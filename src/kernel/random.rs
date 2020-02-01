use x86_64::instructions::random::RdRand;

pub fn rand64() -> Option<u64> {
    match RdRand::new() {
        Some(rand) => rand.get_u64(),
        None => None
    }
}

pub fn rand32() -> Option<u32> {
    match RdRand::new() {
        Some(rand) => rand.get_u32(),
        None => None
    }
}

pub fn rand16() -> Option<u16> {
    match RdRand::new() {
        Some(rand) => rand.get_u16(),
        None => None
    }
}
