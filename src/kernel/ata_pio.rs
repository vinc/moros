use crate::{print, kernel};
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;

const REG_DATA       : u16 = 0x00;
const REG_ERROR      : u16 = 0x01;
const REG_FEATURES   : u16 = 0x01;
const REG_SECCOUNT0  : u16 = 0x02;
const REG_LBA0       : u16 = 0x03;
const REG_LBA1       : u16 = 0x04;
const REG_LBA2       : u16 = 0x05;
const REG_DEVSEL     : u16 = 0x06;
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

pub fn init() {
    let ctrl_base = 0x1F0;
    //let ctrl_dev_ctl = 0x3F6;
    let slavebit = 1;

    kernel::sleep::sleep(0.1);
    let mut reg_devsel = Port::new(ctrl_base + REG_DEVSEL);
    let mut reg_command = Port::new(ctrl_base + REG_COMMAND);
    //let dev_ctl = Port::new(ctrl_dev_ctl);
    let mut reg_lba1 = Port::new(ctrl_base + REG_LBA1);
    let mut reg_lba2 = Port::new(ctrl_base + REG_LBA2);

    //interrupts::without_interrupts(|| {
        unsafe {
            reg_devsel.write((0xA0 | (slavebit << 4)) as u8);
        }
    //});
    kernel::sleep::sleep(0.1);
    //interrupts::without_interrupts(|| {
        unsafe {
            reg_command.write(CMD_IDENTIFY);
        }
    //});
    kernel::sleep::sleep(1.0);
    //unsafe {
        //dev_ctl.read();
        //dev_ctl.read();
        //dev_ctl.read();
        //dev_ctl.read();
    //};
    let cl: u8 = unsafe {
        reg_lba1.read()
    };
    let ch: u8 = unsafe {
        reg_lba2.read()
    };
    print!("disk: {:02X} {:02X}\n", cl, ch);
}
