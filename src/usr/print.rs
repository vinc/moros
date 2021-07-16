use crate::{usr, print};

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    let n = args.len();
    for i in 1..n {
        print!("{}", args[i]);
        if i < n - 1 {
            print!(" ");
        }
    }
    print!("\n");
    usr::shell::ExitCode::CommandSuccessful
}
