pub const OPEN:   usize = 0;
pub const CREATE: usize = 1;
pub const READ:   usize = 2;
pub const WRITE:  usize = 3;
pub const DELETE: usize = 4;
pub const SLEEP:  usize = 5;

#[repr(usize)]
pub enum SysCallNumber {
    // File operations
    Open,
    Create,
    Read,
    Write,
    Delete,

    // Process operations
    Sleep,
}
