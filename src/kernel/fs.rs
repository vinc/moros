use bit_field::BitField;
use crate::kernel;
use heapless::{String, Vec};
use heapless::consts::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Dir = 0,
    File = 1,
}

#[derive(Debug, Clone, Copy)]
pub struct File {
    addr: u32
}

pub fn dirname(pathname: &str) -> &str {
    let n = pathname.len();
    let i = match pathname.rfind('/') {
        Some(0) => 1,
        Some(i) => i,
        None => n,
    };
    &pathname[0..i] 
}

pub fn filename(pathname: &str) -> &str {
    let n = pathname.len();
    let i = match pathname.rfind('/') {
        Some(i) => i + 1,
        None => 0,
    };
    &pathname[i..n] 
}

impl File {
    pub fn create(pathname: &str) -> Option<Self> {
        let dirname = dirname(pathname);
        let filename = filename(pathname);
        if let Some(dir) = Dir::open(dirname) {
            if let Some(block) = dir.create_file(filename) {
                return Some(Self { addr: block.addr });
            }
        }
        None
    }

    pub fn open(pathname: &str) -> Option<Self> {
        let dirname = dirname(pathname);
        let filename = filename(pathname);
        if let Some(dir) = Dir::open(dirname) {
            if let Some(dir_entry) = dir.find(filename) {
                return Some(Self { addr: dir_entry.addr });
            }
        }
        None
    }

    pub fn read(&self, buf: &mut [u8]) -> usize {
        let buf_len = buf.len();
        let mut addr = self.addr;
        let mut i = 0;
        loop {
            let block = Block::read(addr);
            let data = block.data();
            let data_len = data.len();
            for j in 0..data_len {
                if i == buf_len {
                    return i;
                }
                if data[j] == 0 { // TODO: Use filesize instead
                    return i;
                }
                buf[i] = data[j];
                i += 1;
            }
            match block.next() {
                Some(next_block) => addr = next_block.addr(),
                None => return i,
            }
        }
    }

    pub fn read_to_string(&self) -> String<U2048> {
        // TODO: We could use [0; 2048] with real String instead of heapless
        let mut buf = Vec::<u8, U2048>::new();
        buf.resize(2048, 0).unwrap();
        let bytes = self.read(&mut buf);
        buf.resize(bytes, 0).unwrap();
        String::from_utf8(buf).unwrap()
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<(), ()> {
        let buf_len = buf.len();
        let mut addr = self.addr;
        let mut i = 0;
        while i < buf_len {
            let mut block = Block::new(addr);
            let data = block.data_mut();
            let data_len = data.len();
            for j in 0..data_len {
                if i == buf_len {
                    break;
                }
                data[j] = buf[i];
                i += 1;
            }

            addr = match block.next() {
                Some(next_block) => {
                    if i < buf_len {
                        next_block.addr()
                    } else {
                        // TODO: Free the next block(s)
                        0
                    }
                }
                None => {
                    if i < buf_len {
                        match Block::alloc() {
                            Some(next_block) => next_block.addr(),
                            None => return Err(()),
                        }
                    } else {
                        0
                    }
                }
            };

            block.set_next(addr);
            block.write();
        }
        Ok(())
    }

    pub fn addr(&self) -> u32 {
        self.addr
    }
}

#[derive(Clone)]
pub struct Block {
    addr: u32,
    buf: [u8; 512]
}

// Block structure:
// 0..4 => next block address
// 4..512 => block data
impl Block {
    pub fn new(addr: u32) -> Self {
        let buf = [0; 512];
        Self { addr, buf }
    }

    pub fn read(addr: u32) -> Self {
        let bus = 1; // TODO
        let dsk = 0; // TODO
        let mut buf = [0; 512];
        kernel::ata::read(bus, dsk, addr, &mut buf);
        Self { addr, buf }
    }

    pub fn write(&self) {
        let bus = 1; // TODO
        let dsk = 0; // TODO
        kernel::ata::write(bus, dsk, self.addr, &self.buf);
    }

    pub fn alloc() -> Option<Self> {
        match BlockBitmap::next_free_addr() {
            None => {
                return None;
            },
            Some(addr) => {
                BlockBitmap::alloc(addr);

                // Initialize block
                let mut block = Block::read(addr);
                for i in 0..512 {
                    block.buf[i] = 0;
                }
                block.write();

                Some(block)
            }
        }
    }

    pub fn addr(&self) -> u32 {
        self.addr
    }

    pub fn data(&self) -> &[u8] {
        &self.buf[4..512]
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.buf[4..512]
    }

