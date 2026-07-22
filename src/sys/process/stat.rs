use crate::api::fs::{FileIO, IO};

use alloc::format;
use alloc::string::String;
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

const SYSCALLS: [&str; 19] = [
    "exit",
    "spawn",
    "read",
    "write",
    "open",
    "close",
    "info",
    "dup",
    "delete",
    "stop",
    "sleep",
    "poll",
    "connect",
    "listen",
    "accept",
    "alloc",
    "free",
    "kind",
    "seek",
];

impl FileIO for ProcStat {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let table = SYSCALLS.iter().enumerate().map(|(index, name)| {
            format!("{} {}", name, super::syscall_count(index + 1))
        }).collect::<Vec<String>>().join("\n");
        let s = format!("label calls\n{}", table);
        let n = s.len();
        buf[0..n].copy_from_slice(s.as_bytes());
        if n > buf.len() {
            return Err(());
        }
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
