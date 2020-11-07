use crate::{kernel, print, user};
use crate::kernel::console::Style;

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    let csi_color = Style::color("Yellow");
    let csi_reset = Style::reset();
    print!("{}MOROS has reached its fate, the system is now halted.{}\n", csi_color, csi_reset);
    kernel::acpi::shutdown();
    loop { kernel::time::sleep(1.0) }
}
