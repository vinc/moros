use vga::fonts::VgaFont;

// psf2raw example.psf > example.bin

pub fn vga_font(name: &str) -> Option<&VgaFont> {
    match name {
        "ibm-vga-9x16"        => Some(&IBM_VGA_9X16),
        "lat15-terminus-8x16" => Some(&LAT15_TERMINUS_8X16),
        "zap-light-8x16"      => Some(&ZAP_LIGHT_8X16),
        "zap-vga-8x16"        => Some(&ZAP_VGA_8X16),
        _                     => None,
    }
}

pub const IBM_VGA_9X16: VgaFont = VgaFont {
    characters: 256,
    character_height: 16,
    font_data: include_bytes!("ibm-vga-9x16.bin"),
};

pub const LAT15_TERMINUS_8X16: VgaFont = VgaFont {
    characters: 256,
    character_height: 16,
    font_data: include_bytes!("lat15-terminus-8x16.bin"),
};

pub const ZAP_LIGHT_8X16: VgaFont = VgaFont {
    characters: 256,
    character_height: 16,
    font_data: include_bytes!("zap-light-8x16.bin"),
};

pub const ZAP_VGA_8X16: VgaFont = VgaFont {
    characters: 256,
    character_height: 16,
    font_data: include_bytes!("zap-vga-8x16.bin"),
};
