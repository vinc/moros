use crate::{print, kernel, user};
use alloc::vec::Vec;

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 2 {
        return user::shell::ExitCode::CommandError;
    }
    let path: Vec<_> = args[1].split("/").collect();
    if path.len() != 8 || path[1] != "dev" || path[2] != "ata" || path[4] != "dsk" || path[6] != "blk" {
        return user::shell::ExitCode::CommandError;
    }
    let bus = path[3].parse().unwrap();
    let dsk = path[5].parse().unwrap();
    let blk = path[7].parse().unwrap();
    let mut buf = [0u8; 512];
    kernel::ata::read(bus, dsk, blk, &mut buf);
    print_hex(&buf);
    user::shell::ExitCode::CommandSuccessful
}

pub fn print_hex(buf: &[u8]) {
    let n = buf.len() / 2;
    for i in 0..n {
        if i % 8 == 0 {
            print!("\n{:08X}: ", i * 2);
        }
        print!("{:02X}{:02X} ", buf[i * 2], buf[i * 2 + 1]);
        if i % 8 == 7 {
            for j in 0..16 {
                let c = buf[(i * 2 + 1) - 15 + j] as char;
                if c.is_ascii_graphic() || c.is_ascii_whitespace() {
                    print!("{}", c);
                } else {
                    print!(".");
                }
            }
        }
    }
    print!("\n");
}
