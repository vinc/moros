use crate::sys;
use alloc::string::{String, ToString};

pub fn canonicalize(path: &str) -> Result<String, ()> {
    match sys::process::env("HOME") {
        Some(home) => {
            if path.starts_with('~') {
                Ok(path.replace("~", &home))
            } else {
                Ok(path.to_string())
            }
        },
        None => {
            Ok(path.to_string())
        }
    }
}

pub fn read_to_string(path: &str) -> Result<String, ()> {
    let path = match canonicalize(path) {
        Ok(path) => path,
        Err(_) => return Err(()),
    };
    match sys::fs::File::open(&path) {
        Some(mut file) => {
            Ok(file.read_to_string())
        },
        None => {
            Err(())
        }
    }
}

pub fn write(path: &str, buf: &[u8]) -> Result<(), ()> {
    let path = match canonicalize(path) {
        Ok(path) => path,
        Err(_) => return Err(()),
    };
    let mut file = match sys::fs::File::open(&path) {
        None => match sys::fs::File::create(&path) {
            None => return Err(()),
            Some(file) => file,
        },
        Some(file) => file,
    };
    // TODO: add File::write_all to split buf if needed
    match file.write(buf) {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}
