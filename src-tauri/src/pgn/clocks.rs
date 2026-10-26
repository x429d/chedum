use crate::domain::game::TimeClass;

/// Извлекает секунды из комментария `[%clk H:MM:SS.d]` или `[%clk MM:SS]`.
pub fn parse_clock_comment(comment: &str) -> Option<f64> {
    let start = comment.find("[%clk")?;
    let rest = &comment[start + 5..];
    let rest = rest.trim_start();
    let end = rest.find(']')?;
    let value = rest[..end].trim();
    parse_clock_value(value)
}

fn parse_clock_value(value: &str) -> Option<f64> {
    let parts: Vec<&str> = value.split(':').collect();
    let seconds = match parts.as_slice() {
        [minutes, seconds] => parse_num(minutes)? * 60.0 + parse_num(seconds)?,
        [hours, minutes, seconds] => {
            parse_num(hours)? * 3600.0 + parse_num(minutes)? * 60.0 + parse_num(seconds)?
        }
        _ => return None,
    };
    if seconds.is_finite() && seconds >= 0.0 {
        Some(seconds)
    } else {
        None
    }
}

fn parse_num(value: &str) -> Option<f64> {
    value.trim().parse::<f64>().ok()
}

/// Разбирает `[%emt 0:00:12]` в секунды.
pub fn parse_elapsed_comment(comment: &str) -> Option<f64> {
    let start = comment.find("[%emt")?;
    let rest = &comment[start + 5..];
    let end = rest.find(']')?;
    parse_clock_value(rest[..end].trim())
}

/// Базовое время и инкремент из строки `600+5` или `40/7200:1800`.
pub fn parse_time_control(value: &str) -> Option<(u32, u32)> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let first = trimmed.split(':').next().unwrap_or(trimmed);
    let (base_part, increment_part) = first.split_once('+').unwrap_or((first, "0"));
    let base_part = base_part.rsplit('/').next().unwrap_or(base_part);
    let base = base_part.trim().parse::<u32>().ok()?;
    let increment = increment_part.trim().parse::<u32>().ok().unwrap_or(0);
    Some((base, increment))
}

/// Класс контроля времени по базовому времени и инкременту.
pub fn classify_time_class(base_secs: u32, increment_secs: u32) -> TimeClass {
    let estimated = u64::from(base_secs) + 40 * u64::from(increment_secs);
    if estimated < 180 {
        TimeClass::Bullet
    } else if estimated < 600 {
        TimeClass::Blitz
    } else if estimated < 1800 {
        TimeClass::Rapid
    } else {
        TimeClass::Classical
    }
}

pub fn time_class_from_control(value: Option<&str>) -> Option<TimeClass> {
    let (base, increment) = parse_time_control(value?)?;
    Some(classify_time_class(base, increment))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_clock() {
        assert_eq!(parse_clock_comment("{[%clk 0:03:00]}"), Some(180.0));
        assert_eq!(parse_clock_comment("[%clk 1:02:03.4]"), Some(3723.4));
        assert_eq!(parse_clock_comment("[%clk 0:00:09.5]"), Some(9.5));
    }

    #[test]
    fn parses_short_clock() {
        assert_eq!(parse_clock_comment("[%clk 03:07]"), Some(187.0));
    }

    #[test]
    fn rejects_malformed_clock() {
        assert_eq!(parse_clock_comment("no clock here"), None);
        assert_eq!(parse_clock_comment("[%clk abc]"), None);
        assert_eq!(parse_clock_comment("[%clk 1:xx:00]"), None);
    }

    #[test]
    fn parses_elapsed() {
        assert_eq!(parse_elapsed_comment("[%emt 0:00:12.5]"), Some(12.5));
    }

    #[test]
    fn parses_time_controls() {
        assert_eq!(parse_time_control("600+5"), Some((600, 5)));
        assert_eq!(parse_time_control("180"), Some((180, 0)));
        assert_eq!(parse_time_control("40/7200:1800"), Some((7200, 0)));
        assert_eq!(parse_time_control("garbage"), None);
    }

    #[test]
    fn classifies_time_controls() {
        assert_eq!(classify_time_class(60, 0), TimeClass::Bullet);
        assert_eq!(classify_time_class(300, 0), TimeClass::Blitz);
        assert_eq!(classify_time_class(600, 0), TimeClass::Rapid);
        assert_eq!(classify_time_class(1800, 0), TimeClass::Classical);
    }
}
