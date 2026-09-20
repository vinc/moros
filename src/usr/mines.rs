use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::{console, io, rng};

use alloc::string::ToString;
use core::fmt;
use vte::{Params, Parser, Perform};

#[derive(Clone, Copy, PartialEq)]
enum Cell {
    Empty,
    Flag,
    Unsure,
    Mine,
    Blank,
    Number(u8),
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reset = Style::reset();
        match self {
            Cell::Empty    => write!(f, " "),
            Cell::Flag     => write!(f, "!"),
            Cell::Unsure   => write!(f, "?"),
            Cell::Blank    => write!(f, "."),
            Cell::Mine     => write!(f, "#"),
            Cell::Number(n) => {
                let color = match n {
                    1 => Style::color("aqua"),
                    2 => Style::color("lime"),
                    3 => Style::color("red"),
                    _ => Style::color("fushia"),
                };
                write!(f, "{}{}{}", color, n, reset)
            }
        }
    }
}

struct Game {
    cursor: (usize, usize),
    board: [[Cell; 8]; 8],
    mines: [[bool; 8]; 8],
}

impl Game {
    pub fn new(mut count: usize) -> Self {
        let cursor = (0, 0);
        let board = [[Cell::Blank; 8]; 8];
        let mut mines = [[false; 8]; 8];
        while count > 0 {
            let y = (rng::get_u16() % 8) as usize;
            let x = (rng::get_u16() % 8) as usize;
            if !mines[y][x] {
                mines[y][x] = true;
                count -= 1;
            }
        }
        Self { cursor, board, mines }
    }

    pub fn is_flagged(&self, y: usize, x: usize) -> bool {
        match self.board[y][x] {
            Cell::Flag | Cell::Unsure => true,
            _ => false,
        }
    }

    pub fn is_empty(&self, y: usize, x: usize) -> bool {
        self.board[y][x] == Cell::Empty
    }

    pub fn is_blank(&self, y: usize, x: usize) -> bool {
        self.board[y][x] == Cell::Blank
    }

    pub fn render(&mut self) {
        for y in 0..8 {
            for x in 0..8 {
                let mut n = 0;
                if self.mines[y][x] || self.is_blank(y, x) || !self.is_empty(y, x) {
                    continue;
                }
                if y > 0 {
                    if x > 0 && self.mines[y - 1][x - 1] {
                        n += 1;
                    }
                    if self.mines[y - 1][x] {
                        n += 1;
                    }
                    if x < 7 && self.mines[y - 1][x + 1] {
                        n += 1;
                    }
                }
                if x > 0 && self.mines[y][x - 1] {
                    n += 1;
                }
                if x < 7 && self.mines[y][x + 1] {
                    n += 1;
                }
                if y < 7 {
                    if x > 0 && self.mines[y + 1][x - 1] {
                        n += 1;
                    }
                    if self.mines[y + 1][x] {
                        n += 1;
                    }
                    if x < 7 && self.mines[y + 1][x + 1] {
                        n += 1;
                    }
                }
                if n > 0 {
                    self.board[y][x] = Cell::Number(n);
                }
            }
        }
    }

    pub fn run(&mut self) {
        self.render();
        print!("\n{}", self);
        self.move_to_cursor();
        let mut parser = Parser::new();
        while let Some(c) = io::stdin().read_char() {
            print!("\x1b[?25l"); // Disable cursor
            match c {
                'q' | console::ETX_KEY | console::EOT_KEY => {
                    self.move_to_bottom();
                    print!("\x1b[?25h"); // Enable cursor
                    return;
                }
                ' ' => {
                        self.move_to_top();
                        let y = self.cursor.0;
                        let x = self.cursor.1;
                        if self.board[y][x] == Cell::Blank {
                            self.board[y][x] = Cell::Flag;
                        } else if self.board[y][x] == Cell::Flag {
                            self.board[y][x] = Cell::Blank;
                        }
                        print!("{}", self);
                        self.move_to_cursor();
                }
                '\n' => {
                        self.move_to_top();
                        let y = self.cursor.0;
                        let x = self.cursor.1;
                        if self.mines[y][x] {
                            self.board[y][x] = Cell::Mine;
                            self.render();
                            print!("{}", self);
                            self.move_to_cursor();
                            self.move_to_bottom();
                            print!("\x1b[?25h"); // Enable cursor
                            return;
                        }
                        if self.board[y][x] == Cell::Blank {
                            self.board[y][x] = Cell::Empty;
                            self.render();
                        }
                        print!("{}", self);
                        self.move_to_cursor();
                }
                c => {
                    for b in c.to_string().as_bytes() {
                        self.move_to_top();
                        parser.advance(self, *b);
                        print!("{}", self);
                        self.move_to_cursor();
                    }
                }
            }
            print!("\x1b[?25h"); // Enable cursor
        }
    }

    fn move_to_top(&self) {
        print!("\x1b[{}A", 2 * self.cursor.0 + 1);
        print!("\x1b[{}D", 4 * self.cursor.1 + 4);
    }

    fn move_to_cursor(&self) {
        print!("\x1b[{}A", 16 - 2 * self.cursor.0);
        print!("\x1b[{}C", 4 * self.cursor.1 + 4);
    }

    fn move_to_bottom(&self) {
        print!("\x1b[{}B", 16 - 2 * self.cursor.0);
        print!("\x1b[{}D", 4 * self.cursor.1 + 4);
    }

    fn handle_up_key(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
        }
    }

    fn handle_down_key(&mut self) {
        if self.cursor.0 < 7 {
            self.cursor.0 += 1;
        }
    }

    fn handle_forward_key(&mut self) {
        if self.cursor.1 < 7 {
            self.cursor.1 += 1;
        }
    }

    fn handle_backward_key(&mut self) {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        }
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..8 {
            write!(f, "  +---+---+---+---+---+---+---+---+\n")?;
            write!(f, "  |")?;
            for x in 0..8 {
                let cell = self.board[y][x];
                write!(f, " {} |", cell)?;
            }
            write!(f, "\n")?;
        }
        write!(f, "  +---+---+---+---+---+---+---+---+\n")
    }
}

impl Perform for Game {
    fn csi_dispatch(&mut self, _: &Params, _: &[u8], _: bool, c: char) {
        match c {
            'A' => self.handle_up_key(),
            'B' => self.handle_down_key(),
            'C' => self.handle_forward_key(),
            'D' => self.handle_backward_key(),
            _ => {}
        }
    }
}

pub fn main(_args: &[&str]) -> Result<(), ExitCode> {
    Game::new(10).run();
    Ok(())
}
