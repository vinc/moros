use crate::usr;
use crate::api::console::Style;
use crate::api::{io, random};
use core::fmt;
use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;
use vte::{Params, Parser, Perform};

struct Game {
    pub score: usize,
    pub board: [usize; 16],
}

impl Game {
    pub fn new() -> Self {
        Self {
            score: 0,
            board: [0; 16],
        }
    }

    pub fn compute(&mut self) {
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

    pub fn rotate(&mut self, times: usize) {
        for _ in 0..times {
            let mut rotate = [3, 7, 11, 15, 2, 6, 10, 14, 1, 5, 9, 13, 0, 4, 8, 12];
            for i in 0..16 {
                rotate[i] = self.board[rotate[i]];
            }
            self.board = rotate;
        }
    }

    pub fn seed(&mut self) {
        let zeros: Vec<_> = (0..16).filter(|i| self.board[*i] == 0).collect();
        let n = zeros.len();
        if n == 0 {
            return;
        }
        let r = random::get_u64() as usize;
        let i = r % n;
        self.board[zeros[i]] = 2;
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
        self.rotate(1);
        self.compute();
        self.rotate(3);
        self.seed();
    }

    fn handle_backward_key(&mut self) {
        self.rotate(3);
        self.compute();
        self.rotate(1);
        self.seed();
    }

    pub fn run(&mut self) {
        let mut parser = Parser::new();
        print!("{}", self);
        while let Some(c) = io::stdin().read_char() {
            match c {
                'q' | '\x03' | '\x04' => { // ^C and ^D
                    return;
                },
                c => {
                    for b in c.to_string().as_bytes() {
                        parser.advance(self, *b);
                    }
                    print!("\x1b[20A{}", self);
                }
            }
        }
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reset = Style::reset();
        let color = Style::color("Yellow");
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
                        2    => Style::color("LightGray"),
                        4    => Style::color("LightBlue"),
                        8    => Style::color("LightCyan"),
                        16   => Style::color("LightGreen"),
                        32   => Style::color("Yellow"),
                        64   => Style::color("LightRed"),
                        128  => Style::color("Pink"),
                        256  => Style::color("Magenta"),
                        512  => Style::color("Pink"),
                        1024 => Style::color("Red"),
                        2048 => Style::color("Brown"),
                        _    => Style::color("White"),
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
    fn execute(&mut self, _b: u8) {
    }

    fn print(&mut self, _c: char) {
    }

    fn csi_dispatch(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, c: char) {
        match c {
            'A' => self.handle_up_key(),
            'B' => self.handle_down_key(),
            'C' => self.handle_forward_key(),
            'D' => self.handle_backward_key(),
            _ => {},
        }
    }
}

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    print!("\x1b[?25l"); // Disable cursor
    let mut game = Game::new();
    game.seed();
    game.seed();
    game.run();
    print!("\x1b[?25h"); // Enable cursor
    usr::shell::ExitCode::CommandSuccessful
}
