use crate::{sys, usr};

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() == 1 {
        eprintln!("Usage: keyboard <command>");
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

// TODO: Move that to API
fn error(message: &str) -> usr::shell::ExitCode {
    eprintln!("Error: {}", message);
    usr::shell::ExitCode::CommandError
}