    pub fn next(&self) -> Option<Self> {
        let addr = (self.buf[0] as u32) << 24
                 | (self.buf[1] as u32) << 16
                 | (self.buf[2] as u32) << 8
                 | (self.buf[3] as u32);

        if addr == 0 {
            None
        } else {
            Some(Self::read(addr))
        }
    }

    // FIXME: next() returns a Block, but set_next() takes a u32
    pub fn set_next(&mut self, addr: u32) {
        self.buf[0] = addr.get_bits(24..32) as u8;
        self.buf[1] = addr.get_bits(16..24) as u8;
        self.buf[2] = addr.get_bits(8..16) as u8;
        self.buf[3] = addr.get_bits(0..8) as u8;
    }
}

const BITMAP_SIZE: u32 = 512 - 4; // TODO: Bitmap should use the full block
const MAX_BLOCKS: u32 = 2 * 2048;

const BITMAP_ADDR_OFFSET: u32 = 2048 + 2;
const DATA_ADDR_OFFSET: u32 = BITMAP_ADDR_OFFSET + MAX_BLOCKS;

/* Disk Areas
 * 1 => Reserved
 * 2 => Bitmap (allocated blocks (1 bit per block)
 * 3 => Data (directories and files)
 */

pub struct BlockBitmap {}

impl BlockBitmap {
    pub fn is_free(addr: u32) -> bool {
        let block = Block::read(BITMAP_ADDR_OFFSET + ((addr - DATA_ADDR_OFFSET) / BITMAP_SIZE));
        let bitmap = block.data(); // TODO: Add block.buffer()
        bitmap[((addr - DATA_ADDR_OFFSET) % BITMAP_SIZE) as usize] == 0
    }

    pub fn alloc(addr: u32) {
        let mut block = Block::read(BITMAP_ADDR_OFFSET + ((addr - DATA_ADDR_OFFSET) / BITMAP_SIZE));
        let bitmap = block.data_mut();
        bitmap[((addr - DATA_ADDR_OFFSET) % BITMAP_SIZE) as usize] = 1;
        block.write();
    }

    pub fn free(addr: u32) {
        let mut block = Block::read(BITMAP_ADDR_OFFSET + ((addr - DATA_ADDR_OFFSET) / BITMAP_SIZE));
        let bitmap = block.data_mut();
        bitmap[((addr - DATA_ADDR_OFFSET) % BITMAP_SIZE) as usize] = 0;
        block.write();
    }

    pub fn next_free_addr() -> Option<u32> {
        let n = MAX_BLOCKS / BITMAP_SIZE;
        for i in 0..n {
            let block = Block::read(BITMAP_ADDR_OFFSET + i);
            let bitmap = block.data();
            for j in 0..BITMAP_SIZE {
                if bitmap[j as usize] == 0 {
                    let addr = DATA_ADDR_OFFSET + i * 512 + j;
                    return Some(addr);
                }
            }
        }
        None
    }
}

pub struct DirEntry {
    kind: FileType,
    addr: u32,
    size: u32,
    name: String<U256>,
}

impl DirEntry {
    pub fn new(kind: FileType, addr: u32, size: u32, name: &str) -> Self {
        let name = String::from(name);
        Self { kind, addr, size, name }
    }

    pub fn is_dir(&self) -> bool {
        self.kind == FileType::Dir
    }

    pub fn is_file(&self) -> bool {
        self.kind == FileType::File
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn name(&self) -> String<U256> {
        self.name.clone()
    }

    pub fn to_dir(&self) -> Dir {
        Dir { addr: self.addr }
    }
}

pub struct Dir {
    addr: u32,
}

impl Dir {
    pub fn root() -> Self {
        Self { addr: DATA_ADDR_OFFSET }
    }

    pub fn open(path: &str) -> Option<Self> {
        let mut dir = Dir::root();
        if path == "/" {
            return Some(dir);
        }
        for name in path.trim_start_matches('/').split('/') {
            match dir.find(name) {
                Some(dir_entry) => {
                    if dir_entry.is_dir() {
                        dir = dir_entry.to_dir()
                    } else {
                        return None;
                    }
                },
                None => {
                    return None
                },
            }
        }
        Some(dir)
    }

    pub fn addr(&self) -> u32 {
        self.addr
    }

    pub fn find(&self, name: &str) -> Option<DirEntry> {
        for entry in self.read() {
            if entry.name == name {
                return Some(entry);
            }
        }
        None
    }

    pub fn create_file(&self, name: &str) -> Option<Block> {
        self.create(FileType::File, name)
    }

