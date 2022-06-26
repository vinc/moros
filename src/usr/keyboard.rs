use crate::sys;
use crate::api::console::Style;

pub fn main(args: &[&str]) -> Result<usize, usize> {
    if args.len() == 1 {
        help();
        return Err(1);
    }
    match args[1] {
        "set" => {
            if args.len() == 2 {
                error!("Keyboard layout missing");
                Err(1)
            } else {
                let layout = args[2];
                if sys::keyboard::set_keyboard(layout) {
                    Ok(0)
                } else {
                    error!("Unknown keyboard layout");
                    Err(1)
                }
            }
        }
        "-h" | "--help" | "help" => {
            help();
            Ok(0)
        }
        _ => {
            error!("Invalid command");
            Err(1)
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
