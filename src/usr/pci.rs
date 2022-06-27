use crate::sys;
use crate::api::console::Style;

pub fn main(args: &[&str]) -> Result<(), usize> {
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

fn list(verbose: bool) -> Result<(), usize> {
    let color1 = Style::color("Blue");
    let color2 = Style::color("LightBlue");
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
    Ok(())
}

fn help() -> Result<(), usize> {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} pci {}<command> <options>{1}", csi_title, csi_reset, csi_option);
    println!();
    println!("{}Commands:{}", csi_title, csi_reset);
    println!("  {}list{}             List PCI devices", csi_option, csi_reset);
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!("  {0}-v{1}, {0}--verbose{1}    Increase verbosity", csi_option, csi_reset);
    Ok(())
}
