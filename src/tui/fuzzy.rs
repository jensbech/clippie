/// Fuzzy search with match tracking
/// Returns (matched, match_positions, is_exact)
/// where match_positions is a vec of (start, length) tuples for each matching region

#[derive(Debug, Clone)]
pub struct FuzzyMatch {
    pub matched: bool,
    pub match_positions: Vec<(usize, usize)>,
    pub is_exact: bool,
}

/// Perform fuzzy matching on text
/// Matches if all characters in query appear in order in text
/// Also checks for exact substring matches
pub fn fuzzy_match(text: &str, query: &str) -> FuzzyMatch {
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    // Check for exact substring match first
    if let Some(pos) = text_lower.find(&query_lower) {
        return FuzzyMatch {
            matched: true,
            match_positions: vec![(pos, query_lower.len())],
            is_exact: true,
        };
    }

    // Fuzzy match: find all positions where query characters appear
    let mut match_positions = Vec::new();
    let mut query_chars = query_lower.chars().peekable();
    let mut text_chars = text_lower.chars().enumerate().peekable();

    while let Some(q_char) = query_chars.peek() {
        let mut found = false;

        while let Some((idx, t_char)) = text_chars.peek() {
            if t_char == q_char {
                let start = *idx;
                // Track this match
                match_positions.push((start, 1));
                query_chars.next();
                text_chars.next();
                found = true;
                break;
            }
            text_chars.next();
        }

        if !found {
            // Query character not found in remaining text
            return FuzzyMatch {
                matched: false,
                match_positions: Vec::new(),
                is_exact: false,
            };
        }
    }

    // Merge adjacent positions into ranges
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (pos, len) in match_positions {
        if let Some(last) = merged.last_mut() {
            if last.0 + last.1 == pos {
                // Adjacent, merge them
                last.1 += len;
            } else {
                merged.push((pos, len));
            }
        } else {
            merged.push((pos, len));
        }
    }

    FuzzyMatch {
        matched: true,
        match_positions: merged,
        is_exact: false,
    }
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
        assert!(!result.match_positions.is_empty());
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
