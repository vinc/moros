use crate::print;

pub fn main(args: &[&str]) {
    let n = args.len();
    for i in 1..n {
        print!("{}", args[i]);
        if i < n {
            print!(" ");
        }
    }
    print!("\n");
}
