//! UTF-8-safe text helpers.
//!
//! Slicing a `&str` with a byte index (`&s[..n]`) panics when `n` falls inside a
//! multibyte char, and with `panic = "abort"` in release a single stored memory
//! could kill the server (Q2-B01..B06). Every byte budget applied to user text
//! must go through these helpers instead.
//!
//! All functions are total: they never panic for any `&str` / index pair, and the
//! returned slices always borrow from the original string (no normalization, so
//! callers never mix offsets from a transformed copy with the original).

/// Largest char boundary of `s` that is `<= index` (clamped to `s.len()`).
pub fn floor_char_boundary(s: &str, index: usize) -> usize {
    if index >= s.len() {
        return s.len();
    }
    let mut boundary = index;
    while !s.is_char_boundary(boundary) {
        boundary -= 1; // index 0 is always a boundary, so this terminates.
    }
    boundary
}

/// Smallest char boundary of `s` that is `>= index` (clamped to `s.len()`).
pub fn ceil_char_boundary(s: &str, index: usize) -> usize {
    if index >= s.len() {
        return s.len();
    }
    let mut boundary = index;
    while !s.is_char_boundary(boundary) {
        boundary += 1; // s.len() is always a boundary, so this terminates.
    }
    boundary
}

/// The longest prefix of `s` that is at most `max_bytes` bytes long and ends on
/// a char boundary.
pub fn truncate_bytes(s: &str, max_bytes: usize) -> &str {
    &s[..floor_char_boundary(s, max_bytes)]
}

/// The longest suffix of `s` that is at most `max_bytes` bytes long and starts on
/// a char boundary.
pub fn suffix_bytes(s: &str, max_bytes: usize) -> &str {
    let start = ceil_char_boundary(s, s.len().saturating_sub(max_bytes));
    &s[start..]
}

/// Greek capital sigma: the one char `str::to_lowercase` maps by context
/// (final sigma `ς` vs `σ`), so its lowercase depends on the window around it.
const CAPITAL_SIGMA: char = 'Σ';

/// `text` lowercased once, char by char, with a map back to the original, for
/// case-insensitive searches of many needles in the same text.
///
/// Offsets are never shared between the lowercased copy and the original
/// (casing changes UTF-8 lengths, e.g. `ẞ` -> `ß`, `İ` -> `i̇`): every match is
/// mapped back through the recorded char offsets, so returned slices always
/// start and end on char boundaries of the original.
pub struct LowercaseIndex<'a> {
    original: &'a str,
    /// Concatenation of `char::to_lowercase` of every original char.
    lower: String,
    /// One `(original_offset, lower_offset)` per original char plus a final
    /// `(original.len(), lower.len())`; `None` when `original` is ASCII, where
    /// both offsets are always equal.
    char_offsets: Option<Vec<(usize, usize)>>,
    /// Windows containing `Σ` are compared exactly instead of via `lower`.
    has_capital_sigma: bool,
}

impl<'a> LowercaseIndex<'a> {
    /// Lowercase `original` once (one allocation, plus the offset map for
    /// non-ASCII text).
    pub fn new(original: &'a str) -> Self {
        if original.is_ascii() {
            return Self {
                original,
                lower: original.to_ascii_lowercase(),
                char_offsets: None,
                has_capital_sigma: false,
            };
        }
        let mut lower = String::with_capacity(original.len());
        let mut char_offsets = Vec::with_capacity(original.len() + 1);
        let mut has_capital_sigma = false;
        for (offset, c) in original.char_indices() {
            char_offsets.push((offset, lower.len()));
            has_capital_sigma |= c == CAPITAL_SIGMA;
            lower.extend(c.to_lowercase());
        }
        char_offsets.push((original.len(), lower.len()));
        Self {
            original,
            lower,
            char_offsets: Some(char_offsets),
            has_capital_sigma,
        }
    }

