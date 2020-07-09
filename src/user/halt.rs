use crate::{print, kernel, user};

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    let csi_color = kernel::console::color("Yellow");
    let csi_reset = kernel::console::color("Reset");
    print!("{}MOROS has reached its fate, the system is now halted.{}\n", csi_color, csi_reset);
    kernel::time::sleep(3.0);
    kernel::acpi::poweroff();
    user::shell::ExitCode::CommandSuccessful
}

