use chess::{BitBoard, Coord};

fn main() {
    let mut pawn_moves = [BitBoard::new(0); 64];

    for i in 0..64 {
        let coord = Coord::from_offset(i);

        if coord.row() == 8 {
            continue;
        }

        let mut moves = BitBoard::new(0);

        if let Some(mv) = coord.mv(0, -1) {
            moves.set(mv);
        }

        if coord.row() == 7 {
            moves.set(coord.mv(0, -2).unwrap());
        }

        pawn_moves[i] = moves;
    }

    for moves in pawn_moves {
        println!("{moves:?},");
    }
}

