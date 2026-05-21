// This file includes functions for pregenerating certain datastructures, like knight attack maps

use crate::board::Bitboard;

struct BoardCoordinate {
    x: i8,
    y: i8,
}

impl BoardCoordinate {
    pub(self) fn to_pos(&self) -> u8 {
        self.x as u8 + 8 * self.y as u8
    }

    pub(self) fn next_square(&self) -> Option<BoardCoordinate> {
        let mut x = self.x;
        let mut y = self.y;

        if x == 7 && y == 7 {
            return None;
        }
        if x == 7 {
            x = 0;
            y += 1;
        } else {
            x += 1;
        }
        return Some(BoardCoordinate { x, y });
    }

    pub(self) fn offset(&self, x: i8, y: i8) -> Option<BoardCoordinate> {
        let new_x = self.x + x;
        let new_y = self.y + y;

        if new_x < 0 || new_y < 0 || new_x > 7 || new_y > 7 {
            None
        } else {
            Some(BoardCoordinate { x: new_x, y: new_y })
        }
    }
}

pub fn generate_knight_attack_map() -> [u64; 64] {
    let mut map = [0u64; 64];
    let mut coord = BoardCoordinate { x: -1, y: 0 };

    while coord.next_square().is_some() {
        coord = coord.next_square().unwrap();
        let mut board = Bitboard::empty();

        let targets = vec![
            coord.offset(2, 1),
            coord.offset(1, 2),
            coord.offset(-1, 2),
            coord.offset(-2, 1),
            coord.offset(-2, -1),
            coord.offset(-1, -2),
            coord.offset(1, -2),
            coord.offset(2, 1),
        ];

        for c in targets {
            if c.is_some() {
                board.add_piece(c.unwrap().to_pos());
            }
        }

        map[coord.to_pos() as usize] = board.get_u64();
    }

    return map;
}
