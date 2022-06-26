use crate::usr;
use crate::api::clock;
use crate::api::console::Style;

pub fn main(args: &[&str]) -> Result<usize, usize> {
    let csi_color = Style::color("LightBlue");
    let csi_reset = Style::reset();
    let cmd = args[1..].join(" ");
    let start = clock::realtime();
    let res = usr::shell::exec(&cmd);
    let duration = clock::realtime() - start;
    println!("{}Executed '{}' in {:.6}s{}", csi_color, cmd, duration, csi_reset);
    res
}
