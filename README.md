# Chessr

Chessr is a chess engine written in rust. I started it as a personal side project to gain insights into the workings of chess engines
and to deepen my understanding of the rust programming language. Because this is intended as a portfolio project, this chess engine was - to great extent - coded without the use of AI code generation.

## How it works

Chess engines really need to things: They need to be fast and have a good evaluation function. These two things determine how strong a chess engine is.
This means that many operations are done low level to make sure you get optimal performance. A beautiful coincidence is, that a chess board has 64 squares and
many programming languages support 64-bit unsigned integers (ulong / u64). This means that we can work on a chess board using bitwise operations which are very fast.
This is why these chess board representations are call bitboards.
