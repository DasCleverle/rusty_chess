use crate::{
    bitboard::BitBoard,
    moves::lookup::{BISHOP_MAGICS, ROOK_MAGICS},
    Coord,
};

#[inline(always)]
pub fn get_rook_move_mask(from: Coord, blockers: &BitBoard, friendly_pieces: &BitBoard) -> BitBoard {
    let offset = from.offset();
    let (magic, shift) = ROOK_MAGICS[offset];
    let magic_index = get_magic_index(&ROOK_MOVES[offset], shift, magic, blockers);

    return BLOCKED_ROOK_MOVES[offset][magic_index] & !friendly_pieces;
}

#[inline(always)]
pub fn get_bishop_move_mask(from: Coord, blockers: &BitBoard, friendly_pieces: &BitBoard) -> BitBoard {
    let offset = from.offset();
    let (magic, shift) = BISHOP_MAGICS[offset];
    let magic_index = get_magic_index(&BISHOP_MOVES[offset], shift, magic, blockers);

    return BLOCKED_BISHOP_MOVES[offset][magic_index] & !friendly_pieces;
}

fn get_magic_index(mask: &BitBoard, shift: u8, magic: u64, blockers: &BitBoard) -> usize {
    let blockers = blockers & mask;
    let hash = blockers.0.wrapping_mul(magic);
    let index = (hash >> shift) as usize;

    return index;
}

lazy_static! {
    static ref ROOK_MOVES: [BitBoard; 64] = get_all_move_masks(ROOK_DIRECTIONS);
    static ref BISHOP_MOVES: [BitBoard; 64] = get_all_move_masks(BISHOP_DIRECTIONS);
    static ref BLOCKED_ROOK_MOVES: [Vec<BitBoard>; 64] = get_blocked_moves(ROOK_DIRECTIONS, &ROOK_MAGICS);
    static ref BLOCKED_BISHOP_MOVES: [Vec<BitBoard>; 64] = get_blocked_moves(BISHOP_DIRECTIONS, &BISHOP_MAGICS);
}

const ROOK_DIRECTIONS: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const BISHOP_DIRECTIONS: [(isize, isize); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

fn get_blocked_moves(directions: [(isize, isize); 4], magics: &[(u64, u8); 64]) -> [Vec<BitBoard>; 64] {
    let mut all_blocked_moves: [Vec<BitBoard>; 64] = std::array::from_fn(|_| {
        let mut v = Vec::with_capacity(1 << 12);
        v.resize(1 << 12, BitBoard::new(0));
        return v;
    });

    for i in 0..64 {
        let moves = &mut all_blocked_moves[i];

        let coord = Coord::from_offset(i);
        let one_off_mask = get_one_off_move_mask(coord, directions);
        let move_mask = get_move_mask(coord, directions);
        let (magic, bits) = magics[i];

        for blockers in subsets(&move_mask) {
            let magic_index = get_magic_index(&one_off_mask, bits, magic, &blockers);
            let blocked_moves = get_blocked_move_mask(coord, &move_mask, &blockers, &directions);

            moves[magic_index] = blocked_moves;
        }
    }

    return all_blocked_moves;
}

fn get_blocked_move_mask(from: Coord, moves: &BitBoard, blockers: &BitBoard, directions: &[(isize, isize); 4]) -> BitBoard {
    let mut blocked_moves = *moves;

    for direction in directions {
        let mut coord = from.clone();
        let mut found_blocker = false;

        while coord.mv_mut(direction.0, direction.1) {
            if found_blocker {
                blocked_moves.unset(coord);
            }

            if blockers.is_set(coord) {
                found_blocker = true;
            }
        }
    }

    return blocked_moves;
}

fn subsets(pattern: &BitBoard) -> Vec<BitBoard> {
    let mut subsets: Vec<BitBoard> = Vec::new();
    let mut subset = 0;

    loop {
        if subset != 0 {
            subsets.push(BitBoard::new(subset));
        }

        subset = subset.wrapping_sub(pattern.0) & pattern.0;

        if subset == 0 {
            break;
        }
    }

    return subsets;
}

fn get_all_move_masks(directions: [(isize, isize); 4]) -> [BitBoard; 64] {
    let mut moves = [BitBoard::default(); 64];

    for i in 0..64 {
        moves[i] = get_one_off_move_mask(Coord::from_offset(i), directions);
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

fn get_one_off_move_mask(coord: Coord, directions: [(isize, isize); 4]) -> BitBoard {
    let mut mask = BitBoard::new(0);

    for direction in directions {
        let mut coord = coord.clone();

        while coord.mv_mut(direction.0, direction.1) && coord.mv(direction.0, direction.1).is_some() {
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
        for i in 0..64 {
            let coord = Coord::from_offset(i);
            let moves = get_move_mask(coord, ROOK_DIRECTIONS);

            for blockers in subsets(&moves) {
                let magic_move_mask = get_rook_move_mask(coord, &blockers, &BitBoard::new(0));
                let expected = get_blocked_move_mask(coord, &moves, &blockers, &ROOK_DIRECTIONS);

                if magic_move_mask != expected {
                    println!("coord: {coord}");
                    println!("blockers");
                    println!("{blockers}");

                    println!("expected");
                    println!("{expected}");

                    println!("magic_move_mask");
                    println!("{magic_move_mask}");
                }

                assert_eq!(expected, magic_move_mask);
            }
        }
    }

    #[test]
    fn bishop_moves() {
        for i in 0..64 {
            let coord = Coord::from_offset(i);
            let moves = get_move_mask(coord, BISHOP_DIRECTIONS);

            for blockers in subsets(&moves) {
                let magic_move_mask = get_bishop_move_mask(coord, &blockers, &BitBoard::new(0));
                let expected = get_blocked_move_mask(coord, &moves, &blockers, &BISHOP_DIRECTIONS);

                if magic_move_mask != expected {
                    println!("coord: {coord}");
                    println!("blockers");
                    println!("{blockers}");

                    println!("expected");
                    println!("{expected}");

                    println!("magic_move_mask");
                    println!("{magic_move_mask}");
                }

                assert_eq!(expected, magic_move_mask);
            }
        }
    }
}
