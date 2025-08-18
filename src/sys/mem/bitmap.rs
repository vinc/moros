use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use spin::{Once, Mutex};
use x86_64::structures::paging::{
    FrameAllocator, FrameDeallocator,
    PhysFrame, Size4KiB
};
use x86_64::PhysAddr;

#[derive(Debug, Clone, Copy)]
struct UsableRegion {
    base: PhysFrame,
    frame_count: usize,
}

impl UsableRegion {
    pub fn start_frame(&self) -> PhysFrame {
        self.base
    }

    pub fn end_frame(&self) -> PhysFrame {
        self.base + (self.frame_count - 1) as u64
    }

    pub fn contains(&self, frame: PhysFrame) -> bool {
        self.start_frame() <= frame && frame <= self.end_frame()
    }

    pub fn offset(&self, frame: PhysFrame) -> u64 {
        (frame.start_address() - self.base.start_address()) / 4096
    }
}

fn frame_at(addr: u64) -> PhysFrame<Size4KiB> {
    PhysFrame::containing_address(PhysAddr::new(addr))
}

const MAX_REGIONS: usize = 32;
const MAX_FRAMES: usize = (4 << 30) / 4096; // 4 GB of RAM
const BITMAP_SIZE: usize = MAX_FRAMES / 64;
static FRAME_ALLOCATOR: Once<Mutex<BitmapFrameAllocator>> = Once::new();

pub fn init_frame_allocator(memory_map: &'static MemoryMap) {
    FRAME_ALLOCATOR.call_once(|| {
        Mutex::new(BitmapFrameAllocator::init(memory_map))
    });
}

pub struct BitmapFrameAllocator {
    usable_regions: [Option<UsableRegion>; MAX_REGIONS],
    regions_count: usize,
    allocated_bitmap: [u64; BITMAP_SIZE],
    next_free_index: usize,
    frames_count: usize,
}

impl BitmapFrameAllocator {
    pub fn init(memory_map: &'static MemoryMap) -> Self {
        let mut allocator = Self {
            usable_regions: [None; MAX_REGIONS],
            regions_count: 0,
            allocated_bitmap: [0; BITMAP_SIZE],
            next_free_index: 0,
            frames_count: 0,
        };

        let regions = memory_map.iter();
        let usable_regions = regions.filter(|r| {
            r.region_type == MemoryRegionType::Usable
        });
        for region in usable_regions {
            let start_frame = frame_at(region.range.start_addr());
            let end_frame = frame_at(region.range.end_addr() - 1);
            let size = end_frame.start_address() - start_frame.start_address();
            let frame_count = 1 + (size / 4096) as usize;

            debug_assert!(allocator.regions_count < MAX_REGIONS);
            let desc = UsableRegion { base: start_frame, frame_count };
            allocator.usable_regions[allocator.regions_count] = Some(desc);
            allocator.regions_count += 1;
            allocator.frames_count += frame_count;
        }
        allocator.frames_count = allocator.frames_count.min(MAX_FRAMES);
        allocator
    }

    fn index_to_frame(&self, index: usize) -> Option<PhysFrame> {
        if index >= self.frames_count {
            return None;
        }

        let mut current_base_index = 0;
        for i in 0..self.regions_count {
            if let Some(region) = self.usable_regions[i] {
                if index < current_base_index + region.frame_count {
                    let frame_offset = index - current_base_index;
                    return Some(region.base + frame_offset as u64);
                }
                current_base_index += region.frame_count;
            }
        }
        None
    }

    fn frame_to_index(&self, frame: PhysFrame) -> Option<usize> {
        let mut current_base_index = 0;
        for i in 0..self.regions_count {
            if let Some(region) = self.usable_regions[i] {
                if region.contains(frame) {
                    let frame_offset = region.offset(frame);
                    return Some(current_base_index + frame_offset as usize);
                }
                current_base_index += region.frame_count;
            }
        }
        None
    }

    fn is_frame_allocated(&self, index: usize) -> bool {
        let word_index = index / 64;
        let bit_index = index % 64;
        (self.allocated_bitmap[word_index] & (1 << bit_index)) != 0
    }

    fn set_frame_allocated(&mut self, index: usize, allocated: bool) {
        let word_index = index / 64;
        let bit_index = index % 64;
        if allocated {
            self.allocated_bitmap[word_index] |= 1 << bit_index;
        } else {
            self.allocated_bitmap[word_index] &= !(1 << bit_index);
        }
    }

    pub fn total_usable_frames(&self) -> usize {
        self.frames_count
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
                self.next_free_index = self.next_free_index.min(index);
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
    f(&mut *allocator)
}
