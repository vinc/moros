use crate::{sys, usr};

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() == 1 {
        println!("Usage: keyboard <command>");
        return usr::shell::ExitCode::CommandError;
    }
    match args[1] {
        "set" => {
            if args.len() == 2 {
                return error("keyboard layout missing");
            } else {
                let layout = args[2];
                if !sys::keyboard::set_keyboard(layout) {
                    return error("unknown keyboard layout");
                }
            }
        },
        _ => {
            return error("invalid command");
        }
    }
    usr::shell::ExitCode::CommandSuccessful
}

fn error(message: &str) -> usr::shell::ExitCode {
    println!("Error: {}", message);
    usr::shell::ExitCode::CommandError
}
