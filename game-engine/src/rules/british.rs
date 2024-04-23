use crate::{rules::Rules, board::BitBoard, ai::{Move, Pos}};

// no flying kings, 8x8, pawns cannot move backwards, 
// any capture sequence can be chosen, but captures are forced, 
// red moves first 
pub struct BritishRules {
}

impl BritishRules {
    pub fn new() -> BritishRules {
        BritishRules { }
    }
}

impl Rules for BritishRules {
    fn get_possible_moves(&self, _board: &BitBoard) -> Vec<Move> {
        vec![
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
        ]

    }
}
