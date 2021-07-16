use crate::{print, user};
use crate::api::syscall;

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    print!("{:.6}\n", syscall::uptime());
    user::shell::ExitCode::CommandSuccessful
}
