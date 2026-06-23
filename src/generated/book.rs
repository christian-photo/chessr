use std::{fs::File, io::Read};

use vampirc_uci::{UciMove, UciSquare};

pub struct OpeningBook {
    entries: Vec<BookEntry>,
}

impl OpeningBook {
    pub fn read_from_file(file: &str) -> Result<Self, std::io::Error> {
        let mut file = File::open(file)?;
        let mut buffer = Vec::new();
        let size = file.read_to_end(&mut buffer)?;

        let entry_size = 8 + 2 + 2 + 4;

        if size % entry_size != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid Polyglot book size",
            ));
        }

        let mut entries = Vec::with_capacity(size / entry_size);

        for chunk in buffer.chunks_exact(entry_size) {
            let entry = BookEntry {
                hash: u64::from_be_bytes(chunk[0..8].try_into().unwrap()),
                book_move: u16::from_be_bytes(chunk[8..10].try_into().unwrap()),
                weight: u16::from_be_bytes(chunk[10..12].try_into().unwrap()),
                learn: u32::from_be_bytes(chunk[12..16].try_into().unwrap()),
            };

            entries.push(entry);
        }

        Ok(Self { entries })
    }

    pub fn find_entries(&self, hash: u64) -> Vec<BookEntry> {
        let index = match self.entries.binary_search_by_key(&hash, |e| e.hash) {
            Ok(index) => Some(index),
            Err(_) => self.entries.iter().position(|e| e.hash == hash), // Fall back to normal search if book is not sorted
        };

        if let Some(index) = index {
            let mut lower_bound = index;
            let mut upper_bound = index + 1;

            while lower_bound > 0 && self.entries[lower_bound - 1].hash == hash {
                lower_bound -= 1;
            }

            while upper_bound < self.entries.len() && self.entries[upper_bound].hash == hash {
                upper_bound += 1;
            }

            let mut entries = Vec::with_capacity(upper_bound - lower_bound);
            entries.extend_from_slice(&self.entries[lower_bound..upper_bound]);

            entries
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BookEntry {
    pub hash: u64,
    pub book_move: u16,
    pub weight: u16,
    pub learn: u32,
}

impl BookEntry {
    pub fn to_uci(&self) -> UciMove {
        let mut to_file = (self.book_move & 0b111) as u8;
        let to_rank = ((self.book_move >> 3) & 0b111) as u8;
        let from_file = ((self.book_move >> 6) & 0b111) as u8;
        let from_rank = ((self.book_move >> 9) & 0b111) as u8;
        let promo = ((self.book_move >> 12) & 0b111) as u8;

        let from = from_rank * 8 + from_file;
        let to = to_rank * 8 + to_file;

        // Fix castling notation
        if from == 4 && to == 7 || from == 60 && to == 63 {
            to_file = 6;
        } else if from == 4 && to == 0 || from == 60 && to == 56 {
            to_file = 1;
        }

        let square_from = UciSquare::from((b'a' + from_file) as char, from_rank + 1);
        let square_to = UciSquare::from((b'a' + to_file) as char, to_rank + 1);

        let mut m = UciMove::from_to(square_from, square_to);

        m.promotion = match promo {
            1 => Some(vampirc_uci::UciPiece::Knight),
            2 => Some(vampirc_uci::UciPiece::Bishop),
            3 => Some(vampirc_uci::UciPiece::Rook),
            4 => Some(vampirc_uci::UciPiece::Queen),
            _ => None,
        };

        m
    }
}
