use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::syscall;
use alloc::string::{String, ToString};
use crate::sys;
use super::shell::Config;

/// Main function for the cd command.
///
/// Behavior:
/// - With no arguments: prints the current directory.
/// - With one argument:
///   - If the argument is ".", remains in the current directory.
///   - If the argument is "..", changes to the parent directory.
///   - Otherwise, attempts to change to the specified directory.
pub fn main(args: &[&str], config: &mut Config) -> Result<(), ExitCode> {
    // Check if the user requested help.
    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        help();
        return Ok(());
    }

    match args.len() {
        // No arguments: print the current directory.
        1 => {
            println!("{}", sys::process::dir());
            Ok(())
        }
        // One argument: process the path and attempt to change the directory.
        2 => {
            let new_path = resolve_path(args[1]);
            if !fs::exists(&new_path) {
                error!("cd: no such file or directory: {}", new_path);
                return Err(ExitCode::Failure);
            }
            if !is_dir(&new_path) {
                error!("cd: not a directory: {}", new_path);
                return Err(ExitCode::Failure);
            }
            sys::process::set_dir(&new_path);
            config.env.insert("DIR".to_string(), sys::process::dir());
            Ok(())
        }
        // More than one argument: usage error.
        _ => {
            help();
            Err(ExitCode::UsageError)
        }
    }
}

/// Resolves the provided path, handling special cases:
/// - "." returns the current directory;
/// - ".." returns the parent directory;
/// - Absolute paths are returned unchanged;
/// - Relative paths are concatenated with the current directory.
fn resolve_path(arg: &str) -> String {
    let current_dir = sys::process::dir();
    match arg {
        "." => current_dir,
        ".." => fs::dirname(current_dir.trim_end_matches('/')).to_string(),
        path if path.starts_with('/') => path.to_string(),
        path => {
            let mut new_path = current_dir;
            if !new_path.ends_with('/') {
                new_path.push('/');
            }
            new_path.push_str(path);
            new_path
        }
    }
}

/// Checks if the specified path corresponds to a valid directory.
pub fn is_dir(path: &str) -> bool {
    if let Some(info) = syscall::info(path) {
        info.is_dir()
    } else {
        false
    }
}

/// Displays the help message for the cd command.
fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} cd [OPTION]... [DIRECTORY]", csi_title, csi_reset);
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!("  {0}-h{1}, {0}--help{1}      display this help and exit", csi_option, csi_reset);
    println!();
    println!("If no DIRECTORY is provided, the current directory is printed.");
    println!("Examples:");
    println!("  cd ..     - change to the parent directory");
    println!("  cd .      - remain in the current directory");
    println!("  cd path   - change to the specified directory");
}