use crate::sys;
use crate::api::process::ExitCode;
use crate::api::fs::{FileIO, IO};
use crate::sys::fs::FileInfo;
use crate::sys::process::Process;

use alloc::vec;
use core::arch::asm;
use smoltcp::wire::IpAddress;
use crate::sys::fs::Device;

pub fn exit(code: ExitCode) -> ExitCode {
    sys::process::exit();
    code
}

pub fn sleep(seconds: f64) {
    sys::time::sleep(seconds);
}

pub fn delete(path: &str) -> isize {
    if sys::fs::delete(path).is_ok() {
        0
    } else {
        -1
    }
}

pub fn info(path: &str, info: &mut FileInfo) -> isize {
    let path = match sys::fs::canonicalize(path) {
        Ok(path) => path,
        Err(_) => return -1,
    };
    if let Some(res) = sys::fs::info(&path) {
        *info = res;
        0
    } else {
        -1
    }
}

pub fn open(path: &str, flags: usize) -> isize {
    let path = match sys::fs::canonicalize(path) {
        Ok(path) => path,
        Err(_) => return -1,
    };
    if let Some(resource) = sys::fs::open(&path, flags) {
        if let Ok(handle) = sys::process::create_handle(resource) {
            return handle as isize;
        }
    }
    -1
}

pub fn dup(old_handle: usize, new_handle: usize) -> isize {
    if let Some(file) = sys::process::handle(old_handle) {
        sys::process::update_handle(new_handle, *file);
        return new_handle as isize;
    }
    -1
}

pub fn read(handle: usize, buf: &mut [u8]) -> isize {
    if let Some(mut file) = sys::process::handle(handle) {
        if let Ok(bytes) = file.read(buf) {
            sys::process::update_handle(handle, *file);
            return bytes as isize;
        }
    }
    -1
}

pub fn write(handle: usize, buf: &mut [u8]) -> isize {
    if let Some(mut file) = sys::process::handle(handle) {
        if let Ok(bytes) = file.write(buf) {
            sys::process::update_handle(handle, *file);
            return bytes as isize;
        }
    }
    -1
}

pub fn close(handle: usize) {
    if let Some(mut file) = sys::process::handle(handle) {
        file.close();
        sys::process::delete_handle(handle);
    }
}

pub fn spawn(path: &str, args_ptr: usize, args_len: usize) -> ExitCode {
    let path = match sys::fs::canonicalize(path) {
        Ok(path) => path,
        Err(_) => return ExitCode::OpenError,
    };
    if let Some(mut file) = sys::fs::File::open(&path) {
        let mut buf = vec![0; file.size()];
        if let Ok(bytes) = file.read(&mut buf) {
            buf.resize(bytes, 0);
            if let Err(code) = Process::spawn(&buf, args_ptr, args_len) {
                code
            } else {
                ExitCode::Success
            }
        } else {
            ExitCode::ReadError
        }
    } else {
        ExitCode::OpenError
    }
}

pub fn stop(code: usize) -> usize {
    match code {
        0xcafe => { // Reboot
            unsafe {
                asm!(
                    "xor rax, rax",
                    "mov cr3, rax"
                );
            }
        }
        0xdead => { // Halt
            sys::process::exit();
            sys::acpi::shutdown();
        }
        _ => {
            debug!("STOP SYSCALL: Invalid code '{:#x}' received", code);
        }
    }
    0
}

pub fn poll(list: &[(usize, IO)]) -> isize {
    /*
    let timeout = 5.0;
    let started = sys::clock::realtime();
    loop {
        if sys::clock::realtime() - started > timeout {
            break;
        }
        if sys::console::end_of_text() || sys::console::end_of_transmission() {
            break;
        }
    */
        for (i, (handle, event)) in list.iter().enumerate() {
            if let Some(mut file) = sys::process::handle(*handle) {
                if file.poll(*event) {
                    return i as isize;
                }
            }
        }
    /*
        sys::time::halt();
    }
    */
    -1
}

pub fn connect(handle: usize, addr: IpAddress, port: u16) -> isize {
    if let Some(file) = sys::process::handle(handle) {
        if let sys::fs::Resource::Device(Device::TcpSocket(mut dev)) = *file {
            if dev.connect(addr, port).is_ok() {
                return 0;
            }
        }
    }
    -1
}

pub fn listen(handle: usize, port: u16) -> isize {
    if let Some(file) = sys::process::handle(handle) {
        if let sys::fs::Resource::Device(Device::TcpSocket(mut dev)) = *file {
            if dev.listen(port).is_ok() {
                return 0;
            }
        }
    }
    -1
}

pub fn accept(handle: usize) -> Result<IpAddress, ()> {
    if let Some(file) = sys::process::handle(handle) {
        if let sys::fs::Resource::Device(Device::TcpSocket(mut dev)) = *file {
            return dev.accept();
        }
    }
    Err(())
}
