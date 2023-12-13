use crate::api;
use crate::api::clock::DATE_TIME_ZONE;
use crate::api::console::Style;
use crate::api::process::ExitCode;
use time::validate_format_string;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() > 2 {
        return Err(ExitCode::UsageError);
    }
    let format = if args.len() > 1 { args[1] } else { DATE_TIME_ZONE };
    if format == "-h" || format == "--help" {
        return help();
    }
    match validate_format_string(format) {
        Ok(()) => {
            println!("{}", api::time::now().format(format));
            Ok(())
        }
        Err(e) => {
            error!("{}", e);
            Err(ExitCode::Failure)
        }
    }
}

fn help() -> Result<(), ExitCode> {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} date {}[<format>]{}", csi_title, csi_reset, csi_option, csi_reset);
    Ok(())
}
