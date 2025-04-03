use crate::api::{console, io};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use vte::{Params, Parser, Perform};
use alloc::vec;

use core::sync::atomic::Ordering;
use crate::sys::keyboard::SHIFT;

/// Estrutura que representa o prompt com suporte a múltiplas linhas.
pub struct Prompt {
    /// Indica se há fim de linha.
    eol: bool,
    /// Offset do prompt (tamanho da string do prompt).
    offset: usize,
    /// Índice da linha atual em que o cursor se encontra.
    cursor_row: usize,
    /// Posição do cursor na linha atual (valor absoluto na tela).
    cursor_col: usize,
    /// Posição horizontal desejada (relativa à linha, sem contar o offset da primeira linha).
    desired_horizontal: usize,
    /// Conteúdo do prompt dividido em múltiplas linhas.
    lines: Vec<Vec<char>>,
}

impl Prompt {
    /// Cria uma nova instância de `Prompt`.
    pub fn new() -> Self {
        Self {
            eol: true,
            offset: 0,
            cursor_row: 0,
            cursor_col: 0,
            desired_horizontal: 0,
            lines: vec![Vec::with_capacity(console::cols())],
        }
    }

    /// Retorna o número máximo de caracteres que podem ser exibidos na linha atual.
    /// Na primeira linha, considera-se o offset do prompt.
    fn max(&self) -> usize {
        console::cols() - if self.cursor_row == 0 { self.offset } else { 1 }
    }

    /// Retorna o índice efetivo (relativo) da posição do cursor na linha atual.
    fn current_index(&self) -> usize {
        if self.cursor_row == 0 {
            self.cursor_col.saturating_sub(self.offset)
        } else {
            self.cursor_col
        }
    }

    /// Verifica se a linha atual está vazia.
    fn current_line_empty(&self) -> bool {
        self.lines[self.cursor_row].is_empty()
    }

    /// Junta todas as linhas em uma única String, separando-as por '\n'.
    fn collect_input(&self) -> String {
        let mut result = String::new();
        for (i, line) in self.lines.iter().enumerate() {
            let s: String = line.iter().collect();
            result.push_str(&s);
            if i + 1 < self.lines.len() {
                result.push('\n');
            }
        }
        result
    }

    /// Insere uma nova linha no buffer e move o cursor para ela.
    fn handle_newline(&mut self) {
        println!();
        self.lines.push(Vec::with_capacity(console::cols()));
        self.cursor_row += 1;
        self.cursor_col = 0;
        self.desired_horizontal = 0;
    }

    /// Trata a tecla backspace, removendo o caractere anterior ao cursor.
    fn handle_backspace_key(&mut self) {
        let idx = self.current_index();
        if idx > 0 {
            self.lines[self.cursor_row].remove(idx - 1);
            self.cursor_col -= 1;
            self.desired_horizontal = self.current_index();
            let s: String = self.lines[self.cursor_row][idx - 1..].iter().collect();
            print!("\x08{} \x1b[{}D", s, s.len() + 1);
        } else if self.cursor_row > 0 {
            let prev_len = self.lines[self.cursor_row - 1].len();
            let current_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = prev_len + if self.cursor_row == 0 { self.offset } else { 0 };
            self.desired_horizontal = prev_len;
            self.lines[self.cursor_row].extend(current_line);
            let s: String = self.lines[self.cursor_row][prev_len..].iter().collect();
            print!("\r{}", s);
        }
    }

    fn handle_forward_key(&mut self) {
        let idx = self.current_index();
        if idx < self.lines[self.cursor_row].len() {
            print!("\x1b[1C");
            self.cursor_col += 1;
            self.desired_horizontal = self.current_index();
        }
    }

    fn handle_backward_key(&mut self) {
        let idx = self.current_index();
        if idx > 0 {
            print!("\x1b[1D");
            self.cursor_col -= 1;
            self.desired_horizontal = self.current_index();
        }
    }

    /// Insere um caractere imprimível na posição atual do cursor.
    fn handle_printable_key(&mut self, c: char) {
        if console::is_printable(c) {
            if self.lines[self.cursor_row].len() >= self.max() {
                self.handle_newline();
            }
            
            let idx = self.current_index();
            self.lines[self.cursor_row].insert(idx, c);
            self.cursor_col += 1;
            self.desired_horizontal = self.current_index();
            
            let s: String = self.lines[self.cursor_row][idx..].iter().collect();
            print!("{} \x1b[{}D", s, s.len());
            
            if self.lines[self.cursor_row].len() == self.max() {
                self.handle_newline();
            }
        }
    }

    fn handle_page_up_key(&mut self) {
        print!("\x1b[5~");
    }

    fn handle_page_down_key(&mut self) {
        print!("\x1b[6~");
    }

    /// Deleta o caractere na posição atual do cursor.
    fn handle_delete_key(&mut self) {
        let idx = self.current_index();
        if idx < self.lines[self.cursor_row].len() {
            self.lines[self.cursor_row].remove(idx);
            let s: String = self.lines[self.cursor_row][idx..].iter().collect();
            print!("{} \x1b[{}D", s, s.len() + 1);
        }
    }

    /// Lê a entrada do usuário exibindo o prompt passado.
    pub fn input(&mut self, prompt: &str) -> Option<String> {
        print!("{}", prompt);
        self.offset = offset_from_prompt(prompt);
        self.cursor_row = 0;
        self.cursor_col = self.offset;
        self.desired_horizontal = 0;
        self.lines.clear();
        self.lines.push(Vec::with_capacity(console::cols()));
        let mut parser = Parser::new();
        while let Some(c) = io::stdin().read_char() {
            match c {
                console::ETX_KEY => {
                    if self.eol {
                        println!();
                    }
                    return Some(String::new());
                }
                console::EOT_KEY => {
                    if self.current_line_empty() {
                        if self.eol {
                            println!();
                        }
                        return Some(self.collect_input());
                    } else {
                        self.handle_delete_key();
                    }
                }
                '\n' => {
                    if SHIFT.load(Ordering::Relaxed) {
                        self.handle_newline();
                    } else {
                        if self.eol {
                            println!();
                        }
                        return Some(self.collect_input());
                    }
                }
                _ => {
                    for b in c.to_string().as_bytes() {
                        parser.advance(self, *b);
                    }
                }
            }
        }
        None
    }
}

impl Perform for Prompt {
    fn execute(&mut self, b: u8) {
        let c = b as char;
        match c {
            '\x08' => self.handle_backspace_key(),
            _ => {}
        }
    }

    fn print(&mut self, c: char) {
        match c {
            '\x7f' => self.handle_delete_key(),
            c => self.handle_printable_key(c),
        }
    }

    fn csi_dispatch(&mut self, params: &Params, _: &[u8], _: bool, c: char) {
        match c {
            'C' => self.handle_forward_key(),
            'D' => self.handle_backward_key(),
            '~' => {
                for param in params.iter() {
                    match param[0] {
                        3 => self.handle_delete_key(),
                        5 => self.handle_page_up_key(),
                        6 => self.handle_page_down_key(),
                        _ => continue,
                    }
                }
            }
            _ => {}
        }
    }
}

struct Offset(usize);

impl Perform for Offset {
    fn print(&mut self, c: char) {
        self.0 += c.len_utf8();
    }
}

fn offset_from_prompt(s: &str) -> usize {
    let mut parser = Parser::new();
    let mut offset = Offset(0);
    for b in s.bytes() {
        parser.advance(&mut offset, b);
    }
    offset.0
}