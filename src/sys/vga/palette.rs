pub struct Palette {
    pub colors: [(u8, u8, u8); 16],
}

impl Palette {
    pub fn default() -> Palette {
        Palette {
            colors: [
                (0x00, 0x00, 0x00), // Black
                (0x00, 0x00, 0x80), // Blue
                (0x00, 0x80, 0x00), // Green
                (0x00, 0x80, 0x80), // Cyan
                (0x80, 0x00, 0x00), // Red
                (0x80, 0x00, 0x80), // Magenta
                (0x80, 0x80, 0x00), // Brown (Dark Yellow)
                (0xC0, 0xC0, 0xC0), // Light Gray
                (0x80, 0x80, 0x80), // Dark Gray (Gray)
                (0x00, 0x00, 0xFF), // Light Blue
                (0x00, 0xFF, 0x00), // Light Green
                (0x00, 0xFF, 0xFF), // Light Cyan
                (0xFF, 0x00, 0x00), // Light Red
                (0xFF, 0x00, 0xFF), // Pink (Light Magenta)
                (0xFF, 0xFF, 0x00), // Yellow (Light Yellow)
                (0xFF, 0xFF, 0xFF), // White
            ],
        }
    }
}
