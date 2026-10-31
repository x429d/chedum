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
    // White loses 30 Win% points on their second move.
    let moves = [
        (50.0, 50.0),
        (50.0, 50.0),
        (50.0, 20.0),
        (20.0, 20.0),
        (20.0, 20.0),
    ];
    let [white, black] = game_accuracy(&moves, Color::White);
    assert!(white.unwrap() < 90.0);
    assert!((black.unwrap() - 100.0).abs() < 1e-9);
}

#[test]
fn black_first_color_flips_the_alignment() {
    let moves = [
        (50.0, 50.0),
        (50.0, 50.0),
        (50.0, 20.0),
        (20.0, 20.0),
        (20.0, 20.0),
    ];
    let [white, black] = game_accuracy(&moves, Color::Black);
    assert!((white.unwrap() - 100.0).abs() < 1e-9);
    assert!(black.unwrap() < 90.0);
}

#[test]
fn harmonic_mean_punishes_single_bad_move() {
    let moves = [
        (90.0, 90.0),
        (90.0, 90.0),
        (90.0, 20.0),
        (20.0, 90.0),
        (90.0, 90.0),
    ];
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
