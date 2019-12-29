use crate::print;
use x86_64::instructions::port::Port;

#[repr(u8)]
enum Register {
    Second = 0x00,
    Minute = 0x02,
    Hour = 0x04,
    Day = 0x07,
    Month = 0x08,
    Year = 0x09,
    B = 0x0B,
}

#[derive(Debug)]
pub struct RTC {
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u8,
}

pub struct CMOS {
    addr: Port<u8>,
    data: Port<u8>,
}

impl CMOS {
    pub fn new() -> Self {
        CMOS {
            addr: Port::new(0x70),
            data: Port::new(0x71)
        }
    }

    pub fn read(&mut self) -> RTC {
        while self.is_updating() {
            print!(""); // TODO: sleep
        }
        let mut second = self.read_register(Register::Second);
        let mut minute = self.read_register(Register::Minute);
        let mut hour = self.read_register(Register::Hour);
        let mut day = self.read_register(Register::Day);
        let mut month = self.read_register(Register::Month);
        let mut year = self.read_register(Register::Year);

        let b = self.read_register(Register::B);
        if b & 0x04 == 0 {
            second = (second & 0x0F) + ((second / 16) * 10);
            minute = (minute & 0x0F) + ((minute / 16) * 10);
            hour = ( (hour & 0x0F) + (((hour & 0x70) / 16) * 10) ) | (hour & 0x80);
            day = (day & 0x0F) + ((day / 16) * 10);
            month = (month & 0x0F) + ((month / 16) * 10);
            year = (year & 0x0F) + ((year / 16) * 10);
        }

        RTC { second, minute, hour, day, month, year }
    }

    fn is_updating(&mut self) -> bool {
        unsafe {
            self.addr.write(0x0A as u8);
            (self.data.read() & 0x80 as u8) == 1
        }
    }

    fn read_register(&mut self, reg: Register) -> u8 {
        unsafe {
            self.addr.write(reg as u8);
            self.data.read()
        }
    }
}
