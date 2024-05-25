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

fn kings_without_moved(pre: u32, mov: u32, empty: u32) -> u32 {
    let end = empty & mov;
    let end = match end {
        0 => !empty & mov,
        _ => end,
    };
    (pre ^ (pre & mov)) | end
}

fn without_moved_and_promoted(pre: u32, mov: u32, empty: u32, mask: u32) -> u32 {
    (pre ^ (pre & mov)) | (empty & mov & !mask)
}

fn without_captures(pre: u32, mov: u32) -> u32 {
    pre ^ (pre & mov)
}

const RED_PROMOTION: u32 = 0b1111_0000_0000_0000_0000_0000_0000_0000;
const WHITE_PROMOTION: u32 = 0b0000_0000_0000_0000_0000_0000_0000_1111;
fn with_promoted(kings: u32, mov: u32, mask: u32) -> u32 {
    kings | (mask & mov)
}

impl BitBoard {
    pub fn apply_move(&self, mov: u32, color: &Color) -> BitBoard {
        let empty: u32 = !(self.white_pawns | self.red_pawns | self.red_kings | self.white_kings);
        let pawn_move = match color {
            Color::White => self.white_pawns,
            Color::Red => self.red_pawns,
        } & mov > 0;
        BitBoard { 
            white_pawns: match color {
                Color::White => match pawn_move {
                    true => without_moved_and_promoted(self.white_pawns, mov, empty, WHITE_PROMOTION),
                    false => self.white_pawns,
                },
                Color::Red => without_captures(self.white_pawns, mov),
            },
            white_kings: match color {
                Color::White => match pawn_move {
                    false => kings_without_moved(self.white_kings, mov, empty),
                    true => with_promoted(self.white_kings, mov, WHITE_PROMOTION),
                },
                Color::Red => without_captures(self.white_kings, mov),
            },
            red_pawns: match color {
                Color::White => without_captures(self.red_pawns, mov),
                Color::Red => match pawn_move {
                    true => without_moved_and_promoted(self.red_pawns, mov, empty, RED_PROMOTION),
                    false => self.red_pawns,
                },
            },
            red_kings: match color {
                Color::White => without_captures(self.red_kings, mov),
                Color::Red => match pawn_move {
                    false => kings_without_moved(self.red_kings, mov, empty),
                    true => with_promoted(self.red_kings, mov, RED_PROMOTION),
                },
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

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for i in 1..=32 {
            let white_pawns = self.white_pawns & (BIT_MASK >> (i-1)) != 0;
            let white_kings = self.white_kings & (BIT_MASK >> (i-1)) != 0;
            let red_pawns = self.red_pawns & (BIT_MASK >> (i-1)) != 0;
            let red_kings = self.red_kings & (BIT_MASK >> (i-1)) != 0;
            if white_pawns {
                result += "x";
            } else if white_kings {
                result += "X";
            } else if red_pawns {
                result += "o";
            } else if red_kings {
                result += "O";
            } else {
                result += ".";
            }
        }
        result
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

pub fn have_captures(old: &BitBoard, new: &BitBoard, color: &Color) -> bool {
    let enemy_old = match color {
        Color::White => old.red_pawns | old.red_kings,
        Color::Red => old.white_pawns | old.white_kings,
    }.count_ones();
    let enemy_new = match color {
        Color::White => new.red_pawns | new.red_kings,
        Color::Red => new.white_pawns | new.white_kings,
    }.count_ones();
    enemy_old < enemy_new
}

pub fn have_promotions(old: &BitBoard, new: &BitBoard, color: &Color) -> bool {
    let my_old = match color {
        Color::White => old.white_kings,
        Color::Red => old.red_kings,
    }.count_ones();
    let my_new = match color {
        Color::White => new.white_kings,
        Color::Red => new.red_kings,
    }.count_ones();
    my_new > my_old
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_input() {
        let bit = move_to_bitboard("4-7".to_string()).unwrap();
        assert_eq!(bit.mov, 0b00010010000000000000000000000000);
        assert_eq!(bit.start_end, 0b00010010000000000000000000000000);
    }

    #[test]
    fn test_valid_input_with_capture() {
        let bit = move_to_bitboard("5x4".to_string()).unwrap();
        assert_eq!(bit.mov, 0b00011000000000000000000000000000);
        assert_eq!(bit.start_end, 0b00011000000000000000000000000000);
    }

    #[test]
    fn test_valid_input_multiple_moves() {
        let bit = move_to_bitboard("3x4-7".to_string()).unwrap();
        assert_eq!(bit.mov, 0b00100010000000000000000000000000);
        assert_eq!(bit.start_end, 0b00100010000000000000000000000000);
    }

    #[test]
    fn test_valid_input_multiple_captures() {
        let bit = move_to_bitboard("3x4x7".to_string()).unwrap();
        assert_eq!(bit.mov, 0b00110010000000000000000000000000);
        assert_eq!(bit.start_end, 0b00100010000000000000000000000000);
    }

    #[test]
    fn test_invalid_input_invalid_character() {
        let bit = move_to_bitboard("4&7".to_string());
        assert!(bit.is_err());
        assert_eq!(format!("{:?}", bit.unwrap_err()), format!("{:?}", ParseError::InvalidFormat));
    }

    #[test]
    fn test_invalid_input_number_overflow() {
        let bit = move_to_bitboard("40x7".to_string());
        assert_eq!(format!("{:?}", bit.unwrap_err()), format!("{:?}", ParseError::NumberOverflow));
    }

    #[test]
    fn test_valid_input_single_move() {
        let bit = move_to_bitboard("8".to_string()).unwrap();
        assert_eq!(bit.mov, 0b00000001000000000000000000000000);
        assert_eq!(bit.start_end, 0b00000001000000000000000000000000);
    }

    #[test]
    fn test_valid_input_reversed_start_end() {
        let bit = move_to_bitboard("7-4".to_string()).unwrap();
        assert_eq!(bit.mov, 0b00010010000000000000000000000000);
        assert_eq!(bit.start_end, 0b00010010000000000000000000000000);
    }

    #[test]
    fn test_valid_input_full_board_move() {
        let bit = move_to_bitboard("1-32".to_string()).unwrap();
        assert_eq!(bit.mov, 0b10000000000000000000000000000001);
        assert_eq!(bit.start_end, 0b10000000000000000000000000000001);
    }
}
