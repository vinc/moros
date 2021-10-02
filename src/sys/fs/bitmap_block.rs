use super::block::Block;
use super::block_device::BlockDeviceIO;
use super::super_block;
use super::super_block::SuperBlock;

use alloc::vec;
use bit_field::BitField;

// A BitmapBlock store the allocation status of (8 * BLOCK_SIZE) blocks, or 8
// data blocks per byte of a bitmap block.
pub struct BitmapBlock {}

impl BitmapBlock {
    fn block_index(addr: u32) -> u32 {
        let sb = SuperBlock::read();
        let size = sb.block_size();
        let i = addr - sb.data_area();
        sb.bitmap_area() + (i / size / 8)
    }

    fn buffer_index(addr: u32) -> usize {
        let sb = SuperBlock::read();
        let i = (addr - sb.data_area()) as usize;
        i % sb.block_size() as usize
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
        let sb = SuperBlock::read();
        let size = sb.block_size();
        let n = sb.block_count() / size / 8;
        for i in 0..n {
            let block = Block::read(sb.bitmap_area() + i);
            let bitmap = block.data();
            for j in 0..size {
                for k in 0..8 {
                    if !bitmap[j as usize].get_bit(k) {
                        let addr = sb.data_area() + i * 512 * 8 + j * 8 + k as u32;
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
        let sb = SuperBlock::read();
        let a = sb.bitmap_area();
        let b = sb.data_area();
        let n = sb.block_size() as usize;
        let buf = vec![0; n];
        for addr in a..b {
            dev.write(addr, &buf);
        }
    }
}