    /// Leftmost window of exactly `needle.chars().count()` chars of the
    /// original text whose `str::to_lowercase()` equals `needle`, as
    /// `(byte offset, original slice)`. `needle` is compared as given, so pass
    /// it lowercased. An empty needle never matches.
    pub fn find_window(&self, needle: &str) -> Option<(usize, &'a str)> {
        let needle_chars = needle.chars().count();
        if needle_chars == 0 {
            return None;
        }
        if self.has_capital_sigma {
            return find_window_exact(self.original, needle, needle_chars);
        }
        let Some(char_offsets) = &self.char_offsets else {
            // ASCII: a match is ASCII too, so bytes == chars on both sides.
            let start = self.lower.find(needle)?;
            return Some((start, &self.original[start..start + needle.len()]));
        };

        let mut from = 0;
        while let Some(relative) = self.lower[from..].find(needle) {
            let lower_start = from + relative;
            if let Ok(first) = char_offsets.binary_search_by_key(&lower_start, |&(_, low)| low) {
                // The window must cover exactly `needle_chars` original chars
                // whose lowercase ends where the needle ends.
                let aligned_end = char_offsets
                    .get(first + needle_chars)
                    .filter(|&&(_, low)| low == lower_start + needle.len());
                if let Some(&(end, _)) = aligned_end {
                    let start = char_offsets[first].0;
                    return Some((start, &self.original[start..end]));
                }
            }
            let step = self.lower[lower_start..]
                .chars()
                .next()
                .map_or(1, char::len_utf8);
            from = lower_start + step;
        }
        None
    }
}

/// Exact window scan for text containing `Σ`. Each start is first compared
/// without allocating (`Σ` may stand for `σ` or `ς`); only windows that pass
/// are lowercased with `str::to_lowercase` to settle the final-sigma context.
fn find_window_exact<'a>(
    text: &'a str,
    needle: &str,
    needle_chars: usize,
) -> Option<(usize, &'a str)> {
    for (start, _) in text.char_indices() {
        let Some(end) = window_end_if_plausible(&text[start..], needle, needle_chars) else {
            continue;
        };
        let window = &text[start..start + end];
        if window.to_lowercase() == needle {
            return Some((start, window));
        }
    }
    None
}

