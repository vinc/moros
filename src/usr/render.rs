use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::usr::shell;
use crate::sys;

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use core::mem::size_of;

const FRAMEBUFFER: usize = 0xA0000;
const WIDTH: usize = 320;
const HEIGHT: usize = 200;

#[derive(Debug)]
#[repr(C, packed)]
struct BmpHeader {
    signature: [u8; 2],
    file_size: u32,
    reserved: u32,
    data_offset: u32,
}

#[derive(Debug)]
#[repr(C, packed)]
struct DibHeader {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bits_per_pixel: u16,
    compression: u32,
    image_size: u32,
    x_pixels_per_meter: i32,
    y_pixels_per_meter: i32,
    colors_used: u32,
    colors_important: u32,
}

#[derive(Debug)]
struct BmpInfo {
    pub width: u32,
    pub height: u32,
    pub palette: [(u8, u8, u8); 256],
    pub pixels: Vec<u8>,
}

fn parse_bmp(data: &[u8]) -> Result<BmpInfo, String> {
    if data.len() < size_of::<BmpHeader>() + size_of::<DibHeader>() {
        return Err("Invalid BMP file: too small".to_string());
    }

    let bmp_header: &BmpHeader = unsafe { &*(data.as_ptr() as *const BmpHeader) };
    if &bmp_header.signature != b"BM" {
        return Err("Invalid BMP signature".to_string());
    }

    let dib_header: &DibHeader = unsafe { &*(data[size_of::<BmpHeader>()..].as_ptr() as *const DibHeader) };
    if dib_header.bits_per_pixel != 8 {
        return Err("Only 8-bit (256 color) BMPs are supported".to_string());
    }

    let pixels_offset = bmp_header.data_offset as usize;
    let palette_size = 256 * 4; // 256 colors, 4 bytes per color (BGRA)
    let palette_offset = pixels_offset - palette_size;

    let mut palette = [(0, 0, 0); 256];
    for (i, chunk) in data[palette_offset..pixels_offset].chunks(4).enumerate() {
        // Convert BGRA to RGB
        palette[i] = (chunk[2], chunk[1], chunk[0]);
    }

    let pixels = data[pixels_offset..].to_vec();
    let width = dib_header.width as u32;
    let height = dib_header.height.abs() as u32;
    if pixels.len() != (width * height) as usize {
        return Err("Invalid BMP file: wrong pixels count".to_string());
    }

    Ok(BmpInfo { width, height, palette, pixels })
}

fn clear() {
    let ptr = FRAMEBUFFER as *mut u8;
    let size = WIDTH * HEIGHT;
    unsafe {
        let buf = core::slice::from_raw_parts_mut(ptr, size);
        buf.fill(0x00);
    }
}

fn reset() {
    shell::exec("shell /ini/palettes/gruvbox-dark.sh").ok();
    shell::exec("read /ini/fonts/zap-light-8x16.psf => /dev/vga/font").ok();
}

fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} image {}<file>{1}",
        csi_title, csi_reset, csi_option
    );
}


pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() == 1 {
        help();
        return Err(ExitCode::UsageError);
    }

    if args[1].starts_with("-h") || args[1].starts_with("--help") {
        help();
        return Ok(());
    }

    let path = args[1];
    if let Ok(buf) = fs::read_to_bytes(&path) {
        if let Ok(bmp) = parse_bmp(&buf) {
            let width = bmp.width as usize;
            let height = bmp.height as usize;
            if width != WIDTH || height != HEIGHT {
                error!("Unsupported BMP size");
                return Err(ExitCode::Failure);
            }
            let size = width * height;
            let mut img = Vec::with_capacity(size);

            // BMP rows are padded to multiples of 4 bytes
            let row_padding = (4 - (width % 4)) % 4;

            for y in 0..height {
                for x in 0..width {
                    // BMP stores images bottom-up
                    let bmp_y = height - 1 - y;

                    let i = (bmp_y * (width + row_padding) + x) as usize;
                    img.push(bmp.pixels[i]);
                }
            }

            sys::vga::set_320x200_mode();
            clear();

            // Load palette
            for (i, (r, g, b)) in bmp.palette.iter().enumerate() {
                sys::vga::set_palette_color(i, *r, *g, *b);
            }

            // Display image
            let src = img.as_ptr();
            let dst = FRAMEBUFFER as *mut u8;
            unsafe {
                core::ptr::copy_nonoverlapping(src, dst, size);
            }

            while !sys::console::end_of_text() {
                x86_64::instructions::hlt();
            }

            clear();
            sys::vga::set_80x25_mode();
            reset();
            print!("\x1b[2J\x1b[1;1H"); // Clear screen and move to top
        } else {
            error!("Could not parse BMP");
            return Err(ExitCode::Failure);
        }
    }
    Ok(())
}
