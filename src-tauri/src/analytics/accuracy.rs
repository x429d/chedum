use crate::domain::game::Color;

/// Точность одного хода по потере вероятности победы.
///
/// Формула Lichess (`AccuracyPercent.scala`) вместе с бонусом неопределённости +1:
/// `A = 103.1668100711649 · e^(−0.04354415386753951 · ΔW) − 3.166924740191411 + 1`.
pub fn move_accuracy(delta_w: f64) -> f64 {
    let raw = 103.166_810_071_164_9 * (-0.043_544_153_867_539_51 * delta_w.max(0.0)).exp()
        - 3.166_924_740_191_411
        + 1.0;
    raw.clamp(0.0, 100.0)
}

/// Размер скользящего окна волатильности: `(позиций − 1) / 10`, зажатый в 2..8.
fn window_size(moves: usize) -> usize {
    (moves / 10).clamp(2, 8)
}

/// Стандартное отклонение генеральной совокупности.
fn standard_deviation(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let len = values.len() as f64;
    let mean = values.iter().sum::<f64>() / len;
    (values.iter().map(|value| (value - mean).powi(2)).sum::<f64>() / len).sqrt()
}

/// Вес хода по волатильности Win% в окне вокруг него.
///
/// Первые `window − 2` ходов получают окно из начала партии — как в Lichess.
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

/// Точность партии по методу Lichess: среднее взвешенного по волатильности и
/// гармонического среднего точности ходов.
///
/// `moves` — пары `(Win% до хода, Win% после хода)` в перспективе ходящего, по порядку.
/// `first_color` — сторона, которая делает первый ход этой последовательности.
/// Возвращает `[белые, чёрные]`.
pub fn game_accuracy(moves: &[(f64, f64)], first_color: Color) -> [Option<f64>; 2] {
    if moves.is_empty() {
        return [None, None];
    }
    // Позиции в перспективе белых: перед первым ходом и после каждого хода.
    let first_white = matches!(first_color, Color::White);
    let mut win_probs = Vec::with_capacity(moves.len() + 1);
    win_probs.push(if first_white { moves[0].0 } else { 100.0 - moves[0].0 });
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
            // Плоская партия без волатильности: весов нет, остаётся гармоническое среднее.
            (None, Some(harmonic)) => Some(harmonic.clamp(0.0, 100.0)),
            (Some(weighted), Some(harmonic)) => Some(((weighted + harmonic) / 2.0).clamp(0.0, 100.0)),
            _ => None,
        }
    })
}

/// Точность по фазам: для каждой фазы — метод Lichess на её собственных ходах.
/// Возвращает `[белые, чёрные]`, внутри — opening / middlegame / endgame.
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

/// ACPL с кэпом 300 сантипешек на полуход.
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

/// Estimated Game Performance Elo.
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
mod tests {
    use super::*;

    #[test]
    fn perfect_move_is_one_hundred() {
        assert!((move_accuracy(0.0) - 100.0).abs() < 0.001);
    }

    #[test]
    fn accuracy_decreases_with_loss() {
        let small = move_accuracy(2.0);
        let medium = move_accuracy(10.0);
        let large = move_accuracy(30.0);
        assert!(small > medium);
        assert!(medium > large);
        assert!(large >= 0.0);
        assert!(move_accuracy(500.0) >= 0.0);
    }

    #[test]
    fn accuracy_matches_lichess_reference_points() {
        // 103.1668·e^(−0.04354·Δ) − 3.1669 + 1.
        assert!((move_accuracy(10.0) - 64.6).abs() < 0.2);
        assert!((move_accuracy(20.0) - 41.0).abs() < 0.2);
        assert!((move_accuracy(30.0) - 25.8).abs() < 0.2);
    }

    #[test]
    fn flat_game_is_one_hundred_for_both_colors() {
        let moves = [(50.0, 50.0); 5];
        let [white, black] = game_accuracy(&moves, Color::White);
        assert!((white.unwrap() - 100.0).abs() < 1e-9);
        assert!((black.unwrap() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn blunder_lowers_only_the_mover() {
        // Белые теряют 30 пунктов Win% на своём втором ходу.
        let moves = [(50.0, 50.0), (50.0, 50.0), (50.0, 20.0), (20.0, 20.0), (20.0, 20.0)];
        let [white, black] = game_accuracy(&moves, Color::White);
        assert!(white.unwrap() < 90.0);
        assert!((black.unwrap() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn black_first_color_flips_the_alignment() {
        let moves = [(50.0, 50.0), (50.0, 50.0), (50.0, 20.0), (20.0, 20.0), (20.0, 20.0)];
        let [white, black] = game_accuracy(&moves, Color::Black);
        assert!((white.unwrap() - 100.0).abs() < 1e-9);
        assert!(black.unwrap() < 90.0);
    }

    #[test]
    fn harmonic_mean_punishes_single_bad_move() {
        let moves = [(90.0, 90.0), (90.0, 90.0), (90.0, 20.0), (20.0, 90.0), (90.0, 90.0)];
        let [white, _] = game_accuracy(&moves, Color::White);
        let accuracy = white.unwrap();
        assert!(accuracy < 80.0);
        assert!(accuracy > 0.0);
    }

    #[test]
    fn empty_game_is_none() {
        assert_eq!(game_accuracy(&[], Color::White), [None, None]);
    }

    #[test]
    fn window_size_follows_lichess_rule() {
        assert_eq!(window_size(0), 2);
        assert_eq!(window_size(10), 2);
        assert_eq!(window_size(40), 4);
        assert_eq!(window_size(100), 8);
    }

    #[test]
    fn acpl_caps_single_move() {
        let value = average_cp_loss([0.0, 100.0, 1000.0]).unwrap();
        assert!((value - 133.333333).abs() < 0.001);
    }

    #[test]
    fn empty_accuracy_is_none() {
        assert_eq!(average_cp_loss(Vec::new()), None);
    }

    #[test]
    fn perf_elo_formula() {
        let value = performance_elo(Some(1500), Some(75.0), Some(35.0)).unwrap();
        assert_eq!(value, 1500);
        let high = performance_elo(Some(1500), Some(100.0), Some(0.0)).unwrap();
        assert!(high > 2000);
        assert_eq!(
            performance_elo(Some(3200), Some(100.0), Some(0.0)).unwrap(),
            3200
        );
        assert_eq!(performance_elo(None, None, None), None);
    }
}
