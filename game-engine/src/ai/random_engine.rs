use crate::{ai::Engine, board::BitBoard};

use super::{Move, Pos};

pub struct RandomEngine {
}

impl RandomEngine {
    pub fn new() -> RandomEngine {
        RandomEngine { }
    }
}

impl Engine for RandomEngine {
    fn get_name(&self) -> String {
        String::from("Random Engine")
    }

    fn get_move(&mut self, _board: &BitBoard) -> Move {
        Move {
            start: Pos { x: 0, y: 0 },
            end: Pos { x: 0, y: 0 },
            result: BitBoard {
                red_pawns: 0,
                red_kings: 0,
                white_pawns: 0,
                white_kings: 0,
            },
        }
    }
}
