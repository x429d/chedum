use crate::domain::analysis::{MoveCategory, TimeFlag};
use crate::domain::game::TimeClass;

#[derive(Debug, Clone, Copy)]
pub struct TimeContext {
    pub remaining_sec: Option<f64>,
    pub spent_sec: Option<f64>,
    pub initial_sec: f64,
    pub time_class: Option<TimeClass>,
    pub category: MoveCategory,
}

/// Флаги тайм-менеджмента. Приоритет: impulsive_error > time_trouble > overthink.
pub fn time_flag(ctx: TimeContext) -> TimeFlag {
    let remaining = ctx.remaining_sec.unwrap_or(f64::INFINITY);
    let spent = ctx.spent_sec.unwrap_or(0.0);
    let impulsive = ctx.category.is_mistake_or_blunder()
        && spent < 2.5
        && ctx.remaining_sec.is_some_and(|value| value > 30.0);
    if impulsive {
        return TimeFlag::ImpulsiveError;
    }
    let threshold = match ctx.time_class {
        Some(TimeClass::Blitz) | Some(TimeClass::Bullet) => 10.0,
        Some(TimeClass::Rapid) | Some(TimeClass::Classical) => 30.0,
        None => 30.0,
    };
    if remaining < threshold {
        return TimeFlag::TimeTrouble;
    }
    if ctx.initial_sec > 0.0 && spent > ctx.initial_sec * 0.2 {
        return TimeFlag::Overthink;
    }
    TimeFlag::Normal
}

/// Время, потраченное на ход, с учётом инкремента.
pub fn spent_time(
    previous_clock: Option<f64>,
    current_clock: Option<f64>,
    increment_sec: f64,
) -> Option<f64> {
    let previous = previous_clock?;
    let current = current_clock?;
    Some((previous - current + increment_sec).max(0.0))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn ctx(remaining: f64, spent: f64, class: TimeClass, category: MoveCategory) -> TimeContext {
        TimeContext {
            remaining_sec: Some(remaining),
            spent_sec: Some(spent),
            initial_sec: 300.0,
            time_class: Some(class),
            category,
        }
    }

    #[test]
    fn impulsive_error_needs_speed_and_clock() {
        let flag = time_flag(ctx(120.0, 1.2, TimeClass::Blitz, MoveCategory::Blunder));
        assert_eq!(flag, TimeFlag::ImpulsiveError);
        let safe = time_flag(ctx(120.0, 8.0, TimeClass::Blitz, MoveCategory::Blunder));
        assert_eq!(safe, TimeFlag::Normal);
    }

    #[test]
    fn time_trouble_thresholds() {
        assert_eq!(
            time_flag(ctx(9.0, 4.0, TimeClass::Blitz, MoveCategory::Best)),
            TimeFlag::TimeTrouble
        );
        assert_eq!(
            time_flag(ctx(29.0, 4.0, TimeClass::Rapid, MoveCategory::Best)),
            TimeFlag::TimeTrouble
        );
        assert_eq!(
            time_flag(ctx(31.0, 4.0, TimeClass::Rapid, MoveCategory::Best)),
            TimeFlag::Normal
        );
    }

    #[test]
    fn overthink_uses_initial_time() {
        let flag = time_flag(ctx(200.0, 70.0, TimeClass::Blitz, MoveCategory::Best));
        assert_eq!(flag, TimeFlag::Overthink);
    }

    #[test]
    fn spent_time_handles_increment() {
        assert_eq!(spent_time(Some(100.0), Some(90.0), 2.0), Some(12.0));
        assert_eq!(spent_time(Some(100.0), Some(90.0), 0.0), Some(10.0));
        assert_eq!(spent_time(None, Some(90.0), 0.0), None);
    }
}
