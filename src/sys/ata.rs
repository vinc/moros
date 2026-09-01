use crate::sys;

#[cfg(target_arch = "x86_64")]
use crate::sys::fs::{FileIO, IO};

use crate::sys::x86::port::*;

use alloc::string::String;
use alloc::vec::Vec;
use bit_field::BitField;
use core::convert::TryInto;
use core::fmt;
use core::hint::spin_loop;
use lazy_static::lazy_static;
use spin::Mutex;

// Information Technology
// AT Attachment with Packet Interface Extension (ATA/ATAPI-4)
// (1998)

pub const BLOCK_SIZE: usize = 512;

// Keep track of the last selected bus and drive pair to speed up operations
pub static LAST_SELECTED: Mutex<Option<(u8, u8)>> = Mutex::new(None);

const DATA_REGISTER:             u16 = 0;
const ERROR_REGISTER:            u16 = 1;
const SECTOR_COUNT_REGISTER:     u16 = 2;
const LBA0_REGISTER:             u16 = 3;
const LBA1_REGISTER:             u16 = 4;
const LBA2_REGISTER:             u16 = 5;
const DRIVE_REGISTER:            u16 = 6;
const STATUS_REGISTER:           u16 = 7;
const COMMAND_REGISTER:          u16 = 7;

const ALTERNATE_STATUS_REGISTER: u16 = 0;
const CONTROL_REGISTER:          u16 = 0;

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
enum Command {
    Read     = 0x20,
    Write    = 0x30,
    Identify = 0xEC,
}

enum IdentifyResponse {
    Ata([u16; 256]),
    Atapi,
    Sata,
    None,
}

#[allow(dead_code)]
#[repr(usize)]
#[derive(Debug, Clone, Copy)]
enum Status {
    ERR  = 0, // Error
    IDX  = 1, // (obsolete)
    CORR = 2, // (obsolete)
    DRQ  = 3, // Data Request
    DSC  = 4, // (command dependant)
    DF   = 5, // (command dependant)
    DRDY = 6, // Device Ready
    BSY  = 7, // Busy
}

type Res = Result<(), ()>;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Bus {
    id: u8,
    irq: u8,
    io_base: u16,
    ctrl_base: u16,
}

impl Bus {
    pub fn new(id: u8, io_base: u16, ctrl_base: u16, irq: u8) -> Self {
        Self { id, io_base, ctrl_base, irq }
    }

    fn check_floating_bus(&self) -> Res {
        match self.status() {
            0xFF | 0x7F => Err(()),
            _ => Ok(()),
        }
    }

    fn wait(&self, ns: u64) {
        sys::clk::wait(ns);
    }

    fn clear_interrupt(&self) -> u8 {
        unsafe { inb(self.io_base + STATUS_REGISTER) }
    }

    fn status(&self) -> u8 {
        unsafe { inb(self.ctrl_base + ALTERNATE_STATUS_REGISTER) }
    }

    fn lba1(&self) -> u8 {
        unsafe { inb(self.io_base + LBA1_REGISTER) }
    }

    fn lba2(&self) -> u8 {
        unsafe { inb(self.io_base + LBA2_REGISTER) }
    }

    fn read_data(&self) -> u16 {
        unsafe { inw(self.io_base + DATA_REGISTER) }
    }

    fn write_data(&self, data: u16) {
        unsafe { outw(self.io_base + DATA_REGISTER, data) }
    }

    fn is_error(&self) -> bool {
        self.status().get_bit(Status::ERR as usize)
    }

    fn poll(&self, bit: Status, val: bool) -> Res {
        let start = sys::clk::boot_time();
        while self.status().get_bit(bit as usize) != val {
            if sys::clk::boot_time() - start > 1.0 {
                debug!(
                    "ATA hanged while polling {:?} bit in status register",
                    bit
                );
                self.debug();
                return Err(());
            }
            spin_loop();
        }
        Ok(())
    }

    fn select_drive(&self, drive: u8) -> Res {
        self.poll(Status::BSY, false)?;
        self.poll(Status::DRQ, false)?;

        // Skip the rest if this drive was already selected
        if *LAST_SELECTED.lock() == Some((self.id, drive)) {
            return Ok(());
        } else {
            *LAST_SELECTED.lock() = Some((self.id, drive));
        }

        unsafe {
            // Bit 4 => DEV
            // Bit 5 => 1
            // Bit 7 => 1
            outb(self.io_base + DRIVE_REGISTER, 0xA0 | (drive << 4));
        }
        sys::clk::wait(400); // Wait at least 400 ns
        self.poll(Status::BSY, false)?;
        self.poll(Status::DRQ, false)?;
        Ok(())
    }

