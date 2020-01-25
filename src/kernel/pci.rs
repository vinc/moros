use bit_field::BitField;
use crate::{print, kernel};
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;

const CONFIG_ADDR: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

#[derive(Debug, Clone, Copy)]
pub struct DeviceConfig {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub base_addresses: [u32; 6],
    pub interrupt_pin: u8,
    pub interrupt_line: u8,
}

lazy_static! {
    pub static ref PCI_DEVICES: Mutex<Vec<DeviceConfig>> = Mutex::new(Vec::new());
}

pub fn find_device(vendor_id: u16, device_id: u16) -> Option<DeviceConfig> {
    for &device in PCI_DEVICES.lock().iter() {
        if device.vendor_id == vendor_id && device.device_id == device_id {
            return Some(device);
        }
    }
    None
}

pub fn init() {
    for bus in 0..256 {
        check_bus(bus as u8);
    }
}

fn check_bus(bus: u8) {
    for device in 0..32 {
        check_device(bus, device);
    }
}

fn check_device(bus: u8, device: u8) {
    let function = 0;

    let vendor_id = get_vendor_id(bus, device, function);
    if vendor_id == 0xFFFF {
        return; // Device doesn't exist
    }
    add_device(bus, device, function);

    // Multi-function devices
    let header_type = get_header_type(bus, device, function);
    if header_type & 0x80 != 0 {
        for function in 1..8 {
            let vendor_id = get_vendor_id(bus, device, function);
            if vendor_id != 0xFFFF {
                add_device(bus, device, function);
            }
        }
    }
}

fn add_device(bus: u8, device: u8, function: u8) {
    let vendor_id = get_vendor_id(bus, device, function);
    let device_id = get_device_id(bus, device, function);

    let mut base_addresses: [u32; 6] = [0; 6];
    for i in 0..6 {
        let offset = 0x10 + ((i as u8) << 2);
        base_addresses[i] = read_config(bus, device, function, offset);
    }

    let register = read_config(bus, device, function, 0x3C);
    let interrupt_line = register.get_bits(0..8) as u8;
    let interrupt_pin = register.get_bits(8..16) as u8;

    let uptime = kernel::clock::clock_monotonic();
    print!("[{:.6}] PCI {:04}:{:02}:{:02} [{:04X}:{:04X}]\n", uptime, bus, device, function, vendor_id, device_id);
    let device_config = DeviceConfig {
        bus, device, function, vendor_id, device_id, base_addresses, interrupt_pin, interrupt_line
    };
    PCI_DEVICES.lock().push(device_config);
}

fn get_vendor_id(bus: u8, device: u8, function: u8) -> u16 {
    read_config(bus, device, function, 0x00).get_bits(0..16) as u16
}

fn get_device_id(bus: u8, device: u8, function: u8) -> u16 {
    read_config(bus, device, function, 0x00).get_bits(16..32) as u16
}

fn get_header_type(bus: u8, device: u8, function: u8) -> u8 {
    read_config(bus, device, function, 0x0C).get_bits(16..24) as u8
}

fn read_config(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let addr = 0x8000_0000
             | ((bus as u32) << 16)
             | ((device as u32) << 11)
             | ((function as u32) << 8)
             | ((offset as u32) & 0xFC);

    let mut addr_port = Port::new(CONFIG_ADDR);
    let mut data_port = Port::new(CONFIG_DATA);

    unsafe {
        addr_port.write(addr);
        data_port.read()
    }
}
