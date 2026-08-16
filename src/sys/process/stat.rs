use crate::sys::fs::{FileIO, IO};
use crate::sys::syscall;

use alloc::format;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct ProcStat;

impl ProcStat {
    pub fn new() -> Self {
        Self
    }

    pub fn size() -> usize {
        1024
    }
}

impl FileIO for ProcStat {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let n = syscall::number::count();
        let rows = (0..n).map(|i| {
            let name = syscall::number::name(i + 1).unwrap();
            let count = super::syscall_count(i + 1);
            format!("{name} {count}")
        }).collect::<Vec<_>>().join("\n");
        let s = format!("label calls\n{}", rows);
        let n = s.len();
        if n > buf.len() {
            return Err(());
        }
        buf[0..n].copy_from_slice(s.as_bytes());
        Ok(n)
    }

    fn write(&mut self, _buf: &[u8]) -> Result<usize, ()> {
        Err(())
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => true,
            IO::Write => false,
        }
    }
}
