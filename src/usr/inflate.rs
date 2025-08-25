use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::syscall;

use alloc::format;
use miniz_oxide::inflate::decompress_to_vec_zlib as inflate;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() != 2 {
        help();
        return Err(ExitCode::UsageError);
    }
    if args[1] == "-h" || args[1] == "--help" {
        help();
        return Ok(());
    }

    let path = args[1];
    if let Some(info) = syscall::info(path) {
        if info.is_file() {
            if let Ok(bytes) = fs::read_to_bytes(path) {
                if let Ok(buf) = inflate(&bytes) {
                    if path.ends_with(".z") {
                        let n = path.len() - 2;
                        if fs::write(&path[0..n], &buf).is_ok() {
                            if fs::delete(path).is_ok() {
                                Ok(())
                            } else {
                                error!("Could not drop '{}'", path);
                                Err(ExitCode::Failure)
                            }
                        } else {
                            error!("Could not inflate to '{}'", &path[0..n]);
                            Err(ExitCode::Failure)
                        }
                    } else {
                        error!("Could not drop .z extension from '{}'", path);
                        Err(ExitCode::Failure)
                    }
                } else {
                    error!("Could not inflate '{}'", path);
                    Err(ExitCode::Failure)
                }
            } else {
                error!("Could not read '{}'", path);
                Err(ExitCode::Failure)
            }
        } else {
            error!("Could not read type of '{}'", path);
            Err(ExitCode::Failure)
        }
    } else {
        error!("Could not find file '{}'", path);
        Err(ExitCode::Failure)
    }
}

fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} inflate {}<file.z>{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
}
