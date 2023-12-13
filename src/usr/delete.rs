use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::syscall;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let n = args.len();
    if n < 2 {
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

    for arg in &args[1..] {
        let mut pathname = *arg;

        // The commands `delete /usr/alice/` and `delete /usr/alice` are equivalent,
        // but `delete /` should not be modified.
        if pathname.len() > 1 {
            pathname = pathname.trim_end_matches('/');
        }

        if !fs::exists(pathname) {
            error!("Could not find file '{}'", pathname);
            return Err(ExitCode::Failure);
        }

        if let Some(info) = syscall::info(pathname) {
            if info.is_dir() && info.size() > 0 {
                error!("Directory '{}' not empty", pathname);
                return Err(ExitCode::Failure);
            }
        }

        if fs::delete(pathname).is_err() {
            error!("Could not delete file '{}'", pathname);
            return Err(ExitCode::Failure);
        }
    }
    Ok(())
}

fn help() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} delete {}<path>{}", csi_title, csi_reset, csi_option, csi_reset);
    println!();
    println!("{}Paths:{}", csi_title, csi_reset);
    println!("  {0}<dir>/{1}     Delete directory", csi_option, csi_reset);
    println!("  {0}<file>{1}     Delete file", csi_option, csi_reset);
}
