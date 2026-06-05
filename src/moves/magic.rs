#[derive(Debug, Clone, Copy)]
pub struct MagicEntry {
    pub mask: u64,
    pub magic: u64,
    pub shift: u32,
    pub offset: usize,
}

impl MagicEntry {
    #[inline(always)]
    pub fn hash_masked(&self, board: u64) -> u64 {
        (board * self.magic) >> self.shift
    }

    #[inline(always)]
    pub fn hash_unmasked(&self, board: u64) -> u64 {
        ((board & self.mask) * self.magic) >> self.shift
    }
}

// pub fn find_magic_number(pos: u8, rook: bool, random: &mut Random) -> MagicEntry {
//     let mask = if rook {
//         ROOK_MOVE_MASK[pos as usize]
//     } else {
//         BISHOP_MOVE_MASK[pos as usize]
//     };
//     let offset = 1 << mask.count_ones();

//     MagicEntry {
//         mask,
//         magic: 0,
//         shift: 0,
//         offset,
//     }
// }

struct Random(u64);

impl Random {
    pub fn new(seed: u64) -> Self {
        Self { 0: seed }
    }

    pub fn random(&mut self) -> u64 {
        let mut num = self.0;

        num ^= num << 13;
        num ^= num >> 7;
        num ^= num << 17;

        self.0 = num;

        return num;
    }
}

pub const ROOK_MAGICS: [MagicEntry; 64] = [MagicEntry {
    mask: 0,
    magic: 0,
    shift: 0,
    offset: 0,
}; 64];
pub const BISHOP_MAGICS: [MagicEntry; 64] = [MagicEntry {
    mask: 0,
    magic: 0,
    shift: 0,
    offset: 0,
}; 64];
