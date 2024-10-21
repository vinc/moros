use super::cmos::CMOS;
use super::timer;

use crate::api::fs::{FileIO, IO};

const DAYS_BEFORE_MONTH: [u64; 13] = [
    0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365
];

#[derive(Debug, Clone)]
pub struct EpochTime;

impl EpochTime {
    pub fn new() -> Self {
        Self {}
    }

    pub fn size() -> usize {
        core::mem::size_of::<f64>()
    }
}

impl FileIO for EpochTime {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let time = epoch_time().to_be_bytes();
        let n = time.len();
        if buf.len() >= n {
            buf[0..n].clone_from_slice(&time);
            Ok(n)
        } else {
            Err(())
        }
    }

    fn write(&mut self, _buf: &[u8]) -> Result<usize, ()> {
        Err(())
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => true,
            IO::Write => false,
        }
    }
}

/// Returns the number of seconds since Unix epoch (1970-01-01 00:00:00 UTC).
///
/// This clock is not monotonic.
pub fn epoch_time() -> f64 {
    let rtc = CMOS::new().rtc(); // Assuming GMT

    let ts = 86400 * days_before_year(rtc.year as u64)
           + 86400 * days_before_month(rtc.year as u64, rtc.month as u64)
           + 86400 * (rtc.day - 1) as u64
           + 3600 * rtc.hour as u64
           + 60 * rtc.minute as u64
           + rtc.second as u64;

    let fract = timer::time_between_ticks()
              * (timer::ticks() - timer::last_rtc_update()) as f64;

    (ts as f64) + fract
}

fn days_before_year(year: u64) -> u64 {
    (1970..year).fold(0, |days, y|
        days + if is_leap_year(y) { 366 } else { 365 }
    )
}

fn days_before_month(year: u64, month: u64) -> u64 {
    let leap_day = is_leap_year(year) && month > 2;
    DAYS_BEFORE_MONTH[(month as usize) - 1] + (leap_day as u64)
}

fn is_leap_year(year: u64) -> bool {
    if year % 4 != 0 {
        false
    } else if year % 100 != 0 {
        true
    } else if year % 400 != 0 {
        false
    } else {
        true
    }
}

#[test_case]
fn test_epoch_time() {
    assert!(epoch_time() > 1234567890.0);
}