    pub fn create_dir(&self, name: &str) -> Option<Block> {
        self.create(FileType::Dir, name)
    }

    pub fn create(&self, kind: FileType, name: &str) -> Option<Block> {
        if self.find(name).is_some() {
            return None;
        }

        let mut read_dir = self.read();
        while read_dir.next().is_some() {}

        if read_dir.block.data().len() - read_dir.data_offset < name.len() + 10 {
            let new_block = Block::alloc().unwrap(); // TODO
            read_dir.block.set_next(new_block.addr);
            read_dir.block.write();
            read_dir.block = new_block;
            read_dir.data_offset = 0;
        }
        
        let new_block = Block::alloc().unwrap();

        let entry_kind = kind;
        let entry_size = 0;
        let entry_addr = new_block.addr();
        let entry_name = name.as_bytes();

        let n = entry_name.len();
        let i = read_dir.data_offset;
        let data = read_dir.block.data_mut();
        data[i + 0] = entry_kind as u8;
        data[i + 1] = entry_addr.get_bits(24..32) as u8;
        data[i + 2] = entry_addr.get_bits(16..24) as u8;
        data[i + 3] = entry_addr.get_bits(8..16) as u8;
        data[i + 4] = entry_addr.get_bits(0..8) as u8;
        data[i + 5] = entry_size.get_bits(24..32) as u8;
        data[i + 6] = entry_size.get_bits(16..24) as u8;
        data[i + 7] = entry_size.get_bits(8..16) as u8;
        data[i + 8] = entry_size.get_bits(0..8) as u8;
        data[i + 9] = n as u8;
        for j in 0..n {
            data[i + 10 + j] = entry_name[j];
        }
        read_dir.block.write();

        Some(new_block)
    }

    pub fn read(&self) -> ReadDir {
        ReadDir {
            block: Block::read(self.addr),
            data_offset: 0,
        }
    }
}

pub struct ReadDir {
    block: Block,
    data_offset: usize,
}

impl Iterator for ReadDir {
    type Item = DirEntry;
    
    fn next(&mut self) -> Option<DirEntry> {
        loop {
            let data = self.block.data();
            let mut i = self.data_offset;

            loop {
                if i == data.len() - 10 { // No space left for another entry in the block
                    break;
                }

                let entry_kind = match data[i + 0] {
                    0 => FileType::Dir,
                    1 => FileType::File,
                    _ => break,
                };
                let entry_addr = (data[i + 1] as u32) << 24
                               | (data[i + 2] as u32) << 16
                               | (data[i + 3] as u32) << 8
                               | (data[i + 4] as u32);
                let entry_size = (data[i + 5] as u32) << 24
                               | (data[i + 6] as u32) << 16
                               | (data[i + 7] as u32) << 8
                               | (data[i + 8] as u32);
                if entry_addr == 0 {
                    break;
                }
                i += 9;

                let mut n = data[i];
                if n == 0 || n as usize >= data.len() - i {
                    break;
                }
                i += 1;

                // The rest of the entry is the pathname string.
                let mut entry_name = String::<U256>::new();
                loop {
                    if n == 0 {
                        break;
                    }
                    entry_name.push(data[i] as char).expect("Name too long");
                    n -= 1;
                    i += 1;
                }

                self.data_offset = i;
                return Some(DirEntry::new(entry_kind, entry_addr, entry_size, &entry_name));
            }

            match self.block.next() {
                Some(next_block) => {
                    self.block = next_block;
                    self.data_offset = 0;
                }
                None => break,
            }
        }

        None
    }
}

pub fn init() {
    let root = Dir::root();

    // Allocate root dir on new filesystems
    if BlockBitmap::is_free(root.addr()) {
        BlockBitmap::alloc(root.addr());
    }

    /*
    if root.find("test").is_none() {
        match File::create("/test") {
            Some(test) => {
                print!("Created '/test' at block 0x{:08X}\n", test.addr());
            },
            None => {
                print!("Could not create '/test'\n");
            }
        }
    }

    if let Some(mut file) = File::open("/test") {
        let contents = "Yolo";
        file.write(&contents.as_bytes()).unwrap();
        print!("Wrote to '/test'\n");
    } else {
        print!("Could not open '/test'\n");
    }

    if let Some(file) = File::open("/test") {
        print!("Reading '/test':\n");
        print!("{}\n", file.read_to_string());
    } else {
        print!("Could not open '/test'\n");
    }

    let uptime = kernel::clock::clock_monotonic();
    print!("[{:.6}] FS Reading root directory ({} entries)\n", uptime, root.read().count());
    */
}
