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

const WIDTH: usize = 320;
const HEIGHT: usize = 200;

fn mandelbrot(buffer: &mut [u8], x_offset: f64, y_offset: f64, zoom: f64) {
    let n = 256; // Max number of iterations
    let x_scale = 3.0 / (zoom * WIDTH as f64);
    let y_scale = 2.0 / (zoom * HEIGHT as f64);
    for py in 0..HEIGHT {
        for px in 0..WIDTH {
            // Map pixel position to complex plane
            let x0 = x_offset + ((px as f64) - (WIDTH as f64) / 2.0) * x_scale;
            let y0 = y_offset + ((py as f64) - (HEIGHT as f64) / 2.0) * y_scale;

            // Compute whether the point is in the Mandelbrot Set
            let mut x = 0.0;
            let mut y = 0.0;
            let mut x2 = 0.0;
            let mut y2 = 0.0;
            let mut i = 0;


            // Cardioid check
            let q = libm::pow(x0 - 0.25, 2.0) + libm::pow(y0, 2.0);
            if q * (q + (x0 - 0.25)) <= 0.25 * libm::pow(y0, 2.0) {
                buffer[py * 320 + px] = 0;
                continue;
            }

            // Period-2 bulb check
            if libm::pow(x0 + 1.0, 2.0) + libm::pow(y0, 2.0) <= 0.0625 {
                buffer[py * 320 + px] = 0;
                continue;
            }

            while i < n {
                y = 2.0 * x * y + y0;
                x = x2 - y2 + x0;
                x2 = x * x;
                y2 = y * y;

                if x2 + y2 > 4.0 {
                    break;
                }

                i += 1;
            }

            buffer[py * 320 + px] = if i < n {
                // Color the pixel based on the number of iterations
                (i % 255) as u8
            } else {
                // Or black for points that are in the set
                0
            };
        }
    }
}

fn main(args: &[&str]) {
    let mut color = false;
    let mut x = -0.5;
    let mut y = 0.0;
    let mut z = 1.0;

    let n = args.len();
    let mut i = 0;
    while i < n {
        match args[i] {
            "-h" | "--help" => {
                help();
                return;
            }
            "-c" | "--color" => {
                color = true;
            }
            "-x" if i < n - 1 => {
                i += 1;
                x = args[i].parse().unwrap_or(x);
            }
            "-y" if i < n - 1 => {
                i += 1;
                y = args[i].parse().unwrap_or(y);
            }
            "-z" | "--zoom" if i < n - 1 => {
                i += 1;
                z = args[i].parse().unwrap_or(z);
            }
            _ => {}
        }
        i += 1;
    }

    vga::graphic_mode();
    fs::write("/dev/vga/palette", &palette(color)).ok();

    let mut escape = false;
    let mut csi = false;
    let mut buffer = [0; WIDTH * HEIGHT];
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
            ' ' => { // Space: zoom in
                z *= 1.5;
            }
            '\x08' => { // Backspace: zoom out
                z /= 1.5;
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
