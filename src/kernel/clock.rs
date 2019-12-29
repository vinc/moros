use crate::kernel::cmos::CMOS;
use lazy_static::lazy_static;
use spin::Mutex;

const DAYS_IN_MONTH: [u16; 12] = [
    31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31
];

lazy_static! {
    static ref DAYS_SINCE_MONTH: [u16; 12] = {
        let mut days_since_month = [0; 12];
        for m in 0..12 {
            days_since_month[m] = DAYS_IN_MONTH[m];
            if m > 0 {
                days_since_month[m] += days_since_month[m - 1]
            }
        }
        days_since_month
    };

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
    let rtc = CMOS::new().rtc();

    let days_since_year = 365.25 * (rtc.year - 1970) as f64;
    let days_since_month = DAYS_SINCE_MONTH[(rtc.month as usize) - 1] as f64;

    let t = 86400.0 * days_since_year
          + 86400.0 * days_since_month
          + 86400.0 * rtc.day as f64
          +  3600.0 * rtc.hour as f64
          +    60.0 * rtc.minute as f64
          +           rtc.second as f64;

    t as f64
}
