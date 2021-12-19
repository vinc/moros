use crate::{api, sys, usr};
use crate::api::fs;
use crate::api::io;
use crate::api::random;
use crate::api::syscall;
use alloc::collections::btree_map::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::convert::TryInto;
use core::str;
use hmac::Hmac;
use sha2::Sha256;

const PASSWORDS: &str = "/ini/passwords.csv";
const COMMANDS: [&str; 2] = ["create", "login"];

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() == 1 || !COMMANDS.contains(&args[1]) {
        return usage();
    }

    let username: String = if args.len() == 2 {
        print!("Username: ");
        io::stdin().read_line().trim_end().to_string()
    } else {
        args[2].to_string()
    };

    match args[1] {
        "create" => create(&username),
        "login" => login(&username),
        _ => usage(),
    }
}

fn usage() -> usr::shell::ExitCode {
    eprintln!("Usage: user [{}] <username>", COMMANDS.join("|"));
    usr::shell::ExitCode::CommandError
}

// TODO: Add max number of attempts
pub fn login(username: &str) -> usr::shell::ExitCode {
    if !fs::exists(PASSWORDS) {
        eprintln!("Could not read '{}'", PASSWORDS);
        return usr::shell::ExitCode::CommandError;
    }

    if username.is_empty() {
        println!();
        syscall::sleep(1.0);
        return main(&["user", "login"]);
    }

    match hashed_password(username) {
        Some(hash) => {
            print!("Password: ");
            sys::console::disable_echo();
            let password = io::stdin().read_line().trim_end().to_string();
            sys::console::enable_echo();
            println!();
            if !check(&password, &hash) {
                println!();
                syscall::sleep(1.0);
                return main(&["user", "login"]);
            }
        },
        None => {
            println!();
            syscall::sleep(1.0);
            return main(&["user", "login"]);
        },
    }

    let home = format!("/usr/{}", username);
    sys::process::set_user(username);
    sys::process::set_env("HOME", &home);
    sys::process::set_dir(&home);

    // TODO: load shell
    usr::shell::ExitCode::CommandSuccessful
}

pub fn create(username: &str) -> usr::shell::ExitCode {
    if username.is_empty() {
        return usr::shell::ExitCode::CommandError;
    }

    if hashed_password(username).is_some() {
        eprintln!("Username exists");
        return usr::shell::ExitCode::CommandError;
    }

    print!("Password: ");
    sys::console::disable_echo();
    let password = io::stdin().read_line().trim_end().to_string();
    sys::console::enable_echo();
    println!();

    if password.is_empty() {
        return usr::shell::ExitCode::CommandError;
    }

    print!("Confirm: ");
    sys::console::disable_echo();
    let confirm = io::stdin().read_line().trim_end().to_string();
    sys::console::enable_echo();
    println!();

    if password != confirm {
        eprintln!("Password confirmation failed");
        return usr::shell::ExitCode::CommandError;
    }

    if save_hashed_password(username, &hash(&password)).is_err() {
        eprintln!("Could not save user");
        return usr::shell::ExitCode::CommandError;
    }

    // Create home dir
    sys::fs::Dir::create(&format!("/usr/{}", username)).unwrap();

    usr::shell::ExitCode::CommandSuccessful
}

pub fn check(password: &str, hashed_password: &str) -> bool {
    let fields: Vec<_> = hashed_password.split('$').collect();
    if fields.len() != 4 || fields[0] != "1" {
        return false;
    }

    let decoded_field = usr::base64::decode(fields[1].as_bytes());
    let c = u32::from_be_bytes(decoded_field[0..4].try_into().unwrap());

    let decoded_field = usr::base64::decode(fields[2].as_bytes());
    let salt: [u8; 16] = decoded_field[0..16].try_into().unwrap();

    let mut hash = [0u8; 32];
    pbkdf2::pbkdf2::<Hmac<Sha256>>(password.as_bytes(), &salt, c, &mut hash);
    let encoded_hash = String::from_utf8(usr::base64::encode(&hash)).unwrap();

    encoded_hash == fields[3]
}

// Password hashing version 1 => PBKDF2-HMAC-SHA256 + BASE64
// Fields: "<version>$<c>$<salt>$<hash>"
// Example: "1$AAAQAA$PDkXP0I8O7SxNOxvUKmHHQ$BwIUWBxKs50BTpH6i4ImF3SZOxADv7dh4xtu3IKc3o8"
pub fn hash(password: &str) -> String {
    let v = "1"; // Password hashing version
    let c = 4096u32; // Number of iterations
    let mut salt = [0u8; 16];
    let mut hash = [0u8; 32];

    // Generating salt
    for i in 0..2 {
        let num = random::get_u64();
        let buf = num.to_be_bytes();
        let n = buf.len();
        for j in 0..n {
            salt[i * n + j] = buf[j];
        }
    }

    // Hashing password with PBKDF2-HMAC-SHA256
    pbkdf2::pbkdf2::<Hmac<Sha256>>(password.as_bytes(), &salt, c, &mut hash);

    // Encoding in Base64 standard without padding
    let c = c.to_be_bytes();
    let mut res: String = String::from(v);
    res.push('$');
    res.push_str(&String::from_utf8(usr::base64::encode(&c)).unwrap());
    res.push('$');
    res.push_str(&String::from_utf8(usr::base64::encode(&salt)).unwrap());
    res.push('$');
    res.push_str(&String::from_utf8(usr::base64::encode(&hash)).unwrap());
    res
}

fn read_hashed_passwords() -> BTreeMap<String, String> {
    let mut hashed_passwords = BTreeMap::new();
    if let Ok(contents) = api::fs::read_to_string(PASSWORDS) {
        for line in contents.split('\n') {
            let mut rows = line.split(',');
            if let Some(username) = rows.next() {
                if let Some(hash) = rows.next() {
                    hashed_passwords.insert(username.into(), hash.into());
                }
            }
        }
    }
    hashed_passwords
}

fn hashed_password(username: &str) -> Option<String> {
    read_hashed_passwords().get(username).map(|hash| hash.into())
}

fn save_hashed_password(username: &str, hash: &str) -> Result<usize, ()> {
    let mut hashed_passwords = read_hashed_passwords();
    hashed_passwords.remove(username);
    hashed_passwords.insert(username.into(), hash.into());

    let mut csv = String::new();
    for (u, h) in hashed_passwords {
        csv.push_str(&format!("{},{}\n", u, h));
    }

    fs::write(PASSWORDS, csv.as_bytes())
}
