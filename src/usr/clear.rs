use crate::usr;

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    print!("\x1b[2J\x1b[1;1H"); // Clear screen and move cursor to top
    usr::shell::ExitCode::CommandSuccessful
}
