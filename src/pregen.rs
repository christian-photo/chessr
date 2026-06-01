// This file includes functions for pregenerating certain datastructures, like knight attack maps

use crate::board::Bitboard;

#[derive(Debug)]
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

    pub(self) fn prev_square(&self) -> Option<BoardCoordinate> {
        let mut x = self.x;
        let mut y = self.y;

        if x == 0 && y == 0 {
            return None;
        }
        if x == 0 {
            x = 7;
            y -= 1;
        } else {
            x -= 1;
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
            coord.offset(2, -1),
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

pub fn generate_white_pawn_attack_map() -> [u64; 64] {
    let mut map = [0u64; 64];
    let mut coord = BoardCoordinate { x: 7, y: 0 };

    while coord.next_square().is_some() {
        coord = coord.next_square().unwrap();
        let mut board = Bitboard::empty();
        let mut targets = vec![];

        if coord.x > 0 {
            targets.push(coord.offset(-1, 1));
        }
        if coord.x < 7 {
            targets.push(coord.offset(1, 1));
        }

        for c in targets {
            if c.is_some() {
                board.add_piece(c.unwrap().to_pos());
            }
        }

        map[coord.to_pos() as usize] = board.get_u64();
    }

    return map;
}

pub fn generate_black_pawn_attack_map() -> [u64; 64] {
    let mut map = [0u64; 64];
    let mut coord = BoardCoordinate { x: 0, y: 7 };

    while coord.prev_square().is_some() {
        coord = coord.prev_square().unwrap();
        let mut board = Bitboard::empty();
        let mut targets = vec![];

        if coord.x > 0 {
            targets.push(coord.offset(-1, -1));
        }
        if coord.x < 7 {
            targets.push(coord.offset(1, -1));
        }

        for c in targets {
            if c.is_some() {
                board.add_piece(c.unwrap().to_pos());
            }
        }

        map[coord.to_pos() as usize] = board.get_u64();
    }

    return map;
}

pub fn generate_king_map() -> [u64; 64] {
    let mut map = [0u64; 64];
    let mut coord = BoardCoordinate { x: -1, y: 0 };

    while coord.next_square().is_some() {
        coord = coord.next_square().unwrap();
        let mut board = Bitboard::empty();
        let targets = vec![
            coord.offset(-1, -1),
            coord.offset(0, -1),
            coord.offset(1, -1),
            coord.offset(-1, 0),
            // coord.offset(0, 0),
            coord.offset(1, 0),
            coord.offset(-1, 1),
            coord.offset(0, 1),
            coord.offset(1, 1),
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

pub const KING_MOVE_MAP: [u64; 64] = [
    770,
    1797,
    3594,
    7188,
    14376,
    28752,
    57504,
    49216,
    197123,
    460039,
    920078,
    1840156,
    3680312,
    7360624,
    14721248,
    12599488,
    50463488,
    117769984,
    235539968,
    471079936,
    942159872,
    1884319744,
    3768639488,
    3225468928,
    12918652928,
    30149115904,
    60298231808,
    120596463616,
    241192927232,
    482385854464,
    964771708928,
    825720045568,
    3307175149568,
    7718173671424,
    15436347342848,
    30872694685696,
    61745389371392,
    123490778742784,
    246981557485568,
    211384331665408,
    846636838289408,
    1975852459884544,
    3951704919769088,
    7903409839538176,
    15806819679076352,
    31613639358152704,
    63227278716305408,
    54114388906344448,
    216739030602088448,
    505818229730443264,
    1011636459460886528,
    2023272918921773056,
    4046545837843546112,
    8093091675687092224,
    16186183351374184448,
    13853283560024178688,
    144959613005987840,
    362258295026614272,
    724516590053228544,
    1449033180106457088,
    2898066360212914176,
    5796132720425828352,
    11592265440851656704,
    4665729213955833856,
];

pub const BLACK_PAWN_ATTACK: [u64; 64] = [
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    2,
    5,
    10,
    20,
    40,
    80,
    160,
    64,
    512,
    1280,
    2560,
    5120,
    10240,
    20480,
    40960,
    16384,
    131072,
    327680,
    655360,
    1310720,
    2621440,
    5242880,
    10485760,
    4194304,
    33554432,
    83886080,
    167772160,
    335544320,
    671088640,
    1342177280,
    2684354560,
    1073741824,
    8589934592,
    21474836480,
    42949672960,
    85899345920,
    171798691840,
    343597383680,
    687194767360,
    274877906944,
    2199023255552,
    5497558138880,
    10995116277760,
    21990232555520,
    43980465111040,
    87960930222080,
    175921860444160,
    70368744177664,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
];

pub const WHITE_PAWN_ATTACK: [u64; 64] = [
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    131072,
    327680,
    655360,
    1310720,
    2621440,
    5242880,
    10485760,
    4194304,
    33554432,
    83886080,
    167772160,
    335544320,
    671088640,
    1342177280,
    2684354560,
    1073741824,
    8589934592,
    21474836480,
    42949672960,
    85899345920,
    171798691840,
    343597383680,
    687194767360,
    274877906944,
    2199023255552,
    5497558138880,
    10995116277760,
    21990232555520,
    43980465111040,
    87960930222080,
    175921860444160,
    70368744177664,
    562949953421312,
    1407374883553280,
    2814749767106560,
    5629499534213120,
    11258999068426240,
    22517998136852480,
    45035996273704960,
    18014398509481984,
    144115188075855872,
    360287970189639680,
    720575940379279360,
    1441151880758558720,
    2882303761517117440,
    5764607523034234880,
    11529215046068469760,
    4611686018427387904,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
];

pub const KNIGHT_ATTACK: [u64; 64] = [
    132096,
    329728,
    659712,
    1319424,
    2638848,
    5277696,
    10489856,
    4202496,
    33816580,
    84410376,
    168886289,
    337772578,
    675545156,
    1351090312,
    2685403152,
    1075839008,
    8657044482,
    21609056261,
    43234889994,
    86469779988,
    172939559976,
    345879119952,
    687463207072,
    275414786112,
    2216203387392,
    5531918402816,
    11068131838464,
    22136263676928,
    44272527353856,
    88545054707712,
    175990581010432,
    70506185244672,
    567348067172352,
    1416171111120896,
    2833441750646784,
    5666883501293568,
    11333767002587136,
    22667534005174272,
    45053588738670592,
    18049583422636032,
    145241105196122112,
    362539804446949376,
    725361088165576704,
    1450722176331153408,
    2901444352662306816,
    5802888705324613632,
    11533718717099671552,
    4620693356194824192,
    288234782788157440,
    576469569871282176,
    1224997833292120064,
    2449995666584240128,
    4899991333168480256,
    9799982666336960512,
    1152939783987658752,
    2305878468463689728,
    1128098930098176,
    2257297371824128,
    4796069720358912,
    9592139440717824,
    19184278881435648,
    38368557762871296,
    4679521487814656,
    9077567998918656,
];
