use alloc::slice::SliceIndex;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::ops::{Index, IndexMut};
use spin::Mutex;
use x86_64::VirtAddr;

#[derive(Clone)]
pub struct PhysBuf {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl PhysBuf {
    pub fn new(len: usize) -> Self {
        Self::from(vec![0; len])
    }

    // Realloc vec until it uses a chunk of contiguous physical memory
    fn from(vec: Vec<u8>) -> Self {
        let buffer_end = vec.len() - 1;
        let memory_end = phys_addr(&vec[buffer_end]) - phys_addr(&vec[0]);
        if buffer_end == memory_end as usize {
            Self {
                buf: Arc::new(Mutex::new(vec)),
            }
        } else {
            Self::from(vec.clone()) // Clone vec and try again
        }
    }

    pub fn addr(&self) -> u64 {
        phys_addr(&self.buf.lock()[0])
    }
}

impl<I: SliceIndex<[u8]>> Index<I> for PhysBuf {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<I: SliceIndex<[u8]>> IndexMut<I> for PhysBuf {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}

impl core::ops::Deref for PhysBuf {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        let vec = self.buf.lock();
        unsafe { alloc::slice::from_raw_parts(vec.as_ptr(), vec.len()) }
    }
}

impl core::ops::DerefMut for PhysBuf {
    fn deref_mut(&mut self) -> &mut [u8] {
        let mut vec = self.buf.lock();
        unsafe {
            alloc::slice::from_raw_parts_mut(vec.as_mut_ptr(), vec.len())
        }
    }
}

pub fn phys_addr(ptr: *const u8) -> u64 {
    let virt_addr = VirtAddr::new(ptr as u64);
    let phys_addr = super::virt_to_phys(virt_addr).unwrap();
    phys_addr.as_u64()
}
