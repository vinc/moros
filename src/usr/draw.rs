use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::vga;
use crate::api::font::Font;
use crate::api::syscall;
use crate::api::clock;
use crate::sys::console;
use crate::usr::shell;

use alloc::string::{String, ToString};
use alloc::format;
use alloc::vec::Vec;
use bit_field::BitField;
use core::convert::TryFrom;

const WIDTH: usize = 320;
const HEIGHT: usize = 200;

const BLACK: u8 = 0x00;
const GREEN: u8 = 0x3A;

const FONT: &str = "/ini/fonts/zap-light-8x16.psf";

#[derive(PartialEq)]
enum Mode {
    Text,
    Graphic,
}

struct Config {
    mode: Mode
}

impl Config {
    pub fn new() -> Self {
        Self { mode: Mode::Text }
    }

    pub fn text_mode(&mut self) {
        if self.mode == Mode::Graphic {
            vga::text_mode();
            self.mode = Mode::Text;
        }
    }

    pub fn graphic_mode(&mut self) {
        if self.mode == Mode::Text {
            vga::graphic_mode();
            self.mode = Mode::Graphic;
        }
    }
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() == 1 {
        help();
        return Err(ExitCode::UsageError);
    }
    if args.contains(&"-h") || args.contains(&"--help") {
        help();
        return Ok(());
    }
    let mut center = false;
    let mut interval: Option<f64> = None;
    let mut command = None;
    let mut text = None;
    let mut i = 1;
    let n = args.len();
    while i < n {
        match args[i] {
            "-h" | "--help" => {
                help();
                return Ok(());
            }
            "-c" | "--center" => {
                center = true;
            }
            "-i" | "--interval" => {
                if i + 1 < n {
                    i += 1;
                    interval = args[i].parse().ok();
                } else {
                    error!("Missing interval");
                    return Err(ExitCode::UsageError);
                }
            }
            "-e" | "--execute" => {
                if i + 1 < n {
                    i += 1;
                    command = Some(args[i]);
                } else {
                    error!("Missing command");
                    return Err(ExitCode::UsageError);
                }
            }
            "-t" | "--text" => {
                if i + 1 < n {
                    i += 1;
                    text = Some(args[i]);
                } else {
                    error!("Missing text");
                    return Err(ExitCode::UsageError);
                }
            }
            _ => {
                error!("Invalid argument");
                return Err(ExitCode::UsageError);
            }
        }
        i += 1;
    }
    let mut config = Config::new();
    if let Ok(buf) = fs::read_to_bytes(&FONT) {
        if let Ok(fnt) = Font::try_from(&buf[..]) {
            let w = 8;
            let h = fnt.height as usize;
            let max_cols = WIDTH / w;
            let max_rows = HEIGHT / h;
            let chars: Vec<_> = fnt.data.chunks(h).collect();

            let mut refresh = true;
            let mut start = clock::epoch_time(); 
            loop {
                if console::end_of_text() {
                    break;
                }
                if let Some(i) = interval {
                    refresh = refresh || (clock::epoch_time() > start + i);
                }
                if refresh {
                    start = clock::epoch_time(); 
                    let out = if let Some(cmd) = command {
                        if let Ok(buf) = shell::exec_to_bytes(&cmd) {
                            if let Ok(txt) = String::from_utf8(buf) {
                                txt.trim().to_string()
                            } else {
                                config.text_mode();
                                return Err(ExitCode::Failure);
                            }
                        } else {
                            config.text_mode();
                            return Err(ExitCode::Failure);
                        }
                    } else {
                        if let Some(txt) = text {
                            txt.to_string()
                        } else {
                            help();
                            return Err(ExitCode::UsageError);
                        }
                    };
                    let pad = " ".repeat(if center {
                        (max_cols.saturating_sub(out.len()) / 2)
                            + (max_rows / 2) * max_cols
                    } else {
                        0
                    });
                    let out = format!("{}{}", pad, out);
                    let mut img = [BLACK; WIDTH * HEIGHT];
                    for (i, c) in out.chars().enumerate() {
                        if i >= max_cols * max_rows {
                            break;
                        }
                        let col = i % max_cols;
                        let row = i / max_cols;
                        let offset = col * w + row * h * WIDTH;
                        for x in 0..w {
                            for y in 0..h {
                                if chars[c as usize][y].get_bit(w - 1 - x) {
                                    img[x + y * WIDTH + offset] = GREEN;
                                }
                            }
                        }
                    }

                    config.graphic_mode();

                    let dev = "/dev/vga/buffer";
                    if !fs::is_device(dev) || fs::write(dev, &img).is_err() {
                        config.text_mode();
                        error!("Could not write to {:?}", dev);
                        return Err(ExitCode::Failure);
                    }
                    refresh = false;
                }
                syscall::sleep(0.1);
            }
        }
    }

    config.text_mode();
    Ok(())
}

fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} draw {}<options>{1}",
        csi_title, csi_reset, csi_option
    );
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!(
        "  {0}-t{1}, {0}--text <text>{1}         Draw <text>",
        csi_option, csi_reset
    );
    println!(
        "  {0}-e{1}, {0}--execute <command>{1}   Draw output of <command>",
        csi_option, csi_reset
    );
    println!(
        "  {0}-i{1}, {0}--interval <seconds>{1}  Run every <seconds>",
        csi_option, csi_reset
    );
    println!(
        "  {0}-c{1}, {0}--center{1}              Center text",
        csi_option, csi_reset
    );
}
