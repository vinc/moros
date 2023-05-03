use crate::api::syscall;

use core::convert::From;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitCode {
    Success        = 0,
    Failure        = 1,
    UsageError     = 64,
    DataError      = 65,
    OpenError      = 128,
    ReadError      = 129,
    ExecError      = 130,
    PageFaultError = 200,
    ShellExit      = 255,
}

impl From<usize> for ExitCode {
    fn from(code: usize) -> Self {
        match code {
            0 => ExitCode::Success,
            64 => ExitCode::UsageError,
            65 => ExitCode::DataError,
            128 => ExitCode::OpenError,
            129 => ExitCode::ReadError,
            130 => ExitCode::ExecError,
            200 => ExitCode::PageFaultError,
            255 => ExitCode::ShellExit,
            _ => ExitCode::Failure,
        }
    }
}

pub fn spawn(path: &str, args: &[&str]) -> Result<(), ExitCode> {
    if syscall::info(path).is_some() {
        syscall::spawn(path, args)
    } else {
        Err(ExitCode::OpenError)
    }
}
