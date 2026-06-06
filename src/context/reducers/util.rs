pub(crate) fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }

        out.push(ch);
    }

    out
}

pub(crate) fn truncate(input: &str, max_chars: usize) -> (String, bool) {
    let char_count = input.chars().count();
    if char_count <= max_chars {
        return (input.to_string(), false);
    }

    if max_chars == 0 {
        return (String::new(), true);
    }

    const MARKER: &str = "...[truncated]";
    if max_chars <= MARKER.len() {
        return (input.chars().take(max_chars).collect(), true);
    }

    let keep = max_chars - MARKER.len();
    let mut truncated: String = input.chars().take(keep).collect();
    truncated.push_str(MARKER);
    (truncated, true)
}

pub(crate) fn parse_path_line_col(candidate: &str) -> Option<(String, u64, Option<u64>)> {
    let cleaned = candidate
        .trim()
        .trim_matches('`')
        .trim_matches('\'')
        .trim_matches('"')
        .trim_end_matches(':')
        .trim_end_matches(',');
    let parts: Vec<&str> = cleaned.split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    let last = parts.last()?;
    if let Ok(column) = last.parse::<u64>() {
        if parts.len() >= 3 {
            let line_part = parts.get(parts.len() - 2)?;
            if let Ok(line) = line_part.parse::<u64>() {
                let path = parts[..parts.len() - 2].join(":");
                if !path.is_empty() {
                    return Some((path, line, Some(column)));
                }
            }
        }
    }

    if let Ok(line) = last.parse::<u64>() {
        let path = parts[..parts.len() - 1].join(":");
        if !path.is_empty() {
            return Some((path, line, None));
        }
    }

    None
}

pub(crate) fn extract_clippy_rule(input: &str) -> Option<String> {
    let start = input.find("clippy::")?;
    let rule: String = input[start..]
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == ':')
        .collect();

    if rule.len() > "clippy::".len() {
        Some(rule)
    } else {
        None
    }
}

pub(crate) fn is_numeric(input: &str) -> bool {
    !input.is_empty() && input.chars().all(|ch| ch.is_ascii_digit())
}
