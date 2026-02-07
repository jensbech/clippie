#[derive(Debug, Clone)]
pub struct FuzzyMatch {
    pub matched: bool,
    pub match_positions: Vec<(usize, usize)>,
    pub is_exact: bool,
}

pub fn fuzzy_match(text: &str, query: &str) -> FuzzyMatch {
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    if let Some(pos) = text_lower.find(&query_lower) {
        return FuzzyMatch {
            matched: true,
            match_positions: vec![(pos, query_lower.len())],
            is_exact: true,
        };
    }

    let mut match_positions = Vec::new();
    let mut query_chars = query_lower.chars().peekable();
    let mut text_chars = text_lower.chars().enumerate().peekable();

    while let Some(q_char) = query_chars.peek() {
        let mut found = false;
        while let Some((idx, t_char)) = text_chars.peek() {
            if t_char == q_char {
                match_positions.push((*idx, 1));
                query_chars.next();
                text_chars.next();
                found = true;
                break;
            }
            text_chars.next();
        }
        if !found {
            return FuzzyMatch {
                matched: false,
                match_positions: Vec::new(),
                is_exact: false,
            };
        }
    }

    let merged = merge_adjacent_positions(match_positions);

    FuzzyMatch {
        matched: true,
        match_positions: merged,
        is_exact: false,
    }
}

fn merge_adjacent_positions(positions: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (pos, len) in positions {
        if let Some(last) = merged.last_mut() {
            if last.0 + last.1 == pos {
                last.1 += len;
                continue;
            }
        }
        merged.push((pos, len));
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let result = fuzzy_match("hello world", "world");
        assert!(result.matched);
        assert!(result.is_exact);
        assert_eq!(result.match_positions, vec![(6, 5)]);
    }

    #[test]
    fn test_fuzzy_match() {
        let result = fuzzy_match("hello world", "hlo");
        assert!(result.matched);
        assert!(!result.is_exact);
    }

    #[test]
    fn test_no_match() {
        let result = fuzzy_match("hello world", "xyz");
        assert!(!result.matched);
    }

    #[test]
    fn test_case_insensitive() {
        let result = fuzzy_match("Hello World", "hello");
        assert!(result.matched);
        assert!(result.is_exact);
    }

    #[test]
    fn test_fuzzy_multi_word() {
        let result = fuzzy_match(
            "dotnet nuget add source https://nuget.pkg.github.com",
            "dotnet source",
        );
        assert!(result.matched);
    }
}
