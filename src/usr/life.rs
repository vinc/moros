use crate::{api, sys};
use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::rng;
use crate::sys::console;

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

struct Game {
    cols: usize,
    rows: usize,
    grid: BTreeSet<(i64, i64)>,
    step: usize,
    speed: f64,
    quiet: bool,
    seed_interval: usize,
    seed_population: usize,
}

impl Game {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            grid: BTreeSet::new(),
            step: 0,
            speed: 2.0,
            quiet: false,
            seed_interval: 1,
            seed_population: 30,
        }
    }

    pub fn load_file(&mut self, path: &str) {
        if let Ok(lines) = fs::read_to_string(path) {
            for (y, line) in lines.split('\n').enumerate() {
                for (x, c) in line.chars().enumerate() {
                    let cell = (x as i64, y as i64);
                    match c {
                        ' ' | '.' | '0' => self.grid.remove(&cell),
                        _ => self.grid.insert(cell),
                    };
                }
            }
        }
    }

    pub fn run(&mut self) {
        loop {
            if self.seed_interval > 0 && self.step % self.seed_interval == 0 {
                self.seed();
            }
            if console::end_of_text() || (self.is_game_over() && self.quiet) {
                print!("\x1b[2J\x1b[1;1H"); // Clear screen and move to top
                return;
            }
            print!("{}", self);
            sys::time::sleep(1.0 / self.speed);
            if self.is_game_over() {
                continue; // Display the screen until ^C is received
            }

            // Rules of the game (B3/S23)
            // - Birth if three live neighbors
            // - Survival if two or three live neighbors
            self.step += 1;
            let mut cells_to_insert = vec![];
            let mut cells_to_remove = vec![];
            for x in 0..self.cols {
                for y in 0..self.rows {
                    let cell = (x as i64, y as i64);
                    let n = neighboors(&cell).iter().fold(0, |s, c|
                        s + self.grid.contains(c) as u8
                    );
                    match n {
                        2 => continue,
                        3 => cells_to_insert.push(cell),
                        _ => cells_to_remove.push(cell),
                    }
                }
            }
            for cell in cells_to_insert {
                self.grid.insert(cell);
            }
            for cell in cells_to_remove {
                self.grid.remove(&cell);
            }
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.grid.is_empty()
    }

    pub fn population(&self) -> usize {
        self.grid.len()
    }

    pub fn generation(&self) -> usize {
        self.step
    }

    fn seed(&mut self) {
        let n = self.seed_population;
        for _ in 0..n {
            let x = (rng::get_u64() % (self.cols as u64)) as i64;
            let y = (rng::get_u64() % (self.rows as u64)) as i64;
            self.grid.insert((x, y));
        }
    }

    fn status(&self, title: &str, bg: &str) -> String {
        let gen = self.generation();
        let pop = self.population();
        let color = Style::color("Black").with_background(bg);
        let reset = Style::reset();
        let stats = format!("GEN: {:04} | POP: {:04}", gen, pop);
        let size = self.cols - stats.len();
        format!("\n{}{:n$}{}{}", color, title, stats, reset, n = size)
    }
}

fn neighboors(&(x, y): &(i64, i64)) -> Vec<(i64, i64)> {
    vec![
        (x - 1, y - 1),
        (x, y - 1),
        (x + 1, y - 1),
        (x - 1, y),
        (x + 1, y),
        (x - 1, y + 1),
        (x, y + 1),
        (x + 1, y + 1),
    ]
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        for y in 0..self.rows {
            for x in 0..self.cols {
                out.push(if self.grid.contains(&(x as i64, y as i64)) {
                    '#'
                } else {
                    ' '
                });
            }
            if y < self.rows - 1 {
                out.push('\n');
            }
        }

        if !self.quiet {
            let line = if self.is_game_over() {
                self.status("GAME OVER", "Yellow")
            } else {
                self.status("GAME OF LIFE", "White")
            };
            out.push_str(&line);
        }

        write!(f, "\x1b[1;1H{}", out)?; // Move cursor to top then print screen
        Ok(())
    }
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut game = Game::new(api::console::cols(), api::console::rows() - 1);
    let mut i = 0;
    let n = args.len();
    while i < n {
        match args[i] {
            "-h" | "--help" => {
                usage();
                return Ok(());
            }
            "-p" | "--population" => {
                if i + 1 < n {
                    i += 1;
                    let opt = &mut game.seed_population;
                    *opt = args[i].parse().unwrap_or(*opt);
                } else {
                    error!("Missing --population <num>");
                    return Err(ExitCode::UsageError);
                }
            }
            "-i" | "--interval" => {
                if i + 1 < n {
                    i += 1;
                    let opt = &mut game.seed_interval;
                    *opt = args[i].parse().unwrap_or(*opt);
                } else {
                    error!("Missing --interval <num>");
                    return Err(ExitCode::UsageError);
                }
            }
            "-s" | "--speed" => {
                if i + 1 < n {
                    i += 1;
                    let opt = &mut game.speed;
                    *opt = args[i].parse().unwrap_or(*opt);
                } else {
                    error!("Missing --speed <num>");
                    return Err(ExitCode::UsageError);
                }
            }
            "-q" | "--quiet" => {
                game.quiet = true;
                game.rows += 1;
            }
            arg => {
                if arg.starts_with('-') {
                    error!("Unknown option '{}'", arg);
                    return Err(ExitCode::UsageError);
                }
                if i > 0 {
                    let path = arg;
                    if !fs::exists(path) {
                        error!("Could not find file '{}'", path);
                        return Err(ExitCode::UsageError);
                    }
                    game.load_file(path);
                    game.seed_population = 0;
                    game.seed_interval = 0;
                }
            }
        }
        i += 1;
    }
    print!("\x1b[2J"); // Clear screen
    print!("\x1b[?25l"); // Disable cursor
    print!("\x1b[12l"); // Disable echo
    game.run();
    print!("\x1b[12h"); // Enable echo
    print!("\x1b[?25h"); // Enable cursor
    Ok(())
}

fn usage() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} life {}<options> [<path>]{1}",
        csi_title, csi_reset, csi_option
    );
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!(
        "  {0}-p{1}, {0}--population <num>{1}   \
        Set the seed population to {0}<num>{1}",
        csi_option, csi_reset
    );
    println!(
        "  {0}-i{1}, {0}--interval <num>{1}     \
        Set the seed interval to {0}<num>{1}",
        csi_option, csi_reset
    );
    println!(
        "  {0}-s{1}, {0}--speed <num>{1}        \
        Set the simulation speed to {0}<num>{1}",
        csi_option, csi_reset
    );
    println!(
        "  {0}-q{1}, {0}--quiet{1}              \
        Enable quiet mode",
        csi_option, csi_reset
    );
}
