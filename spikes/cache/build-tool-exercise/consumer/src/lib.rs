//! The crate whose compilation drives expansion-cache resolutions.
//!
//! Four invocations, so one build of this crate is four expansions over four
//! distinct keys. The count is fixed and stated here because the driver judges a
//! run by comparing the events it collected against the number it can prove it
//! should have seen — a scenario that silently expanded nothing would otherwise
//! be indistinguishable from one where every expansion hit.

/// The number of `resolve!` invocations one build of this crate performs.
pub const INVOCATIONS: usize = 4;

exercise_macro::resolve!(KEY_A);
exercise_macro::resolve!(KEY_B);
exercise_macro::resolve!(KEY_C);
exercise_macro::resolve!(KEY_D);
