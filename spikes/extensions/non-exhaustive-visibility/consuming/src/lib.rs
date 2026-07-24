//! The crate that consumes the `#[non_exhaustive]` vocabulary from outside its
//! defining crate.
//!
//! Everything in this module is a cross-crate form that *does* compile. The
//! forms that must not compile are the `trybuild` fixtures under
//! `tests/ui/fail/`, where the exact diagnostic is the evidence rather than a
//! side effect, and their sibling `tests/ui/pass/` cases pin the compiling
//! shapes the fail cases are contrasted against.

use non_exhaustive_defining::Growing;

/// Recognize the supported subset and reject anything else.
///
/// This is ADR 0074 convention 5c's recognizer shape. Across the crate
/// boundary `#[non_exhaustive]` makes the wildcard arm mandatory, so a third
/// variant would silently route into the rejection here with no compiler
/// diagnostic anywhere. That is the failure mode the amended convention
/// records, and it is why this compiles rather than why it is desirable.
#[must_use]
pub fn recognize(value: &Growing) -> Option<u8> {
    match value {
        Growing::A => Some(1),
        _ => None,
    }
}

/// Partially classify the vocabulary without enumerating it.
///
/// ADR 0074 convention 5a's case: a consumer that only asks one question of
/// the value never had to list the variants, so a new variant is genuinely
/// additive here.
#[must_use]
pub fn is_first(value: &Growing) -> bool {
    matches!(value, Growing::A)
}

#[cfg(test)]
mod tests {
    use super::{Growing, is_first, recognize};

    #[test]
    fn cross_crate_wildcard_recognizer_compiles_and_rejects_the_unlisted_variant() {
        assert_eq!(recognize(&Growing::A), Some(1));
        // `Growing::B` is a variant this consumer could have listed and did
        // not; the wildcard absorbs it with no diagnostic. Nothing but this
        // assertion reports it.
        assert_eq!(recognize(&Growing::B), None);
        assert!(is_first(&Growing::A));
        assert!(!is_first(&Growing::B));
    }
}
