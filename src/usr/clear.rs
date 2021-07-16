use crate::{kernel, user};

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    kernel::vga::clear_screen();
    user::shell::ExitCode::CommandSuccessful
}
