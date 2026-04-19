mod ac97;
mod sb16;

use crate::api::fs::{FileIO, IO};
use crate::sys::pci::DeviceConfig;
use crate::sys;

use core::cmp;
use core::convert::TryFrom;
use core::convert::TryInto;
use spin::Mutex;

pub static SND: Mutex<Option<(SoundDevice, SoundConfig)>> = Mutex::new(None);

pub enum SoundDevice {
    AC97(ac97::Device),
    SB16(sb16::Device),
}

pub trait SoundDeviceIO {
    fn play(&mut self, buffer: &[u8], config: &SoundConfig);
    fn stop(&mut self);
    fn handle_interrupt(&mut self);
}

impl SoundDeviceIO for SoundDevice {
    fn play(&mut self, buffer: &[u8], config: &SoundConfig) {
        match self {
            SoundDevice::AC97(dev) => dev.play(buffer, config),
            SoundDevice::SB16(dev) => dev.play(buffer, config),
        }
    }

    fn stop(&mut self) {
        match self {
            SoundDevice::AC97(dev) => dev.stop(),
            SoundDevice::SB16(dev) => dev.stop(),
        }
    }

    fn handle_interrupt(&mut self) {
        match self {
            SoundDevice::AC97(dev) => dev.handle_interrupt(),
            SoundDevice::SB16(dev) => dev.handle_interrupt(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SoundConfig {
    channels: u16,
    sample_bits: u16,
    sample_rate: u32,
    data_pos: u32,
    data_len: u32,
}

impl SoundConfig {
    pub fn new() -> Self {
        Self {
            channels: 1,
            sample_bits: 8,
            sample_rate: 44100,
            data_pos: 0,
            data_len: 0,
        }
    }
}

impl TryFrom<&[u8]> for SoundConfig {
    type Error = ();

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        // Try to parse a WAV inside a RIFF container without additionnal
        // metadata and only one single contiguous array of audio samples
        if buf.len() < 44 {
            debug!("SND: Invalid buf size");
            return Err(());
        }
        if buf[0..4] != *b"RIFF" {
            debug!("SND: Error parsing 'RIFF'");
            return Err(());
        }
        if buf[8..12] != *b"WAVE" {
            debug!("SND: Error parsing 'WAVE'");
            return Err(());
        }
        if buf[12..16] != *b"fmt " {
            debug!("SND: Error parsing 'fmt '");
            return Err(());
        }
        if buf[20..22] != 1u16.to_le_bytes() { // Audio format
            debug!("SND: Error parsing audio format");
            return Err(());
        }

        let channels = u16::from_le_bytes(
            buf[22..24].try_into().map_err(|_| ())?
        );
        let sample_rate = u32::from_le_bytes(
            buf[24..28].try_into().map_err(|_| ())?
        );
        let sample_bits = u16::from_le_bytes(
            buf[34..36].try_into().map_err(|_| ())?
        );
        if buf[36..40] != *b"data" {
            debug!("SND: Error parsing 'data'");
            return Err(());
        }
        let data_pos = 44;
        let data_len = u32::from_le_bytes(
            buf[40..44].try_into().map_err(|_| ())?
        );

        Ok(SoundConfig {
            channels,
            sample_bits,
            sample_rate,
            data_pos,
            data_len,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SoundBuffer;

impl SoundBuffer {
    pub fn new() -> Self {
        Self {}
    }

    pub const fn size() -> usize {
        32 << 10
    }
}

impl FileIO for SoundBuffer {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, ()> {
        Err(())
    }

    fn write(&mut self, buffer: &[u8]) -> Result<usize, ()> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            if let Some((ref mut device, ref mut config)) = *SND.lock() {
                if buffer.is_empty() {
                    device.stop();
                } else {
                    let mut i = 0;
                    let mut j = buffer.len();
                    if buffer.get(0..4) == Some(b"RIFF") {
                        device.stop();
                        *config = SoundConfig::try_from(buffer)?;
                        i = config.data_pos as usize; // Skip the header
                    }
                    if config.data_len > 0 {
                        // The buffer can contain less than the whole data and
                        // subsequent writes will play the rest
                        j = cmp::min(j, i + (config.data_len as usize));
                        config.data_len -= (j as u32) - config.data_pos;
                        config.data_pos = 0;
                    } else {
                        device.stop();
                        *config = SoundConfig::new();
                    }
                    device.play(&buffer[i..j], config);
                }
                Ok(buffer.len())
            } else {
                Err(())
            }
        })
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => false,
            IO::Write => true,
        }
    }
}

fn find_device(vendor_id: u16, device_id: u16) -> Option<DeviceConfig> {
    if let Some(mut dev) = sys::pci::find_device(vendor_id, device_id) {
        dev.enable_io_space();
        dev.enable_bus_mastering();
        Some(dev)
    } else {
        None
    }
}

const AC97_DEVICES: [(u16, u16); 2] = [
    (0x8086, 0x2415), // Intel ICH
    (0x1002, 0x4370), // ATI SB400
];

pub fn init() {
    let config = SoundConfig::new();

    if let Some(device) = sb16::find() {
        *SND.lock() = Some((SoundDevice::SB16(device), config.clone()));

        let irq = sb16::IRQ;
        sys::idt::set_irq_handler(irq, interrupt_handler);

        log!("SND DRV SB16 (IRQ {})", irq);
        return;
    }

    for (vendor_id, device_id) in AC97_DEVICES {
        if let Some(pci) = find_device(vendor_id, device_id) {
            let mut device = ac97::Device::new(pci.bar_io(0), pci.bar_io(1));
            //debug!("PCI BAR0: {:#010X}", pci.base_addresses[0]);
            //debug!("PCI BAR1: {:#010X}", pci.base_addresses[1]);
            //debug!("PCI CMD_REG: {:#016b} ({:#08X})", pci.command, pci.command);
            device.init();
            *SND.lock() = Some((SoundDevice::AC97(device), config.clone()));

            let irq = pci.interrupt_line;
            sys::idt::set_irq_handler(irq, interrupt_handler);

            log!("SND DRV AC97 (IRQ {})", irq);
            return;
        }
    }
}

fn interrupt_handler() {
    if let Some((ref mut dev, _)) = *SND.lock() {
        dev.handle_interrupt();
    }
}
