use crate::{Color, BIT_MASK};

#[derive(Debug)]
pub struct BitBoard {
    pub white_pawns: u32,
    pub white_kings: u32,
    pub red_pawns: u32,
    pub red_kings: u32,
}

pub fn generate_bit_board(string_board: String) -> Result<BitBoard, String> {
        let mut white_pawns = 0;
        let mut white_kings = 0;
        let mut red_pawns = 0;
        let mut red_kings = 0;
        for a in string_board.chars() {
            white_pawns = white_pawns << 1;
            white_kings = white_kings << 1;
            red_pawns = red_pawns << 1;
            red_kings = red_kings << 1;
            match a {
                'x' => white_pawns += 1,
                'X' => white_kings += 1,
                'o' => red_pawns += 1,
                'O' => red_kings += 1,
                '.' => {},
                _ => return Err("".into()),
            }
        }
        Ok(BitBoard {white_pawns, white_kings, red_pawns, red_kings})
}

impl BitBoard {
    pub fn apply_move(&self, mov: u32, color: Color) -> BitBoard {
        let not_occupied: u32 = !(self.white_pawns | self.red_pawns | self.red_kings | self.white_kings);
        BitBoard { 
            white_pawns: match color {
                Color::White => (self.white_pawns ^ mov) | (not_occupied & mov),
                Color::Red => self.white_pawns ^ mov,
            },
            white_kings: match color {
                Color::White => (self.white_kings ^ mov) | (not_occupied & mov),
                Color::Red => self.white_kings ^ mov,
            },
            red_pawns: match color {
                Color::White => self.red_pawns ^ mov,
                Color::Red => (self.red_pawns ^ mov) | (not_occupied & mov),
            },
            red_kings: match color {
                Color::White => self.red_kings ^ mov,
                Color::Red => (self.red_kings ^ mov) | (not_occupied & mov),
            },
        }
    }

    pub fn print(&self) {
        let mut text = String::from("");
        let mut num = 0;
        for row in 1..=8 {
            for column in 1..=8 {
                if row % 2 != column % 2 {
                    let white_pawn = (self.white_pawns & (BIT_MASK >> num)) != 0;
                    let red_pawn = (self.red_pawns & (BIT_MASK >> num)) != 0;
                    let white_king = (self.white_kings & (BIT_MASK >> num)) != 0;
                    let red_king = (self.red_kings & (BIT_MASK >> num)) != 0;
                    if white_pawn {
                        text += " ⛀ ";
                    } else if red_pawn {
                        text += " ⛂ ";
                    } else if white_king {
                        text += " ⛁ ";
                    } else if red_king {
                        text += " ⛃ ";
                    } else {
                        text += "   ";
                    }
                    num += 1;
                } else {
                    text += "   ";
                }
            }
            text += "\n";

        }
        println!("{}", text);
    }
}
