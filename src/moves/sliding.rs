#[cfg(not(feature = "pext"))]
use crate::moves::magic::Random;
use crate::{
    bit_ops::pop_lsb,
    pregen::{BISHOP_OCCUPANCY_MASK, ROOK_OCCUPANCY_MASK},
};

pub fn precompute_attacks() -> SlidingAttackLookup {
    let mut rook_offset = 0usize;
    let mut bishop_offset = 0usize;
    let mut lookup = SlidingAttackLookup::empty();

    #[cfg(not(feature = "pext"))]
    let mut random = Random::new(230482);

    for square in 0..64u8 {
        lookup.rook_offsets[square as usize] = rook_offset;
        lookup.bishop_offsets[square as usize] = bishop_offset;

        // Rook attacks --------
        let rook_occupancy_mask = ROOK_OCCUPANCY_MASK[square as usize];

        #[cfg(feature = "pext")]
        {
            let rook_blockers = generate_blockers(rook_occupancy_mask);
            for blocker_config in rook_blockers {
                let attack =
                    generate_attacks(blocker_config, rook_occupancy_mask, square, [8, 1, -8, -1]);

                use bitintr::Pext;

                let index = blocker_config.pext(rook_occupancy_mask) as usize + rook_offset;
                lookup.rook_attacks[index] = attack;
            }
        }

        #[cfg(not(feature = "pext"))]
        for _ in 0..100_000 {
            let mut shift = 50u8;
            'outer: while shift > 30 {
                let rook_blockers = generate_blockers(rook_occupancy_mask);
                let magic_candidate = random.random();
                let mut hashes = vec![0u64; rook_blockers.len()];
                let mut attacks = vec![0u64; rook_blockers.len()];
                for blocker_config in rook_blockers {
                    let attack = generate_attacks(
                        blocker_config,
                        rook_occupancy_mask,
                        square,
                        [8, 1, -8, -1],
                    );

                    let hash = (blocker_config * magic_candidate) >> shift;

                    if hashes.contains(&hash) && !attacks.contains(&attack) {
                        shift -= 1;
                        continue 'outer; // Hash collision
                    }
                    if hashes.contains(&hash) && attacks.contains(&attack) {
                        continue; // A desired collision
                    }
                    hashes.push(hash);
                    attacks.push(attack);
                }

                rook_offset += hashes.len();
            }
        }

        rook_offset += 1 << rook_occupancy_mask.count_ones();

        // Bishop attacks --------
        let bishop_occupancy_mask = BISHOP_OCCUPANCY_MASK[square as usize];
        let bishop_blockers = generate_blockers(bishop_occupancy_mask);

        for blocker_config in bishop_blockers {
            let attack = generate_attacks(
                blocker_config,
                bishop_occupancy_mask,
                square,
                [7, 9, -7, -9],
            );

            #[cfg(feature = "pext")]
            {
                use bitintr::Pext;

                let index = blocker_config.pext(bishop_occupancy_mask) as usize + bishop_offset;
                lookup.bishop_attacks[index] = attack;
            }

            #[cfg(not(feature = "pext"))]
            {}
        }

        bishop_offset += 1 << bishop_occupancy_mask.count_ones();
    }

    lookup
}

pub fn rook_array_length() -> usize {
    let mut length = 0usize;
    for mask in ROOK_OCCUPANCY_MASK {
        length += generate_blockers(mask).len();
    }

    length
}

pub fn bishop_array_length() -> usize {
    let mut length = 0usize;
    for mask in BISHOP_OCCUPANCY_MASK {
        length += generate_blockers(mask).len();
    }

    length
}

/// Generates all possible sliding attack / blocker combinations for a given mask
fn generate_blockers(mask: u64) -> Vec<u64> {
    let possible_blockers = mask.count_ones();
    let mut blocker_squares = Vec::with_capacity(possible_blockers as usize);

    let mut blocker_mask = mask;
    while blocker_mask != 0 {
        let pos = pop_lsb(&mut blocker_mask);
        blocker_squares.push(pos);
    }

    let total_combinations = 1u32 << possible_blockers;
    let mut blockers = Vec::with_capacity(total_combinations as usize);

    // We know that there are in total n (total_combinations) possible combinations of blockers for the given mask
    // This means that we can just use the binary representation of any number between 0 and total_combinations to give us a unique
    // arrangement of blockers on the mask which we need to map to the correct squares by using the blocker_squares vec
    for i in 0..total_combinations {
        let mut blocker = 0u64;
        let mut pattern = i as u64;
        while pattern != 0 {
            let bit = pop_lsb(&mut pattern);
            blocker |= 1u64 << blocker_squares[bit as usize];
        }

        blockers.push(blocker);
    }

    blockers
}

