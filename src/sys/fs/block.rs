use super::block_bitmap::BlockBitmap;

use bit_field::BitField;

#[derive(Clone)]
pub struct Block {
    addr: u32,
    buf: [u8; 512],
}

// Block structure:
// 0..4 => next block address
// 4..512 => block data
impl Block {
    pub fn new(addr: u32) -> Self {
        let buf = [0; 512];
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
                for i in 0..512 {
                    block.buf[i] = 0;
                }
                block.write();

                Some(block)
            }
        }
    }

    pub fn read(addr: u32) -> Self {
        let mut buf = [0; 512];
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
        &self.buf[4..512]
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.buf[4..512]
    }

    // TODO: Return addr instead of block?
    pub fn next(&self) -> Option<Self> {
        let addr = (self.buf[0] as u32) << 24
                 | (self.buf[1] as u32) << 16
                 | (self.buf[2] as u32) << 8
                 | (self.buf[3] as u32);

        if addr == 0 {
            None
        } else {
            Some(Self::read(addr))
        }
    }

    // FIXME: next() returns a Block, but set_next() takes a u32
    pub fn set_next(&mut self, addr: u32) {
        self.buf[0] = addr.get_bits(24..32) as u8;
        self.buf[1] = addr.get_bits(16..24) as u8;
        self.buf[2] = addr.get_bits(8..16) as u8;
        self.buf[3] = addr.get_bits(0..8) as u8;
    }
}
