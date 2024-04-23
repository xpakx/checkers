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
