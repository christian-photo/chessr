use crate::moves::Move;

pub fn order_moves(moves: &mut [Move]) {
    moves.sort_by(|m1, m2| rate_move(m2).cmp(&rate_move(m1)));
}

fn rate_move(m: &Move) -> i32 {
    let mut score = 0;

    if let Some(capture) = m.capture {
        score += 50;

        if capture.get_value() > m.get_piece().get_value() {
            score += capture.get_value() - m.get_piece().get_value();
        }
    } else if m.promotion.is_some() {
        score += 100;
    }

    score
}
