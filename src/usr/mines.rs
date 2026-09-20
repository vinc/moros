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
}

impl Game {
    pub fn new(mut mines: usize) -> Self {
        let cursor = (0, 0);
        let mut board = [[Cell::Blank; 8]; 8];
        while mines > 0 {
            let y = (rng::get_u16() % 8) as usize;
            let x = (rng::get_u16() % 8) as usize;
            if board[y][x] != Cell::Mine {
                board[y][x] = Cell::Mine;
                mines -= 1;
            }
            for y in 0..8 {
                for x in 0..8 {
                    let mut n = 0;
                    if board[y][x] == Cell::Mine {
                        continue;
                    }
                    if y > 0 {
                        if x > 0 && board[y - 1][x - 1] == Cell::Mine {
                            n += 1;
                        }
                        if board[y - 1][x] == Cell::Mine {
                            n += 1;
                        }
                        if x < 7 && board[y - 1][x + 1] == Cell::Mine {
                            n += 1;
                        }
                    }
                    if x > 0 && board[y][x - 1] == Cell::Mine {
                        n += 1;
                    }
                    if x < 7 && board[y][x + 1] == Cell::Mine {
                        n += 1;
                    }
                    if y < 7 {
                        if x > 0 && board[y + 1][x - 1] == Cell::Mine {
                            n += 1;
                        }
                        if board[y + 1][x] == Cell::Mine {
                            n += 1;
                        }
                        if x < 7 && board[y + 1][x + 1] == Cell::Mine {
                            n += 1;
                        }
                    }
                    if n > 0 {
                        board[y][x] = Cell::Number(n);
                    }
                }
            }
        }
        Self { cursor, board }
    }

    pub fn run(&mut self) {
        print!("\n{}", self);
        self.move_to_cursor();
        let mut parser = Parser::new();
        while let Some(c) = io::stdin().read_char() {
            match c {
                'q' | console::ETX_KEY | console::EOT_KEY => {
                    self.move_to_bottom();
                    return;
                }
                c => {
                    for b in c.to_string().as_bytes() {
                        print!("\x1b[?25l"); // Disable cursor
                        self.move_to_top();
                        parser.advance(self, *b);
                        print!("{}", self);
                        print!("\x1b[?25h"); // Enable cursor
                        self.move_to_cursor();
                    }
                }
            }
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
