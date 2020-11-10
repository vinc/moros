use crate::{kernel, print, user};

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    print!("{:.6}\n", kernel::clock::uptime());
    user::shell::ExitCode::CommandSuccessful
}
