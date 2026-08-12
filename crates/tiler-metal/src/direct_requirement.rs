//! Whether a bound device satisfies a *derived* requirement of a routed entry.
//!
//! A derived requirement is one the verified program already states, so no
//! artifact row carries it and no producer can restate it: the routed entry's
//! [`ResourceRequirements`](tiler_ir::schedule::ResourceRequirements) record is
//! the single authority, and this module is the single place that translates one
//! of its neutral statements into Apple's live-device vocabulary. The neutral
//! layers never learn what an Apple family is, and this layer never reads a
//! target profile, an artifact, or a device.
//!
//! # Why this is not a route requirement
//!
//! [`crate::applicability`]'s sibling contract,
//! `crates/tiler-artifact/src/program/requirement.rs`, states the test a row must
//! pass to belong in the backend-feature family: it must be *consumed by the
//! selected route* **and** *not already derivable from its verified program*.
//! Index arithmetic fails the second half. Every scheduled region derives it
//! from its own coordinate space, so copying it into a backend-feature row would
//! mint a second, independently editable statement about one KIR fact, and two
//! statements can disagree about which arithmetic a program needs. The
//! comparison therefore reads the dispatch record directly, exactly as an
//! adapter compares an entry's `local_memory_bytes` against
//! `maxThreadgroupMemoryLength` rather than looking for a row about it.
//!
//! # The authority for the threshold
//!
//! Apple's Metal Feature Set Tables (2025-10-20), "GPU family 1" table, row
//! `64-bit integer math`, reads `Metal3 | Apple3 | —`: the minimum Metal
//! version, the minimum Apple family, and the minimum Mac family, the last of
//! which the row leaves empty.
//! `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md`
//! holds that reading and labels the one inference it rests on — that a row named
//! "64-bit integer math", enumerating no operation subset, is *operation
//! complete* over the governed index family.
//!
//! Two consequences follow, and together they are why this check exists. A macOS
//! artifact family cannot discharge the requirement, because the row states no
//! Mac-family minimum. And MSL 4.0's ability to *spell* `uint64_t` cannot
//! discharge it either, because a spellable type is a language fact about the
//! compiler while this is a capability fact about the device;
//! `separate-metal-launch-index-from-index-and-address-width` eliminated that
//! inference by name.
//!
//! # The threshold sits below the observation vocabulary, and that is stated
//!
//! [`MetalGpuFamily`](crate::applicability::MetalGpuFamily) names `Apple5` through
//! `Apple9`, and the sourced threshold
//! is `Apple3`. The two do not meet, so the threshold is **not** representable as
//! a member of that vocabulary and this module does not pretend otherwise:
//! [`AppleFamilyFloor`](crate::direct_requirement::AppleFamilyFloor) is its own type,
//! and the gap it creates is carried in the
//! outcome rather than closed by assumption.
//!
//! What follows is decidable, and what does not is refused as `Unknown`:
//!
//! - A device reporting **any** named family reports `Apple5` or newer, which is
//!   above the floor, so the requirement is satisfied. This is proved by the
//!   const assertion below rather than by reading the enum declaration.
//! - A device reporting
//!   [`MetalGpuFamilySupport::NoneNamed`](crate::applicability::MetalGpuFamilySupport::NoneNamed)
//!   supports none of
//!   `Apple5`–`Apple9`. That is consistent with an `Apple3` device that satisfies
//!   the requirement *and* with an `Apple1` device that does not, so the
//!   predicate is genuinely `Unknown` — not false — and ADR 0043's disposal of
//!   `Unknown` refuses it. Reporting it as an unsupported device would claim a
//!   fact about hardware nobody observed.
//!
//! Widening [`MetalGpuFamily`](crate::applicability::MetalGpuFamily) down to the
//! threshold would turn that `Unknown`
//! into a decided refusal. That is a change to a public vocabulary two other
//! crates classify exhaustively, so it is not made here; the gap is named in
//! [`MetalIndexArithmeticRefusal::UndecidableBelowVocabulary`](crate::direct_requirement::MetalIndexArithmeticRefusal::UndecidableBelowVocabulary)
//! so a reader finds
//! it at the outcome rather than by inference.
//!
//! # Draft boundary
//!
//! Every public item here is a reviewed *draft* boundary under ADR 0074 §7 and
//! ADR 0075, prepared under the tested-draft authorization recorded in
//! `tickets/carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit.md`.
//! Its exact surface returns to Tom for acceptance before it is treated as
//! accepted.

