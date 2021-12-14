use crate::sys;
use alloc::string::String;
use alloc::vec::Vec;
use bit_field::BitField;
use core::convert::TryInto;
use core::fmt;
use core::hint::spin_loop;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

pub const BLOCK_SIZE: usize = 512;

#[repr(u16)]
enum Command {
    Read = 0x20,
    Write = 0x30,
    Identify = 0xEC,
}

#[allow(dead_code)]
#[repr(usize)]
enum Status {
    ERR = 0,
    IDX = 1,
    CORR = 2,
    DRQ = 3,
    SRV = 4,
    DF = 5,
    RDY = 6,
    BSY = 7,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Bus {
    id: u8,
    irq: u8,

    data_register: Port<u16>,
    error_register: PortReadOnly<u8>,
    features_register: PortWriteOnly<u8>,
    sector_count_register: Port<u8>,
    lba0_register: Port<u8>,
    lba1_register: Port<u8>,
    lba2_register: Port<u8>,
    drive_register: Port<u8>,
    status_register: PortReadOnly<u8>,
    command_register: PortWriteOnly<u8>,

    alternate_status_register: PortReadOnly<u8>,
    control_register: PortWriteOnly<u8>,
    drive_blockess_register: PortReadOnly<u8>,
}

impl Bus {
    pub fn new(id: u8, io_base: u16, ctrl_base: u16, irq: u8) -> Self {
        Self {
            id, irq,

            data_register: Port::new(io_base + 0),
            error_register: PortReadOnly::new(io_base + 1),
            features_register: PortWriteOnly::new(io_base + 1),
            sector_count_register: Port::new(io_base + 2),
            lba0_register: Port::new(io_base + 3),
            lba1_register: Port::new(io_base + 4),
            lba2_register: Port::new(io_base + 5),
            drive_register: Port::new(io_base + 6),
            status_register: PortReadOnly::new(io_base + 7),
            command_register: PortWriteOnly::new(io_base + 7),

            alternate_status_register: PortReadOnly::new(ctrl_base + 0),
            control_register: PortWriteOnly::new(ctrl_base + 0),
            drive_blockess_register: PortReadOnly::new(ctrl_base + 1),
        }
    }

    fn reset(&mut self) {
        unsafe {
            self.control_register.write(4); // Set SRST bit
            sys::time::nanowait(5); // Wait at least 5 us
            self.control_register.write(0); // Then clear it
            sys::time::nanowait(2000); // Wait at least 2 ms
        }
    }

    fn wait(&mut self) {
        sys::time::nanowait(400); // Wait at least 400 us
    }

    fn write_command(&mut self, cmd: Command) {
        unsafe {
            self.command_register.write(cmd as u8);
        }
    }

    fn status(&mut self) -> u8 {
        unsafe { self.alternate_status_register.read() }
    }

    fn lba1(&mut self) -> u8 {
        unsafe { self.lba1_register.read() }
    }

    fn lba2(&mut self) -> u8 {
        unsafe { self.lba2_register.read() }
    }

    fn read_data(&mut self) -> u16 {
        unsafe { self.data_register.read() }
    }

    fn write_data(&mut self, data: u16) {
        unsafe { self.data_register.write(data) }
    }

    fn is_busy(&mut self) -> bool {
        self.status().get_bit(Status::BSY as usize)
    }

    fn is_error(&mut self) -> bool {
        self.status().get_bit(Status::ERR as usize)
    }

    fn is_ready(&mut self) -> bool {
        self.status().get_bit(Status::DRQ as usize)
    }

    fn busy_loop(&mut self) -> Result<(), ()> {
        let start = sys::clock::uptime();
        while self.is_busy() {
            if sys::clock::uptime() - start > 1.0 {
                //debug!("ATA: busy loop hanged");
                return Err(());
            }
            spin_loop();
        }
        Ok(())
    }

    fn ready_loop(&mut self) -> Result<(), ()> {
        let start = sys::clock::uptime();
        while self.is_ready() {
            if sys::clock::uptime() - start > 1.0 {
                //debug!("ATA: ready loop hanged");
                self.debug();
                return Err(());
            }
            spin_loop();
        }
        Ok(())
    }

    fn select_drive(&mut self, drive: u8) {
        // Drive #0 (primary) = 0xA0
        // Drive #1 (secondary) = 0xB0
        unsafe { self.drive_register.write(0xA0 | (drive << 4)) }
    }

