use super::Context;

use crate::api::console::Style;
use crate::api::fs;
use crate::api::process;
use crate::api::process::ExitCode;
use crate::sys;

use alloc::string::ToString;

pub fn print(args: &[&str], ctx: &mut Context) -> Result<(), ExitCode> {
    println!("{}", args[1..].join(" "));

    // Handle dir changed with `print /tmp => /dev/proc/dir`
    ctx.env.insert("dir".to_string(), process::dir());

    Ok(())
}

pub fn change_dir(args: &[&str], ctx: &mut Context) -> Result<(), ExitCode> {
    match args.len() {
        1 => {
            println!("{}", process::dir());
            Ok(())
        }
        2 => {
            let mut path = fs::realpath(args[1]);
            if path.len() > 1 {
                path = path.trim_end_matches('/').into();
            }
            if fs::is_dir(&path) {
                process::set_dir(&path);
                ctx.env.insert("dir".to_string(), process::dir());
                Ok(())
            } else {
                error!("Could not find file {:?}", path);
                Err(ExitCode::Failure)
            }
        }
        _ => Err(ExitCode::Failure),
    }
}

pub fn alias(args: &[&str], ctx: &mut Context) -> Result<(), ExitCode> {
    if args.len() != 3 {
        let csi_option = Style::color("aqua");
        let csi_title = Style::color("yellow");
        let csi_reset = Style::reset();
        eprintln!(
            "{}Usage:{} alias {}<key> <val>{1}",
            csi_title, csi_reset, csi_option
        );
        return Err(ExitCode::UsageError);
    }
    ctx.aliases.insert(args[1].to_string(), args[2].to_string());
    Ok(())
}

pub fn unalias(args: &[&str], ctx: &mut Context) -> Result<(), ExitCode> {
    if args.len() != 2 {
        let csi_option = Style::color("aqua");
        let csi_title = Style::color("yellow");
        let csi_reset = Style::reset();
        eprintln!(
            "{}Usage:{} unalias {}<key>{1}",
            csi_title, csi_reset, csi_option
        );
        return Err(ExitCode::UsageError);
    }

    if ctx.aliases.remove(args[1]).is_none() {
        error!("Could not unalias {:?}", args[1]);
        return Err(ExitCode::Failure);
    }

    Ok(())
}

#[derive(Debug, PartialEq)]
enum VarScope {
    Local,
    Global,
    Env,
}

pub fn set(args: &[&str], ctx: &mut Context) -> Result<(), ExitCode> {
    let mut scope = VarScope::Local;
    let mut key = None;
    let mut val = None;
    let mut i = 1;
    let n = args.len();
    while i < n {
        match args[i] {
            "-h" | "--help" => {
                let csi_option = Style::color("aqua");
                let csi_title = Style::color("yellow");
                let csi_reset = Style::reset();
                eprintln!(
                    "{}Usage:{} set {}<options> <key> <val>{1}",
                    csi_title, csi_reset, csi_option
                );
                println!();
                println!("{}Options:{}", csi_title, csi_reset);
                println!(
                    "  {0}-e{1}, {0}--env{1}    Set env var",
                    csi_option, csi_reset
                );
                return Ok(());
            }
            "-e" | "--env" => {
                scope = VarScope::Env;
            }
            "-l" | "--local" => {
                scope = VarScope::Local;
            }
            "-g" | "--global" => {
                scope = VarScope::Global; // TODO
            }
            _ => {
                if args[i].starts_with('-') {
                    error!("Invalid option {:?}", args[i]);
                    return Err(ExitCode::UsageError);
                } else if key.is_none() {
                    key = Some(args[i]);
                } else if val.is_none() {
                    val = Some(args[i]);
                } else {
                    error!("Too many arguments");
                    return Err(ExitCode::UsageError);
                }
            }
        }
        i += 1;
    }
    if let Some(key) = key {
        if let Some(val) = val {
            if scope == VarScope::Env {
                process::set_env_var(key, val);
            }
            ctx.env.insert(key.to_string(), val.to_string());
            return Ok(());
        }
    }
    Err(ExitCode::UsageError)
}

pub fn unset(args: &[&str], ctx: &mut Context) -> Result<(), ExitCode> {
    // TODO: Unset env var
    if args.len() != 2 {
        let csi_option = Style::color("aqua");
        let csi_title = Style::color("yellow");
        let csi_reset = Style::reset();
        eprintln!(
            "{}Usage:{} unset {}<key>{1}",
            csi_title, csi_reset, csi_option
        );
        return Err(ExitCode::UsageError);
    }

    if ctx.env.remove(args[1]).is_none() {
        error!("Could not unset {:?}", args[1]);
        return Err(ExitCode::Failure);
    }

    Ok(())
}

pub fn logs() -> Result<(), ExitCode> {
    print!("{}", sys::log::read());
    Ok(())
}

pub fn version() -> Result<(), ExitCode> {
    println!(
        "MOROS v{}",
        option_env!("MOROS_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))
    );
    Ok(())
}

