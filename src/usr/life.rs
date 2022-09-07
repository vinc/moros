use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::random;
use crate::sys;

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

struct Game {
    cols: i64,
    rows: i64,
    grid: BTreeSet<(i64, i64)>
    step: usize,
    seed_popcount: usize,
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut game = Game::new(80, 25);
    let mut i = 0;
    let n = args.len();
    while i < n {
        match args[i] {
            "-h" | "--help" => {
                usage();
                return Ok(());
            }
            "-s" | "--seed" => {
                if i + 1 < n {
                    game.seed_popcount = args[i + 1].parse().unwrap_or(game.seed_popcount);
                    i += 1;
                } else {
                    error!("Missing seed value");
                    return Err(ExitCode::UsageError);
                }
            }
            _ => {}
        }
        i += 1;
    }
    print!("\x1b[?25l"); // Disable cursor
    game.run();
    print!("\x1b[?25h"); // Enable cursor
    Ok(())
}

impl Game {
    pub fn new(cols: i64, rows: i64) -> Self {
        Self {
            cols,
            rows,
            grid: BTreeSet::new(),
            step: 0,
            seed_popcount: 25,
        }
    }

    pub fn run(&mut self) {
        print!("\x1b[2J"); // Clear screen
        loop {
            self.seed();
            print!("{}", self);
            sys::time::sleep(0.5);
            if self.grid.is_empty() || sys::console::end_of_text() {
                let csi_title = Style::color("Yellow");
                let csi_reset = Style::reset();
                println!("\n{}The Game of Life ended after {} generations{}", csi_title, self.step, csi_reset);
                return;
            }
            let mut cells_to_insert = vec![];
            let mut cells_to_remove = vec![];
            for x in 0..self.cols {
                for y in 0..self.rows {
                    let cell = (x, y);
                    let mut sum = 0;
                    for neighboor in neighboors(&cell) {
                        if self.grid.contains(&neighboor) {
                            sum += 1;
                        }
                    }
                    if sum == 3 {
                        cells_to_insert.push(cell);
                    } else if sum != 2 {
                        cells_to_remove.push(cell);
                    }
                }
            }
            for cell in cells_to_insert {
                self.grid.insert(cell);
            }
            for cell in cells_to_remove {
                self.grid.remove(&cell);
            }
            self.step += 1;
        }
    }

    fn seed(&mut self) {
        let n = self.seed_popcount;
        for _ in 0..n {
            let x = (random::get_u64() % (self.cols as u64)) as i64;
            let y = (random::get_u64() % (self.rows as u64)) as i64;
            self.grid.insert((x, y));
        }
    }
}

fn neighboors(&(x, y): &(i64, i64)) -> Vec<(i64, i64)> {
    vec![
        (x - 1, y - 1), (x, y - 1), (x + 1, y - 1),
        (x - 1, y),                 (x + 1, y),
        (x - 1, y + 1), (x, y + 1), (x + 1, y + 1),
    ]
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        for y in 0..self.rows {
            for x in 0..self.cols {
                if self.grid.contains(&(x, y)) {
                    out.push('#');
                } else {
                    out.push(' ');
                }
            }
            if y < self.rows - 1 {
                out.push('\n');
            }
        }
        write!(f, "\x1b[1;1H{}", out)?; // Move cursor to top then print screen
        Ok(())
    }
}

fn usage() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} life {}<options>{1}", csi_title, csi_reset, csi_option);
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!("  {0}-s{1},{0} --seed <num>{1}       Set the seedling population count to {0}<num>{1}", csi_option, csi_reset);
}
