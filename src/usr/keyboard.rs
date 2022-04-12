use crate::{sys, usr};
use crate::api::console::Style;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() == 1 {
        help();
        return usr::shell::ExitCode::CommandError;
    }
    match args[1] {
        "set" => {
            if args.len() == 2 {
                error!("Keyboard layout missing");
                usr::shell::ExitCode::CommandError
            } else {
                let layout = args[2];
                if sys::keyboard::set_keyboard(layout) {
                    usr::shell::ExitCode::CommandSuccessful
                } else {
                    error!("Unknown keyboard layout");
                    usr::shell::ExitCode::CommandError
                }
            }
        }
        "-h" | "--help" | "help" => {
            help()
        }
        _ => {
            error!("Invalid command");
            usr::shell::ExitCode::CommandError
        }
    }
}

fn help() -> usr::shell::ExitCode {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} keyboard {}<command>{}", csi_title, csi_reset, csi_option, csi_reset);
    println!();
    println!("{}Commands:{}", csi_title, csi_reset);
    println!("  {0}set <layout>{1}    Set keyboard layout", csi_option, csi_reset);
    usr::shell::ExitCode::CommandSuccessful
}
