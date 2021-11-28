use crate::{sys, usr};
use crate::api::console::Style;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() == 1 {
        return list(false);
    }

    match args[1] {
        "list" => {
            let verbose = args.contains(&"-v") || args.contains(&"--verbose");
            list(verbose)
        },
        _ => {
            help()
        }
    }
}

fn help() -> usr::shell::ExitCode {
    println!("Usage: <command>");
    println!();
    println!("Commands:");
    println!("  list");

    usr::shell::ExitCode::CommandSuccessful
}


fn list(verbose: bool) -> usr::shell::ExitCode {
    let color1 = Style::color("Cyan");
    let color2 = Style::color("Yellow");
    let reset = Style::reset();
    if verbose {
        println!("{}+-------------------->{} bus num{}", color1, color2, reset);
        println!("{}|    +--------------->{} device num{}", color1, color2, reset);
        println!("{}|    |  +------------>{} function num{}", color1, color2, reset);
        println!("{}|    |  |   +-------->{} vendor id{}", color1, color2, reset);
        println!("{}|    |  |   |    +--->{} device id{}", color1, color2, reset);
        println!("{}|    |  |   |    |{}", color1, reset);
    }
    for d in sys::pci::list() {
        print!("{:04X}:{:02X}:{:02X} [{:04X}:{:04X}]", d.bus, d.device, d.function, d.vendor_id, d.device_id);
        if verbose {
            println!(" {}rev={:#04X} class={:#04X},{:#04X} prog={:#04X}{}", color2, d.rev, d.class, d.subclass, d.prog, reset);
        } else {
            println!();
        }
    }
    usr::shell::ExitCode::CommandSuccessful
}
