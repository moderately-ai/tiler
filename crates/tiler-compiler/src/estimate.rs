//! Uncertain resource estimates, which can never establish hard feasibility.
//!
//! The first slice of `implement-opaque-physical-call-providers`. That ticket
//! requires three evidence classes to stay separate, and this module builds the
//! second of them:
//!
//! 1. exact or proven-upper-bound [`ResourceRequirements`], used for hard
//!    feasibility — already exists, in `tiler_ir::schedule`;
//! 2. **uncertain pressure estimates with provenance and an explicit `Unknown`
//!    state, including registers, occupancy, and source size** — this module;
//! 3. an analytical cost estimate with model provenance and its own `Unknown` —
//!    already exists, as [`crate::component_cost`].
//!
//! # The invariant, and why it is a type rather than a rule
//!
//! `AGENTS.md` requires hard feasibility to stay separate from estimated cost:
//! an infeasible plan is rejected with an explainable reason, never hidden
//! behind an infinite or arbitrary cost. The opaque-call ticket states the
//! sharp end of that — an unknown resource estimate cannot establish hard
//! feasibility.
//!
//! So there is deliberately **no conversion** from a [`ResourceEstimate`] into
//! anything feasibility consults. Not a fallible one, not a documented-unsafe
//! one. A `TryFrom` would put the decision at each call site, and the failure
//! mode is a caller who has an estimate, needs a requirement, and reaches for
//! the conversion that exists — which is exactly how an unproven number ends up
//! deciding whether a plan is legal. The absence is the enforcement.
//!
//! An estimate is for *ranking and reporting*. A requirement is for *deciding*.
//! A provider that wants its call admitted must state a requirement it can
//! prove, and an estimate never becomes one by being confident.

use core::fmt;

/// Where an estimate came from.
///
/// Provenance is not decoration: two estimates of the same quantity carrying
/// different provenance are different claims, and a later calibration pass has
/// to know which it is comparing a measurement against. A provider's own
/// assertion and a compiler-derived bound are not interchangeable even when
/// they agree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "first slice of implement-opaque-physical-call-providers: the estimate vocabulary lands before the providers that carry it"
)]
pub(crate) enum EstimateProvenance {
    /// The provider asserted it, and nothing here checked it.
    ///
    /// Explicitly *not* trusted for anything but reporting. This is the state
    /// the opaque-call ticket has in mind when it says unknown provider
    /// behaviour is never optimizable merely because it is registered.
    ProviderAsserted,
    /// Derived by the compiler from structure it verified itself.
    CompilerDerived,
    /// Measured on a device, under a named profile.
    ///
    /// Reserved: no measurement path reaches this module yet.
    /// `calibrate-device-cost-models` owns device measurement and activation.
    Measured,
}

#[allow(
    dead_code,
    reason = "see the module header: governed keys land with the vocabulary they name, ahead of the explain records that will report them"
)]
impl EstimateProvenance {
    /// The governed canonical key naming this provenance.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::ProviderAsserted => "provider-asserted",
            Self::CompilerDerived => "compiler-derived",
            Self::Measured => "measured",
        }
    }
}

impl fmt::Display for EstimateProvenance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.key())
    }
}

/// A resource dimension an estimate may speak about.
///
/// Closed, and the three the opaque-call ticket names. Registers and occupancy
/// are here rather than in `crate::component_cost` because they are *pressure*
/// rather than *cost*: they bear on whether a call fits, which is a different
/// question from what it costs, and the two must not share a vocabulary or the
/// separation this module exists to keep would be lost at the type level.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[allow(
    dead_code,
    reason = "see the module header: the dimensions land with the estimate type that carries them"
)]
pub(crate) enum PressureDimension {
    /// Registers per thread.
    Registers,
    /// Occupancy, as a percentage of the device maximum.
    Occupancy,
    /// Source size of the provider's call, in bytes.
    SourceBytes,
}

#[allow(
    dead_code,
    reason = "see the module header: governed keys land with the vocabulary they name, ahead of the explain records that will report them"
)]
impl PressureDimension {
    /// The governed canonical key naming this dimension.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Registers => "pressure.registers",
            Self::Occupancy => "pressure.occupancy",
            Self::SourceBytes => "pressure.source-bytes",
        }
    }

    /// Whether `value` is representable on this dimension.
    ///
    /// Occupancy is a percentage and cannot exceed 100; the other two are
    /// unbounded counts. A malformed value is refused at construction rather
    /// than reported as an extreme estimate, because an occupancy of 300 is a
    /// provider fault and reading it as pressure would rank a broken call
    /// above a working one.
    const fn admits(self, value: u64) -> bool {
        match self {
            Self::Occupancy => value <= 100,
            Self::Registers | Self::SourceBytes => true,
        }
    }
}

impl fmt::Display for PressureDimension {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.key())
    }
}

/// An uncertain estimate of one pressure dimension.
///
/// **This can never establish hard feasibility.** There is no conversion from
/// here into `ResourceRequirements` or any feasibility input, deliberately —
/// see the module header for why the absence is the enforcement rather than a
/// documented rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "see the module header: accessors land with the type, ahead of the providers that produce it"
)]
pub(crate) struct ResourceEstimate {
    dimension: PressureDimension,
    value: Option<u64>,
    provenance: EstimateProvenance,
}

