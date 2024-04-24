use std::vec;

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
        let movers = match color {
            Color::White => self.get_white_possible_movers(board, not_occupied),
            Color::Red => self.get_red_possible_movers(board, not_occupied),
        };
        movers
    }
}

impl BritishRules {
    fn get_white_possible_movers(&self, board: &BitBoard, not_occupied: u32) -> u32 {
        let jumpers = self.get_white_jumpers(board, not_occupied);
        if jumpers != 0 {
            return jumpers
        }
        return self.get_white_movers(board, not_occupied)
    }

    fn get_red_possible_movers(&self, board: &BitBoard, not_occupied: u32) -> u32 {
        let jumpers = self.get_red_jumpers(board, not_occupied);
        if jumpers != 0 {
            return jumpers
        }
        return self.get_red_movers(board, not_occupied)
    }

    fn get_white_movers(&self, board: &BitBoard, not_occupied: u32) -> u32 {
        let pieces = board.white_pawns | board.white_kings;
        let movers = (not_occupied << 4) & pieces;
        let movers_3 = (not_occupied & MASK_3_UP) << 3 & pieces;
        let movers_5 = (not_occupied & MASK_5_UP) << 5 & pieces;
        let mut movers = movers | movers_3 | movers_5;
        if board.white_kings != 0 {
            let kmovers = (not_occupied >> 4) & board.white_kings;
            let kmovers_3 = (not_occupied & MASK_3_DOWN) >> 3 & board.white_kings;
            let kmovers_5 = (not_occupied & MASK_5_DOWN) >> 5 & board.white_kings;
            movers = movers | kmovers | kmovers_3 | kmovers_5;
        }
        movers
    }

    fn get_red_movers(&self, board: &BitBoard, not_occupied: u32) -> u32 {
        let pieces = board.red_pawns | board.red_kings;
        let movers = (not_occupied >> 4) & pieces;
        let movers_3 = (not_occupied & MASK_3_DOWN) >> 3 & pieces;
        let movers_5 = (not_occupied & MASK_5_DOWN) >> 5 & pieces;
        let mut movers = movers | movers_3 | movers_5;
        if board.red_kings != 0 {
            let kmovers = (not_occupied << 4) & board.red_kings;
            let kmovers_3 = (not_occupied & MASK_3_UP) << 3 & board.red_kings;
            let kmovers_5 = (not_occupied & MASK_5_UP) << 5 & board.red_kings;
            movers = movers | kmovers | kmovers_3 | kmovers_5;
        }
        movers
    }

    fn get_white_jumpers(&self, board: &BitBoard, not_occupied: u32) -> u32 {
        let mut jumpers = 0;
        let pieces = board.white_pawns | board.white_kings;
        let opponent = board.red_pawns | board.red_kings;

        let targets = (not_occupied << 4) & opponent;
        if targets != 0 {
            jumpers |= (((targets & MASK_3_UP) << 3) | ((targets & MASK_5_UP) << 5)) & pieces;
        }

        let targets = (((not_occupied & MASK_3_UP) << 3) | ((not_occupied & MASK_5_UP) << 5)) & opponent;
        if targets != 0 {
            jumpers |= (targets << 4) & pieces;
        }

        if board.white_kings != 0 {
            let targets = (not_occupied >> 4) & opponent;
            if targets != 0 {
                jumpers |= (((targets & MASK_3_DOWN) >> 3) | ((targets & MASK_5_DOWN) >> 5)) & board.white_kings;
            }

            let targets = (((not_occupied & MASK_3_DOWN) >> 3) | ((not_occupied & MASK_5_DOWN) >> 5)) & opponent;
            if targets != 0 {
                jumpers |= (targets << 4) & board.white_kings;
            }
        }
        jumpers
    }

