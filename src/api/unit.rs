use alloc::format;
use alloc::string::String;

#[derive(Clone)]
pub enum SizeUnit {
    None,
    Binary,
    Decimal,
}

impl SizeUnit {
    pub fn format(&self, bytes: usize) -> String {
        match self {
            SizeUnit::None => format!("{}", bytes),
            SizeUnit::Binary => readable_size(bytes, 1024),
            SizeUnit::Decimal => readable_size(bytes, 1000),
        }
    }
}

fn readable_size(bytes: usize, divisor: usize) -> String {
    let units = ["", "K", "M", "G", "T"];
    let d = divisor as f64;
    let mut s = bytes as f64;
    let mut i = 0;
    while s >= d && i < units.len() - 1 {
        s /= d;
        i += 1;
    }
    let p = if i > 0 && s < 10.0 { 1 } else { 0 };
    format!("{:.2$}{}", s, units[i], p)
}

#[test_case]
fn test_binary_size() {
    let unit = SizeUnit::Binary;
    assert_eq!(unit.format(1),          "1");
    assert_eq!(unit.format(10),        "10");
    assert_eq!(unit.format(100),      "100");
    assert_eq!(unit.format(1000),    "1000");
    assert_eq!(unit.format(1024),    "1.0K");
    assert_eq!(unit.format(1120),    "1.1K");
    assert_eq!(unit.format(1160),    "1.1K");
    assert_eq!(unit.format(15000),    "15K");
    assert_eq!(unit.format(1000000), "977K");
}

#[test_case]
fn test_decimal_size() {
    let unit = SizeUnit::Decimal;
    assert_eq!(unit.format(1),          "1");
    assert_eq!(unit.format(10),        "10");
    assert_eq!(unit.format(100),      "100");
    assert_eq!(unit.format(1000),    "1.0K");
    assert_eq!(unit.format(1024),    "1.0K");
    assert_eq!(unit.format(1120),    "1.1K");
    assert_eq!(unit.format(1160),    "1.2K");
    assert_eq!(unit.format(1500),    "1.5K");
    assert_eq!(unit.format(15000),    "15K");
    assert_eq!(unit.format(1000000), "1.0M");
}
