use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::io;
use crate::api::fs;

use alloc::vec::Vec;
use alloc::string::String;

use chumsky::prelude::*;

#[derive(Clone)]
enum Instr {
    Left, Right,
    Incr, Decr,
    Read, Write,
    Loop(Vec<Self>),
}

fn parser<'a>() -> impl Parser<'a, &'a str, Vec<Instr>, extra::Err<Rich<'a, char>>> {
    recursive(|bf| choice((
        just('<').to(Instr::Left),
        just('>').to(Instr::Right),
        just('+').to(Instr::Incr),
        just('-').to(Instr::Decr),
        just(',').to(Instr::Read),
        just('.').to(Instr::Write),
        bf.delimited_by(just('['), just(']')).map(Instr::Loop),
    )).repeated().collect())
}

const TAPE_LEN: usize = 10_000;

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

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() != 2 {
        //help();
        return Err(ExitCode::UsageError);
    }
    if args[1] == "-h" || args[1] == "--help" {
        //help();
        return Ok(());
    }
    let path = args[1];
    if let Ok(buf) = fs::read_to_string(path) {
        let buf = buf.lines().map(|line| line.trim()).collect::<String>();
        let buf = &buf;
        match parser().parse(buf).into_result() {
            Ok(ast) => execute(&ast, &mut 0, &mut [0; TAPE_LEN]),
            Err(errs) => errs.into_iter().for_each(|e| {
                let col = e.span().start + 1;
                let row = 1; // TODO
                error!("Unexpected token at {path}:{row}:{col}");

                use alloc::format;
                let red = Style::color("red");
                let reset = Style::reset();
                let msg = format!("{}", e.reason());
                let space = " ".repeat(e.span().start);
                let arrow = "^".repeat(e.span().end - e.span().start);
                eprintln!("\n{buf}\n{space}{red}{arrow} {msg}{reset}");
            })
        };
        Ok(())
    } else {
        error!("Could not read '{}'", path);
        Err(ExitCode::Failure)
    }
}
