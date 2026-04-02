use core::convert::TryFrom;

#[repr(usize)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SyscallCode {
    Exit        = 0x1,
    Spawn       = 0x2,
    Read        = 0x3,
    Write       = 0x4,
    Open        = 0x5,
    Close       = 0x6,
    Info        = 0x7,
    Dup         = 0x8,
    Delete      = 0x9,
    Stop        = 0xA,
    Sleep       = 0xB,
    Poll        = 0xC,
    Connect     = 0xD,
    Listen      = 0xE,
    Accept      = 0xF,
    Alloc       = 0x10,
    Free        = 0x11,
    Kind        = 0x12,
    Version     = 0x13,
    CreateDir   = 0x14,
    FileList    = 0x15,
}

impl TryFrom<usize> for SyscallCode {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value >= 0x1 && value <= 0x12 {
            Ok(unsafe { core::mem::transmute(value) })
        } else {
            Err(())
        }
    }
}