use crate::{print, kernel, user};

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 2 {
        return user::shell::ExitCode::CommandError;
    }

    let pathname = args[1];

    if let Some(file) = kernel::fs::File::open(pathname) {
        let contents = file.read_to_string();
        print_hex(contents.as_bytes());
        user::shell::ExitCode::CommandSuccessful
    } else {
        print!("File not found '{}'\n", pathname);
        user::shell::ExitCode::CommandError
    }
}

pub fn print_hex(buf: &[u8]) {
    let n = buf.len() / 2;
    for i in 0..n {
        if i % 8 == 0 {
            print!("\n{:08X}: ", i * 2);
        }
        print!("{:02X}{:02X} ", buf[i * 2], buf[i * 2 + 1]);
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
        }
    }
    print!("\n");
}
