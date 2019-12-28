use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref TICKS: Mutex<usize> = Mutex::new(0);
}

pub fn tick() {
    let mut ticks = TICKS.lock();
    *ticks += 1;
}

pub fn uptime() -> f64 {
    let ticks = *TICKS.lock();
    1.0 / (1.193182 * 1000000.0 / 65536.0) * ticks as f64
}
