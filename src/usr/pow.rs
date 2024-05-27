use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::{console, io, rng};

use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt;
use vte::{Params, Parser, Perform};

struct Game {
    score: usize,
    board: [usize; 16],
}

pub fn main(_args: &[&str]) -> Result<(), ExitCode> {
    print!("\x1b[?25l"); // Disable cursor
    Game::new().run();
    print!("\x1b[?25h"); // Enable cursor
    Ok(())
}

impl Game {
    pub fn new() -> Self {
        Self {
            score: 0,
            board: [0; 16],
        }
    }

    pub fn run(&mut self) {
        self.seed();
        self.seed();
        print!("{}", self);
        let mut parser = Parser::new();
        while let Some(c) = io::stdin().read_char() {
            match c {
                'q' | console::ETX_KEY | console::EOT_KEY => {
                    return;
                }
                c => {
                    for b in c.to_string().as_bytes() {
                        parser.advance(self, *b);
                    }
                    print!("\x1b[20A{}", self);
                }
            }
        }
    }

    fn seed(&mut self) {
        let zeros: Vec<_> = (0..16).filter(|i| self.board[*i] == 0).collect();

        if !zeros.is_empty() {
            let i = (rng::get_u64() as usize) % zeros.len();
            self.board[zeros[i]] = 2;
        }
    }

    fn rotate(&mut self, times: usize) {
        for _ in 0..times {
            let tmp = self.board;
            for x in 0..4 {
                for y in 0..4 {
                    self.board[4 * y + 3 - x] = tmp[4 * x + y];
                }
            }
        }
    }

    fn compute(&mut self) {
        for i in 0..16 {
            let mut j = i;
            while j > 3 {
                j -= 4;
                if self.board[j] == 0 {
                    self.board[j] = self.board[j + 4];
                    self.board[j + 4] = 0;
                    continue;
                }
                if self.board[j] == self.board[j + 4] {
                    self.board[j + 4] = 0;
                    self.board[j] *= 2;
                    self.score += self.board[j];
                    break;
                }
                break;
            }
        }
    }

    fn handle_up_key(&mut self) {
        self.compute();
        self.seed();
    }

    fn handle_down_key(&mut self) {
        self.rotate(2);
        self.compute();
        self.rotate(2);
        self.seed();
    }

    fn handle_forward_key(&mut self) {
        self.rotate(3);
        self.compute();
        self.rotate(1);
        self.seed();
    }

    fn handle_backward_key(&mut self) {
        self.rotate(1);
        self.compute();
        self.rotate(3);
        self.seed();
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reset = Style::reset();
        let color = Style::color("yellow");
        write!(f, "\n  {}SCORE: {:>22}{}\n\n", color, self.score, reset)?;
        for y in 0..4 {
            write!(f, "  +------+------+------+------+\n")?;
            write!(f, "  |      |      |      |      |\n")?;
            write!(f, "  |")?;
            for x in 0..4 {
                let v = self.board[x + y * 4];
                if v == 0 {
                    write!(f, "      |")?;
                } else {
                    let color = match v {
                        2 => Style::color("LightGray"),
                        4 => Style::color("blue"),
                        8 => Style::color("LightCyan"),
                        16 => Style::color("lime"),
                        32 => Style::color("yellow"),
                        64 => Style::color("red"),
                        128 => Style::color("Pink"),
                        256 => Style::color("purple"),
                        512 => Style::color("Pink"),
                        1024 => Style::color("maroon"),
                        2048 => Style::color("olive"),
                        _ => Style::color("White"),
                    };
                    write!(f, " {}{:^5}{}|", color, v, reset)?;
                }
            }
            write!(f, "\n  |      |      |      |      |\n")?;
        }
        write!(f, "  +------+------+------+------+\n")
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

#[test_case]
fn test_2048_rotate() {
    let mut game = Game::new();
    game.seed();
    game.seed();
    game.seed();
    let before = game.board;
    game.rotate(1);
    game.rotate(3);
    assert_eq!(game.board, before);
}
