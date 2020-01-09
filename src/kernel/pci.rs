use crate::{print, kernel};
use bit_field::BitField;
use x86_64::instructions::port::Port;

const CONFIG_ADDR: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;


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
    let device_id = get_device_id(bus, device, function);
    //check_function(bus, device, function);
    let header_type = get_header_type(bus, device, function);
    print!("[{:.6}] PCI {:04}:{:02}:{:02} [{:04X}:{:04X}]\n", kernel::clock::clock_monotonic(), bus, device, function, vendor_id, device_id);
    // Multi-function devices
    if header_type & 0x80 != 0 {
        for function in 1..8 {
            let vendor_id = get_vendor_id(bus, device, function);
            if vendor_id != 0xFFFF {
                print!("[{:.6}] PCI {:04}:{:02}:{:02} [{:04X}:{:04X}]\n", kernel::clock::clock_monotonic(), bus, device, function, vendor_id, device_id);
                //check_function(bus, device, function);
            }
        }
    }
}

/*
fn check_function(bus: u8, device: u8, function: u8) {
}
*/

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
