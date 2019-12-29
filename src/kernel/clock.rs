// use crate::print;
use crate::kernel::cmos::CMOS;
use lazy_static::lazy_static;
use spin::Mutex;

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
    let mut cmos = CMOS::new();
    let rtc = cmos.read();
    // print!("{:?}\n", rtc);
    let t = rtc.second as u64 + 60 * rtc.minute as u64 + 3600 * rtc.hour as u64;

    t as f64
}
