pub fn pop_lsb(board: &mut u64) -> u8 {
    let lsb = board.trailing_zeros() as u8;
    *board &= *board - 1;
    lsb
}
