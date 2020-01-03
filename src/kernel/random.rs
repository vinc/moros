use x86_64::instructions::random::RdRand;

pub fn rand64() -> Option<u64> {
    match RdRand::new() {
        Some(rand) => rand.get_u64(),
        None => None
    }
}
