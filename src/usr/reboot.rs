use crate::usr;
use crate::api::syscall;
use crate::api::console::Style;

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    let csi_color = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}MOROS has reached its fate, the system is now rebooting.{}", csi_color, csi_reset);
    syscall::sleep(0.5);
    syscall::reboot();
    loop { syscall::sleep(1.0) }
}
