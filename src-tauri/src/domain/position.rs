use shakmaty::{
    fen::Fen, san::SanPlus, uci::UciMove, CastlingMode, Chess, Color as ShakColor, EnPassantMode,
    Move, Piece, Position, Role, Square,
};

use crate::domain::error::{AppError, AppResult};
use crate::domain::game::Color;

/// Разбирает FEN в позицию стандартных шахмат.
pub fn parse_fen(fen: &str) -> AppResult<Chess> {
    let parsed = Fen::from_ascii(fen.as_bytes())
        .map_err(|err| AppError::Parse(format!("Некорректный FEN: {err}")))?;
    parsed
        .into_position(CastlingMode::Standard)
        .map_err(|err| AppError::Parse(format!("Недопустимая позиция: {err}")))
}

/// Возвращает канонический FEN позиции (включая счётчики полуходов).
pub fn position_fen(pos: &Chess) -> String {
    Fen::from_position(pos, EnPassantMode::Legal).to_string()
}

/// FEN без счётчиков — используется как ключ дебютного словаря.
pub fn book_key(fen: &str) -> String {
    fen.split(' ').take(4).collect::<Vec<_>>().join(" ")
}

pub fn move_uci(mv: Move) -> String {
    mv.to_uci(CastlingMode::Standard).to_string()
}

pub fn parse_uci(fen: &str, uci: &str) -> AppResult<(Chess, Move)> {
    let pos = parse_fen(fen)?;
    let parsed = UciMove::from_ascii(uci.as_bytes())
        .map_err(|err| AppError::Parse(format!("Некорректный UCI «{uci}»: {err}")))?;
    let mv = parsed
        .to_move(&pos)
        .map_err(|err| AppError::Validation(format!("Ход «{uci}» нелегален: {err}")))?;
    Ok((pos, mv))
}

/// Применяет ход и возвращает SAN с суффиксом шаха/мата.
pub fn san_and_play(pos: &mut Chess, mv: Move) -> String {
    SanPlus::from_move_and_play_unchecked(pos, mv).to_string()
}

/// SAN хода без изменения позиции.
pub fn san_of(pos: &Chess, mv: Move) -> String {
    SanPlus::from_move(pos.clone(), mv).to_string()
}

pub fn parse_san(fen: &str, san: &str) -> AppResult<(Chess, Move)> {
    let pos = parse_fen(fen)?;
    let parsed = SanPlus::from_ascii(san.as_bytes())
        .map_err(|err| AppError::Parse(format!("Некорректный SAN «{san}»: {err}")))?;
    let mv = parsed
        .san
        .to_move(&pos)
        .map_err(|err| AppError::Validation(format!("SAN «{san}» нелегален: {err}")))?;
    Ok((pos, mv))
}

/// Переводит последовательность UCI-ходов в SAN, останавливаясь на первом нелегальном.
pub fn uci_line_to_san(fen: &str, moves: &[String]) -> Vec<String> {
    let mut pos = match parse_fen(fen) {
        Ok(pos) => pos,
        Err(_) => return Vec::new(),
    };
    let mut san = Vec::with_capacity(moves.len());
    for uci in moves {
        let parsed = match UciMove::from_ascii(uci.as_bytes()) {
            Ok(parsed) => parsed,
            Err(_) => break,
        };
        let mv = match parsed.to_move(&pos) {
            Ok(mv) => mv,
            Err(_) => break,
        };
        san.push(san_and_play(&mut pos, mv));
    }
    san
}

/// Итоговый FEN после последовательности UCI-ходов.
pub fn apply_uci_line(fen: &str, moves: &[String]) -> AppResult<String> {
    let mut pos = parse_fen(fen)?;
    for uci in moves {
        let parsed = UciMove::from_ascii(uci.as_bytes())
            .map_err(|err| AppError::Parse(format!("Некорректный UCI «{uci}»: {err}")))?;
        let mv = parsed
            .to_move(&pos)
            .map_err(|err| AppError::Validation(format!("Ход «{uci}» нелегален: {err}")))?;
        pos.play_unchecked(mv);
    }
    Ok(position_fen(&pos))
}

pub fn legal_move_count(pos: &Chess) -> usize {
    pos.legal_moves().len()
}

pub fn piece_value(role: Role) -> i32 {
    match role {
        Role::Pawn => 1,
        Role::Knight => 3,
        Role::Bishop => 3,
        Role::Rook => 5,
        Role::Queen => 9,
        Role::King => 0,
    }
}

fn material(pos: &Chess, color: ShakColor) -> i32 {
    let counts = pos.board().material_side(color);
    piece_value(Role::Pawn) * i32::from(counts.pawn)
        + piece_value(Role::Knight) * i32::from(counts.knight)
        + piece_value(Role::Bishop) * i32::from(counts.bishop)
        + piece_value(Role::Rook) * i32::from(counts.rook)
        + piece_value(Role::Queen) * i32::from(counts.queen)
}

