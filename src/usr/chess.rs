use crate::{api, usr};
use crate::api::console::Style;
use crate::api::prompt::Prompt;

use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use littlewing::game::Game;
use littlewing::fen::FEN;
use littlewing::search::Search;
use littlewing::piece_move_generator::{PieceMoveGenerator, PieceMoveGeneratorExt};
use littlewing::piece_move_notation::PieceMoveNotation;
use littlewing::clock::Clock;

use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref MOVES: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

const COMMANDS: [&str; 2] = ["exit", "move"];

fn update_autocomplete(prompt: &mut Prompt, game: &mut Game) {
    *MOVES.lock() = game.get_moves().into_iter().map(|m| m.to_lan()).collect();

    fn chess_completer(line: &str) -> Vec<String> {
        let mut entries = Vec::new();
        let args: Vec<&str> = line.split(' ').collect();
        let i = args.len() - 1;
        if i == 0 { // Autocomplete command
            for &cmd in &COMMANDS {
                if let Some(entry) = cmd.strip_prefix(args[i]) {
                    entries.push(entry.into());
                }
            }
        } else if i == 1 && args[0] == "move" { // Autocomplete moves
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
    (api::syscall::realtime() * 1000.0) as u128
}

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    println!("MOROS Chess v0.1.0\n");

    let csi_color = Style::color("Cyan");
    let csi_error = Style::color("Red");
    let csi_reset = Style::reset();
    let prompt_string = format!("{}>{} ", csi_color, csi_reset);

    let mut prompt = Prompt::new();
    let history_file = "~/.chess-history";
    prompt.history.load(history_file);

    let mut game = Game::new();
    game.show_coordinates = true;
    game.clock = Clock::new(40, 5 * 60 * 1000); // 40 moves in 5 minutes
    game.clock.system_time = Arc::new(system_time);
    let size = 1 << 20; // MB
    game.tt_resize(size);
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    game.load_fen(fen).unwrap();
    println!("{}", game);

    update_autocomplete(&mut prompt, &mut game);
    while let Some(cmd) = prompt.input(&prompt_string) {
        let args: Vec<&str> = cmd.trim().split(' ').collect();
        match args[0] {
            "exit" => {
                break
            },
            "move" => {
                if args.len() < 2 {
                    println!("{}Error:{} no <move> given\n", csi_error, csi_reset);
                    continue;
                }
                if !is_move(args[1]) {
                    println!("{}Error:{} invalid move '{}'\n", csi_error, csi_reset, args[1]);
                    continue;
                }
                let m = game.move_from_lan(args[1]);
                if !game.is_legal_move(m) {
                    println!("{}Error:{} illegal move '{}'\n", csi_error, csi_reset, args[1]);
                    continue;
                }

                game.make_move(m);
                game.history.push(m);

                let r = game.search(1..99);
                if let Some(m) = r {
                    println!();
                    println!("{}<{} move {}", csi_color, csi_reset, m.to_lan());
                    game.make_move(m);
                    game.history.push(m);
                    println!();
                    println!("{}", game);
                }
            },
            _ => {
                println!();
            }
        }
        prompt.history.add(&cmd);
        prompt.history.save(history_file);
        update_autocomplete(&mut prompt, &mut game);
    }
    usr::shell::ExitCode::CommandSuccessful
}

fn is_move(m: &str) -> bool {
    let m = m.as_bytes();
    let n = m.len();
    if n < 3 || 5 < n {
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
