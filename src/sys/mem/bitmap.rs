use crate::sys::boot::{MemoryMap, MemoryRegionType};
use crate::sys::x86::addr::{align_up, PhysAddr, PhysFrame};

use core::{cmp, slice};
use spin::{Once, Mutex};
use bit_field::BitField;
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, Size4KiB};

#[derive(Debug, Clone, Copy, PartialEq)]
struct UsableRegion {
    first_frame: PhysFrame,
    frame_count: usize,
}

impl UsableRegion {
    // NOTE: end_addr is exclusive
    pub fn new(start_addr: usize, end_addr: usize) -> Self {
        let first_frame = frame_at(start_addr);
        let last_frame = frame_at(end_addr - 1);
        let frame_count = last_frame.number() - first_frame.number() + 1;

        Self {
            first_frame,
            frame_count
        }
    }

    pub fn first_frame(&self) -> PhysFrame {
        self.first_frame
    }

    pub fn last_frame(&self) -> PhysFrame {
        PhysFrame::from_number(self.first_frame.number() + self.frame_count - 1)
    }

    pub fn len(&self) -> usize {
        self.frame_count
    }

    pub fn contains(&self, frame: PhysFrame) -> bool {
        self.first_frame() <= frame && frame <= self.last_frame()
    }

    pub fn offset(&self, frame: PhysFrame) -> usize {
        frame.number() - self.first_frame.number()
    }
}

fn frame_at(addr: usize) -> PhysFrame {
    PhysFrame::containing_address(PhysAddr::new(addr))
}

static FRAME_ALLOCATOR: Once<Mutex<BitmapFrameAllocator>> = Once::new();

pub fn init_frame_allocator(memory_map: &MemoryMap) {
    FRAME_ALLOCATOR.call_once(|| {
        Mutex::new(BitmapFrameAllocator::init(memory_map))
    });
}

pub struct BitmapFrameAllocator {
    bitmap: &'static mut [u8],
    next_free_index: usize,
    usable_regions: [Option<UsableRegion>; MemoryMap::CAPACITY],
    regions_count: usize,
    frames_count: usize,
}

impl BitmapFrameAllocator {
    pub fn init(memory_map: &MemoryMap) -> Self {
        let mut bitmap_addr = None;

        // Compute an upper bound on the number of usable frames, knowing that
        // the bitmap will occupy some of them and the regions may be aligned
        // inward.
        let frames_count: usize = memory_map.iter().map(|region| {
            if region.kind == MemoryRegionType::Usable {
                let size = region.size;
                (size / 4096) as usize
            } else {
                0
            }
        }).sum();

        let bitmap_size = frames_count.div_ceil(8); // 8 frames per byte

        let mut allocator = Self {
            bitmap: &mut [],
            next_free_index: 0,
            usable_regions: [None; MemoryMap::CAPACITY],
            regions_count: 0,
            frames_count: 0,
        };

        for region in memory_map.iter() {
            if region.kind != MemoryRegionType::Usable {
                continue;
            }

            let region_start = region.aligned_start();
            let region_end = region.aligned_end();

            if region_end <= region_start {
                continue;
            }

            let region_size = region_end - region_start;

            // Try to place the bitmap in the region
            if bitmap_addr.is_none() && region_size >= bitmap_size {
                bitmap_addr = Some(region_start);

                // TODO: Check alignment
                let addr = super::phys_to_virt(PhysAddr::new(region_start));
                let ptr = addr.as_mut_ptr();
                let len = bitmap_size;
                unsafe {
                    allocator.bitmap = slice::from_raw_parts_mut(ptr, len);
                    allocator.bitmap.fill(0);
                }
            }

            // Calculate usable portion
            let (usable_start, usable_end) = match bitmap_addr {
                Some(addr) if region_start == addr => {
                    let bitmap_end = align_up(region_start + bitmap_size);
                    if bitmap_end >= region_end {
                        continue; // Entire region consumed by the bitmap
                    }
                    (bitmap_end, region_end)
                },
                _ => (region_start, region_end)
            };

            if usable_end - usable_start >= 4096 {
                if allocator.regions_count >= MemoryMap::CAPACITY {
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

    pub fn used_frames(&self) -> usize {
        self.bitmap.iter().map(|w| w.count_ones() as usize).sum()
    }

    fn index_to_frame(&self, index: usize) -> Option<PhysFrame> {
        if index >= self.frames_count {
            return None;
        }

        let mut base = 0;
        for i in 0..self.regions_count {
            if let Some(region) = self.usable_regions[i] {
                if index < base + region.len() {
                    let offset = index - base;
                    let number = region.first_frame().number() + offset;
                    return Some(PhysFrame::from_number(number));
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
                    let offset = region.offset(frame);
                    return Some(base + offset);
                }
                base += region.len();
            }
        }
        None
    }

    fn is_frame_allocated(&self, index: usize) -> bool {
        self.bitmap[index / 8].get_bit(index % 8)
    }

    fn set_frame_allocated(&mut self, index: usize, allocated: bool) {
        self.bitmap[index / 8].set_bit(index % 8, allocated);
    }

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

    unsafe fn deallocate_frame(&mut self, frame: PhysFrame) {
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

unsafe impl FrameAllocator<Size4KiB> for BitmapFrameAllocator {
    fn allocate_frame(&mut self) -> Option<x86_64::structures::paging::PhysFrame> {
        self.allocate_frame().map(|f| f.into())
    }
}

impl FrameDeallocator<Size4KiB> for BitmapFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: x86_64::structures::paging::PhysFrame<Size4KiB>) {
        self.deallocate_frame(frame.into())
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