/// Материальный перевес белых в пешках (без королей).
pub fn material_diff_white(pos: &Chess) -> i32 {
    material(pos, ShakColor::White) - material(pos, ShakColor::Black)
}

/// Суммарная ценность фигур и пешек указанной стороны (король = 0).
pub fn material_value(pos: &Chess, color: Color) -> i32 {
    material(pos, color_to_shakmaty(color))
}

/// Применяет UCI-ход к позиции.
pub fn play_uci(pos: &mut Chess, uci: &str) -> AppResult<Move> {
    let parsed = UciMove::from_ascii(uci.as_bytes())
        .map_err(|err| AppError::Parse(format!("Некорректный UCI «{uci}»: {err}")))?;
    let mv = parsed
        .to_move(pos)
        .map_err(|err| AppError::Validation(format!("Ход «{uci}» нелегален: {err}")))?;
    pos.play_unchecked(mv);
    Ok(mv)
}

/// Оставлена ли фигура под боем менее ценной фигуры без защиты.
pub fn is_hanging_after(pos: &Chess, mv: Move, player: Color) -> bool {
    let square = mv.to();
    let Some(piece) = pos.board().piece_at(square) else {
        return false;
    };
    let own = color_to_shakmaty(player);
    if piece.color != own {
        return false;
    }
    let occupied = pos.board().occupied();
    let attackers = pos.board().attacks_to(square, own.other(), occupied);
    if attackers.is_empty() {
        return false;
    }
    let defenders = pos.board().attacks_to(square, own, occupied);
    if defenders.is_empty() {
        return true;
    }
    let lowest_attacker = attackers
        .into_iter()
        .filter_map(|sq| pos.board().piece_at(sq))
        .map(|attacker| piece_value(attacker.role))
        .min()
        .unwrap_or(0);
    lowest_attacker < piece_value(piece.role)
}

/// Суммарная ценность фигур (без пешек и королей) обеих сторон.
pub fn heavy_material_total(pos: &Chess) -> i32 {
    let counts = pos.board().material();
    let value = |c: &shakmaty::ByRole<u8>| {
        i32::from(c.knight) * 3
            + i32::from(c.bishop) * 3
            + i32::from(c.rook) * 5
            + i32::from(c.queen) * 9
    };
    value(&counts.white) + value(&counts.black)
}

/// Количество фигур (кроме пешек и королей) на доске.
pub fn piece_count_nonpawn(pos: &Chess) -> u32 {
    let counts = pos.board().material();
    let count = |c: &shakmaty::ByRole<u8>| {
        u32::from(c.knight) + u32::from(c.bishop) + u32::from(c.rook) + u32::from(c.queen)
    };
    count(&counts.white) + count(&counts.black)
}

/// Число развитых фигур (всего, обе стороны): не пешки/короли вне стартовых полей.
pub fn developed_count(pos: &Chess) -> u32 {
    let board = pos.board();
    let mut developed = 0;
    for sq in Square::ALL {
        if let Some(piece) = board.piece_at(sq) {
            if piece.role == Role::Pawn || piece.role == Role::King {
                continue;
            }
            if !is_home_square(piece, sq) {
                developed += 1;
            }
        }
    }
    developed
}

fn is_home_square(piece: Piece, sq: Square) -> bool {
    let back_rank = match piece.color {
        ShakColor::White => shakmaty::Rank::First,
        ShakColor::Black => shakmaty::Rank::Eighth,
    };
    if sq.rank() != back_rank {
        return false;
    }
    match piece.role {
        Role::Knight => matches!(sq.file(), shakmaty::File::B | shakmaty::File::G),
        Role::Bishop => matches!(sq.file(), shakmaty::File::C | shakmaty::File::F),
        Role::Rook => matches!(sq.file(), shakmaty::File::A | shakmaty::File::H),
        Role::Queen => sq.file() == shakmaty::File::D,
        Role::King | Role::Pawn => false,
    }
}

pub fn side_to_move(pos: &Chess) -> Color {
    match pos.turn() {
        ShakColor::White => Color::White,
        ShakColor::Black => Color::Black,
    }
}

pub fn color_to_shakmaty(color: Color) -> ShakColor {
    match color {
        Color::White => ShakColor::White,
        Color::Black => ShakColor::Black,
    }
}

pub fn is_check(pos: &Chess) -> bool {
    pos.is_check()
}

pub fn is_checkmate(pos: &Chess) -> bool {
    pos.is_checkmate()
}

pub fn king_square(pos: &Chess, color: ShakColor) -> Option<Square> {
    pos.board().king_of(color)
}

/// Атакована ли клетка фигурами указанного цвета.
pub fn is_attacked(pos: &Chess, sq: Square, by: ShakColor) -> bool {
    pos.board().attacks_to(sq, by, pos.board().occupied()).any()
}
