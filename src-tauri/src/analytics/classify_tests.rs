use super::*;

fn base() -> ClassifyInput {
    ClassifyInput {
        ply: 10,
        in_book: false,
        legal_moves: 20,
        delta_w: 0.0,
        played_is_best: false,
        best_win: 50.0,
        played_win: 50.0,
        second_gap: 3.0,
        sacrifice: false,
        prev_opponent_blunder: false,
        missed_mate: false,
        allowed_mate: false,
    }
}

fn case(delta: f64, played_is_best: bool) -> ClassifyInput {
    ClassifyInput {
        delta_w: delta,
        played_is_best,
        ..base()
    }
}

#[test]
fn book_has_highest_priority() {
    let input = ClassifyInput {
        in_book: true,
        ..case(30.0, false)
    };
    assert_eq!(classify(&input), MoveCategory::Book);
}

#[test]
fn forced_when_only_one_legal_move() {
    let input = ClassifyInput {
        legal_moves: 1,
        ..case(30.0, false)
    };
    assert_eq!(classify(&input), MoveCategory::Forced);
}

#[test]
fn brilliant_requires_sacrifice_and_high_eval() {
    let input = ClassifyInput {
        sacrifice: true,
        delta_w: 0.8,
        played_win: 62.0,
        best_win: 70.0,
        ..base()
    };
    assert_eq!(classify(&input), MoveCategory::Brilliant);
    let no_sacrifice = ClassifyInput {
        sacrifice: false,
        ..input
    };
    assert_ne!(classify(&no_sacrifice), MoveCategory::Brilliant);
}

#[test]
fn great_needs_wide_gap() {
    let input = ClassifyInput {
        delta_w: 1.0,
        second_gap: 20.0,
        ..base()
    };
    assert_eq!(classify(&input), MoveCategory::Great);
    let narrow = ClassifyInput {
        second_gap: 8.0,
        ..input
    };
    assert_ne!(classify(&narrow), MoveCategory::Great);
}

#[test]
fn best_when_matching_engine_move() {
    assert_eq!(classify(&case(0.0, true)), MoveCategory::Best);
    assert_eq!(classify(&case(0.4, false)), MoveCategory::Best);
}

#[test]
fn excellent_good_and_inaccuracy_bands() {
    assert_eq!(classify(&case(1.8, false)), MoveCategory::Excellent);
    assert_eq!(classify(&case(4.5, false)), MoveCategory::Good);
    assert_eq!(classify(&case(9.0, false)), MoveCategory::Inaccuracy);
}

#[test]
fn miss_after_opponent_blunder() {
    let input = ClassifyInput {
        delta_w: 15.0,
        prev_opponent_blunder: true,
        best_win: 78.0,
        played_win: 55.0,
        ..base()
    };
    assert_eq!(classify(&input), MoveCategory::Miss);
    let not_after_blunder = ClassifyInput {
        prev_opponent_blunder: false,
        ..input
    };
    assert_eq!(classify(&not_after_blunder), MoveCategory::Mistake);
}

#[test]
fn mistake_and_blunder_bands() {
    assert_eq!(classify(&case(18.0, false)), MoveCategory::Mistake);
    assert_eq!(classify(&case(30.0, false)), MoveCategory::Blunder);
}

#[test]
fn mate_conditions_force_blunder() {
    let missed = ClassifyInput {
        missed_mate: true,
        ..case(2.0, false)
    };
    assert_eq!(classify(&missed), MoveCategory::Blunder);
    let allowed = ClassifyInput {
        allowed_mate: true,
        ..case(2.0, false)
    };
    assert_eq!(classify(&allowed), MoveCategory::Blunder);
}

#[test]
fn critical_threshold() {
    assert!(is_critical(10.0));
    assert!(!is_critical(9.99));
}

#[test]
fn every_category_is_reachable() {
    let inputs = vec![
        ClassifyInput {
            in_book: true,
            ..base()
        },
        ClassifyInput {
            legal_moves: 1,
            ..base()
        },
        ClassifyInput {
            sacrifice: true,
            delta_w: 0.5,
            played_win: 60.0,
            ..base()
        },
        ClassifyInput {
            delta_w: 1.0,
            second_gap: 16.0,
            ..base()
        },
        case(0.2, true),
        case(2.0, false),
        case(5.0, false),
        ClassifyInput {
            prev_opponent_blunder: true,
            best_win: 75.0,
            delta_w: 14.0,
            played_win: 50.0,
            ..base()
        },
        case(9.0, false),
        case(16.0, false),
        case(25.0, false),
    ];
    let mut seen = std::collections::BTreeSet::new();
    for input in inputs {
        seen.insert(classify(&input));
    }
    assert_eq!(seen.len(), 11);
}
