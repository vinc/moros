use crate::kernel::cmos::CMOS;
use lazy_static::lazy_static;
use spin::Mutex;

const DAYS_BEFORE_MONTH: [u64; 13] = [
    0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365
];

lazy_static! {
    pub static ref TICKS: Mutex<usize> = Mutex::new(0);
}

pub fn tick() {
    let mut ticks = TICKS.lock();
    *ticks += 1;
}

pub fn clock_monotonic() -> f64 {
    let ticks = *TICKS.lock();
    1.0 / (1.193182 * 1000000.0 / 65536.0) * ticks as f64
}

pub fn clock_realtime() -> f64 {
    let rtc = CMOS::new().rtc(); // Assuming GMT

    let t = 86400 * days_before_year(rtc.year as u64)
          + 86400 * days_before_month(rtc.year as u64, rtc.month as u64)
          + 86400 * (rtc.day - 1) as u64
          +  3600 * rtc.hour as u64
          +    60 * rtc.minute as u64
          +         rtc.second as u64;

    t as f64
}

fn days_before_year(year: u64) -> u64 {
    (1970..year).fold(0, |days, y| {
        days + if is_leap_year(y) { 366 } else { 365 }
    })
}

fn days_before_month(year: u64, month: u64) -> u64 {
    let leap_day = is_leap_year(year) && month > 2;
    DAYS_BEFORE_MONTH[(month as usize) - 1] + if leap_day { 1 } else { 0 }
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
