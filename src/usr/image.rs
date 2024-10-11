use crate::api::console::Style;
use crate::api::fs;
use crate::api::syscall;
use crate::api::process::ExitCode;
use crate::api::font::Font;
use crate::api::vga::palette;
use crate::usr::shell;
use crate::sys;

use alloc::vec::Vec;
use alloc::string::{String, ToString};

use vga::writers::{
    Graphics320x200x256,
    Graphics640x480x16,
    GraphicsWriter,
    PrimitiveDrawing,
};

const PALETTE: [(u8, u8, u8); 256] = [
    (  0,   0,   0), // 0x00
    (  0,   0, 170), // 0x01
    (  0, 170,   0), // 0x02
    (  0, 170, 170), // 0x03
    (170,   0,   0), // 0x04
    (170,   0, 170), // 0x05
    (170,  85,   0), // 0x06
    (170, 170, 170), // 0x07
    ( 85,  85,  85), // 0x08
    ( 85,  85, 255), // 0x09
    ( 85, 255,  85), // 0x0A
    ( 85, 255, 255), // 0x0B
    (255,  85,  85), // 0x0C
    (255,  85, 255), // 0x0D
    (255, 255,  85), // 0x0E
    (255, 255, 255), // 0x0F
    (  0,   0,   0), // 0x10
    ( 16,  16,  16), // 0x11
    ( 32,  32,  32), // 0x12
    ( 53,  53,  53), // 0x13
    ( 69,  69,  69), // 0x14
    ( 85,  85,  85), // 0x15
    (101, 101, 101), // 0x16
    (117, 117, 117), // 0x17
    (138, 138, 138), // 0x18
    (154, 154, 154), // 0x19
    (170, 170, 170), // 0x1A
    (186, 186, 186), // 0x1B
    (202, 202, 202), // 0x1C
    (223, 223, 223), // 0x1D
    (239, 239, 239), // 0x1E
    (255, 255, 255), // 0x1F
    (  0,   0, 255), // 0x20
    ( 65,   0, 255), // 0x21
    (130,   0, 255), // 0x22
    (190,   0, 255), // 0x23
    (255,   0, 255), // 0x24
    (255,   0, 190), // 0x25
    (255,   0, 130), // 0x26
    (255,   0,  65), // 0x27
    (255,   0,   0), // 0x28
    (255,  65,   0), // 0x29
    (255, 130,   0), // 0x2A
    (255, 190,   0), // 0x2B
    (255, 255,   0), // 0x2C
    (190, 255,   0), // 0x2D
    (130, 255,   0), // 0x2E
    ( 65, 255,   0), // 0x2F
    (  0, 255,   0), // 0x30
    (  0, 255,  65), // 0x31
    (  0, 255, 130), // 0x32
    (  0, 255, 190), // 0x33
    (  0, 255, 255), // 0x34
    (  0, 190, 255), // 0x35
    (  0, 130, 255), // 0x36
    (  0,  65, 255), // 0x37
    (130, 130, 255), // 0x38
    (158, 130, 255), // 0x39
    (190, 130, 255), // 0x3A
    (223, 130, 255), // 0x3B
    (255, 130, 255), // 0x3C
    (255, 130, 223), // 0x3D
    (255, 130, 190), // 0x3E
    (255, 130, 158), // 0x3F
    (255, 130, 130), // 0x40
    (255, 158, 130), // 0x41
    (255, 190, 130), // 0x42
    (255, 223, 130), // 0x43
    (255, 255, 130), // 0x44
    (223, 255, 130), // 0x45
    (190, 255, 130), // 0x46
    (158, 255, 130), // 0x47
    (130, 255, 130), // 0x48
    (130, 255, 158), // 0x49
    (130, 255, 190), // 0x4A
    (130, 255, 223), // 0x4B
    (130, 255, 255), // 0x4C
    (130, 223, 255), // 0x4D
    (130, 190, 255), // 0x4E
    (130, 158, 255), // 0x4F
    (186, 186, 255), // 0x50
    (202, 186, 255), // 0x51
    (223, 186, 255), // 0x52
    (239, 186, 255), // 0x53
    (255, 186, 255), // 0x54
    (255, 186, 239), // 0x55
    (255, 186, 223), // 0x56
    (255, 186, 202), // 0x57
    (255, 186, 186), // 0x58
    (255, 202, 186), // 0x59
    (255, 223, 186), // 0x5A
    (255, 239, 186), // 0x5B
    (255, 255, 186), // 0x5C
    (239, 255, 186), // 0x5D
    (223, 255, 186), // 0x5E
    (202, 255, 186), // 0x5F
    (186, 255, 186), // 0x60
    (186, 255, 202), // 0x61
    (186, 255, 223), // 0x62
    (186, 255, 239), // 0x63
    (186, 255, 255), // 0x64
    (186, 239, 255), // 0x65
    (186, 223, 255), // 0x66
    (186, 202, 255), // 0x67
    (  0,   0, 113), // 0x68
    ( 28,   0, 113), // 0x69
    ( 57,   0, 113), // 0x6A
    ( 85,   0, 113), // 0x6B
    (113,   0, 113), // 0x6C
    (113,   0,  85), // 0x6D
    (113,   0,  57), // 0x6E
    (113,   0,  28), // 0x6F
    (113,   0,   0), // 0x70
    (113,  28,   0), // 0x71
    (113,  57,   0), // 0x72
    (113,  85,   0), // 0x73
    (113, 113,   0), // 0x74
    ( 85, 113,   0), // 0x75
    ( 57, 113,   0), // 0x76
    ( 28, 113,   0), // 0x77
    (  0, 113,   0), // 0x78
    (  0, 113,  28), // 0x79
    (  0, 113,  57), // 0x7A
    (  0, 113,  85), // 0x7B
    (  0, 113, 113), // 0x7C
    (  0,  85, 113), // 0x7D
    (  0,  57, 113), // 0x7E
    (  0,  28, 113), // 0x7F
    ( 57,  57, 113), // 0x80
    ( 69,  57, 113), // 0x81
    ( 85,  57, 113), // 0x82
    ( 97,  57, 113), // 0x83
    (113,  57, 113), // 0x84
    (113,  57,  97), // 0x85
    (113,  57,  85), // 0x86
    (113,  57,  69), // 0x87
    (113,  57,  57), // 0x88
    (113,  69,  57), // 0x89
    (113,  85,  57), // 0x8A
    (113,  97,  57), // 0x8B
    (113, 113,  57), // 0x8C
    ( 97, 113,  57), // 0x8D
    ( 85, 113,  57), // 0x8E
    ( 69, 113,  57), // 0x8F
    ( 57, 113,  57), // 0x90
    ( 57, 113,  69), // 0x91
    ( 57, 113,  85), // 0x92
    ( 57, 113,  97), // 0x93
    ( 57, 113, 113), // 0x94
    ( 57,  97, 113), // 0x95
    ( 57,  85, 113), // 0x96
    ( 57,  69, 113), // 0x97
    ( 81,  81, 113), // 0x98
    ( 89,  81, 113), // 0x99
    ( 97,  81, 113), // 0x9A
    (105,  81, 113), // 0x9B
    (113,  81, 113), // 0x9C
    (113,  81, 105), // 0x9D
    (113,  81,  97), // 0x9E
    (113,  81,  89), // 0x9F
    (113,  81,  81), // 0xA0
    (113,  89,  81), // 0xA1
    (113,  97,  81), // 0xA2
    (113, 105,  81), // 0xA3
    (113, 113,  81), // 0xA4
    (105, 113,  81), // 0xA5
    ( 97, 113,  81), // 0xA6
    ( 89, 113,  81), // 0xA7
    ( 81, 113,  81), // 0xA8
    ( 81, 113,  89), // 0xA9
    ( 81, 113,  97), // 0xAA
    ( 81, 113, 105), // 0xAB
    ( 81, 113, 113), // 0xAC
    ( 81, 105, 113), // 0xAD
    ( 81,  97, 113), // 0xAE
    ( 81,  89, 113), // 0xAF
    (  0,   0,  65), // 0xB0
    ( 16,   0,  65), // 0xB1
    ( 32,   0,  65), // 0xB2
    ( 49,   0,  65), // 0xB3
    ( 65,   0,  65), // 0xB4
    ( 65,   0,  49), // 0xB5
    ( 65,   0,  32), // 0xB6
    ( 65,   0,  16), // 0xB7
    ( 65,   0,   0), // 0xB8
    ( 65,  16,   0), // 0xB9
    ( 65,  32,   0), // 0xBA
    ( 65,  49,   0), // 0xBB
    ( 65,  65,   0), // 0xBC
    ( 49,  65,   0), // 0xBD
    ( 32,  65,   0), // 0xBE
    ( 16,  65,   0), // 0xBF
    (  0,  65,   0), // 0xC0
    (  0,  65,  16), // 0xC1
    (  0,  65,  32), // 0xC2
    (  0,  65,  49), // 0xC3
    (  0,  65,  65), // 0xC4
    (  0,  49,  65), // 0xC5
    (  0,  32,  65), // 0xC6
    (  0,  16,  65), // 0xC7
    ( 32,  32,  65), // 0xC8
    ( 40,  32,  65), // 0xC9
    ( 49,  32,  65), // 0xCA
    ( 57,  32,  65), // 0xCB
    ( 65,  32,  65), // 0xCC
    ( 65,  32,  57), // 0xCD
    ( 65,  32,  49), // 0xCE
    ( 65,  32,  40), // 0xCF
    ( 65,  32,  32), // 0xD0
    ( 65,  40,  32), // 0xD1
    ( 65,  49,  32), // 0xD2
    ( 65,  57,  32), // 0xD3
    ( 65,  65,  32), // 0xD4
    ( 57,  65,  32), // 0xD5
    ( 49,  65,  32), // 0xD6
    ( 40,  65,  32), // 0xD7
    ( 32,  65,  32), // 0xD8
    ( 32,  65,  40), // 0xD9
    ( 32,  65,  49), // 0xDA
    ( 32,  65,  57), // 0xDB
    ( 32,  65,  65), // 0xDC
    ( 32,  57,  65), // 0xDD
    ( 32,  49,  65), // 0xDE
    ( 32,  40,  65), // 0xDF
    ( 45,  45,  65), // 0xE0
    ( 49,  45,  65), // 0xE1
    ( 53,  45,  65), // 0xE2
    ( 61,  45,  65), // 0xE3
    ( 65,  45,  65), // 0xE4
    ( 65,  45,  61), // 0xE5
    ( 65,  45,  53), // 0xE6
    ( 65,  45,  49), // 0xE7
    ( 65,  45,  45), // 0xE8
    ( 65,  49,  45), // 0xE9
    ( 65,  53,  45), // 0xEA
    ( 65,  61,  45), // 0xEB
    ( 65,  65,  45), // 0xEC
    ( 61,  65,  45), // 0xED
    ( 53,  65,  45), // 0xEE
    ( 49,  65,  45), // 0xEF
    ( 45,  65,  45), // 0xF0
    ( 45,  65,  49), // 0xF1
    ( 45,  65,  53), // 0xF2
    ( 45,  65,  61), // 0xF3
    ( 45,  65,  65), // 0xF4
    ( 45,  61,  65), // 0xF5
    ( 45,  53,  65), // 0xF6
    ( 45,  49,  65), // 0xF7
    (  0,   0,   0), // 0xF8
    (  0,   0,   0), // 0xF9
    (  0,   0,   0), // 0xFA
    (  0,   0,   0), // 0xFB
    (  0,   0,   0), // 0xFC
    (  0,   0,   0), // 0xFD
    (  0,   0,   0), // 0xFE
    (  0,   0,   0), // 0xFF
];

