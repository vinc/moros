use crate::usr;

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    print!("\x1b[2J"); // Clear screen
    usr::shell::ExitCode::CommandSuccessful
}