#[allow(
    dead_code,
    reason = "see the type's own allow: reviewed draft accessors whose consumer is the not-yet-written provider seam"
)]
impl ResourceEstimate {
    /// An estimate carrying a value.
    ///
    /// Returns `None` if the dimension cannot represent the value, which is a
    /// provider fault rather than an extreme estimate.
    pub(crate) const fn known(
        dimension: PressureDimension,
        value: u64,
        provenance: EstimateProvenance,
    ) -> Option<Self> {
        if !dimension.admits(value) {
            return None;
        }
        Some(Self {
            dimension,
            value: Some(value),
            provenance,
        })
    }

    /// An estimate that declines to state a value.
    ///
    /// Explicit rather than absent, and it still carries provenance: "the
    /// provider was asked and does not know" and "nothing has asked yet" are
    /// different claims, and only the first says anything about the provider.
    pub(crate) const fn unknown(
        dimension: PressureDimension,
        provenance: EstimateProvenance,
    ) -> Self {
        Self {
            dimension,
            value: None,
            provenance,
        }
    }

    /// The dimension estimated.
    pub(crate) const fn dimension(&self) -> PressureDimension {
        self.dimension
    }

    /// The estimated value, if one was stated.
    ///
    /// `None` is `Unknown`. It is not zero, and a caller that substitutes zero
    /// would report a call as costing no registers and occupying nothing.
    pub(crate) const fn value(&self) -> Option<u64> {
        self.value
    }

    /// Where the estimate came from.
    pub(crate) const fn provenance(&self) -> EstimateProvenance {
        self.provenance
    }

    /// Whether this estimate states a value at all.
    pub(crate) const fn is_known(&self) -> bool {
        self.value.is_some()
    }
}

impl fmt::Display for ResourceEstimate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            Some(value) => write!(
                formatter,
                "{}={value} ({})",
                self.dimension, self.provenance
            ),
            None => write!(
                formatter,
                "{}=unknown ({})",
                self.dimension, self.provenance
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An occupancy above 100 percent is a provider fault, not a high estimate.
    ///
    /// Driven against an admitted value too, so a constructor that refused
    /// everything would fail here rather than pass.
    #[test]
    fn an_unrepresentable_value_is_refused() {
        assert!(
            ResourceEstimate::known(
                PressureDimension::Occupancy,
                100,
                EstimateProvenance::CompilerDerived
            )
            .is_some(),
            "full occupancy was refused"
        );
        assert!(
            ResourceEstimate::known(
                PressureDimension::Occupancy,
                101,
                EstimateProvenance::ProviderAsserted
            )
            .is_none(),
            "an occupancy above the device maximum was admitted as pressure"
        );
        assert!(
            ResourceEstimate::known(
                PressureDimension::Registers,
                1_000_000,
                EstimateProvenance::ProviderAsserted
            )
            .is_some(),
            "registers are an unbounded count and must not inherit occupancy's bound"
        );
    }

    /// `Unknown` is not zero, and still carries provenance.
    ///
    /// The zero substitution is the failure this class exists to prevent: it
    /// would report an opaque call as needing no registers at all.
    #[test]
    fn unknown_is_not_zero_and_keeps_its_provenance() {
        let unknown = ResourceEstimate::unknown(
            PressureDimension::Registers,
            EstimateProvenance::ProviderAsserted,
        );
        let zero = ResourceEstimate::known(
            PressureDimension::Registers,
            0,
            EstimateProvenance::ProviderAsserted,
        )
        .expect("zero registers is representable");

        assert!(!unknown.is_known());
        assert!(zero.is_known());
        assert_ne!(unknown, zero);
        assert_eq!(unknown.value(), None);
        assert_eq!(zero.value(), Some(0));
        assert_eq!(
            unknown.provenance(),
            EstimateProvenance::ProviderAsserted,
            "an unknown estimate still says who failed to state it"
        );
    }

    /// Provenance distinguishes otherwise identical estimates.
    ///
    /// A provider's assertion and a compiler-derived bound are different claims
    /// even at the same number, and a calibration pass must be able to tell
    /// which it is comparing a measurement against.
    #[test]
    fn identical_values_from_different_provenance_are_distinct() {
        let asserted = ResourceEstimate::known(
            PressureDimension::Registers,
            32,
            EstimateProvenance::ProviderAsserted,
        )
        .expect("representable");
        let derived = ResourceEstimate::known(
            PressureDimension::Registers,
            32,
            EstimateProvenance::CompilerDerived,
        )
        .expect("representable");

        assert_eq!(asserted.value(), derived.value());
        assert_ne!(
            asserted, derived,
            "an asserted estimate compared equal to a derived one"
        );
    }

    /// Dimension keys are distinct.
    #[test]
    fn pressure_dimension_keys_are_distinct() {
        let mut keys = [
            PressureDimension::Registers,
            PressureDimension::Occupancy,
            PressureDimension::SourceBytes,
        ]
        .map(PressureDimension::key);
        keys.sort_unstable();
        let before = keys.len();
        let mut seen = keys.to_vec();
        seen.dedup();
        assert_eq!(before, seen.len(), "two dimensions share a key");
    }
}
