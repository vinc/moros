use crate::{print, kernel, user};

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    print!("MOROS has reached its fate, the system is now halted.\n");
    kernel::time::sleep(3.0);
    kernel::acpi::poweroff();
    user::shell::ExitCode::CommandSuccessful
}

