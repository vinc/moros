use lazy_static::lazy_static;
use vga::fonts::VgaFont;

mod psf;

// https://www.zap.org.au/projects/console-fonts-zap/src/zap-light16.psf
// https://www.zap.org.au/projects/console-fonts-zap/src/zap-vga16.psf
// https://www.zap.org.au/projects/console-fonts-distributed/psftx-debian-10.5/Lat15-Terminus16.psf

pub fn vga_font(name: &str) -> Option<VgaFont> {
    match name {
        "lat15-terminus-8x16" => Some(from_psf_font(*LAT15_TERMINUS_8X16)),
        "zap-light-8x16"      => Some(from_psf_font(*ZAP_LIGHT_8X16)),
        "zap-vga-8x16"        => Some(from_psf_font(*ZAP_VGA_8X16)),
        _                     => None,
    }
}

fn from_psf_font(font: psf::Font) -> VgaFont {
    VgaFont {
        characters: font.size,
        character_height: font.height as u16,
        font_data: font.data,
    }
}

lazy_static! {
    static ref LAT15_TERMINUS_8X16: psf::Font = psf::from_bytes(include_bytes!("lat15-terminus-8x16.psf")).unwrap();
    static ref ZAP_LIGHT_8X16: psf::Font = psf::from_bytes(include_bytes!("zap-light-8x16.psf")).unwrap();
    static ref ZAP_VGA_8X16: psf::Font = psf::from_bytes(include_bytes!("zap-vga-8x16.psf")).unwrap();
}
