use crate::{moves::Move, search::search::MATE_THRESHOLD};

pub const TRANSPOSITION_TABLE_SIZE_MB: usize = 64;
pub const NUM_ENTRIES: usize =
    TRANSPOSITION_TABLE_SIZE_MB * 1024 * 1024 / size_of::<TranspositionEntry>();

pub struct TranspositionTable {
    table: Box<[Option<TranspositionEntry>]>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            table: vec![None; NUM_ENTRIES].into_boxed_slice(),
        }
    }

    pub fn get(&self, hash: u64) -> Option<TranspositionEntry> {
        self.table[hash as usize % NUM_ENTRIES]
    }

    pub fn lookup(
        &self,
        hash: u64,
        depth: u8,
        ply: u8,
        alpha: i32,
        beta: i32,
    ) -> Option<TranspositionEntry> {
        if let Some(mut entry) = self.table[hash as usize % NUM_ENTRIES]
            && entry.key == hash
            && entry.depth >= depth
        {
            entry.eval = TranspositionTable::correct_mate_score_for_read(entry.eval, ply);

            if entry.node_type == TranspositionNodeType::Exact {
                return Some(entry);
            }

            if entry.node_type == TranspositionNodeType::LowerBound && entry.eval >= beta {
                return Some(entry);
            }

            if entry.node_type == TranspositionNodeType::UpperBound && entry.eval <= alpha {
                return Some(entry);
            }
        }

        return None;
    }

    pub fn store(
        &mut self,
        hash: u64,
        eval: i32,
        depth: u8,
        ply: u8,
        node_type: TranspositionNodeType,
        best_move: Move,
    ) {
        let entry = TranspositionEntry {
            key: hash,
            eval: TranspositionTable::correct_mate_score_for_storage(eval, ply),
            depth,
            node_type,
            best_move,
        };

        self.table[hash as usize % NUM_ENTRIES] = Some(entry);
    }

    fn correct_mate_score_for_storage(eval: i32, ply: u8) -> i32 {
        if eval <= MATE_THRESHOLD {
            return eval - ply as i32;
        }
        if eval >= -MATE_THRESHOLD {
            return eval + ply as i32;
        }
        eval
    }

    fn correct_mate_score_for_read(eval: i32, ply: u8) -> i32 {
        if eval <= MATE_THRESHOLD {
            return eval + ply as i32;
        }
        if eval >= -MATE_THRESHOLD {
            return eval - ply as i32;
        }
        eval
    }
}

#[derive(Copy, Clone)]
pub struct TranspositionEntry {
    pub key: u64,
    pub eval: i32,
    pub depth: u8,
    pub node_type: TranspositionNodeType,
    pub best_move: Move,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum TranspositionNodeType {
    Exact,
    LowerBound,
    UpperBound,
}
