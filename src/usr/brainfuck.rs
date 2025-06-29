use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::io;
use crate::api::fs;

use alloc::vec::Vec;

use chumsky::prelude::*;

#[derive(Clone)]
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

const TAPE_LEN: usize = 30_000;

fn read_byte() -> u8 {
    io::stdin().read_char().unwrap_or('\0') as u8
}

fn execute(ast: &[Instr], ptr: &mut usize, tape: &mut [u8; TAPE_LEN]) {
    for symbol in ast {
        match symbol {
            Instr::Left => *ptr = (*ptr + TAPE_LEN - 1).rem_euclid(TAPE_LEN),
            Instr::Right => *ptr = (*ptr + 1).rem_euclid(TAPE_LEN),
            Instr::Incr => tape[*ptr] = tape[*ptr].wrapping_add(1),
            Instr::Decr => tape[*ptr] = tape[*ptr].wrapping_sub(1),
            Instr::Read => tape[*ptr] = read_byte(),
            Instr::Write => print!("{}", tape[*ptr] as char),
            Instr::Loop(ast) => {
                while tape[*ptr] != 0 {
                    execute(ast, ptr, tape)
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

    let red = Style::color("red");
    let reset = Style::reset();

    let path = args[1];
    if let Ok(buf) = fs::read_to_string(path) {
        match parser().parse(&buf).into_result() {
            Ok(ast) => execute(&ast, &mut 0, &mut [0; TAPE_LEN]),
            Err(errs) => errs.into_iter().for_each(|e| {
                let (row, col) = pos(&buf, e.span().start);
                //let token = e.found().unwrap();
                //error!("Unexpected token '{token}' at {path}:{row}:{col}");
                error!("Unexpected token at {path}:{row}:{col}");

                let line = buf.lines().skip(row - 1).next().unwrap();
                let space = " ".repeat(col - 1);
                let arrow = "^".repeat(e.span().end - e.span().start);
                //let reason = e.reason().to_string();
                let reason = "unexpected token";
                eprintln!("\n{line}\n{space}{red}{arrow} {reason}{reset}");
            })
        };
        Ok(())
    } else {
        error!("Could not read '{}'", path);
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
