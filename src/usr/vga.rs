use crate::api::console::Style;
use crate::api::fs;
use crate::api::syscall;
use crate::api::process::ExitCode;
use crate::api::font::Font;
use crate::api::vga::palette;
use crate::usr::shell;
use crate::sys;

use core::convert::TryFrom;

use vga::writers::{
    Graphics320x200x256,
    Graphics640x480x16,
    GraphicsWriter,
    PrimitiveDrawing,
};

// TODO: Remove this command in the next version of MOROS
pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() == 1 {
        help();
        return Err(ExitCode::UsageError);
    }

    match args[1] {
        "-h" | "--help" => {
            help();
            Ok(())
        }
        "set" => {
            if args.len() == 4 && args[2] == "font" {
                warning!("Use VGA font device");
                if let Ok(buf) = fs::read_to_bytes(args[3]) {
                    if let Ok(font) = Font::try_from(buf.as_slice()) {
                        sys::vga::set_font(&font);
                        Ok(())
                    } else {
                        error!("Could not parse font file");
                        Err(ExitCode::Failure)
                    }
                } else {
                    error!("Could not read font file");
                    Err(ExitCode::Failure)
                }
            } else if args.len() == 4 && args[2] == "palette" {
                warning!("Use ANSI OSC palette sequence");
                if let Ok(csv) = fs::read_to_string(args[3]) {
                    if let Ok(palette) = palette::from_csv(&csv) {
                        sys::vga::set_palette(palette);
                        Ok(())
                    } else {
                        error!("Could not parse palette file");
                        Err(ExitCode::Failure)
                    }
                } else {
                    error!("Could not read palette file");
                    Err(ExitCode::Failure)
                }
            } else if args.len() == 4 && args[2] == "mode" {
                match args[3] {
                    "320x200" => {
                        sys::vga::set_320x200_mode();
                        let black = 0x00;
                        let white = 0x07;
                        let mode = Graphics320x200x256::new();
                        mode.clear_screen(black);
                        mode.draw_line((60, 20), (60, 180), white);
                        mode.draw_line((60, 20), (260, 20), white);
                        mode.draw_line((60, 180), (260, 180), white);
                        mode.draw_line((260, 180), (260, 20), white);
                        mode.draw_line((60, 40), (260, 40), white);
                        for (offset, character) in "Hello World!".chars().enumerate() {
                            mode.draw_character(118 + offset * 8, 27, character, white);
                        }
                        syscall::sleep(5.0);
                        sys::vga::set_80x25_mode();
                        vga_reset();
                    }
                    "640x480" => {
                        sys::vga::set_640x480_mode();
                        use vga::colors::Color16;
                        let black = Color16::Black;
                        let white = Color16::White;
                        let mode = Graphics640x480x16::new();
                        mode.clear_screen(black);
                        mode.draw_line((80, 60), (80, 420), white);
                        mode.draw_line((80, 60), (540, 60), white);
                        mode.draw_line((80, 420), (540, 420), white);
                        mode.draw_line((540, 420), (540, 60), white);
                        mode.draw_line((80, 90), (540, 90), white);
                        for (offset, character) in "Hello World!".chars().enumerate() {
                            mode.draw_character(270 + offset * 8, 72, character, white);
                        }
                        syscall::sleep(5.0);
                        sys::vga::set_80x25_mode();
                        vga_reset();
                    }
                    _ => {}
                }
                Ok(())
            } else {
                error!("Invalid command");
                Err(ExitCode::Failure)
            }
        }
        _ => {
            error!("Invalid command");
            Err(ExitCode::Failure)
        }
    }
}

fn vga_reset() {
    shell::exec("shell /ini/palettes/gruvbox-dark.sh").ok();
    shell::exec("read /ini/fonts/zap-light-8x16.psf => /dev/vga/font").ok();
    shell::exec("clear").ok();
}

fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} vga {}<command>{1}",
        csi_title, csi_reset, csi_option
    );
    println!();
    println!("{}Commands:{}", csi_title, csi_reset);
    println!(
        "  {}set font <file>{}       Set VGA font",
        csi_option, csi_reset
    );
    println!(
        "  {}set palette <file>{}    Set VGA color palette",
        csi_option, csi_reset
    );
}
