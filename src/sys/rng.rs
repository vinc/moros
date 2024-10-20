use crate::api::fs::{FileIO, IO};
use crate::sys;

use lazy_static::lazy_static;
use rand::{RngCore, SeedableRng};
use rand_hc::Hc128Rng;
use sha2::{Digest, Sha256};
use spin::Mutex;
use x86_64::instructions::random::RdRand;

lazy_static! {
    static ref RNG: Mutex<Hc128Rng> = Mutex::new(Hc128Rng::from_seed([0; 32]));
}

#[derive(Debug, Clone)]
pub struct Random;

impl Random {
    pub fn new() -> Self {
        Self {}
    }
}

impl FileIO for Random {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let n = buf.len();
        for chunk in buf.chunks_mut(8) {
            let bytes = get_u64().to_le_bytes();
            let count = chunk.len();
            chunk.clone_from_slice(&bytes[..count]);
        }
        Ok(n)
    }

    fn write(&mut self, _buf: &[u8]) -> Result<usize, ()> {
        unimplemented!();
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => true,
            IO::Write => false,
        }
    }
}

pub fn get_u64() -> u64 {
    RNG.lock().next_u64()
}

pub fn get_u32() -> u32 {
    get_u64() as u32
}

pub fn get_u16() -> u16 {
    get_u64() as u16
}

pub fn init() {
    let mut seed = [0; 32];
    if let Some(rng) = RdRand::new() {
        log!("RNG RDRAND available");
        for chunk in seed.chunks_mut(8) {
            // NOTE: Intel's Software Developer's Manual, Volume 1, 7.3.17.1
            let mut retry = true;
            for _ in 0..10 { // Retry up to 10 times
                if let Some(num) = rng.get_u64() {
                    chunk.clone_from_slice(&num.to_be_bytes());
                    retry = false;
                    break;
                } else {
                    //debug!("RDRAND: read failed");
                }
            }
            if retry {
                //debug!("RDRAND: retry failed");
            }
        }
    } else {
        log!("RNG RDRAND unavailable");
        let mut hasher = Sha256::new();
        hasher.update(sys::clk::ticks().to_be_bytes());
        hasher.update(sys::clk::epoch_time().to_be_bytes());
        hasher.update(sys::clk::boot_time().to_be_bytes());
        seed = hasher.finalize().into();
    }

    *RNG.lock() = Hc128Rng::from_seed(seed);
}
