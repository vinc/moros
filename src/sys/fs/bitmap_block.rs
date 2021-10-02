use super::block::Block;
use super::block_device::BlockDeviceIO;
use super::super_block;

use alloc::vec;
use bit_field::BitField;

// A BitmapBlock store the allocation status of (8 * BLOCK_SIZE) blocks, or 8
// data blocks per byte of a bitmap block.
pub struct BitmapBlock {}

impl BitmapBlock {
    fn block_index(addr: u32) -> u32 {
        let size = super::BLOCK_SIZE as u32;
        let i = addr - super::DATA_ADDR;
        super::BITMAP_ADDR + (i / size / 8)
    }

    fn buffer_index(addr: u32) -> usize {
        let i = (addr - super::DATA_ADDR) as usize;
        i % super::BLOCK_SIZE
    }

    pub fn alloc(addr: u32) {
        let mut block = Block::read(BitmapBlock::block_index(addr));
        let bitmap = block.data_mut();
        let i = BitmapBlock::buffer_index(addr);
        if !bitmap[i / 8].get_bit(i % 8) {
            bitmap[i / 8].set_bit(i % 8, true);
            block.write();
            super_block::inc_alloc_count();
        }
    }

    pub fn free(addr: u32) {
        let mut block = Block::read(BitmapBlock::block_index(addr));
        let bitmap = block.data_mut();
        let i = BitmapBlock::buffer_index(addr);
        bitmap[i / 8].set_bit(i % 8, false);
        block.write();
        super_block::dec_alloc_count();
    }

    pub fn next_free_addr() -> Option<u32> {
        let size = super::BLOCK_SIZE as u32;
        let n = super::MAX_BLOCKS as u32 / size / 8;
        for i in 0..n {
            let block = Block::read(super::BITMAP_ADDR + i);
            let bitmap = block.data();
            for j in 0..size {
                for k in 0..8 {
                    if !bitmap[j as usize].get_bit(k) {
                        let addr = super::DATA_ADDR + i * 512 * 8 + j * 8 + k as u32;
                        return Some(addr);
                    }
                }
            }
        }
        None
    }
}

pub fn free_all() {
    if let Some(ref mut dev) = *super::block_device::BLOCK_DEVICE.lock() {
        let buf = vec![0; super::BLOCK_SIZE];
        for addr in super::BITMAP_ADDR..super::DATA_ADDR {
            dev.write(addr, &buf);
        }
    }
}
