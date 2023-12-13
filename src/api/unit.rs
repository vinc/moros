use alloc::format;
use alloc::string::String;

#[derive(Clone)]
pub enum SizeUnit {
    None,
    Binary,
    Decimal,
}

impl SizeUnit {
    pub fn format(&self, size: usize) -> String {
        match self {
            SizeUnit::None => format!("{}", size),
            SizeUnit::Binary => binary_size(size),
            SizeUnit::Decimal => decimal_size(size),
        }
    }
}

const PREFIXES: [&str; 5] = ["", "K", "M", "G", "T"];

fn binary_size(size: usize) -> String {
    let n = PREFIXES.len();
    for i in 0..n {
        let prefix = PREFIXES[i];
        if size < (1 << ((i + 1) * 10)) || i == n - 1 {
            let s = ((size * 10) >> (i * 10)) as f64 / 10.0;
            let s = if s >= 10.0 { libm::round(s) } else { s };
            return format!("{}{}", s, prefix);
        }
    }
    unreachable!();
}

fn decimal_size(size: usize) -> String {
    let size = size;
    let n = PREFIXES.len();
    for i in 0..n {
        let prefix = PREFIXES[i];
        if size < usize::pow(10, 3 * (i + 1) as u32) || i == n - 1 {
            let s = (size as f64) / libm::pow(10.0, 3.0 * (i as f64));
            let precision = if s >= 10.0 { 0 } else { 1 };
            return format!("{:.2$}{}", s, prefix, precision);
        }
    }
    unreachable!();
}
