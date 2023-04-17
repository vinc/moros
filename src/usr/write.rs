use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::syscall;

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

    // The command `write /usr/alice/` with a trailing slash will create
    // a directory, while the same command without a trailing slash will
    // create a file.
    let res = if pathname.ends_with('/') {
        let pathname = pathname.trim_end_matches('/');
        fs::create_dir(pathname)
    } else {
        fs::create_file(pathname)
    };

    if let Some(handle) = res {
        syscall::close(handle);
        Ok(())
    } else {
        error!("Could not write to '{}'", pathname);
        Err(ExitCode::Failure)
    }
}

fn help() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} write {}<path>{}", csi_title, csi_reset, csi_option, csi_reset);
    println!();
    println!("{}Paths:{}", csi_title, csi_reset);
    println!("  {0}<dir>/{1}     Write directory", csi_option, csi_reset);
    println!("  {0}<file>{1}     Write file", csi_option, csi_reset);
}
