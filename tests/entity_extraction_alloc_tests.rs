//! Allocation regression guard for entity extraction (P1).
//!
//! Known organizations and concepts are matched case-insensitively. Commit
//! bf45c0b made that match lowercase a fresh `String` at every char position
//! for every known term (~57 terms), so `extract` allocated O(text length x
//! terms) times and `entity_extraction/extract_mixed` regressed ~12x (Q7).
//! A deterministic allocation count catches that class of regression without
//! a timing assertion: the number of allocations of one `extract` call must
//! not grow with the length of text that contains no entities.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use engram::intelligence::entities::{EntityExtractionConfig, EntityExtractor};

struct CountingAllocator;

thread_local! {
    // `const` init and a `Copy` payload: no lazy init or destructor, so the
    // allocator never re-enters itself through this thread local.
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

// SAFETY: every call is forwarded unchanged to `System`; the counter only
// touches a thread-local `Cell<usize>`.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.with(|count| count.set(count.get() + 1));
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.with(|count| count.set(count.get() + 1));
        System.realloc(ptr, layout, new_size)
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

/// Upper bound for one `extract` call on entity-free filler, independent of
/// the filler length (the lowercased copy, its offset map and a handful of
/// result vectors; regex caches are warmed before counting).
const MAX_ALLOCATIONS_PER_EXTRACT: usize = 64;

fn allocations_for_one_extract(extractor: &EntityExtractor, text: &str) -> usize {
    // Warm the lazily compiled regexes and their per-thread caches.
    let _ = extractor.extract(text);
    let before = ALLOCATIONS.with(Cell::get);
    let result = extractor.extract(text);
    let after = ALLOCATIONS.with(Cell::get);
    drop(result);
    after - before
}

fn assert_allocations_do_not_scale(filler: &str) {
    let extractor = EntityExtractor::new(EntityExtractionConfig::default());
    for repeats in [64_usize, 512] {
        let text = filler.repeat(repeats);
        let allocations = allocations_for_one_extract(&extractor, &text);
        assert!(
            allocations <= MAX_ALLOCATIONS_PER_EXTRACT,
            "extract allocated {allocations} times for {} chars of {filler:?} \
             (bound {MAX_ALLOCATIONS_PER_EXTRACT}); known-term matching must not \
             allocate per char position",
            text.chars().count()
        );
    }
}

#[test]
fn ascii_text_allocations_do_not_scale_with_length() {
    assert_allocations_do_not_scale("plain words without known names and lowercase filler ");
}

#[test]
fn non_ascii_text_allocations_do_not_scale_with_length() {
    assert_allocations_do_not_scale("café façade naïve déjà vu İ ẞ e\u{301} plain words ");
}

#[test]
fn capital_sigma_text_allocations_do_not_scale_with_length() {
    // `Σ` lowercases by context (final sigma), the one char whose window
    // lowercase cannot be precomputed once for the whole text.
    assert_allocations_do_not_scale("ΟΔΥΣΣΕΑΣ plain words without known names ");
}
