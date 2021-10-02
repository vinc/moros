use super::block::Block;
use super::block_device::BlockDeviceIO;
use super::super_block;

use alloc::vec;
use bit_field::BitField;

pub const BITMAP_SIZE: usize = super::BLOCK_SIZE;

// A BlockBitmap store the allocation status of BITMAP_SIZE * 8 data blocks
pub struct BlockBitmap {}

impl BlockBitmap {
    fn block_index(addr: u32) -> u32 {
        let size = BITMAP_SIZE as u32;
        let i = addr - super::DATA_ADDR;
        super::BITMAP_ADDR + (i / size / 8)
    }

    fn buffer_index(addr: u32) -> usize {
        let i = (addr - super::DATA_ADDR) as usize;
        i % BITMAP_SIZE
    }

    pub fn alloc(addr: u32) {
        let mut block = Block::read(BlockBitmap::block_index(addr));
        let bitmap = block.data_mut();
        let i = BlockBitmap::buffer_index(addr);
        if !bitmap[i / 8].get_bit(i % 8) {
            bitmap[i / 8].set_bit(i % 8, true);
            block.write();
            super_block::inc_alloc_count();
        }
    }

    pub fn free(addr: u32) {
        let mut block = Block::read(BlockBitmap::block_index(addr));
        let bitmap = block.data_mut();
        let i = BlockBitmap::buffer_index(addr);
        bitmap[i / 8].set_bit(i % 8, false);
        block.write();
        super_block::dec_alloc_count();
    }

    pub fn next_free_addr() -> Option<u32> {
        let size = BITMAP_SIZE as u32;
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
