use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use spin::{Once, Mutex};
use x86_64::structures::paging::{
    FrameAllocator, FrameDeallocator,
    PhysFrame, Size4KiB
};
use x86_64::PhysAddr;

const MAX_FRAMES: usize = (4 << 30) / 4096; // 4 GB of RAM
const BITMAP_SIZE: usize = MAX_FRAMES / 64;
static FRAME_ALLOCATOR: Once<Mutex<BitmapFrameAllocator>> = Once::new();

pub fn init_frame_allocator() {
    FRAME_ALLOCATOR.call_once(|| {
        Mutex::new(unsafe {
            BitmapFrameAllocator::init(super::MEMORY_MAP.get_unchecked())
        })
    });
}

pub struct BitmapFrameAllocator {
    memory_map: &'static MemoryMap,
    allocated_bitmap: [u64; BITMAP_SIZE],
}

impl BitmapFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        Self {
            memory_map,
            allocated_bitmap: [0; BITMAP_SIZE],
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r|
            r.region_type == MemoryRegionType::Usable
        );
        let addr_ranges = usable_regions.map(|r|
            r.range.start_addr()..r.range.end_addr()
        );
        let frame_addresses = addr_ranges.flat_map(|r|
            r.step_by(4096)
        );
        frame_addresses.map(|addr|
            PhysFrame::containing_address(PhysAddr::new(addr))
        )
    }

    fn frame_to_bitmap_index(&self, frame: PhysFrame) -> Option<usize> {
        for (i, usable_frame) in self.usable_frames().enumerate() {
            if usable_frame.start_address() == frame.start_address() {
                return if i < MAX_FRAMES { Some(i) } else { None };
            }
            if i >= MAX_FRAMES {
                break;
            }
        }

        None
    }

    fn is_frame_allocated(&self, index: usize) -> bool {
        if index >= MAX_FRAMES {
            return false;
        }
        let word_index = index / 64;
        let bit_index = index % 64;
        (self.allocated_bitmap[word_index] & (1 << bit_index)) != 0
    }

    fn set_frame_allocated(&mut self, index: usize, allocated: bool) {
        if index >= MAX_FRAMES {
            return;
        }
        let word_index = index / 64;
        let bit_index = index % 64;
        if allocated {
            self.allocated_bitmap[word_index] |= 1 << bit_index;
        } else {
            self.allocated_bitmap[word_index] &= !(1 << bit_index);
        }
    }

    pub fn total_usable_frames(&self) -> usize {
        self.usable_frames().take(MAX_FRAMES).count()
    }
}

unsafe impl FrameAllocator<Size4KiB> for BitmapFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        for (i, frame) in self.usable_frames().enumerate() {
            if i >= MAX_FRAMES {
                break;
            }
            if !self.is_frame_allocated(i) {
                self.set_frame_allocated(i, true);
                return Some(frame);
            }
        }
        None
    }
}

impl FrameDeallocator<Size4KiB> for BitmapFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        if let Some(index) = self.frame_to_bitmap_index(frame) {
            if self.is_frame_allocated(index) {
                self.set_frame_allocated(index, false);
            }
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
