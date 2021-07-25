use crate::usr;
use crate::api::syscall;

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    println!("{:.6}", syscall::uptime());
    usr::shell::ExitCode::CommandSuccessful
}
