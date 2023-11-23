use crate::api::fs;
use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::syscall;

use alloc::collections::btree_map::BTreeMap;
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
use nom::sequence::delimited;
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
    let mut labels = BTreeMap::new();
    let mut buf = input;
    loop {
        match parse(buf) {
            Ok((rem, exp)) => {
                debug!("{:?}", exp);
                match exp {
                    Exp::Label(name) => {
                        let label = a.create_label();
                        labels.insert(name, label);
                    }
                    _ => {}
                }
                if rem.trim().is_empty() {
                    break;
                }
                buf = rem;
            }
            Err(err) => {
                debug!("Error: {:#?}", err);
                break;
            }
        }
    }
    let mut buf = input;
    loop {
        match parse(buf) {
            Ok((rem, exp)) => {
                match exp {
                    Exp::Label(name) => {
                        if let Some(mut label) = labels.get_mut(&name) {
                            a.set_label(&mut label)?;
                        }
                    }
                    Exp::Instr(args) => {
                        match args[0].as_str() {
                            "mov" => {
                                if let Ok(reg) = parse_r32(&args[1]) {
                                    if let Ok(num) = parse_u32(&args[2]) {
                                        a.mov(reg, num)?;
                                    } else if let Some(label) = labels.get(&args[2]) {
                                        a.lea(reg, ptr(*label))?;
                                    }
                                } else if let Ok(reg) = parse_r64(&args[1]) {
                                    if let Ok(num) = parse_u64(&args[2]) {
                                        a.mov(reg, num)?;
                                    } else if let Some(label) = labels.get(&args[2]) {
                                        a.lea(reg, ptr(*label))?;
                                    }
                                }
                            }
                            "int" => {
                                if let Ok(num) = parse_u32(&args[1]) {
                                    a.int(num)?;
                                }
                            }
                            "db" => {
                                let mut buf = Vec::new();
                                for arg in args[1..].iter() {
                                    if let Ok(num) = parse_u8(arg) {
                                        buf.push(num);
                                    }
                                }
                                debug!("buf: {:?}", buf);
                                a.db(&buf)?;
                            }
                            _ => {
                                error!("Invalid instruction '{}'\n", args[0]);
                                break;
                            }
                        }
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
        (arg1, None, None)             => Exp::Instr(vec![arg1.to_string()]),
        (arg1, Some(arg2), None)       => Exp::Instr(vec![arg1.to_string(), arg2.to_string()]),
        (arg1, Some(arg2), Some(arg3)) => Exp::Instr(vec![arg1.to_string(), arg2.to_string(), arg3.to_string()]),
        _ => panic!()
    };
    Ok((input, exp))
}

fn parse_label(input: &str) -> IResult<&str, Exp> {
    let (input, label) = delimited(multispace0, terminated(alpha1, tag(":")), multispace0)(input)?;
    Ok((input, Exp::Label(label.to_string())))
}

fn parse_u8(s: &str) -> Result<u8, ParseIntError> {
    if s.starts_with("0x") {
        u8::from_str_radix(&s[2..], 16)
    } else {
        u8::from_str_radix(s, 10)
    }
}

fn parse_u32(s: &str) -> Result<u32, ParseIntError> {
    if s.starts_with("0x") {
        u32::from_str_radix(&s[2..], 16)
    } else {
        u32::from_str_radix(s, 10)
    }
}

fn parse_u64(s: &str) -> Result<u64, ParseIntError> {
    if s.starts_with("0x") {
        u64::from_str_radix(&s[2..], 16)
    } else {
        u64::from_str_radix(s, 10)
    }
}

fn parse_r32(name: &str) -> Result<AsmRegister32, ()> {
    match name {
        "eax" => Ok(eax),
        "ebx" => Ok(ebx),
        "ecx" => Ok(ecx),
        "edx" => Ok(edx),
        "edi" => Ok(edi),
        "esi" => Ok(esi),
        "ebp" => Ok(ebp),
        "esp" => Ok(esp),
        "r8d" => Ok(r8d),
        "r9d" => Ok(r9d),
        "r10d" => Ok(r10d),
        "r11d" => Ok(r11d),
        "r12d" => Ok(r12d),
        "r13d" => Ok(r13d),
        "r14d" => Ok(r14d),
        "r15d" => Ok(r15d),
        _ => Err(()),
    }
}

fn parse_r64(name: &str) -> Result<AsmRegister64, ()> {
    match name {
        "rax" => Ok(rax),
        "rbx" => Ok(rbx),
        "rcx" => Ok(rcx),
        "rdx" => Ok(rdx),
        "rdi" => Ok(rdi),
        "rsi" => Ok(rsi),
        "rbp" => Ok(rbp),
        "rsp" => Ok(rsp),
        "r8" => Ok(r8),
        "r9" => Ok(r9),
        "r10" => Ok(r10),
        "r11" => Ok(r11),
        "r12" => Ok(r12),
        "r13" => Ok(r13),
        "r14" => Ok(r14),
        "r15" => Ok(r15),
        _ => Err(()),
    }
}

fn hex(input: &str) -> IResult<&str, &str> {
  recognize(preceded(alt((tag("0x"), tag("0X"))), alphanumeric1))(input)
}

fn help() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} asm {}<file>{}", csi_title, csi_reset, csi_option, csi_reset);
}
