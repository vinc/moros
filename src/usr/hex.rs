use crate::usr;
use crate::api::fs;
use crate::api::console::Style;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() != 2 {
        return usr::shell::ExitCode::CommandError;
    }

    let pathname = args[1];

    if let Ok(buf) = fs::read(pathname) {
        print_hex(&buf);
        usr::shell::ExitCode::CommandSuccessful
    } else {
        eprintln!("File not found '{}'", pathname);
        usr::shell::ExitCode::CommandError
    }
}

// TODO: move this to api::hex::print_hex
pub fn print_hex(buf: &[u8]) {
    let n = buf.len() / 2;
    for i in 0..n {
        print!("{}", Style::color("Yellow"));
        if i % 8 == 0 {
            print!("{:08X}: ", i * 2);
        }
        print!("{}", Style::color("LightCyan"));
        print!("{:02X}{:02X} ", buf[i * 2], buf[i * 2 + 1]);
        print!("{}", Style::reset());
        if i % 8 == 7 || i == n - 1 {
            for _ in 0..(7 - (i % 8)) {
                print!("     ");
            }
            let m = ((i % 8) + 1) * 2;
            for j in 0..m {
                let c = buf[(i * 2 + 1) - (m - 1) + j] as char;
                if c.is_ascii_graphic() {
                    print!("{}", c);
                } else if c.is_ascii_whitespace() {
                    print!(" ");
                } else {
                    print!(".");
                }
            }
            println!();
        }
    }
}
