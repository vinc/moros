use super::bitmap_block::BitmapBlock;
use super::block_device::BlockDeviceIO;

use core::convert::TryInto;

const DATA_OFFSET: usize = 4;

#[derive(Clone)]
pub struct Block {
    addr: u32,
    buf: [u8; super::BLOCK_SIZE],
}

// Block structure:
// 0..4 => next block address
// 4..512 => block data
impl Block {
    pub fn new(addr: u32) -> Self {
        let buf = [0; super::BLOCK_SIZE];
        Self { addr, buf }
    }

    pub fn alloc() -> Option<Self> {
        match BitmapBlock::next_free_addr() {
            None => {
                None
            }
            Some(addr) => {
                BitmapBlock::alloc(addr);

                // Initialize block
                let mut block = Block::read(addr);
                for i in 0..super::BLOCK_SIZE {
                    block.buf[i] = 0;
                }
                block.write();

                Some(block)
            }
        }
    }

    pub fn read(addr: u32) -> Self {
        let mut buf = [0; super::BLOCK_SIZE];
        if let Some(ref mut block_device) = *super::block_device::BLOCK_DEVICE.lock() {
            if block_device.read(addr, &mut buf).is_err() {
                debug!("MFS: could not read block {:#x}", addr);
            }
        }
        Self { addr, buf }
    }

    pub fn write(&self) {
        if let Some(ref mut block_device) = *super::block_device::BLOCK_DEVICE.lock() {
            if block_device.write(self.addr, &self.buf).is_err() {
                debug!("MFS: could not write block {:#x}", self.addr);
            }
        }
    }

    pub fn addr(&self) -> u32 {
        self.addr
    }

    pub fn data(&self) -> &[u8] {
        &self.buf[..]
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.buf[..]
    }

    /*
    pub fn len(&self) -> usize {
        self.buf.len()
    }
    */
}

pub struct LinkedBlock {
    block: Block
}

impl LinkedBlock {
    pub fn new(addr: u32) -> Self {
        Self { block: Block::new(addr) }
    }

    pub fn alloc() -> Option<Self> {
        Block::alloc().map(|block| Self { block })
    }

    pub fn read(addr: u32) -> Self {
        Self { block: Block::read(addr) }
    }

    pub fn write(&self) {
        self.block.write()
    }

    pub fn addr(&self) -> u32 {
        self.block.addr()
    }

    pub fn data(&self) -> &[u8] {
        &self.block.buf[DATA_OFFSET..super::BLOCK_SIZE]
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.block.buf[DATA_OFFSET..super::BLOCK_SIZE]
    }

    pub fn len(&self) -> usize {
        super::BLOCK_SIZE - DATA_OFFSET
    }

    pub fn next(&self) -> Option<Self> {
        let addr = u32::from_be_bytes(self.block.buf[0..4].try_into().unwrap());
        if addr == 0 {
            None
        } else {
            Some(Self::read(addr))
        }
    }

    pub fn alloc_next(&mut self) -> Option<Self> {
        let new_block = LinkedBlock::alloc()?;
        self.set_next_addr(new_block.addr());
        self.write();
        Some(new_block)
    }

    pub fn set_next_addr(&mut self, addr: u32) {
        self.block.buf[0..4].clone_from_slice(&addr.to_be_bytes());
    }
}
