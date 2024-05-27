use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::{api, sys};

use x86_64::instructions::port::Port;

// See: https://wiki.osdev.org/PC_Speaker

const SPEAKER_PORT: u16 = 0x61;

fn start_sound(freq: f64) {
    let divider = (sys::time::PIT_FREQUENCY / freq) as u16;
    let channel = 2;
    sys::time::set_pit_frequency_divider(divider, channel);

    let mut speaker: Port<u8> = Port::new(SPEAKER_PORT);
    let tmp = unsafe { speaker.read() };
    if tmp != (tmp | 3) {
        unsafe { speaker.write(tmp | 3) };
    }
}

fn stop_sound() {
    let mut speaker: Port<u8> = Port::new(SPEAKER_PORT);
    let tmp = unsafe { speaker.read() } & 0xFC;
    unsafe { speaker.write(tmp) };
}

fn beep(freq: f64, len: f64) {
    start_sound(freq);
    api::syscall::sleep(len);
    stop_sound();
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

    beep(freq, len / 1000.0);
    Ok(())
}

fn help() -> Result<(), ExitCode> {
    let csi_option = Style::color("LightCyan");
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
