#[cfg(not(debug_assertions))]
use rand_chacha::ChaChaRng;
#[cfg(not(debug_assertions))]
use rand_core::{RngCore, SeedableRng};
use x86_64::instructions::random::RdRand;

// FIXME: Compiling this with debug_assertions generate the following error:
// LLVM ERROR: Do not know how to split the result of this operator!
#[cfg(not(debug_assertions))]
pub fn get_u64() -> u64 {
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

#[cfg(debug_assertions)]
pub fn get_u64() -> u64 {
    if let Some(rdrand) = RdRand::new() {
        if let Some(rand) = rdrand.get_u64() {
            return rand;
        }
    }
    0
}

pub fn get_u32() -> u32 {
    get_u64() as u32
}

pub fn get_u16() -> u16 {
    get_u64() as u16
}
