use alloc::format;

pub fn main(_args: &[&str]) -> Result<usize, usize> {
    let csi_reset = "\x1b[0m";

    for i in 30..38 {
        let csi_color = format!("\x1b[{};40m", i);
        print!(" {}{:3}{}", csi_color, i, csi_reset);
    }
    println!();
    for i in 90..98 {
        let csi_color = format!("\x1b[{};40m", i);
        print!(" {}{:3}{}", csi_color, i, csi_reset);
    }
    println!();
    for i in 40..48 {
        let csi_color = format!("\x1b[30;{}m", i);
        print!(" {}{:3}{}", csi_color, i, csi_reset);
    }
    println!();
    for i in 100..108 {
        let csi_color = format!("\x1b[30;{}m", i);
        print!(" {}{:3}{}", csi_color, i, csi_reset);
    }
    println!();

    Ok(0)
}
