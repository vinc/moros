use crate::api::clock::{DATE_TIME, DATE_TIME_LEN};
use crate::api::fs::{FileIO, IO};

use alloc::string::String;
use bit_field::BitField;
use core::hint::spin_loop;
use time::{Date, PrimitiveDateTime};
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;

const RTC_CENTURY: u16 = 2000; // NOTE: Change this at the end of 2099

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RTC {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl RTC {
    pub fn new() -> Self {
        CMOS::new().rtc()
    }

    pub fn size() -> usize {
        DATE_TIME_LEN
    }

    pub fn sync(&mut self) {
        *self = RTC::new();
    }
}

impl FileIO for RTC {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        self.sync();
        let date = Date::try_from_ymd(self.year.into(), self.month, self.day).map_err(|_| ())?;
        let date_time = date.try_with_hms(self.hour, self.minute, self.second).map_err(|_| ())?;
        let out = date_time.format(DATE_TIME);
        buf.copy_from_slice(out.as_bytes());
        Ok(out.len())
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        let s = String::from_utf8_lossy(buf);
        let s = s.trim_end();
        if s.len() != RTC::size() {
            return Err(());
        }
        let date_time = PrimitiveDateTime::parse(s, DATE_TIME).map_err(|_| ())?;
        self.year = date_time.year() as u16;
        self.month = date_time.month();
        self.day = date_time.day();
        self.hour = date_time.hour();
        self.minute = date_time.minute();
        self.second = date_time.second();
        if self.year < RTC_CENTURY || self.year > RTC_CENTURY + 99 {
            return Err(());
        }
        CMOS::new().update_rtc(self);
        Ok(buf.len())
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

        rtc.year += RTC_CENTURY;

        rtc
    }

    pub fn update_rtc(&mut self, rtc: &RTC) {
        self.wait_end_of_update();
        let mut second = rtc.second;
        let mut minute = rtc.minute;
        let mut hour = rtc.hour;
        let mut day = rtc.day;
        let mut month = rtc.month;
        let mut year = rtc.year;

        year -= RTC_CENTURY;

        let b = self.read_register(Register::B);

        if b & 0x02 == 0 { // 12 hour format
            if hour == 0 {
                hour = 24;
            }
            if hour > 12 {
                hour -= 12;
                hour.set_bit(8, true);
            }
        }

        if b & 0x04 == 0 { // BCD Mode
            second = 16 * (second / 10) + (second % 10);
            minute = 16 * (minute / 10) + (minute % 10);
            hour = 16 * (hour / 10) + (hour % 10);
            day = 16 * (day / 10) + (day % 10);
            month = 16 * (month / 10) + (month % 10);
            year = 16 * (year / 10) + (year % 10);
        }

        self.write_register(Register::Second, second);
        self.write_register(Register::Minute, minute);
        self.write_register(Register::Hour, hour);
        self.write_register(Register::Day, day);
        self.write_register(Register::Month, month);
        self.write_register(Register::Year, year as u8);
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

    fn write_register(&mut self, reg: Register, value: u8) {
        unsafe {
            self.addr.write(reg as u8);
            self.data.write(value);
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
