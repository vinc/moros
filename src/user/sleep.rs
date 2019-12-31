use crate::kernel::sleep::sleep;

pub fn main(args: &[&str]) {
    if args.len() == 2 {
        if let Ok(duration) = args[1].parse::<f64>() {
            sleep(duration);
        }
    }
}
