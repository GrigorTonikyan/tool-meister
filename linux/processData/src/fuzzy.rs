/// Fuzzy matching with scoring — similar to fzf/sublime logic.
///
/// Returns `None` if no match, or `Some((score, matched_indices))`.
/// Higher score = better match.
pub fn fuzzy_match(query: &str, target: &str) -> Option<(i64, Vec<usize>)> {
    if query.is_empty() {
        return Some((0, Vec::new()));
    }

    let query_lower: Vec<char> = query.to_lowercase().chars().collect();
    let target_lower: Vec<char> = target.to_lowercase().chars().collect();
    let target_chars: Vec<char> = target.chars().collect();

    // Quick check: all query chars must exist in target in order
    {
        let mut qi = 0;
        for &tc in &target_lower {
            if qi < query_lower.len() && tc == query_lower[qi] {
                qi += 1;
            }
        }
        if qi != query_lower.len() {
            return None;
        }
    }

    // Score the match
    let mut score: i64 = 0;
    let mut matched_indices = Vec::with_capacity(query_lower.len());
    let mut qi = 0;
    let mut prev_match_idx: Option<usize> = None;
    let mut consecutive = 0i64;

    for (ti, &tc) in target_lower.iter().enumerate() {
        if qi < query_lower.len() && tc == query_lower[qi] {
            matched_indices.push(ti);

            // Base score for a match
            score += 1;

            // Consecutive bonus (exponential for longer runs)
            if let Some(prev) = prev_match_idx {
                if ti == prev + 1 {
                    consecutive += 1;
                    score += consecutive * 3;
                } else {
                    consecutive = 0;
                }
            }

            // Word boundary bonus (after space, slash, dash, underscore, dot, or at start)
            if ti == 0
                || matches!(
                    target_chars.get(ti.wrapping_sub(1)),
                    Some(' ' | '/' | '-' | '_' | '.' | '\\')
                )
            {
                score += 8;
            }

            // Camel case boundary bonus
            if ti > 0 {
                if let (Some(&prev_c), Some(&curr_c)) =
                    (target_chars.get(ti - 1), target_chars.get(ti))
                {
                    if prev_c.is_lowercase() && curr_c.is_uppercase() {
                        score += 6;
                    }
                }
            }

            // Exact case match bonus
            if target_chars[ti] == query.chars().nth(qi).unwrap_or(' ') {
                score += 1;
            }

            // Penalty for distance from start
            score -= (ti as i64) / 10;

            prev_match_idx = Some(ti);
            qi += 1;
        }
    }

    // Length penalty — prefer shorter targets for same query
    score -= (target.len() as i64) / 20;

    Some((score, matched_indices))
}

/// Simple contains-based fallback for substring matching.
pub fn contains_match(query: &str, target: &str) -> bool {
    target.to_lowercase().contains(&query.to_lowercase())
}

/// Match against multiple fields, return best score.
pub fn fuzzy_match_multi(query: &str, fields: &[&str]) -> Option<(i64, usize)> {
    let mut best: Option<(i64, usize)> = None;
    for (idx, field) in fields.iter().enumerate() {
        if let Some((score, _)) = fuzzy_match(query, field) {
            match best {
                None => best = Some((score, idx)),
                Some((best_score, _)) if score > best_score => best = Some((score, idx)),
                _ => {}
            }
        }
    }
    best
}
