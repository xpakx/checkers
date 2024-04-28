use rand::{rngs::ThreadRng, thread_rng, Rng};

use crate::{ai::Engine, board::BitBoard, rules::Rules, Color, BIT_MASK};

#[allow(dead_code)]
pub struct CountingEngine {
    rng: ThreadRng,
}

#[allow(dead_code)]
impl CountingEngine {
    pub fn new() -> CountingEngine {
        CountingEngine {
            rng: thread_rng(),
        }
    }
}

#[allow(dead_code)]
impl Engine for CountingEngine {
    fn get_name(&self) -> String {
        String::from("Counting Engine")
    }

    fn get_move(&mut self, board: &BitBoard, color: &Color, rules: &Box<dyn Rules>) -> u32 {
        return 0
    }

}

#[allow(dead_code)]
impl CountingEngine {
    fn evaluate(&self, board: &BitBoard, color: &Color) -> u32 {
        let (kings, opponent_kings) = match color {
            Color::Red => (board.red_kings, board.white_kings),
            Color::White => (board.white_kings, board.red_kings),
        };
        let (pawns, opponent_pawns) = match color {
            Color::Red => (board.red_pawns, board.white_pawns),
            Color::White => (board.white_pawns, board.red_pawns),
        };
        let strength = 10*kings.count_ones() + pawns.count_ones();
        let opponent_strength = 10*opponent_kings.count_zeros() + opponent_pawns.count_ones();
        strength - opponent_strength
    }
}
