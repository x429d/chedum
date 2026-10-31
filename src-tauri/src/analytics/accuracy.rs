use crate::domain::game::Color;

/// Accuracy of a single move from the loss in win probability.
///
/// Lichess formula (`AccuracyPercent.scala`) plus the +1 uncertainty bonus:
/// `A = 103.1668100711649 · e^(−0.04354415386753951 · ΔW) − 3.166924740191411 + 1`.
pub fn move_accuracy(delta_w: f64) -> f64 {
    let raw = 103.166_810_071_164_9 * (-0.043_544_153_867_539_51 * delta_w.max(0.0)).exp()
        - 3.166_924_740_191_411
        + 1.0;
    raw.clamp(0.0, 100.0)
}

/// Sliding volatility window size: `(positions − 1) / 10`, clamped to 2..8.
fn window_size(moves: usize) -> usize {
    (moves / 10).clamp(2, 8)
}

/// Population standard deviation.
fn standard_deviation(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let len = values.len() as f64;
    let mean = values.iter().sum::<f64>() / len;
    (values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / len)
        .sqrt()
}

/// Weight of a move based on Win% volatility in the window around it.
///
/// The first `window − 2` moves get a window anchored at the start of the game —
/// same as Lichess.
fn volatility_weight(win_probs: &[f64], index: usize, window: usize) -> Option<f64> {
    let start = (index + 2).saturating_sub(window);
    let end = (start + window).min(win_probs.len());
    let deviation = standard_deviation(&win_probs[start..end]);
    if deviation <= 0.0 {
        None
    } else {
        Some(deviation.clamp(0.5, 12.0))
    }
}

fn weighted_mean<I>(entries: I) -> Option<f64>
where
    I: IntoIterator<Item = (f64, f64)>,
{
    let mut sum = 0.0;
    let mut weight = 0.0;
    for (value, item_weight) in entries {
        sum += value * item_weight;
        weight += item_weight;
    }
    if weight <= f64::EPSILON {
        None
    } else {
        Some(sum / weight)
    }
}

fn harmonic_mean<I>(values: I) -> Option<f64>
where
    I: IntoIterator<Item = f64>,
{
    let mut sum = 0.0;
    let mut count = 0u32;
    for value in values {
        if value <= 0.0 {
            return Some(0.0);
        }
        sum += 1.0 / value;
        count += 1;
    }
    if count == 0 {
        None
    } else {
        Some(f64::from(count) / sum)
    }
}

/// Game accuracy using the Lichess method: the mean of the volatility-weighted
/// average and the harmonic mean of move accuracies.
///
/// `moves` — `(Win% before the move, Win% after the move)` pairs from the mover's
/// perspective, in order.
/// `first_color` — the side that makes the first move of this sequence.
/// Returns `[white, black]`.
pub fn game_accuracy(moves: &[(f64, f64)], first_color: Color) -> [Option<f64>; 2] {
    if moves.is_empty() {
        return [None, None];
    }
    // Positions from White's perspective: before the first move and after each move.
    let first_white = matches!(first_color, Color::White);
    let mut win_probs = Vec::with_capacity(moves.len() + 1);
    win_probs.push(if first_white {
        moves[0].0
    } else {
        100.0 - moves[0].0
    });
    for (index, (_, after)) in moves.iter().enumerate() {
        let white_moves = (index % 2 == 0) == first_white;
        win_probs.push(if white_moves { *after } else { 100.0 - *after });
    }

    let window = window_size(moves.len());
    let mut samples: [Vec<(f64, Option<f64>)>; 2] = [Vec::new(), Vec::new()];
    for (index, (before, after)) in moves.iter().enumerate() {
        let white_moves = (index % 2 == 0) == first_white;
        let slot = if white_moves { 0 } else { 1 };
        let accuracy = if after >= before {
            100.0
        } else {
            move_accuracy(before - after)
        };
        samples[slot].push((accuracy, volatility_weight(&win_probs, index, window)));
    }

    samples.map(|entries| {
        if entries.is_empty() {
            return None;
        }
        let weighted = weighted_mean(
            entries
                .iter()
                .filter_map(|(accuracy, weight)| weight.map(|weight| (*accuracy, weight))),
        );
        let harmonic = harmonic_mean(entries.iter().map(|(accuracy, _)| *accuracy));
        match (weighted, harmonic) {
            // Flat game without volatility: no weights, so only the harmonic mean remains.
            (None, Some(harmonic)) => Some(harmonic.clamp(0.0, 100.0)),
            (Some(weighted), Some(harmonic)) => {
                Some(((weighted + harmonic) / 2.0).clamp(0.0, 100.0))
            }
            _ => None,
        }
    })
}

/// Accuracy by phase: the Lichess method applied to each phase's own moves.
/// Returns `[white, black]`, each containing opening / middlegame / endgame.
pub fn phase_accuracy(
    samples: &[Vec<(f64, f64)>; 3],
    first: &[Option<Color>; 3],
) -> [[Option<f64>; 3]; 2] {
    let mut result = [[None; 3]; 2];
    for slot in 0..3 {
        if let Some(color) = first[slot] {
            let [white, black] = game_accuracy(&samples[slot], color);
            result[0][slot] = white;
            result[1][slot] = black;
        }
    }
    result
}

/// ACPL with a 300-centipawn cap per ply.
pub fn average_cp_loss<I>(losses: I) -> Option<f64>
where
    I: IntoIterator<Item = f64>,
{
    let mut sum = 0.0;
    let mut count = 0u32;
    for loss in losses {
        sum += loss.clamp(0.0, 300.0);
        count += 1;
    }
    if count == 0 {
        None
    } else {
        Some(sum / f64::from(count))
    }
}

pub fn performance_elo(
    base_elo: Option<i32>,
    accuracy: Option<f64>,
    acpl: Option<f64>,
) -> Option<i32> {
    let accuracy = accuracy?;
    let acpl = acpl.unwrap_or(35.0);
    let base = f64::from(base_elo.unwrap_or(1500));
    let raw = base + (accuracy - 75.0) * 18.0 - (acpl - 35.0) * 4.5;
    let rounded = raw.round() as i32;
    Some(rounded.clamp(400, 3200))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "accuracy_tests.rs"]
mod tests;
