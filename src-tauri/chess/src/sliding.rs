use crate::{Coord, bitboard::BitBoard};

pub fn get_rook_move_mask(from: Coord, blockers: &BitBoard, friendly_pieces: &BitBoard) -> BitBoard {
    get_blocked_move_mask(from, blockers, friendly_pieces, &ROOK_MOVES, &ROOK_DIRECTIONS)
}

pub fn get_bishop_move_mask(from: Coord, blockers: &BitBoard, friendly_pieces: &BitBoard) -> BitBoard {
    get_blocked_move_mask(from, blockers, friendly_pieces, &BISHOP_MOVES, &BISHOP_DIRECTIONS)
}

fn get_blocked_move_mask(
    from: Coord,
    blockers: &BitBoard,
    friendly_pieces: &BitBoard,
    moves: &[BitBoard; 64],
    directions: &[(isize, isize); 4],
) -> BitBoard {
    let mut moves = moves[from.offset()];

    for direction in directions {
        let mut coord = from.clone();
        let mut found_blocker = false;

        while coord.mv_mut(direction.0, direction.1) {
            if found_blocker {
                moves.unset(coord);
            }

            if blockers.is_set(coord) {
                found_blocker = true;
            }
        }
    }

    moves = moves & !friendly_pieces;

    return moves;
}

const ROOK_DIRECTIONS: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const BISHOP_DIRECTIONS: [(isize, isize); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

lazy_static! {
    static ref ROOK_MOVES: [BitBoard; 64] = get_all_move_masks(ROOK_DIRECTIONS);
    static ref BISHOP_MOVES: [BitBoard; 64] = get_all_move_masks(BISHOP_DIRECTIONS);
}

fn get_all_move_masks(directions: [(isize, isize); 4]) -> [BitBoard; 64] {
    let mut moves = [BitBoard::default(); 64];

    for i in 0..64 {
        moves[i] = get_move_mask(Coord::from_offset(i), directions);
    }

    return moves;
}

fn get_move_mask(coord: Coord, directions: [(isize, isize); 4]) -> BitBoard {
    let mut mask = BitBoard::new(0);

    for direction in directions {
        let mut coord = coord.clone();

        while coord.mv_mut(direction.0, direction.1) {
            mask.set(coord);
        }
    }

    return mask;
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn rook_moves() {
        let from = Coord::new('d', 5);

        let mut blockers = BitBoard::new(0);
        blockers.set(Coord::new('d', 2));
        blockers.set(Coord::new('b', 5));
        blockers.set(Coord::new('d', 8));

        let mut friendly_pieces = BitBoard::new(0);
        friendly_pieces.set(Coord::new('b', 5));

        let mut expected = BitBoard::new(0);
        expected.set(Coord::new('c', 5));
        expected.set(Coord::new('d', 2));
        expected.set(Coord::new('d', 3));
        expected.set(Coord::new('d', 4));
        expected.set(Coord::new('d', 6));
        expected.set(Coord::new('d', 7));
        expected.set(Coord::new('d', 8));
        expected.set(Coord::new('e', 5));
        expected.set(Coord::new('f', 5));
        expected.set(Coord::new('g', 5));
        expected.set(Coord::new('h', 5));

        let moves = get_rook_move_mask(from, &blockers, &friendly_pieces);

        println!("{moves}");
        println!("{expected}");

        assert_eq!(expected, moves);
    }

    #[test]
    fn bishop_moves() {
        let from = Coord::new('d', 5);

        let mut blockers = BitBoard::new(0);
        blockers.set(Coord::new('b', 3));
        blockers.set(Coord::new('b', 7));
        blockers.set(Coord::new('g', 8));

        let mut friendly_pieces = BitBoard::new(0);
        friendly_pieces.set(Coord::new('b', 7));

        let mut expected = BitBoard::new(0);
        expected.set(Coord::new('b', 3));
        expected.set(Coord::new('c', 4));
        expected.set(Coord::new('h', 1));
        expected.set(Coord::new('g', 2));
        expected.set(Coord::new('f', 3));
        expected.set(Coord::new('e', 4));
        expected.set(Coord::new('c', 6));
        expected.set(Coord::new('e', 6));
        expected.set(Coord::new('f', 7));
        expected.set(Coord::new('g', 8));

        let moves = get_bishop_move_mask(from, &blockers, &friendly_pieces);

        println!("{moves}");
        println!("{expected}");

        assert_eq!(expected, moves);
    }
}
