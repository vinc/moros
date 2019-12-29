use lazy_static::lazy_static;
use crate::{clock, print};
use spin::Mutex;
use heapless::{String, FnvIndexSet};
use heapless::consts::*;
use pc_keyboard::{KeyCode, DecodedKey};

lazy_static! {
    pub static ref STDIN: Mutex<String<U256>> = Mutex::new(String::new());
    pub static ref HISTORY: Mutex<FnvIndexSet<String<U256>, U256>> = Mutex::new(FnvIndexSet::new());
    pub static ref HISTORY_INDEX: Mutex<usize> = Mutex::new(0);
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
}

pub fn print_prompt() {
    print!("\n> ");
}

pub fn key_handle(key: DecodedKey) {
    let mut stdin = STDIN.lock();
    let mut history = HISTORY.lock();
    let mut history_index = HISTORY_INDEX.lock();
    match key {
        DecodedKey::Unicode('\n') => {
            print!("\n");
            if history.len() == history.capacity() {
                let first = history.iter().next().unwrap().clone();
                history.remove(&first);
            }
            if history.insert((*stdin).clone()).is_ok() {
                *history_index = history.len();
            }
            match stdin.as_str() {
                "help" => {
                    print!("RTFM!");
                },
                "version" => {
                    print!("MOROS v{}", env!("CARGO_PKG_VERSION"));
                },
                "uptime" => {
                    print!("{:.6} seconds", clock::uptime());
                },
                _ => {
                    print!("?");
                }
            }
            stdin.clear();
            print!("\n");
            print_prompt();
        },
        DecodedKey::Unicode('\x08') => {
            if stdin.len() > 0 {
                stdin.pop();
                print!("\x08");
            }
        },
        DecodedKey::Unicode(c) => {
            if stdin.push(c).is_ok() {
                print!("{}", c);
            }
        },
        DecodedKey::RawKey(KeyCode::ArrowUp) => {
            if history.len() > 0 {
                if *history_index > 0 {
                    *history_index -= 1;
                }
                if let Some(cmd) = history.iter().nth(*history_index) {
                    let n = stdin.len();
                    for _ in 0..n {
                        print!("\x08");
                    }
                    *stdin = cmd.clone();
                    print!("{}", cmd);
                }
            }
        },
        DecodedKey::RawKey(KeyCode::ArrowDown) => {
            if history.len() > 0 {
                if *history_index < history.len() - 1 {
                    *history_index += 1;
                }
                if let Some(cmd) = history.iter().nth(*history_index) {
                    let n = stdin.len();
                    for _ in 0..n {
                        print!("\x08");
                    }
                    *stdin = cmd.clone();
                    print!("{}", cmd);
                }
            }
        },
        DecodedKey::RawKey(_) => {}
    }
}