const VGA_FRAMEBUFFER_ADDRESS: usize = 0xA0000;
const SCREEN_WIDTH: usize = 320;
const SCREEN_HEIGHT: usize = 200;
const SCREEN_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

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
            debug!("width: {}", bmp.width);
            debug!("height: {}", bmp.height);
            debug!("pixels.len(): {}", bmp.pixels.len());
            // debug!("palette: {:#?}", bmp.palette);
            /*
            for (i, (r, g, b)) in bmp.palette.iter().enumerate() {
                debug!("{}: ({}, {}, {})", i, r, g, b);
            }
            return Ok(());
            */
            sys::vga::set_320x200_mode();
            let mode = Graphics320x200x256::new();
            for (i, (r, g, b)) in bmp.palette.iter().enumerate() {
                sys::vga::set_palette_color(i, *r, *g, *b);
            }
            mode.clear_screen(0x00);
            let width = bmp.width as usize;
            let height = bmp.height as usize;

            /*
            for y in 0..height {
                for x in 0..width {
                    let c = (x / 3) % 256;
                    mode.set_pixel(x, y, c as u8);
                }
            }
            */

            // BMP rows are padded to multiples of 4 bytes
            let row_padding = (4 - (width % 4)) % 4;

            for y in 0..height {
                for x in 0..width {
                    // BMP stores images bottom-up
                    let bmp_y = height - 1 - y;
                    let i = (bmp_y * (width + row_padding) + x) as usize;
                    if i < bmp.pixels.len() {
                        let c = bmp.pixels[i];
                        mode.set_pixel(x, y, c);
                    }
                }
            }
            while !sys::console::end_of_text() {
                x86_64::instructions::hlt();
            }
            sys::vga::set_80x25_mode();
            vga_reset();
        } else {
            error!("Could not parse BMP");
            return Err(ExitCode::Failure);
        }
    }

    /*
    if let Ok(buf) = read_ppm(path) {
        /*
        for y in 0..200 {
            for x in 0..320 {
                let i = y * 320 + x;
                let r = buf[i * 3];
                let g = buf[i * 3 + 1];
                let b = buf[i * 3 + 2];
                let mut c = None;
                for (i, (r1, g1, b1)) in PALETTE.iter().enumerate() {
                    if r == *r1 && g == *g1 && b == *b1 {
                        c = Some(i as u8);
                        break;
                    }
                }
                if c.is_none() {
                    error!("Color not found");
                    return Ok(());
                }
            }
        }
        */
        sys::vga::set_320x200_mode();
        /*
        let framebuffer = unsafe {
            core::slice::from_raw_parts_mut(VGA_FRAMEBUFFER_ADDRESS as *mut u8, SCREEN_SIZE)
        };
        framebuffer.copy_from_slice(&buf);
        */
        let mode = Graphics320x200x256::new();
        mode.clear_screen(0x00);

        for (i, (r, g, b)) in PALETTE.iter().enumerate() {
            sys::vga::set_palette_color(i, *r, *g, *b);
        }


        for y in 0..200 {
            for x in 0..320 {
                let i = y * 320 + x;
                let r = buf[i * 3];
                let g = buf[i * 3 + 1];
                let b = buf[i * 3 + 2];
                //let c = ((r + g + b) / 3) as u8;
                //let c = (((r / 16) + (g / 16) + (b / 16)) / 3) as u8;
                for (i, (r1, g1, b1)) in PALETTE.iter().enumerate() {
                    if r == *r1 && g == *g1 && b == *b1 {
                        let c = i as u8;
                        mode.set_pixel(x, y, c);
                        break;
                    }
                }
            }
        }

        loop {
            if sys::console::end_of_text() {
                break;
            }
        }
        //syscall::sleep(5.0);
        sys::vga::set_80x25_mode();
        vga_reset();
    }

    /*
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
    */
    */
    Ok(())
}

