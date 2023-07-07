use crate::sys;
use crate::api::process::ExitCode;
use crate::sys::fs::FileInfo;
use crate::sys::fs::FileIO;
use crate::sys::process::Process;

use alloc::vec;
use core::arch::asm;
use smoltcp::time::Instant;
use smoltcp::wire::IpAddress;
use crate::sys::net::SOCKETS;
use crate::sys::fs::Device;
use smoltcp::socket::tcp;

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
        if let Ok(handle) = sys::process::create_file_handle(resource) {
            return handle as isize;
        }
    }
    -1
}

pub fn dup(old_handle: usize, new_handle: usize) -> isize {
    if let Some(file) = sys::process::file_handle(old_handle) {
        sys::process::update_file_handle(new_handle, *file);
        return new_handle as isize;
    }
    -1
}

pub fn read(handle: usize, buf: &mut [u8]) -> isize {
    if let Some(mut file) = sys::process::file_handle(handle) {
        if let Ok(bytes) = file.read(buf) {
            sys::process::update_file_handle(handle, *file);
            return bytes as isize;
        }
    }
    -1
}

pub fn write(handle: usize, buf: &mut [u8]) -> isize {
    if let Some(mut file) = sys::process::file_handle(handle) {
        if let Ok(bytes) = file.write(buf) {
            sys::process::update_file_handle(handle, *file);
            return bytes as isize;
        }
    }
    -1
}

pub fn close(handle: usize) {
    sys::process::delete_file_handle(handle);
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

pub fn connect(handle: usize, addr: IpAddress, port: u16) -> isize {
    let timeout = 5.0;
    let started = sys::clock::realtime();
    if let Some(file) = sys::process::file_handle(handle) {
        if let sys::fs::Resource::Device(Device::TcpSocket(dev)) = *file {
            if let Some((ref mut iface, ref mut device)) = *sys::net::NET.lock() {
                loop {
                    if sys::clock::realtime() - started > timeout {
                        return -1;
                    }
                    let mut sockets = SOCKETS.lock();
                    let time = Instant::from_micros((sys::clock::realtime() * 1000000.0) as i64);
                    iface.poll(time, device, &mut sockets);
                    let socket = sockets.get_mut::<tcp::Socket>(dev.handle);
                    let cx = iface.context();

                    match socket.state() {
                        tcp::State::Closed => {
                            let local_port = 49152 + sys::random::get_u16() % 16384;
                            if socket.connect(cx, (addr, port), local_port).is_err() {
                                return -1;
                            }
                        }
                        tcp::State::SynSent => {
                        }
                        tcp::State::Established => {
                            break;
                        }
                        _ => {
                            return -1;
                        }
                    }

                    if let Some(wait_duration) = iface.poll_delay(time, &sockets) {
                        sys::time::sleep((wait_duration.total_micros() as f64) / 1000000.0);
                    }
                    sys::time::halt();
                }
            }
        }
    }
    0
}
