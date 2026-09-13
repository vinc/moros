use super::addr::PhysAddr;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Frame(PhysAddr);

impl Frame {
    pub fn containing_address(addr: PhysAddr) -> Self {
        Self::from_start_address(addr - addr.page_offset()) // Align down
    }

    pub fn from_start_address(addr: PhysAddr) -> Self {
        debug_assert_eq!(addr.page_offset(), 0);
        Self(addr)
    }

    pub fn start_address(self) -> PhysAddr {
        self.0
    }

    pub fn from_number(number: usize) -> Self {
        Self::from_start_address(PhysAddr::new(number << 12))
    }

    pub fn number(self) -> usize {
        self.0.as_usize() >> 12
    }
}

impl From<Frame> for x86_64::structures::paging::PhysFrame {
    fn from(frame: Frame) -> Self {
        Self::from_start_address(frame.start_address().into()).unwrap()
    }
}

impl From<x86_64::structures::paging::PhysFrame> for Frame {
    fn from(frame: x86_64::structures::paging::PhysFrame) -> Self {
        Self::from_start_address(frame.start_address().into())
    }
}

#[test_case]
fn test_frame() {
    let values = [
        (0x0000, 0x0000),
        (0x1000, 0x1000),
        (0xA000, 0xA000),
        (0xAFFF, 0xA000),
    ];
    for (addr1, addr2) in values {
        assert_eq!(
            Frame::containing_address(PhysAddr::new(addr1)).start_address(),
            PhysAddr::new(addr2)
        );
    }

    for i in 0..10 {
        assert_eq!(
            Frame::from_number(i).start_address(),
            PhysAddr::new(i * super::PAGE_SIZE)
        );
    }
}
