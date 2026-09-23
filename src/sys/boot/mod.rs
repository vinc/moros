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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MemoryRegionType {
    Usable,
    Reserved,
    AcpiUsable,
    AcpiReserved,
    Defective,
    Bootloader,
    Kernel,
    Unaddressable,
    Unknown(u32),
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

    pub fn is_usable(&self) -> bool {
        self.kind == MemoryRegionType::Usable
    }

    pub fn is_addressable(&self) -> bool {
        self.kind != MemoryRegionType::Unaddressable
    }

    pub fn aligned_start(&self) -> usize {
        debug_assert!(self.is_usable()); // Unaddressable would overflow
        crate::sys::x86::addr::align_up(self.addr as usize)
    }

    pub fn aligned_end(&self) -> usize {
        debug_assert!(self.is_usable()); // Unaddressable would overflow
        crate::sys::x86::addr::align_down((self.addr + self.size) as usize)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MemoryMap {
    regions: [MemoryRegion; Self::CAPACITY],
    len: usize,
}

impl MemoryMap {
    pub const CAPACITY: usize = 64;

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

    pub fn insert(&mut self, i: usize, region: MemoryRegion) {
        let mut j = self.len;
        while j > i {
            self.regions[j] = self.regions[j - 1];
            j -= 1;
        }
        self.regions[i] = region;
        self.len += 1;
    }

    pub fn remove(&mut self, mut i: usize) {
        while i + 1 < self.len {
            self.regions[i] = self.regions[i + 1];
            i += 1;
        }
        self.len -= 1;
    }

    pub fn reserve(&mut self, addr: u64, size: u64) {
        let end = addr + size;
        let mut i = 0;
        while i < self.len {
            let r = self.regions[i];
            let r_end = r.addr + r.size;
            if r_end <= addr || end <= r.addr {
                // Keep region outside of reservation
            } else if addr <= r.addr && r_end <= end {
                // Remove region inside of reservation
                self.remove(i);
                continue;
            } else if r.addr < addr && end < r_end {
                // Split region around reservation
                self.regions[i].size = addr - r.addr;
                self.insert(i + 1, MemoryRegion::new(end, r_end - end, r.kind));
            } else if r.addr < addr {
                // Clip end of region
                self.regions[i].size = addr - r.addr;
            } else {
                // Clip start of region
                self.regions[i].addr = end;
                self.regions[i].size = r_end - end;
            }
            i += 1;
        }
        let i = self.iter().position(|r| addr < r.addr).unwrap_or(self.len);
        let region = MemoryRegion::new(addr, size, MemoryRegionType::Reserved);
        self.insert(i, region);
    }

    pub fn iter(&self) -> core::slice::Iter<'_, MemoryRegion> {
        self.regions[..self.len].iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, MemoryRegion> {
        self.regions[..self.len].iter_mut()
    }
}

#[test_case]
fn test_memory_map_add() {
    let mut memory_map = MemoryMap::new();

    assert_eq!(memory_map.len, 0);
    memory_map.add(MemoryRegion::new(0, 1024, MemoryRegionType::Kernel));
    assert_eq!(memory_map.len, 1);
    memory_map.add(MemoryRegion::new(1024, 1024, MemoryRegionType::Usable));
    assert_eq!(memory_map.len, 2);
    memory_map.add(MemoryRegion::new(2048, 4096, MemoryRegionType::Usable));
    assert_eq!(memory_map.len, 3);

    assert_eq!(memory_map.regions[0].addr, 0);
    assert_eq!(memory_map.regions[1].addr, 1024);
    assert_eq!(memory_map.regions[2].addr, 2048);

    assert_eq!(memory_map.regions[0].size, 1024);
    assert_eq!(memory_map.regions[1].size, 1024);
    assert_eq!(memory_map.regions[2].size, 4096);
}

#[test_case]
fn test_memory_map_insert() {
    let mut memory_map = MemoryMap::new();

    assert_eq!(memory_map.len, 0);
    memory_map.add(MemoryRegion::new(1024, 1024, MemoryRegionType::Usable));
    assert_eq!(memory_map.len, 1);
    memory_map.add(MemoryRegion::new(2048, 4096, MemoryRegionType::Usable));
    assert_eq!(memory_map.len, 2);
    memory_map.insert(0, MemoryRegion::new(0, 1024, MemoryRegionType::Kernel));
    assert_eq!(memory_map.len, 3);

    assert_eq!(memory_map.regions[0].addr, 0);
    assert_eq!(memory_map.regions[1].addr, 1024);
    assert_eq!(memory_map.regions[2].addr, 2048);

    assert_eq!(memory_map.regions[0].size, 1024);
    assert_eq!(memory_map.regions[1].size, 1024);
    assert_eq!(memory_map.regions[2].size, 4096);
}

#[test_case]
fn test_memory_map_remove() {
    let mut memory_map = MemoryMap::new();

    assert_eq!(memory_map.len, 0);
    memory_map.add(MemoryRegion::new(0, 1024, MemoryRegionType::Kernel));
    assert_eq!(memory_map.len, 1);
    memory_map.add(MemoryRegion::new(1024, 1024, MemoryRegionType::Usable));
    assert_eq!(memory_map.len, 2);
    memory_map.add(MemoryRegion::new(2048, 4096, MemoryRegionType::Usable));
    assert_eq!(memory_map.len, 3);
    memory_map.remove(2);
    assert_eq!(memory_map.len, 2);
    memory_map.remove(0);
    assert_eq!(memory_map.len, 1);

    assert_eq!(memory_map.regions[0].addr, 1024);
    assert_eq!(memory_map.regions[0].size, 1024);
}

#[test_case]
fn test_memory_map_reserve_span() {
    let mut memory_map = MemoryMap::new();

    memory_map.add(MemoryRegion::new(0, 1024, MemoryRegionType::Kernel));
    memory_map.add(MemoryRegion::new(1024, 1024, MemoryRegionType::Usable));
    memory_map.add(MemoryRegion::new(2048, 4096, MemoryRegionType::Usable));

    assert_eq!(memory_map.len, 3);
    memory_map.reserve(512, 2048);
    assert_eq!(memory_map.len, 3);

    assert_eq!(memory_map.regions[0].addr, 0);
    assert_eq!(memory_map.regions[1].addr, 512);
    assert_eq!(memory_map.regions[2].addr, 2048 + 512);

    assert_eq!(memory_map.regions[0].size, 512);
    assert_eq!(memory_map.regions[1].size, 2048);
    assert_eq!(memory_map.regions[2].size, 4096 - 512);

    assert_eq!(memory_map.regions[0].kind, MemoryRegionType::Kernel);
    assert_eq!(memory_map.regions[1].kind, MemoryRegionType::Reserved);
    assert_eq!(memory_map.regions[2].kind, MemoryRegionType::Usable);
}

#[test_case]
fn test_memory_map_reserve_split() {
    let mut memory_map = MemoryMap::new();

    memory_map.add(MemoryRegion::new(0, 1024, MemoryRegionType::Kernel));
    memory_map.add(MemoryRegion::new(1024, 4096, MemoryRegionType::Usable));
    memory_map.add(MemoryRegion::new(8192, 1024, MemoryRegionType::Reserved));

    assert_eq!(memory_map.len, 3);
    memory_map.reserve(2048, 1024);
    assert_eq!(memory_map.len, 5);

    assert_eq!(memory_map.regions[0].addr, 0);
    assert_eq!(memory_map.regions[1].addr, 1024);
    assert_eq!(memory_map.regions[2].addr, 2048);
    assert_eq!(memory_map.regions[3].addr, 3072);
    assert_eq!(memory_map.regions[4].addr, 8192);

    assert_eq!(memory_map.regions[0].size, 1024);
    assert_eq!(memory_map.regions[1].size, 1024);
    assert_eq!(memory_map.regions[2].size, 1024);
    assert_eq!(memory_map.regions[3].size, 2048);
    assert_eq!(memory_map.regions[4].size, 1024);

    assert_eq!(memory_map.regions[0].kind, MemoryRegionType::Kernel);
    assert_eq!(memory_map.regions[1].kind, MemoryRegionType::Usable);
    assert_eq!(memory_map.regions[2].kind, MemoryRegionType::Reserved);
    assert_eq!(memory_map.regions[3].kind, MemoryRegionType::Usable);
    assert_eq!(memory_map.regions[4].kind, MemoryRegionType::Reserved);
}
