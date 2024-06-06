use super::block::Block;
use super::super_block;
use super::super_block::SuperBlock;

use bit_field::BitField;

pub const BITMAP_SIZE: usize = 8 * super::BLOCK_SIZE;

// A BitmapBlock store the allocation status of BITMAP_SIZE blocks, or 8
// data blocks per byte (1 per bit) of a bitmap block.
pub struct BitmapBlock {}

impl BitmapBlock {
    fn indexes(addr: u32) -> (u32, usize) {
        let sb = SuperBlock::read();
        let i = addr - sb.data_area();
        let n = sb.block_size();
        (sb.bitmap_area() + (i / n / 8), (i % (n * 8)) as usize)
    }

    pub fn alloc(addr: u32) {
        let (a, i) = Self::indexes(addr);
        let mut block = Block::read(a);
        let bitmap = block.data_mut();
        if !bitmap[i / 8].get_bit(i % 8) {
            bitmap[i / 8].set_bit(i % 8, true);
            block.write();
            super_block::inc_alloc_count();
        } else {
            // TODO: alloc failed
        }
    }

    pub fn free(addr: u32) {
        let (a, i) = Self::indexes(addr);
        let mut block = Block::read(a);
        let bitmap = block.data_mut();
        bitmap[i / 8].set_bit(i % 8, false);
        block.write();
        super_block::dec_alloc_count();
    }

    pub fn next_free_addr() -> Option<u32> {
        let sb = SuperBlock::read();
        if sb.alloc_count() == sb.block_count() - 1 {
            return None;
        }

        let n = sb.block_size();
        let m = sb.block_count() / n / 8;
        for i in 0..m {
            let block = Block::read(sb.bitmap_area() + i);
            let bitmap = block.data();
            for j in 0..n {
                for k in 0..8 {
                    if !bitmap[j as usize].get_bit(k as usize) {
                        let addr = sb.data_area() + (i * n * 8) + (j * 8) + k;
                        return Some(addr);
                    }
                }
            }
        }
        None
    }
}

pub fn free_all() {
    let sb = SuperBlock::read();
    let a = sb.bitmap_area();
    let b = sb.data_area();
    for addr in a..b {
        Block::new(addr).write();
    }
}
