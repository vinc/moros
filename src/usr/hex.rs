use crate::api::console::Style;
use crate::api::fs;
use crate::api::hex;
use crate::api::process::ExitCode;

// TODO: add `--skip` and `--length` params
pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() != 2 {
        help();
        return Err(ExitCode::UsageError);
    }
    if args[1] == "-h" || args[1] == "--help" {
        help();
        return Ok(());
    }
    let pathname = args[1];
    if let Ok(buf) = fs::read_to_bytes(pathname) {
        // TODO: read chunks
        print!("{}", hex::format_hex(&buf));
        Ok(())
    } else {
        error!("Could not read file {:?}", pathname);
        Err(ExitCode::Failure)
    }
}

fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} hex {}<file>{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
}