    #[allow(dead_code)]
    fn debug(&mut self) {
        unsafe {
            printk!("drive:  0b{:08b}\n", self.drive_register.read());
            printk!("status: 0b{:08b}\n", self.status_register.read());
        }
    }

    fn setup(&mut self, drive: u8, block: u32) {
        let drive_id = 0xE0 | (drive << 4);
        unsafe {
            self.drive_register.write(drive_id | ((block.get_bits(24..28) as u8) & 0x0F));
            self.sector_count_register.write(1);
            self.lba0_register.write(block.get_bits(0..8) as u8);
            self.lba1_register.write(block.get_bits(8..16) as u8);
            self.lba2_register.write(block.get_bits(16..24) as u8);
        }
    }

    pub fn identify_drive(&mut self, drive: u8) -> Option<[u16; 256]> {
        //debug!("ATA: identify /dev/ata/{}/{}", self.id, drive);
        self.reset();
        if self.busy_loop().is_err() || self.is_error() || self.ready_loop().is_err() {
            //debug!("ATA: identify error: drive not found");
            return None;
        }
        self.select_drive(drive);
        unsafe {
            self.sector_count_register.write(0);
            self.lba0_register.write(0);
            self.lba1_register.write(0);
            self.lba2_register.write(0);
        }
        self.write_command(Command::Identify);
        if self.busy_loop().is_err() {
            //debug!("ATA: identify error: hanged after command");
            return None;
        }

        if !self.is_ready() {
            //debug!("ATA: identify error: command error");
            return None;
        }

        if self.lba1() != 0 || self.lba2() != 0 {
            return None;
        }

        let mut res = [0; 256];
        for i in 0..256 {
            res[i] = self.read_data();
        }

        Some(res)
    }

    pub fn read(&mut self, drive: u8, block: u32, buf: &mut [u8]) {
        assert!(buf.len() == BLOCK_SIZE);
        self.setup(drive, block);
        self.write_command(Command::Read);
        self.busy_loop().ok();
        for i in 0..256 {
            let data = self.read_data();
            buf[i * 2] = data.get_bits(0..8) as u8;
            buf[i * 2 + 1] = data.get_bits(8..16) as u8;
        }
    }

    pub fn write(&mut self, drive: u8, block: u32, buf: &[u8]) {
        assert!(buf.len() == BLOCK_SIZE);
        self.setup(drive, block);
        self.write_command(Command::Write);
        self.busy_loop().ok();
        for i in 0..256 {
            let mut data = 0 as u16;
            data.set_bits(0..8, buf[i * 2] as u16);
            data.set_bits(8..16, buf[i * 2 + 1] as u16);
            self.write_data(data);
        }
        self.busy_loop().ok();
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
        log!("ATA {}:{} {}\n", drive.bus, drive.dsk, drive);
    }
}

#[derive(Clone)]
pub struct Drive {
    pub bus: u8,
    pub dsk: u8,
    blocks: u32,
    model: String,
    serial: String,
}

impl Drive {
    pub fn identify(bus: u8, dsk: u8) -> Option<Self> {
        let mut buses = BUSES.lock();
        if let Some(res) = buses[bus as usize].identify_drive(dsk) {
            let buf = res.map(u16::to_be_bytes).concat();
            let serial = String::from_utf8_lossy(&buf[20..40]).trim().into();
            let model = String::from_utf8_lossy(&buf[54..94]).trim().into();
            let blocks = u32::from_be_bytes(buf[120..124].try_into().unwrap()).rotate_left(16);
            Some(Self { bus, dsk, model, serial, blocks })
        } else {
            None
        }
    }

    pub const fn block_size(&self) -> u32 {
        BLOCK_SIZE as u32
    }

    pub fn block_count(&self) -> u32 {
        self.blocks
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
            if let Some(drive) = Drive::identify(bus, dsk) {
                res.push(drive)
            }
        }
    }
    res
}

pub fn read(bus: u8, drive: u8, block: u32, mut buf: &mut [u8]) {
    let mut buses = BUSES.lock();
    buses[bus as usize].read(drive, block, &mut buf);
}

pub fn write(bus: u8, drive: u8, block: u32, buf: &[u8]) {
    let mut buses = BUSES.lock();
    buses[bus as usize].write(drive, block, buf);
}
