use crate::kernel;

pub fn sleep(seconds: f64) {
    kernel::time::sleep(seconds);
}

pub fn uptime() -> f64 {
    kernel::clock::uptime()
}
