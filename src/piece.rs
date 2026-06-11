#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum PieceType {
    #[default]
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    pub fn abbreviation(&self) -> char {
        match self {
            PieceType::Pawn => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
        }
    }

    pub fn from_abbreviation(abbr: char) -> Option<PieceType> {
        match abbr {
            'p' => Some(PieceType::Pawn),
            'n' => Some(PieceType::Knight),
            'b' => Some(PieceType::Bishop),
            'r' => Some(PieceType::Rook),
            'q' => Some(PieceType::Queen),
            'k' => Some(PieceType::King),
            _ => None,
        }
    }

    /// Gets the value of the piece in centipawns (100 centipawns = 1 pawn)
    pub fn get_value(&self) -> i16 {
        match self {
            PieceType::Pawn => 100,
            PieceType::Knight => 300,
            PieceType::Bishop => 300,
            PieceType::Rook => 500,
            PieceType::Queen => 900,
            PieceType::King => 2000,
        }
    }

    /// Get the piece type from its binary representation. None if it could not be assigned to any piece type
    pub fn from_repr(repr: u8) -> PieceType {
        let cleaned = repr & !(0b1 << 7); // Remove the black/white indication bit

        match cleaned {
            0 => PieceType::Pawn,
            1 => PieceType::Knight,
            2 => PieceType::Bishop,
            3 => PieceType::Rook,
            4 => PieceType::Queen,
            5 => PieceType::King,
            _ => panic!("Unrecognized binary representation"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Piece {
    repr: u8,
}

impl Piece {
    // Each piece is assigned a binary representation, where the position of the 1 indicates the type of piece
    // Black pieces have their first bit set to 1
    pub fn new(piece: PieceType, white: bool) -> Piece {
        Piece {
            repr: piece as u8 | ((!white as u8) << 7),
        }
    }

    pub fn repr(self: &Piece) -> u8 {
        self.repr
    }

    pub fn piece_type(self: &Piece) -> PieceType {
        PieceType::from_repr(self.repr())
    }

    pub fn notation(&self) -> char {
        let mut char = self.piece_type().abbreviation();
        if self.is_white() {
            char = char.to_ascii_uppercase();
        }
        return char;
    }

    pub fn is_white(self: &Piece) -> bool {
        (self.repr() & 0b1 << 7) == 0
    }

    pub fn to_bitboard_index(&self) -> usize {
        (self.repr & !(0b1 << 7)) as usize + (!self.is_white() as usize) * 6
    }
}

impl std::fmt::Debug for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Piece")
            .field("is_white", &self.is_white())
            .field("piece_type", &self.piece_type())
            .finish()
    }
}
