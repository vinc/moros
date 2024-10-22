use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::fs;
use crate::api::syscall;

use alloc::string::ToString;

const SPEAKER: &'static str = "/dev/speaker";

fn start_sound(freq: f64) -> Result<(), ExitCode> {
    let buf = freq.to_string();
    if !fs::is_device(SPEAKER) || fs::write(SPEAKER, buf.as_bytes()).is_err() {
        error!("Could not write to '{}'", SPEAKER);
        Err(ExitCode::Failure)
    } else {
        Ok(())
    }
}

fn stop_sound() -> Result<(), ExitCode> {
    start_sound(0.0)
}

fn beep(freq: f64, len: f64) -> Result<(), ExitCode> {
    start_sound(freq)?;
    syscall::sleep(len);
    stop_sound()
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut freq = 440.0;
    let mut len = 200.0;
    let mut i = 1;
    let n = args.len();
    while i < n {
        match args[i] {
            "-h" | "--help" => {
                return help();
            }
            "-f" | "--freq" => {
                if i + 1 < n {
                    i += 1;
                    if let Ok(value) = args[i].parse() {
                        freq = value;
                    } else {
                        error!("Could not parse freq");
                        return Err(ExitCode::Failure);
                    }
                } else {
                    error!("Missing freq");
                    return Err(ExitCode::UsageError);
                }
            }
            "-l" | "--len" => {
                if i + 1 < n {
                    i += 1;
                    if let Ok(value) = args[i].parse() {
                        len = value;
                    } else {
                        error!("Could not parse len");
                        return Err(ExitCode::Failure);
                    }
                } else {
                    error!("Missing len");
                    return Err(ExitCode::UsageError);
                }
            }
            _ => {}
        }
        i += 1;
    }

    beep(freq, len / 1000.0)
}

fn help() -> Result<(), ExitCode> {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} beep {}<options>{1}",
        csi_title, csi_reset, csi_option
    );
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!(
        "  {0}-f{1}, {0}--freq <hertz>{1}          Tone frequency",
        csi_option, csi_reset
    );
    println!(
        "  {0}-l{1}, {0}--len <milliseconds>{1}    Tone length",
        csi_option, csi_reset
    );
    Ok(())
}