fn read_ppm(path: &str) -> Result<Vec<u8>, ()> {
    let buf = fs::read_to_bytes(&path)?;
    let n = buf.len();
    let i = 0;
    let mut j = 0;
    while j < n {
        if buf[j] == b'\n' {
            break;
        }
        j += 1;
    }
    let line = String::from_utf8_lossy(&buf[i..j]);
    j += 1;
    debug!("line: '{}' (1)", line);
    if line != "P6" {
        error!("Invalid format");
        return Err(());
    }

    let mut dims: Vec<usize> = Vec::with_capacity(3);
    while j < n {
        let i = j;
        while j < n {
            if buf[j] == b'\n' {
                break;
            }
            j += 1;
        }
        let line = String::from_utf8_lossy(&buf[i..j]);
        j += 1;
        debug!("line: '{}' (2)", line);
        if line.starts_with('#') {
            continue;
        }
        for word in line.split_whitespace() {
            if let Ok(dim) = word.parse() {
                dims.push(dim);
            }
        }
        if dims.len() == 3 {
            break;
        }
    }

    let width = dims[0];
    let height = dims[1];
    let colors = dims[2];

    debug!("dims: {:?}", dims);
    if width != 320 && height != 200 && colors != 255 {
        error!("Invalid header dimensions");
        return Err(());
    }

    let pixels = buf[j..n].to_vec();
    debug!("pixels.len(): {}", pixels.len());
    if pixels.len() != 320 * 200 * 3 {
        error!("Invalid buffer dimensions");
        return Err(());
    }

    Ok(pixels)
}

