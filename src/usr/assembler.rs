use crate::api::fs;
use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::syscall;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::vec;
use core::num::ParseIntError;
use iced_x86::code_asm::*;
use nom::IResult;
use nom::character::complete::alphanumeric1;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::sequence::terminated;
use nom::combinator::recognize;
use nom::character::complete::alpha1;
use nom::character::complete::multispace0;
use nom::sequence::preceded;
use nom::sequence::tuple;
use nom::combinator::opt;

#[derive(Clone, Debug)]
pub enum Exp {
    Label(String),
    Instr(Vec<String>),
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
    let path = args[1];
    if let Ok(input) = fs::read_to_string(path) {
        if let Ok(output) = assemble(&input) {
            let mut buf = vec![0x7F, b'B', b'I', b'N']; 
            buf.extend_from_slice(&output);
            syscall::write(1, &buf);
        }
        Ok(())
    } else {
        error!("Could not find file '{}'", path);
        Err(ExitCode::Failure)
    }
}

pub fn assemble(input: &str) -> Result<Vec<u8>, IcedError> {
    let mut a = CodeAssembler::new(64)?;
    let _ = eax;
    let _ = edi;
    let mut main = a.create_label();
    a.set_label(&mut main)?;
    let mut buf = input;
    loop {
        match parse(buf) {
            Ok((rem, exp)) => {
                debug!("{:?}", exp);
                match exp {
                    Exp::Instr(ops) => {
                        match ops[0].as_str() {
                            "mov" => {
                                let op = parse_u32(&ops[2]).unwrap();
                                match ops[1].as_str() {
                                    "eax" => { a.mov(eax, op)?; },
                                    "edi" => { a.mov(edi, op)?; },
                                    _ => {},
                                }
                            }
                            "int" => {
                                let op = parse_u32(&ops[1]).unwrap();
                                a.int(op)?;
                            }
                            _ => {
                            }
                        }
                    }
                    _ => {
                    }
                }
                if rem.trim().is_empty() {
                    break;
                }
                buf = rem;
            }
            Err(err) => {
                debug!("asm: {:#?}", err);
                break;
            }
        }
    }
    a.assemble(0x200_000)
}

// Parser

fn parse(input: &str) -> IResult<&str, Exp> {
    alt((parse_label, parse_instr))(input)
}

fn parse_instr(input: &str) -> IResult<&str, Exp> {
    let (input, instr) = tuple((
        preceded(multispace0, alpha1),
        opt(preceded(multispace0, alt((alpha1, hex)))),
        opt(preceded(tuple((tag(","), multispace0)), alt((alpha1, hex)))),
    ))(input)?;
    let exp = match instr {
        (opcode, None, None)                     => Exp::Instr(vec![opcode.to_string()]),
        (opcode, Some(operand1), None)           => Exp::Instr(vec![opcode.to_string(), operand1.to_string()]),
        (opcode, Some(operand1), Some(operand2)) => Exp::Instr(vec![opcode.to_string(), operand1.to_string(), operand2.to_string()]),
        _ => panic!()
    };
    Ok((input, exp))
}

fn parse_label(input: &str) -> IResult<&str, Exp> {
    let (input, label) = terminated(
        terminated(
            alpha1,
            tag(":")
        ),
        multispace0,
    )(input)?;
    Ok((input, Exp::Label(label.to_string())))
}

fn hex(input: &str) -> IResult<&str, &str> {
  recognize(preceded(
    alt((tag("0x"), tag("0X"))),
    alphanumeric1
  ))(input)
}

fn parse_u32(s: &str) -> Result<u32, ParseIntError> {
    if s.starts_with("0x") || s.starts_with("0X") {
        u32::from_str_radix(&s[2..], 16)
    } else {
        u32::from_str_radix(s, 10)
    }
}

fn help() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} asm {}<file>{}", csi_title, csi_reset, csi_option, csi_reset);
}
