use std::io::{stdin, stdout, Write};

use board::Board;
use mcts::SearchTree;

use crate::board::Player;

mod board;
mod mcts;
const WIDTH: usize = 15;
const HEIGHT: usize = 15;
const MAX_TIME_LIMIT: f32 = 20.0;

fn parse_move(cmd: &str) -> Result<[usize; 2], String> {
    if cmd.len() != 2 {
        return Err(String::from("Input must be exactly two characters"));
    }
    let chars: Vec<char> = cmd.chars().collect();

    let usizeconvert = |n| usize::try_from(n).ok();
    let first_num = chars[0].to_digit(WIDTH as u32 + 1).and_then(usizeconvert);
    let second_num = chars[1].to_digit(HEIGHT as u32 + 1).and_then(usizeconvert);

    match (first_num, second_num) {
        (Some(n1), Some(n2)) => Ok([n1, n2]),
        _ => Err(String::from("Unable to parse input into two numbers")),
    }
}

fn main() {
    let mut board = Board::new(WIDTH, HEIGHT);
    let mut search_tree = SearchTree::new(board.clone());
    let mut move_number = 0;
    loop {
        println!("{board}");
        // println!("{:?}", board.utility(board::Player::X));
        // println!("{:?}", board.actions());
        // println!("{search_tree}");
        print!("X TO MOVE:");
        stdout().flush().expect("Error when printing text");
        let mut cmd = String::new();
        stdin()
            .read_line(&mut cmd)
            .expect("Error when reading command");
        let cmd = cmd.trim();
        if cmd.to_uppercase() == "Q" {
            break;
        }

        let [x, y] = match parse_move(cmd.trim()) {
            Ok(m) => m,
            Err(e) => {
                println!("{e}");
                board.place_random().unwrap();
                continue;
            }
        };
        let m = [
            x.checked_sub(1).unwrap_or(WIDTH),
            y.checked_sub(1).unwrap_or(HEIGHT),
        ];
        if let Err(e) = board.place(m) {
            println!("{e}");
            continue;
        }
        if board.utility(Player::X).is_some() {
            println!("{board}");
            println!("X WINS");
            break;
        }

        search_tree.apply_move(m);
        move_number += 1;
        let time_limit = MAX_TIME_LIMIT * (1.0 - 7.0 / (move_number as f32 + 6.7));
        // search for move using mcts
        let m = search_tree.monte_carlo(time_limit);
        // print!("{search_tree}");
        if let Err(e) = board.place(m) {
            println!("{e}");
            continue;
        }
        if board.utility(Player::O).is_some() {
            println!("{board}");
            println!("O WINS");
            break;
        }
        search_tree.apply_move(m);

        // board.place_random(BoardValue::O);
    }
}
