use crate::print;
use crate::kernel::fs;

pub fn main(args: &[&str]) {
    if args.len() != 3 {
        return;
    }

    let from = args[1];
    let to = args[2];

    if to.starts_with("/dev") || to.starts_with("/sys") {
        print!("Permission denied to write to '{}'\n", to);
    } else {
        if let Some(file_from) = fs::File::open(from) {
            if let Some(mut file_to) = fs::File::create(to) {
                file_to.write(&file_from.read());
            } else {
                print!("Permission denied to write to '{}'\n", to);
            }
        } else {
            print!("File not found '{}'\n", from);
        }
    }
}