    fn write_command_params(&self, drive: u8, block: u32, count: u8) -> Res {
        let lba = true;
        let mut bytes = block.to_le_bytes();
        bytes[3].set_bit(4, drive > 0);
        bytes[3].set_bit(5, true);
        bytes[3].set_bit(6, lba);
        bytes[3].set_bit(7, true);
        unsafe {
            outb(self.io_base + SECTOR_COUNT_REGISTER, count);
            outb(self.io_base + LBA0_REGISTER, bytes[0]);
            outb(self.io_base + LBA1_REGISTER, bytes[1]);
            outb(self.io_base + LBA2_REGISTER, bytes[2]);
            outb(self.io_base + DRIVE_REGISTER, bytes[3]);
        }
        Ok(())
    }

    fn write_command(&self, cmd: Command) -> Res {
        unsafe {
            outb(self.io_base + COMMAND_REGISTER, cmd as u8);
        }
        self.wait(400); // Wait at least 400 ns
        self.status(); // Ignore results of first read
        self.clear_interrupt();
        if self.status() == 0 { // Drive does not exist
            return Err(());
        }
        Ok(())
    }

    // Wait for the drive to be ready to transfer one sector of data
    fn sync(&self) -> Res {
        if self.is_error() {
            //debug!("ATA {:?} command errored", cmd);
            //self.debug();
            return Err(());
        }
        self.poll(Status::BSY, false)?;
        self.poll(Status::DRQ, true)?;
        Ok(())
    }

    fn setup_pio(&self, drive: u8, block: u32, count: u8) -> Res {
        self.select_drive(drive)?;
        self.write_command_params(drive, block, count)?;
        Ok(())
    }

    fn read(&self, drive: u8, block: u32, buf: &mut [u8]) -> Res {
        debug_assert!(buf.len() % BLOCK_SIZE == 0);
        let count = buf.len() / BLOCK_SIZE;
        if count == 0 || count > 255 {
            return Err(());
        }
        self.setup_pio(drive, block, count as u8)?;
        self.write_command(Command::Read)?;
        for sector in buf.chunks_mut(BLOCK_SIZE) {
            self.sync()?;
            for chunk in sector.chunks_mut(2) {
                let data = self.read_data().to_le_bytes();
                chunk.clone_from_slice(&data);
            }
        }
        if self.is_error() {
            debug!("ATA read: data error");
            self.debug();
            Err(())
        } else {
            Ok(())
        }
    }

    fn write(&self, drive: u8, block: u32, buf: &[u8]) -> Res {
        debug_assert!(buf.len() % BLOCK_SIZE == 0);
        let count = buf.len() / BLOCK_SIZE;
        if count == 0 || count > 255 {
            return Err(());
        }
        self.setup_pio(drive, block, count as u8)?;
        self.write_command(Command::Write)?;
        for sector in buf.chunks(BLOCK_SIZE) {
            self.sync()?;
            for chunk in sector.chunks(2) {
                let data = u16::from_le_bytes(chunk.try_into().unwrap());
                self.write_data(data);
            }
        }
        if self.is_error() {
            debug!("ATA write: data error");
            self.debug();
            Err(())
        } else {
            Ok(())
        }
    }

    fn identify_drive(&self, drive: u8) -> Result<IdentifyResponse, ()> {
        if self.check_floating_bus().is_err() {
            return Ok(IdentifyResponse::None);
        }
        self.select_drive(drive)?;
        self.write_command_params(drive, 0, 1)?;
        if self.write_command(Command::Identify).is_err() {
            if self.status() == 0 {
                return Ok(IdentifyResponse::None);
            } else {
                return Err(());
            }
        }
        match (self.lba1(), self.lba2()) {
            (0x00, 0x00) => {
                self.sync()?;
                Ok(IdentifyResponse::Ata([(); 256].map(|_| self.read_data())))
            }
            (0x14, 0xEB) => Ok(IdentifyResponse::Atapi),
            (0x3C, 0xC3) => Ok(IdentifyResponse::Sata),
            (_, _) => Err(()),
        }
    }

    #[allow(dead_code)]
    fn reset(&self) {
        unsafe {
            outb(self.ctrl_base + CONTROL_REGISTER, 4); // Set SRST bit
            self.wait(5); // Wait at least 5 ns
            outb(self.ctrl_base + CONTROL_REGISTER, 0); // Then clear it
            self.wait(2000); // Wait at least 2 ms
        }
    }

