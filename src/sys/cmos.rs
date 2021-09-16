use core::hint::spin_loop;

use bit_field::BitField;
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;

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

#[derive(Debug, PartialEq)]
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
            data: Port::new(0x71),
        }
    }

    fn rtc_unchecked(&mut self) -> RTC {
        RTC {
            second: self.read_register(Register::Second),
            minute: self.read_register(Register::Minute),
            hour: self.read_register(Register::Hour),
            day: self.read_register(Register::Day),
            month: self.read_register(Register::Month),
            year: self.read_register(Register::Year) as u16,
        }
    }

    pub fn rtc(&mut self) -> RTC {
        // Read twice the RTC, discard the result and try again if the reads
        // happened during an update
        let mut rtc;
        loop {
            self.wait_end_of_update();
            rtc = self.rtc_unchecked();
            self.wait_end_of_update();
            if rtc == self.rtc_unchecked() {
                break;
            }
        }

        let b = self.read_register(Register::B);

        if b & 0x04 == 0 { // BCD Mode
            rtc.second = (rtc.second & 0x0F) + ((rtc.second / 16) * 10);
            rtc.minute = (rtc.minute & 0x0F) + ((rtc.minute / 16) * 10);
            rtc.hour = ((rtc.hour & 0x0F) + (((rtc.hour & 0x70) / 16) * 10)) | (rtc.hour & 0x80);
            rtc.day = (rtc.day & 0x0F) + ((rtc.day / 16) * 10);
            rtc.month = (rtc.month & 0x0F) + ((rtc.month / 16) * 10);
            rtc.year = (rtc.year & 0x0F) + ((rtc.year / 16) * 10);
        }

        if (b & 0x02 == 0) && (rtc.hour & 0x80 == 0) { // 12 hour format
            rtc.hour = ((rtc.hour & 0x7F) + 12) % 24;
        }

        rtc.year += 2000; // TODO: Don't forget to change this next century

        rtc
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

    fn wait_end_of_update(&mut self) {
        while self.is_updating() {
            spin_loop();
        }
    }

    fn is_updating(&mut self) -> bool {
        unsafe {
            self.addr.write(Register::A as u8);
            self.data.read().get_bit(7)
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
            self.addr.write(prev | 0x80);
        }
    }
}
