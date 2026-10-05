//! Fuzz target: `engram::text_util` (added by task C3).
//!
//! Every byte budget applied to user text must go through these helpers; they
//! must be total and always return slices that borrow from the original text on
//! a char boundary.
#![no_main]

use engram::text_util::{ceil_char_boundary, floor_char_boundary, suffix_bytes, truncate_bytes};
use libfuzzer_sys::fuzz_target;

// Input layout: bytes 0..2 = little-endian byte budget, rest = UTF-8 text
// (invalid sequences are replaced).
fuzz_target!(|data: &[u8]| {
    let [lo, hi, rest @ ..] = data else {
        return;
    };
    let budget = usize::from(u16::from_le_bytes([*lo, *hi]));
    let text = String::from_utf8_lossy(rest);
    let text = text.as_ref();

    let floor = floor_char_boundary(text, budget);
    let ceil = ceil_char_boundary(text, budget);
    assert!(floor <= text.len() && ceil <= text.len());
    assert!(text.is_char_boundary(floor) && text.is_char_boundary(ceil));
    assert!(floor <= budget.min(text.len()));
    assert!(ceil >= budget.min(text.len()));

    let prefix = truncate_bytes(text, budget);
    assert!(prefix.len() <= budget);
    assert!(text.starts_with(prefix));
    if budget >= text.len() {
        assert_eq!(prefix, text);
    }

    let suffix = suffix_bytes(text, budget);
    assert!(suffix.len() <= budget);
    assert!(text.ends_with(suffix));
    if budget >= text.len() {
        assert_eq!(suffix, text);
    }
});