fn generate_attacks(occupancy: u64, mask: u64, pos: u8, directions: [i8; 4]) -> u64 {
    let mut square;
    let mut attacks = 0u64;

    for direction in directions {
        square = pos;

        // Loop as long as the next square is in the general occupancy mask
        while square.wrapping_add_signed(direction) < 64
            && mask & (1 << square.wrapping_add_signed(direction)) != 0
        {
            // Break out of the loop if the square is occupied, we will still add that square to the attacks
            // because it can be captured if it is occupied by the opponent
            if occupancy & (1u64 << square.wrapping_add_signed(direction)) != 0 {
                break;
            }

            square = square.wrapping_add_signed(direction);
            attacks |= 1u64 << square;
        }
        // We add one more square because this will be the edge square, that was not included in the occupancy mask
        // Or if we broke out of the while loop, it is the square that can be captured if it is occupied by the opponent
        if square.wrapping_add_signed(direction) > 63
            || square % 8 == 0 && square.wrapping_add_signed(direction) % 8 == 7 // Avoid the situation where a rook could jump to the other side of the board
            || square % 8 == 7 && square.wrapping_add_signed(direction) % 8 == 0
        {
            continue;
        }
        square = square.wrapping_add_signed(direction);
        attacks |= 1u64 << square;
    }

    attacks
}

pub const TOTAL_ROOK_ATTACKS: usize = 102400;
pub const TOTAL_BISHOP_ATTACKS: usize = 5248;

pub struct SlidingAttackLookup {
    rook_attacks: Vec<u64>,
    bishop_attacks: Vec<u64>,

    rook_offsets: [usize; 64],
    bishop_offsets: [usize; 64],
}

impl SlidingAttackLookup {
    pub fn empty() -> Self {
        Self {
            #[cfg(feature = "pext")]
            bishop_attacks: vec![0u64; TOTAL_BISHOP_ATTACKS],
            #[cfg(feature = "pext")]
            rook_attacks: vec![0u64; TOTAL_ROOK_ATTACKS],

            #[cfg(not(feature = "pext"))]
            bishop_attacks: vec![0u64; 5248], // Numbers grabbed out of thin air, will need to see how many hash collisions there are
            #[cfg(not(feature = "pext"))]
            rook_attacks: vec![0u64; 102400],

            rook_offsets: [0usize; 64],
            bishop_offsets: [0usize; 64],
        }
    }

    pub fn get_rook_attacks(&self, pos: u8, board_occupancy: u64) -> u64 {
        let mask = ROOK_OCCUPANCY_MASK[pos as usize];
        let square_offset = self.rook_offsets[pos as usize];

        #[cfg(feature = "pext")]
        {
            use bitintr::Pext;

            let index = board_occupancy.pext(mask) as usize + square_offset;
            return self.rook_attacks[index];
        }

        #[cfg(not(feature = "pext"))]
        {}
    }

    pub fn get_bishop_attacks(&self, pos: u8, board_occupancy: u64) -> u64 {
        let mask = BISHOP_OCCUPANCY_MASK[pos as usize];
        let square_offset = self.bishop_offsets[pos as usize];

        #[cfg(feature = "pext")]
        {
            use bitintr::Pext;

            let index = board_occupancy.pext(mask) as usize + square_offset;
            return self.bishop_attacks[index];
        }

        #[cfg(not(feature = "pext"))]
        {}
    }

    pub fn get_queen_attacks(&self, pos: u8, board_occupancy: u64) -> u64 {
        self.get_bishop_attacks(pos, board_occupancy) | self.get_rook_attacks(pos, board_occupancy)
    }
}