/// Byte length of the `needle_chars`-char prefix of `rest` when its char-wise
/// lowercase can equal `needle` (`Σ` accepted as `σ` or `ς`).
fn window_end_if_plausible(rest: &str, needle: &str, needle_chars: usize) -> Option<usize> {
    let mut expected = needle.chars();
    let mut end = 0;
    let mut window_chars = 0;
    for c in rest.chars().take(needle_chars) {
        if c == CAPITAL_SIGMA {
            if !matches!(expected.next(), Some('σ' | 'ς')) {
                return None;
            }
        } else {
            for lowered in c.to_lowercase() {
                if expected.next() != Some(lowered) {
                    return None;
                }
            }
        }
        end += c.len_utf8();
        window_chars += 1;
    }
    // Too few chars left, or the needle is longer than the window's lowercase.
    if window_chars != needle_chars || expected.next().is_some() {
        return None;
    }
    Some(end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_and_ceil_snap_to_boundaries() {
        let s = "aé"; // 'é' = bytes 1..3
        assert_eq!(floor_char_boundary(s, 2), 1);
        assert_eq!(ceil_char_boundary(s, 2), 3);
        assert_eq!(floor_char_boundary(s, 99), 3);
        assert_eq!(ceil_char_boundary(s, 99), 3);
        assert_eq!(floor_char_boundary(s, 0), 0);
        assert_eq!(ceil_char_boundary("", 5), 0);
    }

    #[test]
    fn truncate_and_suffix_respect_budget() {
        let s = "a😀b"; // 😀 = bytes 1..5
        assert_eq!(truncate_bytes(s, 0), "");
        assert_eq!(truncate_bytes(s, 1), "a");
        assert_eq!(truncate_bytes(s, 4), "a");
        assert_eq!(truncate_bytes(s, 5), "a😀");
        assert_eq!(truncate_bytes(s, 100), s);
        assert_eq!(suffix_bytes(s, 0), "");
        assert_eq!(suffix_bytes(s, 1), "b");
        assert_eq!(suffix_bytes(s, 4), "b");
        assert_eq!(suffix_bytes(s, 5), "😀b");
        assert_eq!(suffix_bytes(s, 100), s);
    }

    /// The window matcher that `LowercaseIndex::find_window` replaced
    /// (entities.rs, bf45c0b), kept verbatim as the behavioral oracle.
    fn reference_find_window<'a>(text: &'a str, needle: &str) -> Option<(usize, &'a str)> {
        let needle_len = needle.chars().count();
        if needle_len == 0 {
            return None;
        }
        for (start, _) in text.char_indices() {
            let end = match text[start..].char_indices().nth(needle_len) {
                Some((offset, _)) => start + offset,
                None => text.len(),
            };
            let candidate = &text[start..end];
            if candidate.chars().count() == needle_len && candidate.to_lowercase() == needle {
                return Some((start, candidate));
            }
        }
        None
    }

    fn assert_matches_reference(text: &str, needle: &str) {
        assert_eq!(
            LowercaseIndex::new(text).find_window(needle),
            reference_find_window(text, needle),
            "text {text:?}, needle {needle:?}"
        );
    }

    #[test]
    fn find_window_maps_matches_back_to_original_slices() {
        let index = LowercaseIndex::new("İ OpenAI and ẞtraße");
        assert_eq!(index.find_window("openai"), Some((3, "OpenAI")));
        assert_eq!(index.find_window("ßtraße"), Some((14, "ẞtraße")));
        // Windows are counted in original chars and `İ` lowercases to two
        // ("i̇"), so no window containing it can equal its lowercase.
        assert_eq!(index.find_window("i\u{307}"), None);
        assert_eq!(index.find_window("i\u{307} "), None);
        assert_eq!(index.find_window("missing"), None);
        assert_eq!(index.find_window(""), None);
        assert_eq!(
            LowercaseIndex::new("Use RUST").find_window("rust"),
            Some((4, "RUST"))
        );
        assert_eq!(LowercaseIndex::new("").find_window("a"), None);
    }

    #[test]
    fn find_window_matches_reference_on_adversarial_unicode() {
        let texts = [
            "İ OpenAI",
            "İstanbul ISTANBUL",
            "ẞtraße STRASSE",
            "Cafe\u{301} CAFÉ café",
            "\u{212A}afka KAFKA",
            "ΟΔΥΣΣΕΑΣ Σ σ ς ΣΑΣ",
            "ΑΣ ΑΣΑ",
            "aaab AAAB",
            "ǅemal ǄEMAL",
            "ﬁle FILE",
            "😀GitHub😀",
            "a",
            "",
        ];
        let needles = [
            "openai",
            "i\u{307}stanbul",
            "istanbul",
            "ßtraße",
            "strasse",
            "ss",
            "café",
            "cafe\u{301}",
            "e\u{301}",
            "kafka",
            "σ",
            "ς",
            "ας",
            "ασ",
            "οδυσσεας",
            "οδυσσεασ",
            "σας",
            "σασ",
            "aab",
            "ǆemal",
            "ﬁle",
            "github",
            "a",
            "",
        ];
        for text in texts {
            for needle in needles {
                assert_matches_reference(text, needle);
            }
        }
    }

    /// Deterministic fuzz (fixed seed, no external RNG): random texts over an
    /// alphabet of case-changing, length-changing and context-sensitive chars,
    /// needles taken as lowercased windows of the text or random strings.
    #[test]
    fn find_window_matches_reference_on_seeded_random_inputs() {
        const ALPHABET: &[char] = &[
            'a', 'A', 'b', 'B', 's', 'S', ' ', 'İ', 'i', 'ẞ', 'ß', 'Σ', 'σ', 'ς', 'Α', 'α',
            '\u{301}', '\u{307}', '\u{212A}', 'k', 'ǅ', 'ǆ', 'ﬁ', 'é', '😀', '-',
        ];
        let mut state: u64 = 0x5EED_0FE1_7000_0001;
        let mut next = |bound: usize| {
            // xorshift64*; deterministic across platforms.
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            (state.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 33) as usize % bound
        };
        for _ in 0..20_000 {
            let text: String = (0..next(12))
                .map(|_| ALPHABET[next(ALPHABET.len())])
                .collect();
            let chars: Vec<char> = text.chars().collect();
            let needle = if !chars.is_empty() && next(3) > 0 {
                let start = next(chars.len());
                let len = 1 + next(chars.len() - start);
                chars[start..start + len]
                    .iter()
                    .collect::<String>()
                    .to_lowercase()
            } else {
                (0..next(4))
                    .map(|_| ALPHABET[next(ALPHABET.len())])
                    .collect()
            };
            assert_matches_reference(&text, &needle);
        }
    }
}
