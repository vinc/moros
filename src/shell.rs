use lazy_static::lazy_static;
use crate::{clock, print};
use crate::string::String;
use spin::Mutex;

lazy_static! {
    pub static ref STDIN: Mutex<String> = Mutex::new(String::new());
}

pub fn print_banner() {
    print!("                                      _M_\n");
    print!("                                     (o o)\n");
    print!("+--------------------------------ooO--(_)--Ooo---------------------------------+\n");
    print!("|                                                                              |\n");
    print!("|                                    MOROS                                     |\n");
    print!("|                                                                              |\n");
    print!("|                       Omniscient Rust Operating System                       |\n");
    print!("|                                                                              |\n");
    print!("+------------------------------------------------------------------------------+\n");
    print!("\n");
}

pub fn print_prompt() {
    print!("> ");
}

pub fn key_handle(c: char) {
    let mut stdin = STDIN.lock();
    if c == '\n' {
        print!("\n");
        match stdin.as_str() {
            "help" => {
                print!("< rtfm!");
            },
            "uptime" => {
                print!("{:.6} seconds\n", clock::uptime());
            },
            _ => {
                print!("?");
            }
        }
        stdin.clear();
        print!("\n");
        print_prompt();
    } else {
        if c == 0x08 as char {
            if stdin.len() > 0 {
                stdin.pop();
                print!("{}", c);
            }
        } else {
            stdin.push(c as u8);
            print!("{}", c);
        }
    }
}
