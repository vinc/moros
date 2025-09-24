use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use core::{cmp, slice};
use spin::{Once, Mutex};
use bit_field::BitField;
use x86_64::structures::paging::{
    FrameAllocator, FrameDeallocator,
    PhysFrame, Size4KiB
};
use x86_64::PhysAddr;

#[derive(Debug, Clone, Copy, PartialEq)]
struct UsableRegion {
    first_frame: PhysFrame,
    frame_count: usize,
}

impl UsableRegion {
    // NOTE: end_addr is exclusive
    pub fn new(start_addr: u64, end_addr: u64) -> Self {
        let first_frame = frame_at(start_addr);
        let last_frame = frame_at(end_addr - 1);
        let a = first_frame.start_address();
        let b = last_frame.start_address();
        let frame_count = ((b - a) / 4096) as usize + 1;

        Self {
            first_frame,
            frame_count
        }
    }

    pub fn first_frame(&self) -> PhysFrame {
        self.first_frame
    }

    pub fn last_frame(&self) -> PhysFrame {
        self.first_frame + (self.frame_count - 1) as u64
    }

    pub fn len(&self) -> usize {
        self.frame_count
    }

    pub fn contains(&self, frame: PhysFrame) -> bool {
        self.first_frame() <= frame && frame <= self.last_frame()
    }

    pub fn offset(&self, frame: PhysFrame) -> usize {
        let addr = frame.start_address() - self.first_frame.start_address();
        (addr / 4096) as usize
    }
}

fn frame_at(addr: u64) -> PhysFrame<Size4KiB> {
    PhysFrame::containing_address(PhysAddr::new(addr))
}

static FRAME_ALLOCATOR: Once<Mutex<BitmapFrameAllocator>> = Once::new();

pub fn init_frame_allocator(memory_map: &'static MemoryMap) {
    FRAME_ALLOCATOR.call_once(|| {
        Mutex::new(BitmapFrameAllocator::init(memory_map))
    });
}

const MAX_REGIONS: usize = 32;

pub struct BitmapFrameAllocator {
    bitmap: &'static mut [u64],
    next_free_index: usize,
    usable_regions: [Option<UsableRegion>; MAX_REGIONS],
    regions_count: usize,
    frames_count: usize,
}

impl BitmapFrameAllocator {
    pub fn init(memory_map: &'static MemoryMap) -> Self {
        let mut bitmap_addr = None;

        let frames_count: usize = memory_map.iter().map(|region| {
            if region.region_type == MemoryRegionType::Usable {
                let size = region.range.end_addr() - region.range.start_addr();
                debug_assert_eq!(size % 4096, 0);
                (size / 4096) as usize
            } else {
                0
            }
        }).sum();
        let bitmap_size = ((frames_count + 63) / 64) * 8;

        let mut allocator = Self {
            bitmap: &mut [],
            next_free_index: 0,
            usable_regions: [None; MAX_REGIONS],
            regions_count: 0,
            frames_count: 0,
        };

        for region in memory_map.iter() {
            if region.region_type != MemoryRegionType::Usable {
                continue;
            }

            let region_start = region.range.start_addr();
            let region_end = region.range.end_addr();
            let region_size = (region_end - region_start) as usize;

            // Try to place the bitmap in the region
            if bitmap_addr.is_none() && region_size >= bitmap_size {
                bitmap_addr = Some(region_start);

                // TODO: Check alignment
                let addr = super::phys_to_virt(PhysAddr::new(region_start));
                let ptr = addr.as_mut_ptr();
                let len = bitmap_size / 8;
                unsafe {
                    allocator.bitmap = slice::from_raw_parts_mut(ptr, len);
                    allocator.bitmap.fill(0);
                }
            }

            // Calculate usable portion
            let (usable_start, usable_end) = match bitmap_addr {
                Some(addr) if region_start == addr => {
                    let bitmap_end = region_start + bitmap_size as u64;
                    if bitmap_end >= region_end {
                        continue; // Entire region consumed by the bitmap
                    }
                    (bitmap_end, region_end)
                },
                _ => (region_start, region_end)
            };

            if usable_end - usable_start >= 4096 {
                if allocator.regions_count >= MAX_REGIONS {
                    debug!("MEM: Could not add usable region");
                    break;
                }
                let r = UsableRegion::new(usable_start, usable_end);
                allocator.usable_regions[allocator.regions_count] = Some(r);
                allocator.regions_count += 1;
                allocator.frames_count += r.len();
            }
        }

        if bitmap_addr.is_none() {
            panic!("MEM: No usable region large enough to host bitmap");
        }

        allocator
    }

    fn index_to_frame(&self, index: usize) -> Option<PhysFrame> {
        if index >= self.frames_count {
            return None;
        }

        let mut base = 0;
        for i in 0..self.regions_count {
            if let Some(region) = self.usable_regions[i] {
                if index < base + region.len() {
                    let frame_offset = index - base;
                    return Some(region.first_frame() + frame_offset as u64);
                }
                base += region.len();
            }
        }
        None
    }

    fn frame_to_index(&self, frame: PhysFrame) -> Option<usize> {
        let mut base = 0;
        for i in 0..self.regions_count {
            if let Some(region) = self.usable_regions[i] {
                if region.contains(frame) {
                    let frame_offset = region.offset(frame);
                    return Some(base + frame_offset);
                }
                base += region.len();
            }
        }
        None
    }

    fn is_frame_allocated(&self, index: usize) -> bool {
        let word_index = index / 64;
        let bit_index = index % 64;
        self.bitmap[word_index].get_bit(bit_index)
    }

    fn set_frame_allocated(&mut self, index: usize, allocated: bool) {
        let word_index = index / 64;
        let bit_index = index % 64;
        self.bitmap[word_index].set_bit(bit_index, allocated);
    }
}

unsafe impl FrameAllocator<Size4KiB> for BitmapFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        for i in 0..self.frames_count {
            let index = (self.next_free_index + i) % self.frames_count;
            if !self.is_frame_allocated(index) {
                self.set_frame_allocated(index, true);
                self.next_free_index = index + 1;
                return self.index_to_frame(index);
            }
        }
        None // No free frames
    }
}

impl FrameDeallocator<Size4KiB> for BitmapFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        if let Some(index) = self.frame_to_index(frame) {
            if self.is_frame_allocated(index) {
                self.set_frame_allocated(index, false);
                self.next_free_index = cmp::min(self.next_free_index, index);
            } else {
                //panic!("Double free detected");
            }
        } else {
            //panic!("Deallocating a frame not managed by the allocator");
        }
    }
}

pub fn frame_allocator() -> &'static Mutex<BitmapFrameAllocator> {
    FRAME_ALLOCATOR.get().expect("frame allocator not initialized")
}

pub fn with_frame_allocator<F, R>(f: F) -> R
where
    F: FnOnce(&mut BitmapFrameAllocator) -> R,
{
    let mut allocator = frame_allocator().lock();
    f(&mut allocator)
}

#[test_case]
fn test_usable_region() {
    let region = UsableRegion {
        first_frame: frame_at(4096),
        frame_count: 10,
    };

    assert_eq!(region, UsableRegion::new(4096, 4096 * 11));

    assert_eq!(region.len(), 10);

    assert_eq!(region.first_frame(), frame_at(4096));
    assert_eq!(region.last_frame(), frame_at(4096 * 10));

    assert!(!region.contains(frame_at(0)));
    assert!(region.contains(frame_at(4096)));
    assert!(region.contains(frame_at(4096 * 3)));
    assert!(region.contains(frame_at(4096 * 10)));
    assert!(!region.contains(frame_at(4096 * 11)));
}
