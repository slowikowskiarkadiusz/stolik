use crate::scenes::astro_duel::ship::Ship;

const WIN_SCORE: u8 = 5;
pub const GAME_OVER_DELAY: f32 = 4.0;
pub const DEATH_DELAY: f32 = 2.5;

pub fn check_winner(score: &[u8; 2]) -> Option<u8> {
    for i in 0..2usize {
        let j = 1 - i;
        if score[i] >= WIN_SCORE && score[i] >= score[j] + 2 {
            return Some((i + 1) as u8);
        }
    }
    None
}

pub fn apply_hit(
    ship: &mut Option<Ship>,
    score: &mut [u8; 2],
    winner: &mut Option<u8>,
    game_over_timer: &mut f32,
    death_timer: &mut Option<(f32, usize)>,
    scorer: usize,
) {
    if let Some(s) = ship {
        let (died, _shield) = s.take_hit();
        if died && scorer < 2 {
            score[scorer] += 1;
            if let Some(w) = check_winner(score) {
                *winner = Some(w);
                *game_over_timer = GAME_OVER_DELAY;
            } else if death_timer.is_none() {
                *death_timer = Some((DEATH_DELAY, scorer));
            }
        }
    }
}

pub fn apply_damage(
    ship: &mut Option<Ship>,
    amount: u8,
    score: &mut [u8; 2],
    winner: &mut Option<u8>,
    game_over_timer: &mut f32,
    death_timer: &mut Option<(f32, usize)>,
    scorer: usize,
) {
    if let Some(s) = ship {
        let died = s.take_damage(amount);
        if died && scorer < 2 {
            score[scorer] += 1;
            if let Some(w) = check_winner(score) {
                *winner = Some(w);
                *game_over_timer = GAME_OVER_DELAY;
            } else if death_timer.is_none() {
                *death_timer = Some((DEATH_DELAY, scorer));
            }
        }
    }
}