use core::fmt;
use std::error::Error;

use tiler_ir::schedule::IndexArithmetic;

use crate::applicability::{MetalGpuFamily, MetalGpuFamilySupport};

/// The minimum Apple GPU family one governed index arithmetic requires.
///
/// **Deliberately not a [`MetalGpuFamily`].** That vocabulary names the families
/// a device is *observed* to support and begins at `Apple5`; this names the
/// family a capability *requires* and its only value is below that floor. They
/// are different questions with different evidence — one is a device answer, the
/// other is a row in Apple's feature tables — and spelling the floor as an
/// observation type would have required either widening the observation
/// vocabulary to hold a value no probe here produces, or rounding the sourced
/// `Apple3` up to `Apple5` and refusing devices the table admits.
///
/// **An ADR 0074 convention 5b type, deliberately exhaustive.** A variant added
/// here must state its own comparison against an observation in
/// `evaluate_against`, because a floor at or above the observation vocabulary
/// is decided differently from one below it, and a wildcard could only reuse
/// whichever comparison the first floor happened to need.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AppleFamilyFloor {
    /// `MTLGPUFamilyApple3`, as the `64-bit integer math` row names it.
    Apple3,
}

impl AppleFamilyFloor {
    /// Returns the raw `MTLGPUFamily` enumerator Apple declares for this floor.
    ///
    /// Transcribed from `MTLDevice.h`
    /// (`$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLDevice.h`),
    /// which declares `MTLGPUFamilyApple1 = 1001` through
    /// `MTLGPUFamilyApple10 = 1010`; `MTLGPUFamilyApple3 = 1003` is line 235 of
    /// the macOS 27.0 SDK copy on this host. The raw value rather than an
    /// [`AppleGpuFamilyConstant`](crate::applicability::AppleGpuFamilyConstant)
    /// because that type is opaque by construction — its field is private
    /// precisely so nothing can mint an enumerator for a family
    /// [`MetalGpuFamily`] does not name, and this floor is exactly such a family.
    #[must_use]
    pub const fn apple_constant_value(self) -> isize {
        match self {
            Self::Apple3 => 1003,
        }
    }
}

impl fmt::Display for AppleFamilyFloor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Apple3 => formatter.write_str("Apple3"),
        }
    }
}

/// The minimum Apple family that carries one governed index arithmetic.
///
/// A total map rather than one constant, and exhaustive with no wildcard: a
/// variant added to [`IndexArithmetic`] must state the floor that carries it
/// here, in the crate that owns Apple's vocabulary, rather than inherit
/// whichever threshold the first arithmetic happened to need. Answering at all
/// for an unrecognized arithmetic is the silently-wrong fast path this
/// vocabulary exists to prevent.
#[must_use]
pub const fn minimum_gpu_family(index_arithmetic: IndexArithmetic) -> AppleFamilyFloor {
    match index_arithmetic {
        IndexArithmetic::CompleteU64 => AppleFamilyFloor::Apple3,
    }
}

/// Compiles only while every observable family is above every requirable floor.
///
/// [`evaluate_against`] answers `Ok` for *any* named family without comparing
/// it, and this is why that is sound rather than convenient: the lowest family
/// [`MetalGpuFamily`] can report is strictly above the highest floor
/// [`minimum_gpu_family`] can return, so the comparison has one answer for the
/// whole vocabulary. Written as a `const` assertion over `MetalGpuFamily::ALL`
/// rather than as a test, because the property is what makes the `Ok` arm
/// correct — widening the vocabulary downward, or raising a floor above
/// `Apple5`, must stop the build here and send whoever did it to the arm that
/// then has to compare.
///
/// It reads `ALL[0]` rather than a named variant so that widening the vocabulary
/// downward is caught even though `ALL`'s own assertion already fixes its order.
const _: () = {
    let lowest_observable = MetalGpuFamily::ALL[0].apple_constant().value();
    assert!(
        lowest_observable > AppleFamilyFloor::Apple3.apple_constant_value(),
        "every family MetalGpuFamily can observe must be strictly above every floor \
         minimum_gpu_family can require; if that stops holding, the named-family arm of \
         evaluate_against has to compare rather than admit",
    );
};

