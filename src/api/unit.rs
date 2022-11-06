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
            SizeUnit::Decimal => todo!(),
        }
    }
}

const PREFIXES: [&str; 5] = ["", "K", "M", "G", "T"];

fn binary_size(size: usize) -> String {
    let n = PREFIXES.len();
    for i in 0..n {
        let prefix = PREFIXES[i];
        if size < (1 << ((i + 1) * 10)) || i == n - 1 {
            let s = ((10 * size) >> (i * 10)) as f64 / 10.0;
            let s = if s >= 10.0 { libm::round(s) } else { s };
            return format!("{}{}", s, prefix);
        }
    }
    unreachable!();
}
