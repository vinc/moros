use crate::{kernel, print, user};
use crate::kernel::console::Style;

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    let csi_color = Style::color("Yellow");
    let csi_reset = Style::reset();
    print!("{}MOROS has reached its fate, the system is now halted.{}\n", csi_color, csi_reset);
    kernel::time::sleep(3.0);
    kernel::acpi::poweroff();
    user::shell::ExitCode::CommandSuccessful
}
