/// The standard color palette in VGA text mode
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black      = 0x0,
    Blue       = 0x1,
    Green      = 0x2,
    Cyan       = 0x3,
    Red        = 0x4,
    Magenta    = 0x5,
    Brown      = 0x6,
    LightGray  = 0x7,
    DarkGray   = 0x8,
    LightBlue  = 0x9,
    LightGreen = 0xA,
    LightCyan  = 0xB,
    LightRed   = 0xC,
    Pink       = 0xD,
    Yellow     = 0xE,
    White      = 0xF,
}

const COLORS: [Color; 16] = [
    Color::Black,
    Color::Blue,
    Color::Green,
    Color::Cyan,
    Color::Red,
    Color::Magenta,
    Color::Brown,
    Color::LightGray,
    Color::DarkGray,
    Color::LightBlue,
    Color::LightGreen,
    Color::LightCyan,
    Color::LightRed,
    Color::Pink,
    Color::Yellow,
    Color::White,
];

pub fn colors() -> [Color; 16] {
    COLORS
}

pub fn from_index(index: usize) -> Color {
    COLORS[index]
}

pub fn from_ansi(code: u8) -> Color {
    match code {
        30 => Color::Black,
        31 => Color::Red,
        32 => Color::Green,
        33 => Color::Brown,
        34 => Color::Blue,
        35 => Color::Magenta,
        36 => Color::Cyan,
        37 => Color::LightGray,
        90 => Color::DarkGray,
        91 => Color::LightRed,
        92 => Color::LightGreen,
        93 => Color::Yellow,
        94 => Color::LightBlue,
        95 => Color::Pink,
        96 => Color::LightCyan,
        97 => Color::White,
        _ => Color::Black, // TODO: Error
    }
}

impl Color {
    pub fn to_vga_reg(&self) -> u8 {
        match self {
            Color::Black      => 0x00,
            Color::Blue       => 0x01,
            Color::Green      => 0x02,
            Color::Cyan       => 0x03,
            Color::Red        => 0x04,
            Color::Magenta    => 0x05,
            Color::Brown      => 0x14,
            Color::LightGray  => 0x07,
            Color::DarkGray   => 0x38,
            Color::LightBlue  => 0x39,
            Color::LightGreen => 0x3A,
            Color::LightCyan  => 0x3B,
            Color::LightRed   => 0x3C,
            Color::Pink       => 0x3D,
            Color::Yellow     => 0x3E,
            Color::White      => 0x3F,
        }
    }
}
