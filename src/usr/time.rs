use crate::usr;
use crate::api::syscall;
use crate::api::console::Style;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    let csi_color = Style::color("LightBlue");
    let csi_reset = Style::reset();
    let cmd = args[1..].join(" ");
    let start = syscall::realtime();
    usr::shell::exec(&cmd);
    let duration = syscall::realtime() - start;
    println!("{}Executed '{}' in {:.6}s{}", csi_color, cmd, duration, csi_reset);
    usr::shell::ExitCode::CommandSuccessful
}
