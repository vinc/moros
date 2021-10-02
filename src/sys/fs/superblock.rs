use super::block::Block;
use super::block_device::AtaBlockDevice;
use core::convert::TryInto;

pub const SIGNATURE: &[u8; 8] = b"MOROS FS";

#[derive(Debug)]
pub struct Superblock {
    signature: &'static[u8; 8],
    version: u8,
    block_size: u32,
    pub block_count: u32,
    pub alloc_count: u32,
}

impl Superblock {
    pub fn from_ata(dev: &AtaBlockDevice) -> Self {
        Self {
            signature: SIGNATURE,
            version: super::VERSION,
            block_size: dev.block_size() as u32,
            block_count: dev.block_count() as u32,
            alloc_count: 0,
        }
    }

    pub fn read() -> Self {
        let block = Block::read(super::SUPERBLOCK_ADDR);
        let data = block.data();
        debug_assert_eq!(&data[0..8], SIGNATURE);
        Self {
            signature: SIGNATURE,
            version: data[8],
            block_size: 2 << (8 + data[9] as u32),
            block_count: u32::from_be_bytes(data[10..14].try_into().unwrap()),
            alloc_count: u32::from_be_bytes(data[14..18].try_into().unwrap()),
        }
    }

    pub fn write(&self) {
        let mut block = Block::new(super::SUPERBLOCK_ADDR);
        let data = block.data_mut();

        data[0..8].clone_from_slice(self.signature);
        data[8] = self.version;

        let size = self.block_size;
        debug_assert!(size >= 512);
        debug_assert!(size.is_power_of_two());
        data[9] = (size.trailing_zeros() as u8) - 9; // 2 ^ (9 + n)
        data[10..14].clone_from_slice(&self.block_count.to_be_bytes());
        data[14..18].clone_from_slice(&self.alloc_count.to_be_bytes());

        block.write();
    }
}

pub fn inc_alloc_count() {
    let mut sb = Superblock::read();
    sb.alloc_count += 1;
    sb.write();
}

pub fn dec_alloc_count() {
    let mut sb = Superblock::read();
    sb.alloc_count -= 1;
    sb.write();
}
