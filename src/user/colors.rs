use alloc::format;
use crate::{print, user};

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    let csi_reset = "\x1b[0m";

    for i in 30..38 {
        let csi_color = format!("\x1b[{};40m", i);
        print!(" {}{:3}{}", csi_color, i, csi_reset);
    }
    print!("\n");
    for i in 90..98 {
        let csi_color = format!("\x1b[{};40m", i);
        print!(" {}{:3}{}", csi_color, i, csi_reset);
    }
    print!("\n");
    for i in 40..48 {
        let csi_color = format!("\x1b[30;{}m", i);
        print!(" {}{:3}{}", csi_color, i, csi_reset);
    }
    print!("\n");
    for i in 100..108 {
        let csi_color = format!("\x1b[30;{}m", i);
        print!(" {}{:3}{}", csi_color, i, csi_reset);
    }
    print!("\n");

    user::shell::ExitCode::CommandSuccessful
}