    #[allow(dead_code)]
    fn debug(&self) {
        unsafe {
            debug!(
                "ATA status register: 0b{:08b} <BSY|DRDY|#|#|DRQ|#|#|ERR>",
                inb(self.ctrl_base + ALTERNATE_STATUS_REGISTER)
            );
            debug!(
                "ATA error register:  0b{:08b} <#|#|#|#|#|ABRT|#|#>",
                inb(self.io_base + ERROR_REGISTER)
            );
        }
    }
}

lazy_static! {
    pub static ref BUSES: Mutex<Vec<Bus>> = Mutex::new(Vec::new());
}

pub fn init() {
    {
        let mut buses = BUSES.lock();
        buses.push(Bus::new(0, 0x1F0, 0x3F6, 14));
        buses.push(Bus::new(1, 0x170, 0x376, 15));
    }

    for drive in list() {
        log!("ATA {}:{} {}", drive.bus, drive.dsk, drive);
    }
}

#[derive(Clone, Debug)]
pub struct Drive {
    pub bus: u8,
    pub dsk: u8,
    model: String,
    serial: String,
    block_count: u32,
    block_index: u32,
}

impl Drive {
    pub fn size() -> usize {
        BLOCK_SIZE
    }

    pub fn open(bus: u8, dsk: u8) -> Option<Self> {
        let buses = BUSES.lock();
        let res = buses[bus as usize].identify_drive(dsk);
        if let Ok(IdentifyResponse::Ata(res)) = res {
            let buf = res.map(u16::to_be_bytes).concat();
            let model = String::from_utf8_lossy(&buf[54..94]).trim().into();
            let serial = String::from_utf8_lossy(&buf[20..40]).trim().into();
            let block_count = u32::from_be_bytes(
                buf[120..124].try_into().unwrap()
            ).rotate_left(16);
            let block_index = 0;

            Some(Self {
                bus,
                dsk,
                model,
                serial,
                block_count,
                block_index,
            })
        } else {
            None
        }
    }

    pub const fn block_size(&self) -> u32 {
        BLOCK_SIZE as u32
    }

    pub fn block_count(&self) -> u32 {
        self.block_count
    }

    fn humanized_size(&self) -> (usize, String) {
        let size = self.block_size() as usize;
        let count = self.block_count() as usize;
        let bytes = size * count;
        if bytes >> 20 < 1000 {
            (bytes >> 20, String::from("MB"))
        } else {
            (bytes >> 30, String::from("GB"))
        }
    }
}

#[cfg(target_arch = "x86_64")]
impl FileIO for Drive {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        if self.block_index == self.block_count {
            return Ok(0);
        }

        let mut buses = BUSES.lock();
        let bus = &mut buses[self.bus as usize];
        bus.read(self.dsk, self.block_index, buf)?;
        self.block_index += 1;
        Ok(buf.len())
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        let mut buses = BUSES.lock();
        let bus = &mut buses[self.bus as usize];

        let mut count = 0;
        for chunk in buf.chunks(BLOCK_SIZE) {
            if self.block_index == self.block_count {
                return Err(());
            }
            let n = chunk.len();
            if n == BLOCK_SIZE {
                bus.write(self.dsk, self.block_index, chunk)?;
            } else {
                let mut block = [0; BLOCK_SIZE];
                block[0..n].clone_from_slice(chunk);
                bus.write(self.dsk, self.block_index, &block)?;
            }
            self.block_index += 1;
            count += chunk.len();
        }
        Ok(count)
    }

    fn close(&mut self) {
    }

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => true,
            IO::Write => true,
        }
    }
}

impl fmt::Display for Drive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (size, unit) = self.humanized_size();
        write!(f, "{} {} ({} {})", self.model, self.serial, size, unit)
    }
}

pub fn list() -> Vec<Drive> {
    let mut res = Vec::new();
    for bus in 0..2 {
        for dsk in 0..2 {
            if let Some(drive) = Drive::open(bus, dsk) {
                res.push(drive)
            }
        }
    }
    res
}

pub fn read(bus: u8, drive: u8, block: u32, buf: &mut [u8]) -> Res {
    let buses = BUSES.lock();
    buses[bus as usize].read(drive, block, buf)
}

pub fn write(bus: u8, drive: u8, block: u32, buf: &[u8]) -> Res {
    let buses = BUSES.lock();
    buses[bus as usize].write(drive, block, buf)
}
