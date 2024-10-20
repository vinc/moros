use super::CMOS;

use crate::sys;
use crate::api::clock::{DATE_TIME, DATE_TIME_LEN};
use crate::api::fs::{FileIO, IO};

use alloc::string::String;
use time::{Date, PrimitiveDateTime};

pub const RTC_CENTURY: u16 = 2000; // NOTE: Change this at the end of 2099

#[repr(u8)]
pub enum Register {
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
pub enum Interrupt {
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
        let date = Date::try_from_ymd(
            self.year.into(), self.month, self.day
        ).map_err(|_| ())?;
        let date_time = date.try_with_hms(
            self.hour, self.minute, self.second
        ).map_err(|_| ())?;
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
        sys::clk::log_rtc();
        Ok(buf.len())
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => true,
            IO::Write => true,
        }
    }
}
