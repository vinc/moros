use crate::sys;
use crate::api::console::Style;
use crate::api::process;

pub fn main(args: &[&str]) -> Result<(), usize> {
    if args.len() == 1 {
        help();
        return Err(process::EXIT_FAILURE);
    }
    match args[1] {
        "set" => {
            if args.len() == 2 {
                error!("Keyboard layout missing");
                Err(process::EXIT_FAILURE)
            } else {
                let layout = args[2];
                if sys::keyboard::set_keyboard(layout) {
                    Ok(())
                } else {
                    error!("Unknown keyboard layout");
                    Err(process::EXIT_FAILURE)
                }
            }
        }
        "-h" | "--help" | "help" => {
            help();
            Ok(())
        }
        _ => {
            error!("Invalid command");
            Err(process::EXIT_FAILURE)
        }
    }
}

fn help() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} keyboard {}<command>{}", csi_title, csi_reset, csi_option, csi_reset);
    println!();
    println!("{}Commands:{}", csi_title, csi_reset);
    println!("  {0}set <layout>{1}    Set keyboard layout", csi_option, csi_reset);
}
