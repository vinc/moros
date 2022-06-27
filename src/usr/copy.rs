use crate::api::fs;

pub fn main(args: &[&str]) -> Result<(), usize> {
    if args.len() != 3 {
        eprintln!("Usage: copy <source> <dest>");
        return Err(1);
    }

    let source = args[1];
    let dest = args[2];

    if let Ok(contents) = fs::read_to_bytes(source) {
        if fs::write(dest, &contents).is_ok() {
            Ok(())
        } else {
            error!("Could not write to '{}'", dest);
            Err(1)
        }
    } else {
        error!("File not found '{}'", source);
        Err(1)
    }
}
