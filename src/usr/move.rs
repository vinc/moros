use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::usr;

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
            _ => continue,
        }
    }

    // TODO: Avoid doing copy+delete
    match usr::copy::main(args) {
        Ok(()) => usr::delete::main(&args[0..2]),
        _ => Err(ExitCode::Failure),
    }
}

fn help() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} move {}<src> <dst>{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
}
