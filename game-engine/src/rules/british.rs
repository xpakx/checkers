use crate::{board::BitBoard, rules::Rules, Color};

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

const MASK_3_DOWN: u32 = 0b0000_0111_0000_0111_0000_0111_0000_0000;
const MASK_3_UP: u32 = 0b0000_0000_1110_0000_1110_0000_1110_0000;
const MASK_5_DOWN: u32 = 0b1110_0000_1110_0000_1110_0000_1110_0000;
const MASK_5_UP: u32 = 0b0000_0111_0000_0111_0000_0111_0000_0111;

impl Rules for BritishRules {
    fn get_possible_movers(&self, board: &BitBoard, color: Color) -> u32 {
        let not_occupied: u32 = !(board.white_pawns | board.red_pawns | board.red_kings | board.white_kings);
        // TODO: jumpers
        let movers = match color {
            Color::White => self.get_white_movers(board, not_occupied),
            Color::Red => self.get_red_movers(board, not_occupied),
        };
        movers
    }
}

impl BritishRules {
    fn get_white_movers(&self, board: &BitBoard, not_occupied: u32) -> u32 {
        let pieces = board.white_pawns | board.white_kings;
        let movers = (not_occupied << 4) & pieces;
        let movers_3 = (not_occupied & MASK_3_UP) << 3 & pieces;
        let movers_5 = (not_occupied & MASK_5_UP) << 5 & pieces;
        println!("WHITE");
        println!("4: {:032b}", movers);
        println!("3: {:032b}", movers_3);
        println!("5: {:032b}", movers_5);
        movers | movers_3 | movers_5
    }

    fn get_red_movers(&self, board: &BitBoard, not_occupied: u32) -> u32 {
        let pieces = board.red_pawns | board.red_kings;
        let movers = (not_occupied >> 4) & pieces;
        let movers_3 = (not_occupied & MASK_3_DOWN) >> 3 & pieces;
        let movers_5 = (not_occupied & MASK_5_DOWN) >> 5 & pieces;
        println!("RED");
        println!("4: {:032b}", movers);
        println!("3: {:032b}", movers_3);
        println!("5: {:032b}", movers_5);
        movers | movers_3 | movers_5
    }
}
