#[cfg(all(feature = "limine", feature = "multiboot"))]
compile_error!("features limine and multiboot are mutually exclusive");

#[cfg(all(target_arch = "x86", not(feature = "multiboot")))]
compile_error!("target i686 requires feature multiboot");

#[cfg(all(target_arch = "x86_64", feature = "multiboot"))]
compile_error!("feature multiboot requires target i686");

#[cfg(not(any(feature = "limine", feature = "multiboot")))]
pub mod bootloader;

#[cfg(feature = "limine")]
pub mod limine;

#[cfg(feature = "multiboot")]
pub mod multiboot;

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
    regions: [MemoryRegion; Self::CAPACITY],
    len: usize,
}

impl MemoryMap {
    pub const CAPACITY: usize = 32;

    pub fn new() -> Self {
        let empty = MemoryRegion::new(0, 0, MemoryRegionType::Reserved);
        Self {
            regions: [empty; Self::CAPACITY],
            len: 0,
        }
    }

    pub fn add(&mut self, region: MemoryRegion) {
        if self.len < Self::CAPACITY {
            self.regions[self.len] = region;
            self.len += 1;
        }
    }

    pub fn as_slice(&self) -> &[MemoryRegion] {
        &self.regions[..self.len]
    }

    pub fn iter(&self) -> core::slice::Iter<'_, MemoryRegion> {
        self.as_slice().iter()
    }
}