    fn get_red_jumpers(&self, board: &BitBoard, not_occupied: u32) -> u32 {
        let mut jumpers = 0;
        let pieces = board.red_pawns | board.red_kings;
        let opponent = board.white_pawns | board.white_kings;

        let targets = (not_occupied << 4) & opponent;
        if targets != 0 {
            jumpers |= (((targets & MASK_3_DOWN) >> 3) | ((targets & MASK_5_DOWN) >> 5)) & pieces;
        }

        let targets = (((not_occupied & MASK_3_DOWN) >> 3) | ((not_occupied & MASK_5_DOWN) >> 5)) & opponent;
        if targets != 0 {
            jumpers |= (targets << 4) & pieces;
        }

        if board.red_kings != 0 {
            let targets = (not_occupied >> 4) & opponent;
            if targets != 0 {
                jumpers |= (((targets & MASK_3_UP) << 3) | ((targets & MASK_5_UP) << 5)) & board.red_kings;
            }

            let targets = (((not_occupied & MASK_3_UP) << 3) | ((not_occupied & MASK_5_UP) << 5)) & opponent;
            if targets != 0 {
                jumpers |= (targets << 4) & board.red_kings;
            }
        }
        jumpers
    }

    // mover should have only one bit set
    fn get_moves(&self, board: &BitBoard, mover: u32, color: Color) -> Vec<u32> {
        match color {
            Color::White => self.get_white_moves(board, mover),
            Color::Red => self.get_red_moves(board, mover),
        }
    }

    fn get_white_moves(&self, board: &BitBoard, mover: u32) -> Vec<u32> {
        let not_occupied: u32 = !(board.white_pawns | board.red_pawns | board.red_kings | board.white_kings);
    
        let mut moves = Vec::new();
        if (not_occupied << 4) & mover != 0 {
            moves.push(mover | (mover >> 4));
        }
        if (not_occupied & MASK_3_UP) << 3 & mover != 0 {
            moves.push(mover | (mover >> 3));
        }
        if (not_occupied & MASK_5_UP) << 5 & mover != 0 {
            moves.push(mover | (mover >> 5));
        }

        if (board.white_kings & mover) != 0 {
            if (not_occupied >> 4) & mover != 0 {
                moves.push(mover | (mover << 4));
            }
            if (not_occupied & MASK_3_DOWN) >> 3 & mover != 0 {
                moves.push(mover | (mover << 3));
            }
            if (not_occupied & MASK_5_DOWN) >> 5 & mover != 0 {
                moves.push(mover | (mover << 5));
            }
        }

        return moves
    }

    fn get_red_moves(&self, board: &BitBoard, mover: u32) -> Vec<u32> {
        let not_occupied: u32 = !(board.white_pawns | board.red_pawns | board.red_kings | board.white_kings);
        let mut moves = Vec::new();
        if (not_occupied >> 4) & mover != 0 {
            moves.push(mover | (mover << 4));
        }
        if (not_occupied & MASK_3_DOWN) >> 3 & mover != 0 {
            moves.push(mover | (mover << 3));
        }
        if (not_occupied & MASK_5_DOWN) >> 5 & mover != 0 {
            moves.push(mover | (mover << 5));
        }

        if (board.red_kings & mover) != 0 {
            if (not_occupied << 4) & mover != 0 {
                moves.push(mover | (mover >> 4));
            }
            if (not_occupied & MASK_3_DOWN) << 3 & mover != 0 {
                moves.push(mover | (mover >> 3));
            }
            if (not_occupied & MASK_5_DOWN) << 5 & mover != 0 {
                moves.push(mover | (mover >> 5));
            }
        }

        return moves
    }

