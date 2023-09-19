use chess::{BitBoard, Coord};

fn random_magic() -> u64 {
    rand::random::<u64>() & rand::random::<u64>() & rand::random::<u64>()
}

// fn spread_index_over_mask(index: usize, mask: &BitBoard) -> BitBoard {
//     let mut result = BitBoard::new(0);
//
//     for (i, coord) in mask.into_iter().enumerate() {
//         if index & (1 << i) != 0 {
//             result.set(coord);
//         }
//     }
//
//     return result;
// }
//
// fn get_blocker_patterns(mask: BitBoard) -> impl Iterator<Item = BitBoard> {
//     let ones = mask.count_ones();
//
//     return (0..(1 << ones)).map(move |i| spread_index_over_mask(i, &mask));
// }

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

fn magic_index(mask: &BitBoard, shift: u8, magic: u64, blockers: &BitBoard) -> usize {
    let blockers = blockers & mask;
    let hash = blockers.0.wrapping_mul(magic);
    let index = (hash >> (64 - shift)) as usize;

    return index;
}

fn find_magic(coord: Coord, directions: [(isize, isize); 4]) -> (u64, u8) {
    let mask = get_one_off_mask(coord, directions);
    let moves = get_move_mask(coord, directions);
    let shift = mask.count_ones() as u8;

    loop {
        let magic = random_magic();

        if try_make_table(coord, &moves, &mask, shift, magic, &directions) {
            return (magic, shift);
        }
    }
}

fn try_make_table(coord: Coord, moves: &BitBoard, mask: &BitBoard, shift: u8, magic: u64, directions: &[(isize, isize); 4]) -> bool {
    let mut table: [Option<BitBoard>; 1 << 12] = [None; 1 << 12];

    for blockers in subsets(moves) {
        let blocked_moves = get_blocked_move_mask(coord, &blockers, &directions);
        let index = magic_index(&mask, shift, magic, &blockers);

        match table[index] {
            None => {
                table[index] = Some(blocked_moves)
            },
            Some(m) if m != blocked_moves => {
                return false
            },
            Some(_) => { }
        };
    }

    return true;
}

fn main() {
    println!("rooks:");

    for i in 0..64 {
        let coord = Coord::from_offset(i);
        let (magic, shift) = find_magic(coord, ROOK_DIRECTIONS);

        println!("({}, {}),", magic, 64 - shift);
    }

    println!("bishops:");

    for i in 0..64 {
        let coord = Coord::from_offset(i);
        let (magic, shift) = find_magic(coord, BISHOP_DIRECTIONS);

        println!("({}, {}),", magic, 64 - shift);
    }
}

const ROOK_DIRECTIONS: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const BISHOP_DIRECTIONS: [(isize, isize); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

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

fn get_one_off_mask(coord: Coord, directions: [(isize, isize); 4]) -> BitBoard {
    let mut mask = BitBoard::new(0);

    for direction in directions {
        let mut coord = coord.clone();

        while coord.mv_mut(direction.0, direction.1) && coord.mv(direction.0, direction.1).is_some() {
            mask.set(coord);
        }
    }

    return mask;
}

fn get_blocked_move_mask(from: Coord, blockers: &BitBoard, directions: &[(isize, isize); 4]) -> BitBoard {
    let mut moves = BitBoard::new(0);

    for direction in directions {
        let mut coord = from.clone();

        while coord.mv_mut(direction.0, direction.1) {
            moves.set(coord);

            if blockers.is_set(coord) {
                break;
            }
        }
    }

    return moves;
}
