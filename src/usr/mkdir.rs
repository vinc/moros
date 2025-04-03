use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::syscall;
use alloc::vec::Vec;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut parents = false;
    let mut paths: Vec<&str> = Vec::new();
    let mut i = 1;
    let n = args.len();

    // Processa os argumentos
    while i < n {
        match args[i] {
            "-h" | "--help" => {
                help();
                return Ok(());
            }
            "-p" | "--parents" => {
                parents = true;
            }
            _ => {
                paths.push(args[i]);
            }
        }
        i += 1;
    }

    if paths.is_empty() {
        help();
        return Err(ExitCode::UsageError);
    }

    // Para cada diretório informado, tenta criá-lo
    let mut exit_code = ExitCode::Success;
    for path in paths {
        // Se o diretório já existe...
        if fs::exists(path) {
            if !parents {
                error!("mkdir: cannot create directory '{}': File exists", path);
                exit_code = ExitCode::Failure;
            }
            continue;
        }

        // Se a opção -p estiver ativa, cria os diretórios pais necessários
        if parents {
            if let Err(e) = create_parents(fs::dirname(path)) {
                error!("mkdir: cannot create directory '{}': {}", path, e);
                exit_code = ExitCode::Failure;
                continue;
            }
        }

        // Tenta criar o diretório final
        let res = fs::create_dir(path);
        if let Some(handle) = res {
            syscall::close(handle);
        } else {
            error!("mkdir: cannot create directory '{}'", path);
            exit_code = ExitCode::Failure;
        }
    }

    if exit_code == ExitCode::Success {
        Ok(())
    } else {
        Err(exit_code)
    }
}

// Função recursiva para criar diretórios pais conforme necessário
fn create_parents(path: &str) -> Result<(), &'static str> {
    if path.is_empty() || fs::exists(path) {
        return Ok(());
    }
    // Cria o diretório pai primeiro
    create_parents(fs::dirname(path))?;
    if let Some(handle) = fs::create_dir(path) {
        syscall::close(handle);
        Ok(())
    } else {
        Err("failed to create directory")
    }
}

fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} mkdir [OPTION]... DIRECTORY...", csi_title, csi_reset);
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!("  {0}-p{1}, {0}--parents{1}   no error if existing, make parent directories as needed", csi_option, csi_reset);
    println!("  {0}-h{1}, {0}--help{1}      display this help and exit", csi_option, csi_reset);
}
