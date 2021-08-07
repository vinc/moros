use crate::{api, usr};
use crate::api::console::Style;
use crate::api::prompt::Prompt;

use alloc::format;
use alloc::sync::Arc;
use alloc::vec::Vec;

use littlewing::game::Game;
use littlewing::fen::FEN;
use littlewing::search::Search;
use littlewing::piece_move_generator::PieceMoveGenerator;
use littlewing::piece_move_notation::PieceMoveNotation;

use littlewing::clock::Clock;

fn system_time() -> u128 {
    (api::syscall::realtime() * 1000.0) as u128
}

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    println!("MOROS Chess v0.1.0\n");

    let csi_color = Style::color("Cyan");
    //let csi_error = Style::color("Red");
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

    while let Some(cmd) = prompt.input(&prompt_string) {
        let args: Vec<&str> = cmd.trim().split(' ').collect();
        match args[0] {
            "exit" => {
                break
            },
            "move" => {
                if args.len() > 1 {
                    let m = game.move_from_lan(args[1]);
                    game.make_move(m);
                    game.history.push(m);

                    let r = game.search(1..99);
                    if let Some(m) = r {
                        println!("{}<{} move {}", csi_color, csi_reset, m.to_lan());
                        game.make_move(m);
                        game.history.push(m);
                        println!();
                        println!("{}", game);
                    }
                }
            },
            _ => {
                println!();
            }
        }
    }
    usr::shell::ExitCode::CommandSuccessful
}