    // TODO?: eliminate recursion?
    fn get_white_jumps(&self, board: &BitBoard, start: u32, mover: u32, captures: u32) -> Vec<u32> {
        let not_occupied: u32 = start | captures | !(board.white_pawns | board.red_pawns | board.red_kings | board.white_kings);
        let opponent = (board.red_pawns | board.red_kings) ^ captures;

        let mut moves = Vec::new();
        let targets = (not_occupied << 4) & opponent;
        if targets != 0 && ((targets & MASK_3_UP) << 3) & mover != 0 {
            let mut result = self.get_white_jumps(board, start, mover >> 7, captures | (mover >> 3));
            moves.append(&mut result);
        }
        if targets != 0 && ((targets & MASK_5_UP) << 5) & mover != 0 {
            let mut result = self.get_white_jumps(board, start, mover >> 9, captures | (mover >> 5));
            moves.append(&mut result);
        }

        let targets = ((not_occupied & MASK_3_UP) << 3) & opponent;
        if targets != 0 && (targets << 4) & mover != 0 {
            let mut result = self.get_white_jumps(board, start, mover >> 7, captures | (mover >> 4));
            moves.append(&mut result);
        }

        let targets = ((not_occupied & MASK_5_UP) << 5) & opponent;
        if targets != 0 && (targets << 4) & mover != 0 {
            let mut result = self.get_white_jumps(board, start, mover >> 9, captures | (mover >> 4));
            moves.append(&mut result);
        }

        if (board.white_kings & mover) != 0 {
            let targets = (not_occupied >> 4) & opponent;
            if targets != 0 && ((targets & MASK_3_DOWN) >> 3) & mover != 0 {
                let mut result = self.get_white_jumps(board, start, mover << 7, captures | (mover << 3));
                moves.append(&mut result);
            }
            if targets != 0 && ((targets & MASK_5_DOWN) >> 5) & mover != 0 {
                let mut result = self.get_white_jumps(board, start, mover << 9, captures | (mover << 5));
                moves.append(&mut result);
            }
            let targets = ((not_occupied & MASK_3_DOWN) >> 3) & opponent;
            if targets != 0 && (targets >> 4) & mover != 0 {
                let mut result = self.get_white_jumps(board, start, mover << 7, captures | (mover << 4));
                moves.append(&mut result);
            }

            let targets = ((not_occupied & MASK_5_DOWN) >> 5) & opponent;
            if targets != 0 && (targets >> 4) & mover != 0 {
                let mut result = self.get_white_jumps(board, start, mover << 9, captures | (mover << 4));
                moves.append(&mut result);
            }
        }

        if moves.len() == 0 {
            if captures != 0 {
                return vec![captures | start | mover]
            }
            return vec![]
        }
        moves
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;

    #[test]
    fn test_full_white_board() {
        let board = BitBoard {
            white_pawns: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b1111_1111_1111_1111_1111_1111_1111_1111,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let movers = rules.get_possible_movers(&board, Color::White);
        assert_eq!(movers, 0b0000_0000_0000_0000_0000_0000_0000_0000);
    }

    #[test]
    fn test_full_red_board() {
        let board = BitBoard {
            white_pawns: 0b1111_1111_1111_1111_1111_1111_1111_1111,
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let movers = rules.get_possible_movers(&board, Color::Red);
        assert_eq!(movers, 0b0000_0000_0000_0000_0000_0000_0000_0000);
    }

    #[test]
    fn test_start_position_red() {
        let board = BitBoard {
            white_pawns: 0b1111_1111_1111_0000_0000_0000_0000_0000,
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_0000_0000_0000_1111_1111_1111,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let movers = rules.get_possible_movers(&board, Color::Red);
        assert_eq!(movers, 0b0000_0000_0000_0000_0000_1111_0000_0000);
    }

    #[test]
    fn test_start_position_white() {
        let board = BitBoard {
            white_pawns: 0b1111_1111_1111_0000_0000_0000_0000_0000,
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_0000_0000_0000_1111_1111_1111,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let movers = rules.get_possible_movers(&board, Color::White);
        assert_eq!(movers, 0b0000_0000_1111_0000_0000_0000_0000_0000);
    }

    #[test]
    fn test_single_white() {
        let board = BitBoard {
            white_pawns: 0b0000_0000_0000_0010_0000_0000_0000_0000,
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_0000_0000_0000_0000_0000_1111,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let movers = rules.get_possible_movers(&board, Color::White);
        assert_eq!(movers, 0b0000_0000_0000_0010_0000_0000_0000_0000);
    }

    #[test]
    fn test_white_capture_enforced() {
        let board = BitBoard {
            white_pawns: 0b0000_1111_0000_0010_0000_0000_0000_0000,
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_0000_0000_0100_0000_0000_0000,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let movers = rules.get_possible_movers(&board, Color::White);
        assert_eq!(movers, 0b0000_0000_0000_0010_0000_0000_0000_0000);
    }

    #[test]
    fn test_white_without_capture() {
        let board = BitBoard {
            white_pawns: 0b0000_1111_0000_0010_0000_0000_0000_0000,
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_0000_0000_1000_0000_0000_0000,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let movers = rules.get_possible_movers(&board, Color::White);
        assert_eq!(movers, 0b0000_1111_0000_0010_0000_0000_0000_0000);
    }

    #[test]
    fn test_red_capture_enforced() {
        let board = BitBoard {
            white_pawns:   0b0000_0000_0000_0010_0000_0000_0000_0000,
            white_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:     0b0000_0000_0000_0000_0100_0000_1111_0000,
            red_kings:     0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let movers = rules.get_possible_movers(&board, Color::Red);
        assert_eq!(movers, 0b0000_0000_0000_0000_0100_0000_0000_0000);
    }

    #[test]
    fn test_red_without_capture() {
        let board = BitBoard {
            white_pawns:   0b0000_0000_0000_0001_0000_0000_0000_0000,
            white_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:     0b0000_0000_0000_0000_0100_0000_1111_0000,
            red_kings:     0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let movers = rules.get_possible_movers(&board, Color::Red);
        assert_eq!(movers, 0b0000_0000_0000_0000_0100_0000_1111_0000);
    }

    #[test]
    fn test_get_white_moves_regular_pawn() {
        let board = BitBoard {
            white_pawns:   0b0000_0000_0000_0001_0000_0000_0000_0000,
            white_kings:   0,
            red_pawns:     0,
            red_kings:     0,
        };

        let expected_moves = vec![
                           0b0000_0000_0000_0001_0001_0000_0000_0000,
                           0b0000_0000_0000_0001_0010_0000_0000_0000,
        ];

        let mover =        0b0000_0000_0000_0001_0000_0000_0000_0000;
        let rules = BritishRules::new();
        let mut moves = rules.get_moves(&board, mover, Color::White);
        moves.sort();

        assert_eq!(moves, expected_moves);
    }

    #[test]
    fn test_get_white_moves_king() {
        let board = BitBoard {
            white_pawns:   0,
            white_kings:   0b0000_0000_0000_0001_0000_0000_0000_0000,
            red_pawns:     0,
            red_kings:     0,
        };

        let expected_moves = vec![
                           0b0000_0000_0000_0001_0001_0000_0000_0000,
                           0b0000_0000_0000_0001_0010_0000_0000_0000,
                           0b0000_0000_0001_0001_0000_0000_0000_0000,
                           0b0000_0000_0010_0001_0000_0000_0000_0000,
        ];

        let mover =        0b0000_0000_0000_0001_0000_0000_0000_0000;
        let rules = BritishRules::new();
        let mut moves = rules.get_moves(&board, mover, Color::White);
        moves.sort();

        assert_eq!(moves, expected_moves);
    }

    #[test]
    fn test_get_red_moves_regular_pawn() {
        let board = BitBoard {
            white_pawns:   0,
            white_kings:   0,
            red_pawns:     0b0000_0000_0000_0001_0000_0000_0000_0000,
            red_kings:     0,
        };

        let expected_moves = vec![
                           0b0000_0000_0001_0001_0000_0000_0000_0000,
                           0b0000_0000_0010_0001_0000_0000_0000_0000,
        ];

        let mover =        0b0000_0000_0000_0001_0000_0000_0000_0000;
        let rules = BritishRules::new();
        let mut moves = rules.get_moves(&board, mover, Color::Red);
        moves.sort();

        assert_eq!(moves, expected_moves);
    }

    #[test]
    fn test_get_red_moves_king() {
        let board = BitBoard {
            white_pawns:   0,
            white_kings:   0,
            red_pawns:     0,
            red_kings:     0b0000_0000_0000_0001_0000_0000_0000_0000,
        };

        let expected_moves = vec![
                           0b0000_0000_0000_0001_0001_0000_0000_0000,
                           0b0000_0000_0001_0001_0000_0000_0000_0000,
                           0b0000_0000_0010_0001_0000_0000_0000_0000,
        ];

        let mover =        0b0000_0000_0000_0001_0000_0000_0000_0000;
        let rules = BritishRules::new();
        let mut moves = rules.get_moves(&board, mover, Color::Red);
        moves.sort();

        assert_eq!(moves, expected_moves);
    }
}
