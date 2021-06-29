use crate::{kernel, print, user};
use alloc::collections::btree_map::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryInto;
use core::str;
use hmac::Hmac;
use sha2::Sha256;

const PASSWORDS: &'static str = "/ini/passwords.csv";
const COMMANDS: [&'static str; 2] = ["create", "login"];

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() == 1 || !COMMANDS.contains(&args[1]) {
        return usage();
    }

    let username: String = if args.len() == 2 {
        print!("Username: ");
        kernel::console::get_line().trim_end().into()
    } else {
        args[2].into()
    };

    match args[1] {
        "create" => create(&username),
        "login" => login(&username),
        _ => usage(),
    }
}

fn usage() -> user::shell::ExitCode {
    print!("Usage: user [{}] <username>\n", COMMANDS.join("|"));
    return user::shell::ExitCode::CommandError;
}

// TODO: Add max number of attempts
pub fn login(username: &str) -> user::shell::ExitCode {
    if username.is_empty() {
        print!("\n");
        kernel::syscall::sleep(1.0);
        return main(&["user", "login"]);
    }

    match hashed_password(&username) {
        Some(hash) => {
            print!("Password: ");
            kernel::console::disable_echo();
            let mut password = kernel::console::get_line();
            kernel::console::enable_echo();
            print!("\n");
            password.pop();
            if !check(&password, &hash) {
                print!("\n");
                kernel::syscall::sleep(1.0);
                return main(&["user", "login"]);
            }
        },
        None => {
            print!("\n");
            kernel::syscall::sleep(1.0);
            return main(&["user", "login"]);
        },
    }

    let home = format!("/usr/{}", username);
    kernel::process::set_user(&username);
    kernel::process::set_env("HOME", &home);
    kernel::process::set_dir(&home);

    // TODO: load shell
    user::shell::ExitCode::CommandSuccessful
}

pub fn create(username: &str) -> user::shell::ExitCode {
    if username.is_empty() {
        return user::shell::ExitCode::CommandError;
    }

    if hashed_password(&username).is_some() {
        print!("Username exists\n");
        return user::shell::ExitCode::CommandError;
    }

    print!("Password: ");
    kernel::console::disable_echo();
    let mut password = kernel::console::get_line();
    kernel::console::enable_echo();
    print!("\n");
    password.pop();

    if password.is_empty() {
        return user::shell::ExitCode::CommandError;
    }

    print!("Confirm: ");
    kernel::console::disable_echo();
    let mut confirm = kernel::console::get_line();
    kernel::console::enable_echo();
    print!("\n");
    confirm.pop();

    if password != confirm {
        print!("Password confirmation failed\n");
        return user::shell::ExitCode::CommandError;
    }

    if save_hashed_password(&username, &hash(&password)).is_err() {
        print!("Could not save user\n");
        return user::shell::ExitCode::CommandError;
    }

    // Create home dir
    kernel::fs::Dir::create(&format!("/usr/{}", username)).unwrap();

    user::shell::ExitCode::CommandSuccessful
}

pub fn check(password: &str, hashed_password: &str) -> bool {
    let fields: Vec<_> = hashed_password.split('$').collect();
    if fields.len() != 4 || fields[0] != "1" {
        return false;
    }

    let decoded_field = user::base64::decode(&fields[1].as_bytes());
    let c = u32::from_be_bytes(decoded_field[0..4].try_into().unwrap());

    let decoded_field = user::base64::decode(&fields[2].as_bytes());
    let salt: [u8; 16] = decoded_field[0..16].try_into().unwrap();

    let mut hash = [0u8; 32];
    pbkdf2::pbkdf2::<Hmac<Sha256>>(password.as_bytes(), &salt, c as usize, &mut hash);
    let encoded_hash = String::from_utf8(user::base64::encode(&hash)).unwrap();

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
        let num = kernel::random::get_u64();
        let buf = num.to_be_bytes();
        let n = buf.len();
        for j in 0..n {
            salt[i * n + j] = buf[j];
        }
    }

    // Hashing password with PBKDF2-HMAC-SHA256
    pbkdf2::pbkdf2::<Hmac<Sha256>>(password.as_bytes(), &salt, c as usize, &mut hash);

    // Encoding in Base64 standard without padding
    let c = c.to_be_bytes();
    let mut res: String = String::from(v);
    res.push('$');
    res.push_str(&String::from_utf8(user::base64::encode(&c)).unwrap());
    res.push('$');
    res.push_str(&String::from_utf8(user::base64::encode(&salt)).unwrap());
    res.push('$');
    res.push_str(&String::from_utf8(user::base64::encode(&hash)).unwrap());
    res
}

fn read_hashed_passwords() -> BTreeMap<String, String> {
    let mut hashed_passwords = BTreeMap::new();
    if let Some(file) = kernel::fs::File::open(PASSWORDS) {
        for line in file.read_to_string().split("\n") {
            let mut rows = line.split(",");
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
    let hashed_passwords = read_hashed_passwords();

    match hashed_passwords.get(username) {
        Some(hash) => Some(hash.into()),
        None => None,
    }
}

fn save_hashed_password(username: &str, hash: &str) -> Result<(), ()> {
    let mut hashed_passwords = read_hashed_passwords();
    hashed_passwords.remove(username.into());
    hashed_passwords.insert(username.into(), hash.into());

    let mut file = match kernel::fs::File::open(PASSWORDS) {
        None => match kernel::fs::File::create(PASSWORDS) {
            None => return Err(()),
            Some(file) => file,
        },
        Some(file) => file,
    };

    let mut contents = String::new();
    for (u, h) in hashed_passwords {
        contents.push_str(&format!("{},{}\n", u, h));
    }
    file.write(&contents.as_bytes())
}
