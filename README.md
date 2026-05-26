# Chessr

Because this is a portfolio project, this chess engine was coded without the use of AI code generation

## Why rust?

## Performance Improvements

To generate the legal moves, you have to make sure, that the piece does not capture pieces of the same color. In my first approach I used the `pieces` array
in my `Board` struct to check if a piece is present and if it has the same color.
```rs
if let Some(target_piece) = board.pieces[target as usize] {
    if target_piece.is_white() == board.is_white_turn() {
        continue;
    } else {
        found_move.capture = Some(target_piece.piece_type());
    }
}
```

But by moving the capture assignment to the execution of the move and then using the black and white bitboards instead, I was able to reduce the the runtime of the function by 92%!
```rs
let friendly = if board.is_white_turn() {
    board.white.get_u64()
} else {
    board.black.get_u64()
};

if (friendly & (0b1 << target)) != 0 {
    continue;
}

found_move.capture = board.pieces[target as usize].map(|p| p.piece_type());
```

## Project Timeline

- 18.5.2026: Created project and setup board structure
- 22.5.2026: Finished initial pass on bitboard, pieces and board representation
- 25.5.2026: Implemented knight moves (9.3ns)
- 26.5.2026: Implemented first pass on pawn moves (108.4ns)

## How it works

### Positions

Internally, the chess engine uses the `u8` data type to encode positions, however only the first 6 bits are used, because they are sufficient to store encode all 64 squares.

### Bitboard

Before we can work with the bitboard `u64` integer and the `u8` position value, we have to transform the position from encoding a number to encoding a specific square on the chess board.
We do this by shifting a 1, exactly the position's value times to the left. This means, that if our position is 5 we shift the 1 5 times left: `0b1 << 5 = 0b100000`
