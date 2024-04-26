use crate::Color;
use crate::Regex;
use crate::BIT_MASK;

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

fn without_captures(pre: u32, mov: u32, empty: u32) -> u32 {
    (pre ^ (pre & mov)) | (empty & mov)
}

fn without_moved(pre: u32, mov: u32) -> u32 {
    pre ^ (pre & mov)
}

impl BitBoard {
    pub fn apply_move(&self, mov: u32, color: Color) -> BitBoard {
        let empty: u32 = !(self.white_pawns | self.red_pawns | self.red_kings | self.white_kings);
        BitBoard { 
            white_pawns: match color {
                Color::White => without_captures(self.white_pawns, mov, empty),
                Color::Red => without_moved(self.white_pawns, mov),
            },
            white_kings: match color {
                Color::White => without_captures(self.white_kings, mov, empty),
                Color::Red => without_moved(self.white_kings, mov),
            },
            red_pawns: match color {
                Color::White => without_moved(self.red_pawns, mov),
                Color::Red => without_captures(self.red_pawns, mov, empty),
            },
            red_kings: match color {
                Color::White => without_moved(self.red_kings, mov),
                Color::Red => without_captures(self.red_kings, mov, empty),
            },
        }
    }

    #[allow(dead_code)]
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

#[derive(Debug)]
pub enum ParseError {
    InvalidFormat,
    NumberOverflow,
    InvalidDigit,
}



pub fn move_to_bitboard(move_string: String) -> Result<MoveBit, ParseError> {
    let move_regex = Regex::new(r"^(\d+(x|-))*\d+$").unwrap();

    if !move_regex.is_match(move_string.as_str()) {
        return Err(ParseError::InvalidFormat);
    }

    let mut current_num = 0;
    let mut mov: u32 = 0;
    let mut start_end: u32 = 0;

    for c in move_string.chars() {
        if c.is_digit(10) {
            current_num *= 10;
            current_num += c.to_digit(10).ok_or(ParseError::InvalidDigit)?;
            if current_num > 32 {
                return Err(ParseError::NumberOverflow);
            }
        } 
        match c {
            'x' => {
                if start_end == 0 {
                    start_end = BIT_MASK >> (current_num - 1);
                    current_num = 0;
                    continue;
                }
                if current_num != 0 {
                    mov |= BIT_MASK >> (current_num - 1);
                    current_num = 0;
                }
            },
            '-' => {
                if start_end == 0 {
                    start_end = BIT_MASK >> (current_num - 1);
                }
                current_num = 0;
            },
            _ => {}
        }
    }

    start_end |=  BIT_MASK >> (current_num - 1);
    mov |= start_end;
    Ok(MoveBit { start_end, mov })
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct MoveBit {
    pub mov: u32,
    pub start_end: u32,
}
