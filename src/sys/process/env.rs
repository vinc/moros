use crate::api::fs::{FileIO, IO};
use crate::api::ini;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct ProcEnv;

impl ProcEnv {
    pub fn new() -> Self {
        Self
    }

    pub fn size() -> usize {
        4096
    }
}

impl FileIO for ProcEnv {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let env = super::env();
        let max = env.keys().map(|k| k.len()).max().unwrap_or(0);
        let res = env.iter().map(|(k, v)|
            format!("{:max$} = \"{}\"", k, v)
        ).collect::<Vec<String>>().join("\n");
        let n = res.len();
        if n > buf.len() {
            return Err(());
        }
        buf[0..n].copy_from_slice(res.as_bytes());
        Ok(n)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        if let Ok(s) = String::from_utf8(buf.to_vec()) {
            if let Some(h) = ini::parse(&s) {
                for (k, v) in h.iter() {
                    super::set_env_var(k, v);
                }
                return Ok(buf.len());
            }
        }
        Err(())
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => true,
            IO::Write => true,
        }
    }
}
