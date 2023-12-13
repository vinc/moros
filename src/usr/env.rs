use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::sys;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let n = args.len();
    for i in 1..n {
        match args[i] {
            "-h" | "--help" => {
                help();
                return Ok(());
            }
            _ => continue
        }
    }
    match n {
        1 => {
            let width = sys::process::envs().keys().map(|k| k.len()).max().unwrap_or(0);
            for (key, val) in sys::process::envs() {
                println!("{:width$} \"{}\"", key, val, width = width);
            }
            Ok(())
        }
        2 => {
            let key = args[1];
            if let Some(val) = sys::process::env(key) {
                println!("{}", val);
                Ok(())
            } else {
                error!("Could not get '{}'", key);
                Err(ExitCode::Failure)
            }
        }
        3 => {
            sys::process::set_env(args[1], args[2]);
            Ok(())
        }
        _ => {
            help();
            Err(ExitCode::UsageError)
        }
    }
}

fn help() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} env {}[<key> [<value>]]{}", csi_title, csi_reset, csi_option, csi_reset);
}
