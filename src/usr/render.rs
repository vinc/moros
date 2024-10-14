use crate::api::console::Style;
use crate::api::fs;
use crate::api::io;
use crate::api::process::ExitCode;
use crate::api::vga;
use crate::sys;

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use core::mem::size_of;

const FRAMEBUFFER: usize = 0xA0000;
const WIDTH: usize = 320;
const HEIGHT: usize = 200;

fn clear() {
    let ptr = FRAMEBUFFER as *mut u8;
    let size = WIDTH * HEIGHT;
    unsafe {
        let buf = core::slice::from_raw_parts_mut(ptr, size);
        buf.fill(0x00);
    }
}

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
        return Err("Invalid BMP file: size too small".to_string());
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
    if pixels_offset < palette_size || data.len() <= pixels_offset {
        return Err("Invalid BMP palette".to_string());
    }
    let palette_offset = pixels_offset - palette_size;

    let mut palette = [(0, 0, 0); 256];
    for (i, bgra) in data[palette_offset..pixels_offset].chunks(4).enumerate() {
        // Convert BGRA to RGB and discard the Alpha layer
        palette[i] = (bgra[2], bgra[1], bgra[0]);
    }

    let pixels = data[pixels_offset..].to_vec();
    let width = dib_header.width as u32;
    let height = dib_header.height.abs() as u32;
    if pixels.len() != (width * height) as usize {
        return Err("Invalid BMP file: wrong pixels count".to_string());
    }

    Ok(BmpInfo { width, height, palette, pixels })
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

#[derive(PartialEq)]
enum Mode {
    Text,
    Graphic,
}

#[derive(PartialEq)]
enum Command {
    Prev,
    Next,
    Quit,
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
            clear();
            vga::text_mode();
            self.mode = Mode::Text;
        }
    }

    pub fn graphic_mode(&mut self) {
        if self.mode == Mode::Text {
            vga::graphic_mode();
            clear();
            self.mode = Mode::Graphic;
        }
    }
}

fn render_bmp(path: &str, config: &mut Config) -> Result<Command, ExitCode> {
    if let Ok(buf) = fs::read_to_bytes(&path) {
        if let Ok(bmp) = parse_bmp(&buf) {
            let width = bmp.width as usize;
            let height = bmp.height as usize;
            if width != WIDTH || height != HEIGHT {
                config.text_mode();
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

            config.graphic_mode();

            // Load palette
            for (i, (r, g, b)) in bmp.palette.iter().enumerate() {
                sys::vga::set_palette(i, *r, *g, *b);
            }

            // Display image
            let src = img.as_ptr();
            let dst = FRAMEBUFFER as *mut u8;
            unsafe {
                core::ptr::copy_nonoverlapping(src, dst, size);
            }

            Ok(read_command())
        } else {
            config.text_mode();
            error!("Could not parse BMP");
            Err(ExitCode::Failure)
        }
    } else {
        config.text_mode();
        error!("Could not read BMP");
        Err(ExitCode::Failure)
    }
}


fn read_command() -> Command {
    let mut escape = false;
    let mut csi = false;
    let mut csi_params = String::new();
    loop {
        let c = io::stdin().read_char().unwrap_or('\0');
        match c {
            'q' | '\x11' | '\x03' => { // Ctrl Q or Ctrl C
                return Command::Quit;
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
                csi_params.clear();
                continue;
            }
            'C' if csi => { // Arrow Right
                return Command::Next;
            }
            'D' if csi => { // Arrow Left
                return Command::Prev;
            }
            c => {
                if csi {
                    csi_params.push(c);
                    continue;
                } else {
                    return Command::Next;
                }
            }
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
    let files = &args[1..];
    let mut config = Config::new();
    let mut i = 0;
    let n = files.len();
    loop {
        match render_bmp(files[i], &mut config) {
            Err(err) => {
                return Err(err);
            }
            Ok(Command::Quit) => {
                break;
            }
            Ok(Command::Next) => {
                i = (i + 1) % n;
            }
            Ok(Command::Prev) => {
                i = (n + i - 1) % n; // Avoid underflow
            }
        }
    }
    config.text_mode();
    Ok(())
}
