use crate::{print, kernel};
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

    let vendor_id = vendor_id(bus, device, function);
    if vendor_id == 0xFFFF {
        return; // Device doesn't exist
    }
    /*
    check_function(bus, device, function);
    header_type = header_type(bus, device, function);
    // Multi-function devices
    if header_type & 0x80 != 0 {
        for function in i..8 {
            if vendor_id(bus, device, function) != 0xFFFF {
                check_function(bus, device, function);
            }
        }
    }
    */
}

/*
fn check_function(bus: u8, device: u8, function: u8) {
}
*/

fn vendor_id(bus: u8, device: u8, function: u8) -> u16 {
    let vendor_id = read_config(bus, device, function, 0);
    if vendor_id != 0xFFFF {
        let device_id = read_config(bus, device, function, 2);
        print!("[{:.6}] PCI {:04}:{:02}:{:02} [{:04X}:{:04X}]\n", kernel::clock::clock_monotonic(), bus, device, function, vendor_id, device_id);
    }
    vendor_id
}

fn read_config(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    let addr = ((bus as u32) << 16)
             | ((device as u32) << 11)
             | ((function as u32) << 8)
             | ((offset as u32) & 0xFC)
             | 0x8000_0000;

    let mut addr_port = Port::new(CONFIG_ADDR);
    let mut data_port = Port::new(CONFIG_DATA);

    let data: u32 = unsafe {
        addr_port.write(addr);
        data_port.read()
    };

    ((data >> (((offset as u32) & 2) * 8)) as u16) & 0xFFFF
}
