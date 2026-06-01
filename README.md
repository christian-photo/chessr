# Chessr

Because this is a portfolio project, this chess engine was coded without the use of AI code generation

## Why rust?

## Project Timeline

- 18.5.2026: Created project and setup board structure
- 22.5.2026: Finished initial pass on bitboard, pieces and board representation
- 25.5.2026: Implemented knight moves
- 26.5.2026: Implemented first pass on pawn moves

## How it works

### Positions

Internally, the chess engine uses the `u8` data type to encode positions, however only the first 6 bits are used, because they are sufficient to store encode all 64 squares.

### Bitboard

Before we can work with the bitboard `u64` integer and the `u8` position value, we have to transform the position from encoding a number to encoding a specific square on the chess board.
We do this by shifting a 1, exactly the position's value times to the left. This means, that if our position is 5 we shift the 1 5 times left: `0b1 << 5 = 0b100000`
