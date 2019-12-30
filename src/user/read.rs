use crate::print;
use crate::kernel::fs;
use crate::user;

pub fn main(args: &[&str]) {
    if args.len() != 2 {
        return;
    }

    let pathname = args[1];

    match pathname {
        "/dev/rtc" => user::date::main(&["date", "--iso-8601"]),
        "/dev/clk/realtime" => user::date::main(&["date", "--raw"]),
        "/dev/clk/uptime" => user::uptime::main(&["uptime", "--raw"]),
        "/sys/version" => print!("MOROS v{}\n", env!("CARGO_PKG_VERSION")),
        _ => {
            if let Some(file) = fs::File::open(pathname) {
                print!("{}\n", file.read());
            } else {
                print!("File not found '{}'\n", pathname);
            }
        }
    }
}
