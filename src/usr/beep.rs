use crate::{api, usr};

use x86_64::instructions::port::Port;

// See: https://wiki.osdev.org/PC_Speaker

// Play sound using built in speaker
fn play_sound(freq: f64) {
    // Set the PIT to the desired frequency
    let div = (1193180.0 / freq) as u32;

    let mut cmd: Port<u8> = Port::new(0x43);
    let mut data: Port<u8> = Port::new(0x42);
    let mut speaker: Port<u8> = Port::new(0x61);

    unsafe {
        cmd.write(0xb6);
        data.write(div as u8);
        data.write((div >> 8) as u8);
    };

    // And play the sound using the PC speaker
    let tmp = unsafe { speaker.read() };
    if tmp != (tmp | 3) {
        unsafe { speaker.write(tmp | 3) };
    }
}

// Make it stop
fn stop_sound() {
    let mut speaker: Port<u8> = Port::new(0x61);
    let tmp = unsafe { speaker.read() } & 0xFC;
    unsafe { speaker.write(tmp) };
}

fn beep(freq: f64, len: f64) {
    play_sound(freq);
    api::syscall::sleep(len);
    stop_sound();
}

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    let mut freq = 440.0;
    let mut len = 200.0;
    let mut i = 1;
    let n = args.len();
    while i < n {
        match args[i] {
            "--freq" | "-f" => {
                if i + 1 < n {
                    if let Ok(value) = args[i + 1].parse() {
                        freq = value;
                    } else {
                        println!("Could not parse freq");
                        return usr::shell::ExitCode::CommandError;
                    }
                    i += 1;
                } else {
                    println!("Missing freq");
                    return usr::shell::ExitCode::CommandError;
                }
            },
            "--len" | "-l" => {
                if i + 1 < n {
                    if let Ok(value) = args[i + 1].parse() {
                        len = value;
                    } else {
                        println!("Could not parse len");
                        return usr::shell::ExitCode::CommandError;
                    }
                    i += 1;
                } else {
                    println!("Missing len");
                    return usr::shell::ExitCode::CommandError;
                }
            },
            _ => {},
        }
        i += 1;
    }

    beep(freq, len / 1000.0);
    usr::shell::ExitCode::CommandSuccessful
}
