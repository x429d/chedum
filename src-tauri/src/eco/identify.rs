/// Normalizes a name from a PGN header into readable form: collapses whitespace,
/// strips trailing punctuation and repeated segments.
pub fn normalize_opening_name(value: &str) -> Option<String> {
    let cleaned = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = cleaned.trim_matches(|ch: char| ch == '.' || ch == ',' || ch.is_whitespace());
    if trimmed.is_empty() {
        return None;
    }
    let mut segments: Vec<&str> = Vec::new();
    for segment in trimmed.split(", ") {
        if segments.last() != Some(&segment) {
            segments.push(segment);
        }
    }
    Some(segments.join(", "))
}

/// Opening name from a Chess.com slug URL: `.../openings/Vienna-Game` → `Vienna Game`.
pub fn name_from_eco_url(url: &str) -> Option<String> {
    let slug = url.trim().trim_end_matches('/').rsplit('/').next()?;
    if slug.is_empty() {
        return None;
    }
    normalize_opening_name(&slug.replace(['-', '_'], " "))
}
