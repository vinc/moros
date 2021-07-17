use crate::usr;
use crate::api::syscall;

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    print!("{:.6}\n", syscall::uptime());
    usr::shell::ExitCode::CommandSuccessful
}
