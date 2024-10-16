#![no_std]
#![no_main]

extern crate alloc;

use moros::{print, println};
use moros::api::console::Style;
use moros::api::fs;
use moros::api::io;
use moros::api::vga;
use moros::entry_point;

entry_point!(main);

fn palette(color: bool) -> [u8; 768] {
    let mut palette = [0; 768];
    for i in 0..256 {
        let mut r = i as u8;
        let mut g = i as u8;
        let mut b = i as u8;
        if i > 0 && color {
            let t = i as f32 / 255.0;
            r = (9.0 * (1.0 - t) * t * t * t * 255.0) as u8;
            g = (15.0 * (1.0 - t) * (1.0 - t) * t * t * 255.0) as u8;
            b = (8.5 * (1.0 - t) * (1.0 - t) * (1.0 - t) * t * 255.0) as u8;
        }
        palette[i * 3 + 0] = r;
        palette[i * 3 + 1] = g;
        palette[i * 3 + 2] = b;
    }
    palette
}

fn mandelbrot(buffer: &mut [u8], x_offset: f64, y_offset: f64, zoom: f64) {
    let x_scale = 3.0 / 320.0 / zoom;
    let y_scale = 2.0 / 200.0 / zoom;
    for py in 0..200 {
        for px in 0..320 {
            // Map pixel position to complex plane
            let x0 = px as f64 * x_scale - 2.0 + x_offset;
            let y0 = py as f64 * y_scale - 1.0 + y_offset;

            // Compute whether the point is in the Mandelbrot Set
            let mut x = 0.0;
            let mut y = 0.0;
            let mut i = 0;
            let n = 255;

            while x * x + y * y <= 4.0 && i < n {
                let tmp = x * x - y * y + x0;
                y = 2.0 * x * y + y0;
                x = tmp;
                i += 1;
            }

            // Color the pixel based on the number of iterations
            buffer[py * 320 + px] = (i % 256) as u8;
        }
    }
}

fn main(args: &[&str]) {
    let mut color = false;
    for arg in args {
        match *arg {
            "-h" | "--help" => {
                help();
                return;
            }
            "-c" | "--color" => {
                color = true;
            }
            _ => {}
        }
    }

    vga::graphic_mode();
    fs::write("/dev/vga/palette", &palette(color)).ok();

    let mut x = 0.0;
    let mut y = 0.0;
    let mut z = 1.0;

    let mut escape = false;
    let mut csi = false;
    let mut buffer = [0; 320 * 200];
    loop {
        mandelbrot(&mut buffer, x, y, z);
        fs::write("/dev/vga/buffer", &buffer).ok();
        let c = io::stdin().read_char().unwrap_or('\0');
        match c {
            'q' | '\x11' | '\x03' => { // Ctrl Q or Ctrl C
                break;
            }
            '\0' => {
                continue;
            }
            '\x1B' => { // ESC
                escape = true;
                continue;
            }
            '[' if escape => {
                csi = true;
                continue;
            }
            'A' if csi => { // Arrow Up
                y -= 0.2 / z;
            }
            'B' if csi => { // Arrow Down
                y += 0.2 / z;
            }
            'C' if csi => { // Arrow Right
                x += 0.2 / z;
            }
            'D' if csi => { // Arrow Left
                x -= 0.2 / z;
            }
            ' ' => { // Space
                let x_center = x + (1.5 / z);
                let y_center = y + (1.0 / z);
                z *= 1.5; // Increase zoom
                x = x_center - (1.5 / z);
                y = y_center - (1.0 / z);
            }
            '\x08' => { // Backspace
                let x_center = x + (1.5 / z);
                let y_center = y + (1.0 / z);
                z /= 1.5; // Increase zoom
                x = x_center - (1.5 / z);
                y = y_center - (1.0 / z);
            }
            _ => {}
        }
    }

    vga::text_mode();
}

fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} mandelbrot {}<options>{1}",
        csi_title, csi_reset, csi_option
    );
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!(
        "  {0}-c{1}, {0}--color{1}    Colorize output",
        csi_option, csi_reset
    );
}
