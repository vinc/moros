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

const WIDTH: usize = 320;
const HEIGHT: usize = 200;

struct Point {
    x: f64,
    y: f64,
}

struct Config {
    offset: Point,
    zoom: f64,
    color: bool,
    n: usize,
}

fn palette(config: &Config) -> [u8; 768] {
    let mut palette = [0; 768];
    for i in 0..256 {
        let mut r = 255 - i as u8;
        let mut g = 255 - i as u8;
        let mut b = 255 - i as u8;
        if config.color {
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

fn mandelbrot(buffer: &mut [u8], config: &Config) {
    let w = WIDTH as f64;
    let h = HEIGHT as f64;
    let x_scale = 3.0 / (config.zoom * w);
    let y_scale = 2.0 / (config.zoom * h);
    for py in 0..HEIGHT {
        for px in 0..WIDTH {
            // Map pixel position to complex plane
            let x0 = config.offset.x + ((px as f64) - w / 2.0) * x_scale;
            let y0 = config.offset.y + ((py as f64) - h / 2.0) * y_scale;

            // Compute whether the point is in the Mandelbrot Set
            let mut x = 0.0;
            let mut y = 0.0;
            let mut x2 = 0.0;
            let mut y2 = 0.0;
            let mut i = 0;


            // Cardioid check
            let q = libm::pow(x0 - 0.25, 2.0) + libm::pow(y0, 2.0);
            if q * (q + (x0 - 0.25)) <= 0.25 * libm::pow(y0, 2.0) {
                buffer[py * 320 + px] = black(config);
                continue;
            }

            // Period-2 bulb check
            if libm::pow(x0 + 1.0, 2.0) + libm::pow(y0, 2.0) <= 0.0625 {
                buffer[py * 320 + px] = black(config);
                continue;
            }

            while i < config.n {
                y = 2.0 * x * y + y0;
                x = x2 - y2 + x0;
                x2 = x * x;
                y2 = y * y;

                if x2 + y2 > 4.0 {
                    break;
                }

                i += 1;
            }

            buffer[py * 320 + px] = if i < config.n {
                // Color the pixel based on the number of iterations
                (i % 256) as u8
            } else {
                // Or black for points that are in the set
                black(config)
            };
        }
    }
}

fn black(config: &Config) -> u8 {
    if config.color {
        0
    } else {
        255
    }
}

fn main(args: &[&str]) {
    let mut config = Config {
        color: false,
        offset: Point { x: -0.5, y: 0.0 },
        zoom: 1.0,
        n: 256,
    };
    let n = args.len();
    let mut i = 0;
    while i < n {
        match args[i] {
            "-h" | "--help" => {
                help();
                return;
            }
            "-c" | "--color" => {
                config.color = true;
            }
            "-x" if i < n - 1 => {
                i += 1;
                config.offset.x = args[i].parse().unwrap_or(config.offset.x);
            }
            "-y" if i < n - 1 => {
                i += 1;
                config.offset.y = args[i].parse().unwrap_or(config.offset.y);
            }
            "-z" | "--zoom" if i < n - 1 => {
                i += 1;
                config.zoom = args[i].parse().unwrap_or(config.zoom);
            }
            "-n" if i < n - 1 => {
                i += 1;
                config.n = args[i].parse().unwrap_or(config.n);
            }
            _ => {}
        }
        i += 1;
    }

    vga::graphic_mode();
    fs::write("/dev/vga/palette", &palette(&config)).ok();

    let mut escape = false;
    let mut csi = false;
    let mut buffer = [0; WIDTH * HEIGHT];
    loop {
        mandelbrot(&mut buffer, &config);
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
                config.offset.y -= 0.2 / config.zoom;
            }
            'B' if csi => { // Arrow Down
                config.offset.y += 0.2 / config.zoom;
            }
            'C' if csi => { // Arrow Right
                config.offset.x += 0.2 / config.zoom;
            }
            'D' if csi => { // Arrow Left
                config.offset.x -= 0.2 / config.zoom;
            }
            ' ' => { // Space: zoom in
                config.zoom *= 1.5;
            }
            '\x08' => { // Backspace: zoom out
                config.zoom /= 1.5;
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
