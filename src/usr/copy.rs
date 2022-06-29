use crate::api::fs;
use crate::api::process::ExitCode;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() != 3 {
        eprintln!("Usage: copy <source> <dest>");
        return Err(ExitCode::UsageError);
    }

    let source = args[1];
    let dest = args[2];

    if let Ok(contents) = fs::read_to_bytes(source) {
        if fs::write(dest, &contents).is_ok() {
            Ok(())
        } else {
            error!("Could not write to '{}'", dest);
            Err(ExitCode::Failure)
        }
    } else {
        error!("File not found '{}'", source);
        Err(ExitCode::Failure)
    }
}
