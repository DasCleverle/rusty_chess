use chess::{Coord, BitBoard};

const KNIGHT_JUMPS: [(isize, isize); 8] = [(-2, 1), (-1, 2), (1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1)];

fn main() {
    for i in 0..64 {
        let coord = Coord::from_offset(i);
        let mut knight_moves = BitBoard::new(0);

        for direction in KNIGHT_JUMPS {
            if let Some(to) = coord.mv(direction.0, direction.1) {
                knight_moves.set(to);
            }
        }

        println!("BitBoard({}),", knight_moves.0);
    }

}
