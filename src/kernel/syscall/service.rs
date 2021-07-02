use crate::kernel;

pub fn sleep(arg1: usize, _arg2: usize, _arg3: usize) -> usize {
    kernel::time::sleep(f64::from_bits(arg1 as u64));
    0
}

pub fn uptime(_arg1: usize, _arg2: usize, _arg3: usize) -> usize {
    kernel::clock::uptime().to_bits() as usize
}
