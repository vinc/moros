use crate::{print, kernel, user};
use heapless::{String, FnvIndexMap, Vec};
use heapless::consts::*;
use core::convert::TryInto;
use core::str;
use hmac::Hmac;
use sha2::Sha256;

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    login()
}

// TODO: Add max number of attempts
pub fn login() -> user::shell::ExitCode {
    let mut hashed_passwords: FnvIndexMap<String<U256>, String<U1024>, U256> = FnvIndexMap::new();
    if let Some(file) = kernel::fs::File::open("/cfg/passwords.csv") {
        for line in file.read().split("\n") {
            let mut rows = line.split(",");
            if let Some(username) = rows.next() {
                if let Some(hashed_password) = rows.next() {
                    hashed_passwords.insert(username.into(), hashed_password.into()).unwrap();
                }
            }
        }
    }

    print!("\nUsername: ");
    let mut username = kernel::console::get_line();
    username.pop(); // Trim end of string
    match hashed_passwords.get(&username) {
        None => {
            kernel::sleep::sleep(1.0);
            return login();
        },
        Some(hashed_password) => {
            print!("Password: ");
            kernel::console::disable_echo();
            let mut password = kernel::console::get_line();
            kernel::console::enable_echo();
            print!("\n");
            password.pop();
            if !check(&password, hashed_password) {
                kernel::sleep::sleep(1.0);
                return login();
            }
        }
    }

    // TODO: load shell
    user::shell::ExitCode::CommandSuccessful
}

pub fn check(password: &str, hashed_password: &str) -> bool {
    let fields: Vec<_, U4> = hashed_password.split('$').collect();
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
pub fn hash(password: &str) -> String<U1024> {
    let v = "1"; // Password hashing version
    let c = 4096u32; // Number of iterations
    let mut salt = [0u8; 16];
    let mut hash = [0u8; 32];

    // Generating salt
    for i in 0..2 {
        let num = kernel::random::rand64().unwrap();
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
    let mut res: String<U1024> = String::from(v);
    res.push('$').unwrap();
    res.push_str(&String::from_utf8(user::base64::encode(&c)).unwrap()).unwrap();
    res.push('$').unwrap();
    res.push_str(&String::from_utf8(user::base64::encode(&salt)).unwrap()).unwrap();
    res.push('$').unwrap();
    res.push_str(&String::from_utf8(user::base64::encode(&hash)).unwrap()).unwrap();
    res
}