/// Why a bound device does not satisfy an entry's derived index arithmetic.
///
/// One variant per repair, not one carrying a tag, because a caller acts on each
/// differently: an undecidable observation is a vocabulary to widen or a device
/// to change, and an unobserved family is an adapter to fix.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a later derived requirement,
/// or a decided below-floor refusal once the vocabulary reaches the floor, lands
/// additively, and no consumer outside this crate classifies this by exhaustive
/// match.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum MetalIndexArithmeticRefusal {
    /// The device named no family this vocabulary knows, so nothing is decided.
    ///
    /// **`Unknown`, and refused as one.** The device supports none of
    /// `Apple5`–`Apple9`; the requirement's floor is `Apple3`; and no observation
    /// separates a device between the two — which satisfies the requirement —
    /// from one below it, which does not. ADR 0043's disposal of `Unknown` keeps
    /// such a candidate out of an executable frontier, and that is what this is.
    ///
    /// It is deliberately **not** reported as an unsupported device. That would
    /// be a claim about hardware nobody observed, and it would go stale silently
    /// the day the vocabulary reaches the floor and the same observation becomes
    /// decidable.
    UndecidableBelowVocabulary {
        /// The arithmetic the routed entry's dispatch record requires.
        required: IndexArithmetic,
        /// The lowest Apple family that carries it.
        floor: AppleFamilyFloor,
        /// The lowest family the observation vocabulary can report.
        lowest_observable: MetalGpuFamily,
    },
    /// Nothing observed the device's Apple family, so nothing was compared.
    ///
    /// A gap in the adapter rather than a fact about the device, which is why it
    /// is told apart from [`Self::UndecidableBelowVocabulary`]: the repair is to
    /// ask the device, not to widen a vocabulary or change hardware. It refuses
    /// too, because an unasked question is not a satisfied requirement.
    Unobserved {
        /// The arithmetic the routed entry's dispatch record requires.
        required: IndexArithmetic,
        /// The lowest Apple family that carries it.
        floor: AppleFamilyFloor,
    },
}

impl MetalIndexArithmeticRefusal {
    /// Returns the stable rule identifier for this refusal.
    ///
    /// Two rules over two variants, and they stay separate because a caller
    /// routing on the text needs the `Unknown` disposal told apart from a gap in
    /// its own adapter.
    #[must_use]
    pub const fn rule(&self) -> &'static str {
        match self {
            Self::UndecidableBelowVocabulary { .. } => {
                "metal.index-arithmetic.undecidable-below-vocabulary"
            }
            Self::Unobserved { .. } => "metal.index-arithmetic.unobserved-family",
        }
    }

    /// Returns the index arithmetic this refusal names.
    #[must_use]
    pub const fn required(&self) -> IndexArithmetic {
        match self {
            Self::UndecidableBelowVocabulary { required, .. }
            | Self::Unobserved { required, .. } => *required,
        }
    }

    /// Returns the Apple family floor this refusal could not establish.
    #[must_use]
    pub const fn floor(&self) -> AppleFamilyFloor {
        match self {
            Self::UndecidableBelowVocabulary { floor, .. } | Self::Unobserved { floor, .. } => {
                *floor
            }
        }
    }
}

impl fmt::Display for MetalIndexArithmeticRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UndecidableBelowVocabulary {
                required,
                floor,
                lowest_observable,
            } => write!(
                formatter,
                "{}: index arithmetic {required:?} requires {floor} or newer, this device reports \
                 no named Apple family, and the observation vocabulary starts at \
                 {lowest_observable} — so whether this device reaches {floor} is unknown rather \
                 than false",
                self.rule(),
            ),
            Self::Unobserved { required, floor } => write!(
                formatter,
                "{}: index arithmetic {required:?} requires {floor} or newer and nothing observed \
                 this device's Apple family",
                self.rule(),
            ),
        }
    }
}

