/// Точность одного хода по потере вероятности победы (CAPS).
pub fn move_accuracy(delta_w: f64) -> f64 {
    let value = 103.1668 * (-0.04354 * delta_w.max(0.0)).exp() - 3.1669;
    value.clamp(0.0, 100.0)
}

/// Средневзвешенная точность. Критические ходы имеют вес 1.5, остальные 1.0.
pub fn weighted_accuracy<I>(entries: I) -> Option<f64>
where
    I: IntoIterator<Item = (f64, f64)>,
{
    let mut sum = 0.0;
    let mut weight = 0.0;
    for (accuracy, item_weight) in entries {
        sum += accuracy * item_weight;
        weight += item_weight;
    }
    if weight <= f64::EPSILON {
        None
    } else {
        Some((sum / weight).clamp(0.0, 100.0))
    }
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

/// Вес хода для итоговой точности: критические позиции важнее.
pub fn critical_weight(is_critical: bool) -> f64 {
    if is_critical {
        1.5
    } else {
        1.0
    }
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
    fn weighted_accuracy_uses_weights() {
        let value = weighted_accuracy([(100.0, 1.0), (50.0, 1.5)]).unwrap();
        assert!((value - 70.0).abs() < 1e-9);
    }

    #[test]
    fn empty_accuracy_is_none() {
        assert_eq!(weighted_accuracy(Vec::new()), None);
        assert_eq!(average_cp_loss(Vec::new()), None);
    }

    #[test]
    fn acpl_caps_single_move() {
        let value = average_cp_loss([0.0, 100.0, 1000.0]).unwrap();
        assert!((value - 133.333333).abs() < 0.001);
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
