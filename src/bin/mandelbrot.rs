#![no_std]
#![no_main]

extern crate alloc;

//use moros::print;
use moros::api::fs;
use moros::api::io;
use moros::api::vga;
use moros::entry_point;

entry_point!(main);

fn palette() -> [u8; 768] {
    let mut palette = [0; 768];
    for i in 0..256 {
        palette[i * 3 + 0] = i as u8; // R
        palette[i * 3 + 1] = i as u8; // G
        palette[i * 3 + 2] = i as u8; // B
    }
    palette
}

fn mandelbrot() -> [u8; 320 * 200] {
    let mut buffer = [0; 320 * 200];
    let (x_min, x_max) = (-2.0, 1.0);
    let (y_min, y_max) = (-1.0, 1.0);
    for y in 0..200 {
        for x in 0..320 {
            // Map pixel position to complex plane
            let cx = x_min + (x as f32 / 320.0) * (x_max - x_min);
            let cy = y_min + (y as f32 / 200.0) * (y_max - y_min);

            // Compute whether the point is in the Mandelbrot Set
            let mut zx = 0.0;
            let mut zy = 0.0;
            let mut i = 0;
            let n = 255;

            while zx * zx + zy * zy <= 4.0 && i < n {
                let tmp = zx * zx - zy * zy + cx;
                zy = 2.0 * zx * zy + cy;
                zx = tmp;
                i += 1;
            }

            // Color the pixel based on the number of iterations
            buffer[y * 320 + x] = i as u8;
        }
    }
    buffer
}

fn wait() {
    while io::stdin().read_char().is_none() {
        x86_64::instructions::hlt();
    }
}

fn main(_args: &[&str]) {
    vga::graphic_mode();
    fs::write("/dev/vga/palette", &palette()).ok();
    fs::write("/dev/vga/buffer", &mandelbrot()).ok();
    wait();
    vga::text_mode();
}