impl Error for MetalIndexArithmeticRefusal {}

/// Decides whether one observed device carries one entry's index arithmetic.
///
/// Deterministic and pure: nothing here reads a device, a process, an
/// environment variable, or an artifact. The observation is the caller's, made
/// through [`try_observe_highest_gpu_family`](crate::applicability::try_observe_highest_gpu_family)
/// so the families probed and the family named stay one authority.
///
/// `None` is a device nobody asked, which refuses under
/// [`MetalIndexArithmeticRefusal::Unobserved`] rather than reading as an absent
/// requirement.
///
/// # Errors
///
/// Returns the refusal naming the unestablished floor. On both measured devices
/// reachable from this workspace the answer is `Ok(MetalGpuFamily::Apple9)`.
///
/// ```
/// use tiler_ir::schedule::IndexArithmetic;
/// use tiler_metal::applicability::{MetalGpuFamily, MetalGpuFamilySupport};
/// use tiler_metal::direct_requirement::{
///     AppleFamilyFloor, MetalIndexArithmeticRefusal, evaluate_index_arithmetic,
///     minimum_gpu_family,
/// };
///
/// let required = IndexArithmetic::CompleteU64;
/// assert_eq!(minimum_gpu_family(required), AppleFamilyFloor::Apple3);
///
/// // Every family the vocabulary can report is above the sourced floor.
/// for observed in MetalGpuFamily::ALL {
///     assert_eq!(
///         evaluate_index_arithmetic(required, Some(MetalGpuFamilySupport::Highest(observed))),
///         Ok(observed),
///     );
/// }
///
/// // A device naming none of them is undecidable against an Apple3 floor, and
/// // refused as such rather than reported unsupported.
/// assert_eq!(
///     evaluate_index_arithmetic(required, Some(MetalGpuFamilySupport::NoneNamed)),
///     Err(MetalIndexArithmeticRefusal::UndecidableBelowVocabulary {
///         required,
///         floor: AppleFamilyFloor::Apple3,
///         lowest_observable: MetalGpuFamily::Apple5,
///     }),
/// );
///
/// // A device nobody asked is a different refusal from a device that answered.
/// assert_eq!(
///     evaluate_index_arithmetic(required, None),
///     Err(MetalIndexArithmeticRefusal::Unobserved {
///         required,
///         floor: AppleFamilyFloor::Apple3,
///     }),
/// );
/// ```
pub fn evaluate_index_arithmetic(
    required: IndexArithmetic,
    observed: Option<MetalGpuFamilySupport>,
) -> Result<MetalGpuFamily, MetalIndexArithmeticRefusal> {
    evaluate_against(required, minimum_gpu_family(required), observed)
}

/// Decides one observation against an explicitly supplied floor.
///
/// Split from [`evaluate_index_arithmetic`] so the floor is an *input* rather
/// than a constant the comparison reads for itself, which is what lets a test
/// drive a floor the governed map does not currently produce. It is
/// `pub(crate)` rather than public because a caller choosing its own floor would
/// be a second authority over what an arithmetic requires — exactly the
/// duplication [`minimum_gpu_family`] exists to prevent.
///
/// The named-family arm admits without comparing, and the `const` assertion
/// above is what makes that sound: every family the vocabulary reports is
/// strictly above every floor this signature admits.
pub(crate) fn evaluate_against(
    required: IndexArithmetic,
    floor: AppleFamilyFloor,
    observed: Option<MetalGpuFamilySupport>,
) -> Result<MetalGpuFamily, MetalIndexArithmeticRefusal> {
    match observed {
        None => Err(MetalIndexArithmeticRefusal::Unobserved { required, floor }),
        Some(MetalGpuFamilySupport::NoneNamed) => {
            Err(MetalIndexArithmeticRefusal::UndecidableBelowVocabulary {
                required,
                floor,
                lowest_observable: MetalGpuFamily::ALL[0],
            })
        }
        Some(MetalGpuFamilySupport::Highest(highest)) => Ok(highest),
    }
}
