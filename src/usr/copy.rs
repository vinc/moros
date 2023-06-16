use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let n = args.len();
    if n != 3 {
        help();
        return Err(ExitCode::UsageError);
    }
    for i in 1..n {
        match args[i] {
            "-h" | "--help" => {
                help();
                return Ok(());
            }
            _ => continue
        }
    }

    let source = args[1];
    let dest = args[2];

    if let Ok(contents) = fs::read_to_bytes(source) {
        if fs::write(dest, &contents).is_ok() {
            Ok(())
        } else {
            error!("Could not write to '{}'", dest);
            Err(ExitCode::Failure)
        }
    } else {
        error!("Could not find file '{}'", source);
        Err(ExitCode::Failure)
    }
}

fn help() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} copy {}<src> <dst>{}", csi_title, csi_reset, csi_option, csi_reset);
}
