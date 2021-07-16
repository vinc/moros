use crate::{sys, usr};

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    sys::vga::clear_screen();
    usr::shell::ExitCode::CommandSuccessful
}
