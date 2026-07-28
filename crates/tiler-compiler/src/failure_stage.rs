//! Where an opaque call may fail, and where falling back stops being legal.
//!
//! A slice of `implement-opaque-physical-call-providers`, which requires typed
//! failure stages. The stages are not a severity scale — they are positions in
//! the compile-and-run sequence, and what matters about a position is which
//! side of the commit point it falls on.
//!
//! # The rule this encodes
//!
//! `AGENTS.md` states it as a correctness priority: "preflight before routing
//! commit, fallback only before program work, and no fallback after allocation,
//! partial encoding, submission, or semantic validation failure."
//!
//! The reason fallback must stop is not tidiness. Once resources are allocated
//! or a command buffer is partly encoded, a fallback would have to reason about
//! what the abandoned attempt already did to device state, and that reasoning is
//! exactly the thing nobody can do for an *opaque* call — the compiler does not
//! model its body, so it cannot know what it touched before it failed. A
//! fallback there is not a slower correct path; it is a guess.
//!
//! So [`CallFailureStage::fallback_permitted`] is a property of the stage
//! rather than a policy a caller may override, and there is no stage at which a
//! caller may opt back in.

use core::fmt;

/// The point in the sequence at which an opaque call failed.
///
/// Ordered by position in the sequence, and the derived `Ord` is that order —
/// which is why the variants are declared in sequence order and must stay that
/// way. A later stage is not "worse"; it is simply further along, and further
/// along is what removes options.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[allow(
    dead_code,
    reason = "slice of implement-opaque-physical-call-providers: the failure vocabulary lands before the call sites that report it"
)]
pub(crate) enum CallFailureStage {
    /// The provider declined the target before any work was done.
    ///
    /// The cheapest failure and the only one that is entirely ordinary: a
    /// provider saying "not for this target" is not an error.
    Applicability,
    /// Preflight checks against the resolved target profile failed.
    Preflight,
    /// The call's declared ABI, effects, or resources did not validate.
    Validation,
    /// Program construction failed after the call was admitted.
    ProgramConstruction,
    /// Device resources had been allocated when the failure occurred.
    Allocation,
    /// A command buffer had been partly encoded.
    PartialEncoding,
    /// Work had been submitted to the device.
    Submission,
}

/// The last stage at which falling back to another implementation is legal.
///
/// Named rather than written inline at the comparison, so the boundary has one
/// definition and moving it is one edit that every check follows.
#[allow(
    dead_code,
    reason = "the named boundary the exhaustive match is checked against; the consumer that reads it is the not-yet-written opaque-call seam"
)]
const LAST_FALLBACK_STAGE: CallFailureStage = CallFailureStage::Validation;

#[allow(
    dead_code,
    reason = "see the module header: accessors land with the vocabulary, ahead of the call seam that reports through them"
)]
impl CallFailureStage {
    /// The governed canonical key naming this stage.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Applicability => "failure.applicability",
            Self::Preflight => "failure.preflight",
            Self::Validation => "failure.validation",
            Self::ProgramConstruction => "failure.program-construction",
            Self::Allocation => "failure.allocation",
            Self::PartialEncoding => "failure.partial-encoding",
            Self::Submission => "failure.submission",
        }
    }

    /// Whether another implementation may be tried after a failure here.
    ///
    /// True only at or before [`LAST_FALLBACK_STAGE`]. Written as an exhaustive
    /// match rather than a comparison so that inserting a stage forces a
    /// decision about which side of the boundary it belongs on, instead of
    /// inheriting an answer from where it happens to sit in the declaration
    /// order. The `Ord` derive makes that order meaningful, which is exactly why
    /// it must not silently decide this.
    pub(crate) const fn fallback_permitted(self) -> bool {
        match self {
            Self::Applicability | Self::Preflight | Self::Validation => true,
            Self::ProgramConstruction
            | Self::Allocation
            | Self::PartialEncoding
            | Self::Submission => false,
        }
    }

    /// Whether this failure is an ordinary outcome rather than a defect.
    ///
    /// Only a provider declining the target is ordinary. Everything else is
    /// something that should be explained, and treating a preflight rejection
    /// as routine is how an infeasible plan becomes a silent one.
    pub(crate) const fn is_ordinary(self) -> bool {
        match self {
            Self::Applicability => true,
            Self::Preflight
            | Self::Validation
            | Self::ProgramConstruction
            | Self::Allocation
            | Self::PartialEncoding
            | Self::Submission => false,
        }
    }
}

impl fmt::Display for CallFailureStage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.key())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [CallFailureStage; 7] = [
        CallFailureStage::Applicability,
        CallFailureStage::Preflight,
        CallFailureStage::Validation,
        CallFailureStage::ProgramConstruction,
        CallFailureStage::Allocation,
        CallFailureStage::PartialEncoding,
        CallFailureStage::Submission,
    ];

    /// The declaration order is the sequence order.
    ///
    /// `Ord` is derived from it and `LAST_FALLBACK_STAGE` is stated in terms of
    /// it, so a reordering that looked cosmetic would move the fallback
    /// boundary. This pins the order itself rather than trusting it.
    #[test]
    fn stages_are_declared_in_sequence_order() {
        let mut sorted = ALL;
        sorted.sort_unstable();
        assert_eq!(
            ALL, sorted,
            "the declaration order is not the sequence order"
        );
    }

    /// Fallback is permitted up to validation and never after.
    ///
    /// Asserted stage by stage against the rule `AGENTS.md` states, rather than
    /// against `LAST_FALLBACK_STAGE`, so a check that simply re-derived itself
    /// from the constant could not pass vacuously.
    #[test]
    fn fallback_stops_at_the_commit_point() {
        assert!(CallFailureStage::Applicability.fallback_permitted());
        assert!(CallFailureStage::Preflight.fallback_permitted());
        assert!(CallFailureStage::Validation.fallback_permitted());

        for committed in [
            CallFailureStage::ProgramConstruction,
            CallFailureStage::Allocation,
            CallFailureStage::PartialEncoding,
            CallFailureStage::Submission,
        ] {
            assert!(
                !committed.fallback_permitted(),
                "{committed} permitted a fallback after the commit point"
            );
        }
    }

    /// The permitted set is exactly the stages at or before the boundary.
    ///
    /// This is the consistency check between the exhaustive match and the named
    /// constant: they are written independently, and if they disagreed the
    /// match would silently win.
    #[test]
    fn the_match_and_the_named_boundary_agree() {
        for stage in ALL {
            assert_eq!(
                stage.fallback_permitted(),
                stage <= LAST_FALLBACK_STAGE,
                "{stage} disagrees with the named fallback boundary"
            );
        }
    }

    /// Only a declined target is an ordinary failure.
    #[test]
    fn only_applicability_is_an_ordinary_failure() {
        assert!(CallFailureStage::Applicability.is_ordinary());
        for defect in ALL.iter().skip(1) {
            assert!(
                !defect.is_ordinary(),
                "{defect} was reported as an ordinary outcome"
            );
        }
    }

    /// Stage keys are distinct.
    #[test]
    fn stage_keys_are_distinct() {
        let mut keys: Vec<&str> = ALL.iter().copied().map(CallFailureStage::key).collect();
        keys.sort_unstable();
        let before = keys.len();
        keys.dedup();
        assert_eq!(before, keys.len(), "two stages share a key");
    }
}
