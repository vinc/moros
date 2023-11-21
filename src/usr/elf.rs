use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::usr;

use alloc::string::String;
use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, NasmFormatter};
use object::{Object, ObjectSection};

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut path = "";
    let mut disassemble = false;
    let mut i = 1;
    let n = args.len();
    while i < n {
        match args[i] {
            "-h" | "--help" => {
                help();
                return Ok(());
            }
            "-d" | "--disassemble" => {
                disassemble = true;
            }
            _ => {
                if args[i].starts_with('-') {
                    error!("Invalid option '{}'", args[i]);
                    return Err(ExitCode::UsageError);
                } else if path.is_empty() {
                    path = args[i];
                } else {
                    error!("Too many arguments");
                    return Err(ExitCode::UsageError);
                }
            }
        }
        i += 1;
    }

    let color = Style::color("Yellow");
    let reset = Style::reset();
    if let Ok(buf) = fs::read_to_bytes(path) {
        let bin = buf.as_slice();
        if let Ok(obj) = object::File::parse(bin) {
            println!("ELF entry address: {:#X}", obj.entry());
            for section in obj.sections() {
                if let Ok(name) = section.name() {
                    if name.is_empty() {
                        continue;
                    }
                    if disassemble && section.kind() != object::SectionKind::Text {
                        continue;
                    }
                    let addr = section.address() as usize;
                    let size = section.size();
                    let align = section.align();
                    println!();

                    println!("{}{}{} (addr: {:#X}, size: {}, align: {})", color, name, reset, addr, size, align);
                    if let Ok(data) = section.data() {
                        if disassemble {
                            let ip = addr as u64;
                            let mut decoder = Decoder::with_ip(64, data, ip, DecoderOptions::NONE);
                            let mut formatter = NasmFormatter::new();
                            formatter.options_mut().set_hex_prefix("0x");
                            formatter.options_mut().set_hex_suffix("");
                            //formatter.options_mut().set_first_operand_char_index(10);
                            let mut output = String::new();
                            let mut instruction = Instruction::default();
                            while decoder.can_decode() {
                                decoder.decode_out(&mut instruction);
                                output.clear();
                                formatter.format(&instruction, &mut output);
                                print!("{}{:016X}: ", Style::color("LightCyan"), instruction.ip());
                                let start_index = (instruction.ip() - ip) as usize;
                                let instr_bytes = &data[start_index..start_index + instruction.len()];
                                for b in instr_bytes.iter() {
                                    print!("{}{:02X} {}", Style::color("Pink"), b, reset);
                                }
                                if instr_bytes.len() < 10 {
                                    for _ in 0..10 - instr_bytes.len() {
                                        print!("   ");
                                    }
                                }
                                println!(" {}", output);
                            }
                        } else {
                            usr::hex::print_hex(data);
                        }
                    }
                }
            }
            Ok(())
        } else {
            error!("Could not parse ELF");
            Err(ExitCode::Failure)
        }
    } else {
        error!("Could not find file '{}'", path);
        Err(ExitCode::Failure)
    }
}

fn help() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} elf {}<binary>{}", csi_title, csi_reset, csi_option, csi_reset);
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!("  {0}-d{1}, {0}--disassemble{1}          Display assembler contents of executable section", csi_option, csi_reset);
}
