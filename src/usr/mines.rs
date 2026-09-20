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

    pub fn run(&mut self) {
        print!("\n{}", self);
        self.move_to_cursor();
        let mut parser = Parser::new();
        while let Some(c) = io::stdin().read_char() {
            print!("\x1b[?25l"); // Disable cursor
            match c {
                'q' | console::ETX_KEY | console::EOT_KEY => {
                    self.move_to_bottom();
                    print!("\x1b[?25h"); // Enable cursor
                    println!("\n  GAME OVER: You bailed");
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
                '?' => {
                        self.move_to_top();
                        let y = self.cursor.0;
                        let x = self.cursor.1;
                        if self.board[y][x] == Cell::Blank {
                            self.board[y][x] = Cell::Unsure;
                        } else if self.board[y][x] == Cell::Unsure {
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
                            print!("{}", self);
                            self.move_to_cursor();
                            self.move_to_bottom();
                            print!("\x1b[?25h"); // Enable cursor
                            println!("\n  GAME OVER: You failed");
                            return;
                        }
                        self.reveal(y, x);
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
            if self.is_uncovered() {
                self.move_to_bottom();
                println!("\n  GAME OVER: You won");
                return;
            }
        }
    }

    fn is_uncovered(&self) -> bool {
        for y in 0..8 {
            for x in 0..8 {
                if self.board[y][x] == Cell::Blank {
                    return false;
                }
            }
        }
        true
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

    fn reveal(&mut self, y: usize, x: usize) {
        if !self.is_blank(y, x) || self.mines[y][x] {
            return;
        }
        let count = self.count_mines(y, x);
        if count > 0 {
            self.board[y][x] = Cell::Number(count);
            return;
        }
        self.board[y][x] = Cell::Empty;
        for y2 in 0..8 {
            for x2 in 0..8 {
                if Self::is_neighbor(y, x, y2, x2) {
                    self.reveal(y2, x2);
                }
            }
        }
    }

    fn count_mines(&self, y: usize, x: usize) -> u8 {
        let mut count = 0;
        for y2 in 0..8 {
            for x2 in 0..8 {
                if Self::is_neighbor(y, x, y2, x2) && self.mines[y2][x2] {
                    count += 1;
                }
            }
        }
        count
    }

    fn is_blank(&self, y: usize, x: usize) -> bool {
        self.board[y][x] == Cell::Blank
    }

    fn is_neighbor(y: usize, x: usize, y2: usize, x2: usize) -> bool {
        let dy = y.abs_diff(y2);
        let dx = x.abs_diff(x2);
        dy <= 1 && dx <= 1 && (dy, dx) != (0, 0)
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
