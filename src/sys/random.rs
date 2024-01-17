use crate::api::fs::{FileIO, IO};
use crate::sys;

use rand::{RngCore, SeedableRng};
use rand_hc::Hc128Rng;
use x86_64::instructions::random::RdRand;

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
        for i in 0..n {
            buf[i] = get_u64() as u8;
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
    } else {
        // FIXME: RDRAND instruction is not available on old CPUs
        seed[0..8].clone_from_slice(&sys::time::ticks().to_be_bytes());
        seed[8..16].clone_from_slice(&sys::clock::realtime().to_be_bytes());
        seed[16..24].clone_from_slice(&sys::clock::uptime().to_be_bytes());
        seed[24..32].clone_from_slice(&sys::time::ticks().to_be_bytes());
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
