use crate::api::fs::{FileIO, IO};
//use crate::sys;

use rand::{RngCore, SeedableRng};
use rand_hc::Hc128Rng;
use sha2::{Digest, Sha256};
use spin::Mutex;
use x86_64::instructions::random::RdRand;

static SEED: Mutex<[u8; 32]> = Mutex::new([0; 32]);

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
    let mut seed = [0; 32];
    // NOTE: Intel's Software Developer's Manual, Volume 1, 7.3.17.1
    if let Some(rng) = RdRand::new() {
        for chunk in seed.chunks_mut(8) {
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
        //debug!("RDRAND: unavailable");
        let mut old_seed = SEED.lock();
        let mut hasher = Sha256::new();
        hasher.update(*old_seed);
        //hasher.update(&sys::time::ticks().to_be_bytes());
        //hasher.update(&sys::clock::realtime().to_be_bytes());
        //hasher.update(&sys::clock::uptime().to_be_bytes());
        seed = hasher.finalize().into();
        *old_seed = seed;
    }

    let mut rng = Hc128Rng::from_seed(seed);
    rng.next_u64()
}

pub fn get_u32() -> u32 {
    get_u64() as u32
}

pub fn get_u16() -> u16 {
    get_u64() as u16
}

pub fn init() {
    if RdRand::new().is_some() {
        log!("RNG RDRAND available");
    } else {
        log!("RNG RDRAND unavailable");
    }
}
