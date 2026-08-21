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
