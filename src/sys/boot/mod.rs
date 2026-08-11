#[cfg(not(any(feature = "limine", feature = "multiboot")))]
pub mod bootloader;

#[cfg(feature = "limine")]
pub mod limine;

#[cfg(feature = "multiboot")]
pub mod multiboot;

pub const MAX_REGIONS: usize = 32;

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MemoryRegionType {
    Usable,
    Reserved,
    AcpiUsable,
    AcpiReserved,
    Defective,
    Custom(u32),
}

#[derive(Copy, Clone, Debug)]
pub struct MemoryRegion {
    pub addr: u64,
    pub size: u64,
    pub kind: MemoryRegionType,
}

impl MemoryRegion {
    pub fn new(addr: u64, size: u64, kind: MemoryRegionType) -> Self {
        Self { addr, size, kind }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MemoryMap {
    regions: [MemoryRegion; MAX_REGIONS],
    len: usize,
}

impl MemoryMap {
    pub fn new() -> Self {
        let empty = MemoryRegion::new(0, 0, MemoryRegionType::Reserved);
        Self {
            regions: [empty; MAX_REGIONS],
            len: 0,
        }
    }

    pub fn add(&mut self, region: MemoryRegion) {
        self.regions[self.len] = region;
        self.len += 1;
    }

    pub fn as_slice(&self) -> &[MemoryRegion] {
        &self.regions[..self.len]
    }

    pub fn iter(&self) -> core::slice::Iter<'_, MemoryRegion> {
        self.as_slice().iter()
    }
}
