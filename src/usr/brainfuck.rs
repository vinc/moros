use crate::api::console::Style;
use crate::api::fs;
use crate::api::io;
use crate::api::process::ExitCode;

use alloc::vec::Vec;
use chumsky::prelude::*;

const TAPE_LEN: usize = 30_000;

fn read_byte() -> u8 {
    io::stdin().read_char().unwrap_or('\0') as u8
}

#[derive(Clone, Debug, PartialEq)]
enum Instr {
    Left, Right,
    Incr, Decr,
    Read, Write,
    Loop(Vec<Self>),
}

fn parser<'a>() -> impl Parser<'a, &'a str, Vec<Instr>, extra::Err<Rich<'a, char>>> {
    let comment = none_of("<>+-,.[]").ignored();
    recursive(|bf| choice((
        just('<').to(Instr::Left),
        just('>').to(Instr::Right),
        just('+').to(Instr::Incr),
        just('-').to(Instr::Decr),
        just(',').to(Instr::Read),
        just('.').to(Instr::Write),
        bf.delimited_by(just('['), just(']')).map(Instr::Loop),
    )).padded_by(comment.repeated()).repeated().collect())
}

fn eval(ast: &[Instr], ptr: &mut usize, tape: &mut [u8; TAPE_LEN]) {
    for sym in ast {
        match sym {
            Instr::Left => *ptr = (*ptr + TAPE_LEN - 1).rem_euclid(TAPE_LEN),
            Instr::Right => *ptr = (*ptr + 1).rem_euclid(TAPE_LEN),
            Instr::Incr => tape[*ptr] = tape[*ptr].wrapping_add(1),
            Instr::Decr => tape[*ptr] = tape[*ptr].wrapping_sub(1),
            Instr::Read => tape[*ptr] = read_byte(),
            Instr::Write => print!("{}", tape[*ptr] as char),
            Instr::Loop(ast) => {
                while tape[*ptr] != 0 {
                    eval(ast, ptr, tape)
                }
            }
        }
    }
}

fn pos(buf: &str, i: usize) -> (usize, usize) {
    let mut col = 1;
    let mut row = 1;
    let mut j = 0;
    for line in buf.lines() {
        let n = line.len();
        if i < j + n {
            col = i - j + 1;
            break;
        }
        j += n + 1;
        row += 1;
    }
    (row, col)
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() != 2 {
        help();
        return Err(ExitCode::UsageError);
    }
    if args[1] == "-h" || args[1] == "--help" {
        help();
        return Ok(());
    }

    let error = Style::color("red");
    let reset = Style::reset();
    let path = args[1];
    if let Ok(buf) = fs::read_to_string(path) {
        match parser().parse(&buf).into_result() {
            Ok(ast) => eval(&ast, &mut 0, &mut [0; TAPE_LEN]),
            Err(errs) => errs.into_iter().for_each(|e| {
                let (row, col) = pos(&buf, e.span().start);
                error!("Unexpected token at {path}:{row}:{col}");

                let line = buf.lines().nth(row - 1).unwrap();
                let space = " ".repeat(col - 1);
                let arrow = "^".repeat(e.span().end - e.span().start);
                let reason = "unexpected token";
                eprintln!("\n{line}\n{space}{error}{arrow} {reason}{reset}");
            })
        };
        Ok(())
    } else {
        error!("Could not read {:?}", path);
        Err(ExitCode::Failure)
    }
}

fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} brainfuck {}<path>{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
}

#[test_case]
fn test_parser() {
    use alloc::vec;
    let src = "+++++[-] Increment a cell five times then loop to clear it";
    let ast = vec![
        Instr::Incr, Instr::Incr, Instr::Incr, Instr::Incr, Instr::Incr,
        Instr::Loop(vec![Instr::Decr])
    ];
    assert_eq!(parser().parse(src).into_result(), Ok(ast));
}
