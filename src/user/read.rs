use crate::{print, kernel, user};

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 2 {
        return user::shell::ExitCode::CommandError;
    }

    let pathname = args[1];

    match pathname {
        "/dev/rtc" => {
            user::date::main(&["date", "--iso-8601"])
        },
        "/dev/clk/realtime" => {
            user::date::main(&["date", "--raw"])
        },
        "/dev/clk/uptime" => {
            user::uptime::main(&["uptime", "--raw"])
        },
        "/sys/version" => {
            print!("MOROS v{}\n", env!("CARGO_PKG_VERSION"));
            user::shell::ExitCode::CommandSuccessful
        },
        _ => {
            if pathname.ends_with('/') {
                user::list::main(args)
            } else if let Some(file) = kernel::fs::File::open(pathname) {
                print!("{}", file.read_to_string());
                user::shell::ExitCode::CommandSuccessful
            } else {
                print!("File not found '{}'\n", pathname);
                user::shell::ExitCode::CommandError
            }
        }
    }
}
