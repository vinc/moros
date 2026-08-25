// System segment type (Intel SDM 3.5)
pub const TYPE_TSS: u8 = 0b1001; // 32-bit or 64-bit TSS (Available)
pub const TYPE_IG:  u8 = 0b1110; // 32-bit or 64-bit Interrupt Gate

#[derive(Clone, Copy, Default)]
#[repr(transparent)]
pub struct SegmentSelector {
    pub bits: u16,
}

impl SegmentSelector {
    pub const fn new(index: usize, ring: usize) -> Self {
        Self { bits: (index << 3 | ring) as u16 }
    }

    pub const fn index(&self) -> usize {
        (self.bits >> 3) as usize
    }
}
