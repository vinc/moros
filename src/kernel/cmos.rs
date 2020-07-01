use crate::print;
use x86_64::instructions::port::Port;
use x86_64::instructions::interrupts;

#[repr(u8)]
enum Register {
    Second = 0x00,
    Minute = 0x02,
    Hour = 0x04,
    Day = 0x07,
    Month = 0x08,
    Year = 0x09,
    A = 0x0A,
    B = 0x0B,
    C = 0x0C,
}

#[repr(u8)]
enum Interrupt {
    Periodic = 1 << 6,
    Alarm = 1 << 5,
    Update = 1 << 4,
}

#[derive(Debug)]
pub struct RTC {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
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

    pub fn rtc(&mut self) -> RTC {
        while self.is_updating() {
            print!(""); // TODO: sleep
        }
        let mut second = self.read_register(Register::Second);
        let mut minute = self.read_register(Register::Minute);
        let mut hour = self.read_register(Register::Hour);
        let mut day = self.read_register(Register::Day);
        let mut month = self.read_register(Register::Month);
        let mut year = self.read_register(Register::Year) as u16;

        let b = self.read_register(Register::B);
        if b & 0x04 == 0 {
            second = (second & 0x0F) + ((second / 16) * 10);
            minute = (minute & 0x0F) + ((minute / 16) * 10);
            hour = ((hour & 0x0F) + (((hour & 0x70) / 16) * 10) ) | (hour & 0x80);
            day = (day & 0x0F) + ((day / 16) * 10);
            month = (month & 0x0F) + ((month / 16) * 10);
            year = (year & 0x0F) + ((year / 16) * 10);
        }

        year += 2000; // TODO: Don't forget to change this next century

        RTC { year, month, day, hour, minute, second }
    }

    pub fn enable_periodic_interrupt(&mut self) {
        self.enable_interrupt(Interrupt::Periodic);
    }

    pub fn enable_alarm_interrupt(&mut self) {
        self.enable_interrupt(Interrupt::Alarm);
    }

    pub fn enable_update_interrupt(&mut self) {
        self.enable_interrupt(Interrupt::Update);
    }

    /// Rate must be between 3 and 15
    /// Resulting in the following frequency: 32768 >> (rate - 1)
    pub fn set_periodic_interrupt_rate(&mut self, rate: u8) {
        interrupts::without_interrupts(|| {
            self.disable_nmi();
            unsafe {
                self.addr.write(Register::A as u8);
                let prev = self.data.read();
                self.addr.write(Register::A as u8);
                self.data.write((prev & 0xF0) | rate);
            }
            self.enable_nmi();
            self.notify_end_of_interrupt();
        });
    }

    fn enable_interrupt(&mut self, interrupt: Interrupt) {
        interrupts::without_interrupts(|| {
            self.disable_nmi();
            unsafe {
                self.addr.write(Register::B as u8);
                let prev = self.data.read();
                self.addr.write(Register::B as u8);
                self.data.write(prev | interrupt as u8);
            }
            self.enable_nmi();
            self.notify_end_of_interrupt();
        });
    }

    pub fn notify_end_of_interrupt(&mut self) {
        unsafe {
            self.addr.write(Register::C as u8);
            self.data.read();
        }
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

    fn enable_nmi(&mut self) {
        unsafe {
            let prev = self.addr.read();
            self.addr.write(prev & 0x7F);
        }
    }

    fn disable_nmi(&mut self) {
        unsafe {
            let prev = self.addr.read();
            self.addr.write(prev  | 0x80);
        }
    }
}
