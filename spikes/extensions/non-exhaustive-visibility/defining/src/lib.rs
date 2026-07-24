//! The crate that defines the probe's `#[non_exhaustive]` vocabulary.
//!
//! `#[non_exhaustive]` constrains patterns written *outside* the defining
//! crate. [`same_crate_total_map`] is the control for that claim: it is the
//! encoder shape ADR 0074's convention 3 requires — every variant contributes
//! its own tag and there is no wildcard arm — and it compiles here. The
//! byte-identical body in the sibling `non-exhaustive-consuming` crate fails
//! `E0004`, which is the asymmetry the whole probe exists to record.

/// A public vocabulary documented as growing, in ADR 0074 convention 5's sense.
///
/// Two variants are the minimum that can distinguish "no wildcard arm is
/// needed" from "one omitted variant is reported", which the cross-crate
/// `non_exhaustive_omitted_patterns` fixtures depend on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Growing {
    /// The first inhabited variant.
    A,
    /// The second inhabited variant.
    B,
}

/// Map every variant of [`Growing`] to its own tag with no wildcard arm.
///
/// This is a total map in ADR 0074 convention 5b's sense: no wildcard value is
/// derivable from the variant it would cover, so the absence of a catch-all is
/// the mechanism that turns a widened vocabulary into a compile error at this
/// site rather than into two subjects sharing identity bytes.
#[must_use]
pub fn same_crate_total_map(value: &Growing) -> u8 {
    match value {
        Growing::A => 1,
        Growing::B => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::{Growing, same_crate_total_map};

    #[test]
    fn same_crate_total_map_is_exhaustive_without_a_wildcard() {
        // The assertion is secondary; that this module compiles at all is the
        // measurement, because `same_crate_total_map` has no catch-all arm.
        assert_eq!(same_crate_total_map(&Growing::A), 1);
        assert_eq!(same_crate_total_map(&Growing::B), 2);
    }
}
