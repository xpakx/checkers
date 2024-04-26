mod board;
mod ai;
mod rules;
mod rabbit;
mod config;

use crate::ai::{get_engine, EngineType};
use crate::rules::{get_rules, RuleSet, MoveVerification};
use crate::rabbit::lapin_listen;
use crate::board::move_to_bitboard;
use regex::Regex;

#[tokio::main]
async fn main() {
    let board = "xxxxxxxxxxxx........oooooooooooo";
    let bitboard = board::generate_bit_board(board.into());
    println!("{:?}", bitboard);
    let bitboard = bitboard.unwrap();

    println!("{:032b}", bitboard.white_pawns);
    println!("{:032b}", bitboard.red_pawns);
    bitboard.print();
    let mut engine = get_engine(EngineType::Random);
    let rules = get_rules(RuleSet::British);
    println!("{}", engine.get_name());
    let mov = engine.get_move(&bitboard, &Color::White, &rules);
    println!("{:032b}", mov);
    let bitboard = bitboard.apply_move(mov, Color::White);
    bitboard.print();
    println!("{:?}", bitboard);
    let mov = engine.get_move(&bitboard, &Color::Red, &rules);
    let bitboard = bitboard.apply_move(mov, Color::Red);
    bitboard.print();
    println!("{:?}", bitboard);
    let mov = engine.get_move(&bitboard, &Color::White, &rules);
    let bitboard = bitboard.apply_move(mov, Color::White);
    bitboard.print();
    println!("{:?}", bitboard);
    let mov = engine.get_move(&bitboard, &Color::Red, &rules);
    let bitboard = bitboard.apply_move(mov, Color::Red);
    bitboard.print();
    println!("{:?}", bitboard);

    println!("rules: {:?}", rules.get_definition());
    println!("white: {:032b}", rules.get_possible_movers(&bitboard, &Color::White));
    println!("red: {:032b}", rules.get_possible_movers(&bitboard, &Color::Red));

    let moves = vec!["10x13", "10-1", "10-1-15-4", "2x5x4", "12x32x30", "12x34x56", "10xxx10", "x10x10"];
    for mov in moves {
        match move_to_bitboard(String::from(mov)) {
            Ok(bitboard) => println!("{}, Bitboard representation: {:032b}", mov, bitboard.mov),
            Err(err) => println!("{}, Error: {:?}", mov, err),
        }
    }
    let board = "xxxxxxxxxxxx........oooooooooooo";
    let bitboard = board::generate_bit_board(board.into());
    let bitboard = bitboard.unwrap();
    bitboard.print();
    let mov = move_to_bitboard("10-15".into());
    let mov = rules.verify_move(&bitboard, mov.unwrap(), &Color::White);
    if let MoveVerification::Ok(mov) = mov {
        let bitboard = bitboard.apply_move(mov, Color::White);
        bitboard.print();
    }


    let config = config::get_config();
    let mut cfg = deadpool_lapin::Config::default();
    cfg.url = Some(config.rabbit.into());
    let lapin_pool = cfg.create_pool(Some(deadpool_lapin::Runtime::Tokio1)).unwrap();
    lapin_listen(lapin_pool.clone()).await;
}

#[derive(Debug, Clone)]
pub enum Color {
    White,
    Red,
}

const BIT_MASK: u32 = 0b1000_0000_0000_0000_0000_0000_0000_0000;
