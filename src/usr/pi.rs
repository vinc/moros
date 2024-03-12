use crate::api::process::ExitCode;
use crate::{sys, usr};

use alloc::format;
use num_bigint::BigInt;
use usr::lisp::Number;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut digits = None;
    if args.len() == 2 {
        if let Ok(n) = args[1].parse() {
            digits = Some(n);
        }
    }

    let mut q = BigInt::from(1);
    let mut r = BigInt::from(0);
    let mut t = BigInt::from(1);
    let mut k = BigInt::from(1);
    let mut n = BigInt::from(3);
    let mut l = BigInt::from(3);
    let mut first = true;
    loop {
        if sys::console::end_of_text() {
            break;
        }
        if &q * 4 + &r - &t < &n * &t {
            print!("{}", Number::BigInt(n.clone()));
            if first {
                print!(".");
                first = false;
            }
            match digits {
                Some(0) => break,
                Some(i) => digits = Some(i - 1),
                None => {}
            }

            let nr = (&r - &n * &t) * 10;
            n = (&q * 3 + &r) * 10 / &t - &n * 10;
            q *= 10;
            r = nr;
        } else {
            let nr = (&q * 2 + &r) * &l;
            let nn = (&q * &k * 7 + 2 + &r * &l) / (&t * &l);
            q *= &k;
            t *= &l;
            l += 2;
            k += 1;
            n = nn;
            r = nr;
        }
    }
    println!();
    Ok(())
}