use core::mem::size_of;

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
    debug!("{:?}", dib_header);

    /*
    let palette_offset = size_of::<BmpHeader>() + size_of::<DibHeader>();
    let palette_size = 256 * 4; // 256 colors, 4 bytes per color (BGRA)
    if data.len() < palette_offset + palette_size {
        return Err("Invalid BMP file: too small for palette".to_string());
    }
    */

    let pixels_offset = bmp_header.data_offset as usize;
    let palette_size = 256 * 4; // 256 colors, 4 bytes per color (BGRA)
    let palette_offset = pixels_offset - palette_size;

    /*
    if data.len() < pixels_offset || palette_offset < size_of::<BmpHeader>() + size_of::<DibHeader>() {
        return Err("Invalid BMP file: incorrect data offset or palette size");
    }
    */

    let mut palette = [(0, 0, 0); 256];
    //for (i, chunk) in data[palette_offset..palette_offset + palette_size].chunks(4).enumerate() {
    for (i, chunk) in data[palette_offset..pixels_offset].chunks(4).enumerate() {
        // Convert BGRA to RGB
        palette[i] = (chunk[2], chunk[1], chunk[0]);
    }

    //let pixels_offset = bmp_header.data_offset as usize;
    /*
    if data.len() < pixels_offset {
        return Err("Invalid BMP file: data offset out of bounds".to_string());
    }
    */

    let pixels = data[pixels_offset..].to_vec();
    let width = dib_header.width as u32;
    let height = dib_header.height.abs() as u32;
    if pixels.len() != (width * height) as usize {
        return Err("Invalid BMP file: wrong pixels count".to_string());
    }

    Ok(BmpInfo { width, height, palette, pixels })
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
        "{}Usage:{} image {}<file>{1}",
        csi_title, csi_reset, csi_option
    );
}

