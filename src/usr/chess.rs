use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::prompt::Prompt;
use crate::api::rng;
use crate::{api, sys};

use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use lazy_static::lazy_static;
use littlewing::chess::*;
use littlewing::color::*;
use spin::Mutex;

lazy_static! {
    static ref MOVES: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

const FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const COMMANDS: [&str; 11] = [
    "quit", "help", "init", "time", "move", "undo", "show", "perf", "save",
    "load", "puzzle"
];

fn update_autocomplete(prompt: &mut Prompt, game: &mut Game) {
    *MOVES.lock() = game.get_moves().into_iter().map(|m| m.to_lan()).collect();

    fn chess_completer(line: &str) -> Vec<String> {
        let mut entries = Vec::new();
        let args: Vec<&str> = line.split(' ').collect();
        let i = args.len() - 1;
        if i == 0 {
            // Autocomplete command
            for &cmd in &COMMANDS {
                if let Some(entry) = cmd.strip_prefix(args[i]) {
                    entries.push(entry.into());
                }
            }
        } else if i == 1 && (args[0] == "move" || args[0] == "m") {
            // Autocomplete moves
            for m in &*MOVES.lock() {
                if let Some(entry) = m.strip_prefix(args[1]) {
                    entries.push(entry.into());
                }
            }
        }
        entries
    }
    prompt.completion.set(&chess_completer);
}

fn system_time() -> u128 {
    (api::clock::realtime() * 1000.0) as u128
}

struct Chess {
    game: Game,
    side: Color,
    csi_color: Style,
    csi_notif: Style,
    csi_reset: Style,
}

impl Chess {
    fn new() -> Self {
        Self {
            game: Game::new(),
            side: BLACK,
            csi_color: Style::color("teal"),
            csi_notif: Style::color("yellow"),
            csi_reset: Style::reset(),
        }
    }

    fn run(&mut self) {
        println!("MOROS Chess v0.2.0\n");
        let prompt_string = format!("{}>{} ", self.csi_color, self.csi_reset);

        let mut prompt = Prompt::new();
        let history_file = "~/.chess-history";
        prompt.history.load(history_file);

        // 40 moves in 5 minutes
        self.game.clock = Clock::new(40, 5 * 60 * 1000);
        self.game.clock.system_time = Arc::new(system_time);
        self.game.show_coordinates = true;
        let size = 1 << 20; // 1 MB
        self.game.tt_resize(size);
        self.game.load_fen(FEN).unwrap();
        println!("{}", self.game);

        update_autocomplete(&mut prompt, &mut self.game);
        while let Some(cmd) = prompt.input(&prompt_string) {
            let args: Vec<&str> = cmd.trim().split(' ').collect();
            match args[0] {
                "q" | "quit" => break,
                "h" | "help" => self.cmd_help(args),
                "i" | "init" => self.cmd_init(args),
                "t" | "time" => self.cmd_time(args),
                "p" | "play" => self.cmd_play(args),
                "m" | "move" => self.cmd_move(args),
                "u" | "undo" => self.cmd_undo(args),
                "l" | "load" => self.cmd_load(args),
                "s" | "save" => self.cmd_save(args),
                "puzzle" => self.cmd_puzzle(args),
                "perf" => self.cmd_perf(args),
                cmd => {
                    if cmd.is_empty() {
                        println!();
                    } else {
                        error!("Unknown command '{}'\n", cmd);
                    }
                }
            }
            prompt.history.add(&cmd);
            prompt.history.save(history_file);
            update_autocomplete(&mut prompt, &mut self.game);
        }
    }

    fn cmd_help(&mut self, _args: Vec<&str>) {
        println!("{}Commands:{}", self.csi_notif, self.csi_reset);
        let cmds = [
            ("q", "uit", "Exit this program\n"),
            ("h", "elp", "Display this screen\n"),
            ("i", "nit", "Initialize a new game\n"),
            (
                "t",
                "ime <moves> <time>",
                "Set clock to <moves> in <time> (in seconds)\n",
            ),
            ("p", "lay [<side>]", "Play <side> on the board\n"),
            ("m", "ove <move>", "Play <move> on the board\n"),
            ("u", "ndo", "Undo the last move\n"),
            ("l", "oad <file>", "Load game from <file>\n"),
            ("s", "ave <file>", "Save game to <file>\n"),
            ("", "perf [<depth>] ", "Count the nodes at each depth\n"),
        ];
        for (alias, command, usage) in &cmds {
            let csi_col1 = Style::color("lime");
            let csi_col2 = Style::color("aqua");
            print!(
                "  {}{}{}{:20}{}{}",
                csi_col1, alias, csi_col2, command, self.csi_reset, usage
            );
        }
        println!();
    }

    fn cmd_init(&mut self, _args: Vec<&str>) {
        self.game.clear();
        self.game.load_fen(FEN).unwrap();
        println!();
        println!("{}", self.game);
    }

    fn cmd_puzzle(&mut self, args: Vec<&str>) {
        if args.len() != 2 {
            error!("No <path> given\n");
            return;
        }
        let path = args[1];
        if let Ok(text) = fs::read_to_string(path) {
            let lines: Vec<&str> = text.lines().collect();
            let i = (rng::get_u64() as usize) % lines.len();
            let fen = lines[i];
            self.load(fen);
        } else {
            error!("Could not read '{}'\n", path);
        }
    }

    fn cmd_load(&mut self, args: Vec<&str>) {
        if args.len() != 2 {
            error!("No <path> given\n");
            return;
        }
        let path = args[1];
        if let Ok(fen) = fs::read_to_string(path) {
            self.load(&fen);
        } else {
            error!("Could not read '{}'\n", path);
        }
    }

    fn cmd_save(&mut self, args: Vec<&str>) {
        if args.len() != 2 {
            error!("No <path> given\n");
            return;
        }
        let path = args[1];
        let text = format!("{}\n", self.game.to_fen());
        if fs::write(path, text.as_bytes()).is_ok() {
            println!();
        } else {
            error!("Could not write to '{}'\n", path);
        }
    }

    fn cmd_time(&mut self, args: Vec<&str>) {
        match args.len() {
            1 => {
                error!("No <moves> and <time> given\n");
                return;
            }
            2 => {
                error!("No <time> given\n");
                return;
            }
            _ => {}
        }
        if let Ok(moves) = args[1].parse::<u16>() {
            if let Ok(time) = args[2].parse::<f64>() {
                self.game.clock = Clock::new(moves, (time * 1000.0) as u64);
                self.game.clock.system_time = Arc::new(system_time);
                println!();
            } else {
                error!("Could not parse time\n");
            }
        } else {
            error!("Could not parse moves\n");
        }
    }

    fn cmd_play(&mut self, args: Vec<&str>) {
        self.side = match args.get(1) {
            None => self.game.side(),
            Some(&"white") => WHITE,
            Some(&"black") => BLACK,
            Some(_) => {
                error!("Could not parse side\n");
                return;
            }
        };
        println!();
        if self.game.side() == self.side {
            self.play();
        }
    }

    fn cmd_move(&mut self, args: Vec<&str>) {
        if args.len() < 2 {
            error!("No <move> given\n");
            return;
        }
        if !is_move(args[1]) {
            error!("Invalid move '{}'\n", args[1]);
            return;
        }
        let m = self.game.move_from_lan(args[1]);
        if !self.game.is_parsed_move_legal(m) {
            error!("Illegal move '{}'\n", args[1]);
            return;
        }

        print!("\x1b[?25l"); // Disable cursor
        self.game.make_move(m);
        self.game.history.push(m);
        println!();
        println!("{}", self.game);

        if self.game.side() == self.side {
            self.play();
        }

        if self.game.is_mate() {
            if self.game.is_check(color::WHITE) {
                println!("{}<{} black mates", self.csi_color, self.csi_reset);
            } else if self.game.is_check(color::BLACK) {
                println!("{}<{} white mates", self.csi_color, self.csi_reset);
            } else {
                println!("{}<{} draw", self.csi_color, self.csi_reset);
            }
            println!();
        }
        print!("\x1b[?25h"); // Enable cursor
    }

    fn cmd_undo(&mut self, _args: Vec<&str>) {
        if !self.game.history.is_empty() {
            if let Some(m) = self.game.history.pop() {
                self.game.undo_move(m);
            }
        }
        println!();
        println!("{}", self.game);
    }

    fn cmd_perf(&mut self, args: Vec<&str>) {
        let csi_depth = Style::color("aqua");
        let csi_count = Style::color("fushia");
        let csi_reset = Style::reset();

        let mut depth = if args.len() > 1 {
            if let Ok(d) = args[1].parse() {
                d
            } else {
                error!("Invalid depth '{}'\n", args[1]);
                return;
            }
        } else {
            1
        };

        loop {
            let started_at = (self.game.clock.system_time)();
            let n = self.game.perft(depth);
            let t = (self.game.clock.system_time)();
            let s = ((t - started_at) as f64) / 1000.0;
            let nps = (n as f64) / s;
            println!(
                "{}{}:{} {}{} ({:.2} s, {:.2e} nps)",
                csi_depth, depth, csi_count, n, csi_reset, s, nps
            );

            if args.len() > 1 || sys::console::end_of_text() {
                break;
            } else {
                depth += 1;
            }
        }
        println!();
    }

    fn load(&mut self, fen: &str) {
        self.game.clear(); // TODO: Do we need this?
        if self.game.load_fen(fen).is_ok() {
            self.side = self.game.side() ^ 1;
            let color = if self.game.side() == WHITE {
                "white"
            } else {
                "black"
            };
            println!();
            println!(
                "{}<{} play {}", self.csi_color, self.csi_reset, color
            );
            println!();
            println!("{}", self.game);
        } else {
            error!("Could not load game\n");
        }
    }

    fn play(&mut self) {
        let time = (self.game.clock.allocated_time() as f64) / 1000.0;
        print!(
            "{}<{} wait {:.2} seconds{}",
            self.csi_color, self.csi_notif, time, self.csi_reset
        );
        let r = self.game.search(1..99);
        print!("\x1b[2K\x1b[1G");
        if let Some(m) = r {
            let s = m.to_lan();
            println!("{}<{} move {}", self.csi_color, self.csi_reset, s);
            println!();
            self.game.make_move(m);
            self.game.history.push(m);
            println!("{}", self.game);
        }
    }
}

fn is_move(m: &str) -> bool {
    let m = m.as_bytes();
    let n = m.len();
    if n < 4 || 5 < n {
        return false;
    }
    if m[0] < b'a' || b'h' < m[0] {
        return false;
    }
    if m[2] < b'a' || b'h' < m[2] {
        return false;
    }
    if m[1] < b'1' || b'8' < m[1] {
        return false;
    }
    if m[3] < b'1' || b'8' < m[3] {
        return false;
    }
    if n == 4 {
        return true;
    }
    if m[4] == b'b' || m[4] == b'n' || m[4] == b'r' || m[4] == b'q' {
        return true;
    }
    false
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    for &arg in args {
        match arg {
            "-h" | "--help" => return help(),
            _ => {}
        }
    }
    let mut chess = Chess::new();
    chess.run();
    Ok(())
}

pub fn help() -> Result<(), ExitCode> {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} chess {}{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
    Ok(())
}
