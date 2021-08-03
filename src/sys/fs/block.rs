use super::block_size;
use super::block_bitmap::BlockBitmap;

use core::convert::TryInto;

const DATA_OFFSET: usize = 4;

#[derive(Clone)]
pub struct Block {
    addr: u32,
    buf: [u8; block_size()],
}

// Block structure:
// 0..4 => next block address
// 4..512 => block data
impl Block {
    pub fn new(addr: u32) -> Self {
        let buf = [0; block_size()];
        Self { addr, buf }
    }

    pub fn alloc() -> Option<Self> {
        match BlockBitmap::next_free_addr() {
            None => {
                None
            }
            Some(addr) => {
                BlockBitmap::alloc(addr);

                // Initialize block
                let mut block = Block::read(addr);
                for i in 0..block_size() {
                    block.buf[i] = 0;
                }
                block.write();

                Some(block)
            }
        }
    }

    pub fn read(addr: u32) -> Self {
        let mut buf = [0; block_size()];
        if let Some(ref block_device) = *super::block_device::BLOCK_DEVICE.lock() {
            block_device.read(addr, &mut buf);
        }
        Self { addr, buf }
    }

    pub fn write(&self) {
        if let Some(ref block_device) = *super::block_device::BLOCK_DEVICE.lock() {
            block_device.write(self.addr, &self.buf);
        }
    }

    pub fn addr(&self) -> u32 {
        self.addr
    }

    pub fn data(&self) -> &[u8] {
        &self.buf[DATA_OFFSET..block_size()]
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.buf[DATA_OFFSET..block_size()]
    }

    pub fn len(&self) -> usize {
        block_size() - DATA_OFFSET
    }

    // TODO: Return addr instead of block?
    pub fn next(&self) -> Option<Self> {
        let addr = u32::from_be_bytes(self.buf[0..4].try_into().unwrap());
        if addr == 0 {
            None
        } else {
            Some(Self::read(addr))
        }
    }

    // FIXME: next() returns a Block, but set_next() takes a u32
    pub fn set_next(&mut self, addr: u32) {
        self.buf[0..4].clone_from_slice(&addr.to_be_bytes());
    }
}
