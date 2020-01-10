use crate::{print, kernel};
use x86_64::instructions::port::Port;

/*
const REG_DATA       : u16 = 0x00;
const REG_ERROR      : u16 = 0x01;
const REG_FEATURES   : u16 = 0x01;
const REG_SECCOUNT0  : u16 = 0x02;
const REG_LBA0       : u16 = 0x03;
*/

const REG_LBA1       : u16 = 0x04;
const REG_LBA2       : u16 = 0x05;
const REG_DEVSEL     : u16 = 0x06;

/*
const REG_COMMAND    : u16 = 0x07;
const REG_STATUS     : u16 = 0x07;
const REG_SECCOUNT1  : u16 = 0x08;
const REG_LBA3       : u16 = 0x09;
const REG_LBA4       : u16 = 0x0A;
const REG_LBA5       : u16 = 0x0B;
const REG_CONTROL    : u16 = 0x0C;
const REG_ALTSTATUS  : u16 = 0x0C;
const REG_DEVADDRESS : u16 = 0x0D;

const CMD_READ_PIO        : u16 = 0x20;
const CMD_READ_PIO_EXT    : u16 = 0x24;
const CMD_READ_DMA        : u16 = 0xC8;
const CMD_READ_DMA_EXT    : u16 = 0x25;
const CMD_WRITE_PIO       : u16 = 0x30;
const CMD_WRITE_PIO_EXT   : u16 = 0x34;
const CMD_WRITE_DMA       : u16 = 0xCA;
const CMD_WRITE_DMA_EXT   : u16 = 0x35;
const CMD_CACHE_FLUSH     : u16 = 0xE7;
const CMD_CACHE_FLUSH_EXT : u16 = 0xEA;
const CMD_PACKET          : u16 = 0xA0;
const CMD_IDENTIFY_PACKET : u16 = 0xA1;
const CMD_IDENTIFY        : u16 = 0xEC;
*/

pub struct Bus {
    pub id: u8,
    pub io_base: u16,
    pub ctrl_base: u16,
    pub irq: u8,
}

impl Bus {
    pub fn detect_drive(&self, drive: u8) {
        // Drive #0 (primary) = 0xA0
        // Drive #1 (secondary) = 0xB0
        let drive_id = 0xA0 | (drive << 4);

        let mut control_port: Port<u16> = Port::new(self.ctrl_base);
        let mut devsel_port = Port::new(self.io_base + REG_DEVSEL);
        let mut lba1_port = Port::new(self.io_base + REG_LBA1);
        let mut lba2_port = Port::new(self.io_base + REG_LBA2);

        unsafe {
            control_port.write(0); // Reset bus
            devsel_port.write(drive_id); // Select drive
            control_port.read(); // Wait 400ns
            control_port.read();
            control_port.read();
            control_port.read();
        };

        let lo: u8 = unsafe { lba1_port.read() }; // Cylinder low byte
        let hi: u8 = unsafe { lba2_port.read() }; // Cylinder high byte

        let uptime = kernel::clock::clock_monotonic();
        let drive_number = self.id * 2 + drive;
        print!("[{:.6}] DRIVE {} [{:02X}:{:02X}]\n", uptime, drive_number, lo, hi);
    }
}

pub fn init() {
    let primary_bus = Bus {
        id: 0,
        io_base: 0x1F0,
        ctrl_base: 0x3F6,
        irq: 14,
    };
    primary_bus.detect_drive(0);
    primary_bus.detect_drive(1);

    let secondary_bus = Bus {
        id: 1,
        io_base: 0x170,
        ctrl_base: 0x376,
        irq: 15,
    };
    secondary_bus.detect_drive(0);
    secondary_bus.detect_drive(1);
}
