use rand_chacha::ChaChaRng;
use rand_core::{RngCore, SeedableRng};
use x86_64::instructions::random::RdRand;

pub fn rand_u64() -> u64{
    let mut seed = [0u8; 32];
    if let Some(rdrand) = RdRand::new() {
        for i in 0..4 {
            if let Some(rand) = rdrand.get_u64() {
                let bytes = rand.to_be_bytes();
                for j in 0..8 {
                    seed[8 * i + j] = bytes[j];
                }
            }
        }
    }

    let mut chacha = ChaChaRng::from_seed(seed);
    chacha.next_u64()
}
