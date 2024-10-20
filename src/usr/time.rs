use crate::api::clock;
use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::usr;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let csi_color = Style::color("blue");
    let csi_reset = Style::reset();
    let cmd = args[1..].join(" ");
    let start = clock::epoch_time();
    let res = usr::shell::exec(&cmd);
    let duration = clock::epoch_time() - start;
    eprintln!(
        "{}Executed '{}' in {:.6}s{}",
        csi_color, cmd, duration, csi_reset
    );
    res
}
