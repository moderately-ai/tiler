//! The frontend's statement of its artifact-family delivery policy, and the
//! `#[cfg]`-gated tokens that deliver it.
//!
//! ADR 0049 requires every inline AOT compilation request to carry a canonical,
//! typed `ArtifactFamilySelection`, and requires that the proc macro not infer a
//! family from its host environment. This module is where this frontend states
//! one, reads it back, and turns the result of building it into generated Rust.
//! It states a policy and emits tokens; it does not discover Apple tools or read
//! the host, which belongs to [`tiler_metal_aot::driver`].
//!
//! There is exactly one canonical encoder for a selection and it lives in
//! [`tiler_metal_aot::family`]. Nothing here re-derives ordering, duplicate
//! rejection, per-family deployment minimums, language standards, or identity
//! bytes; [`ArtifactFamilySelection::new`] is the only way a value of that type
//! comes into being, on this side of the boundary as on the driver's. The
//! consumer-target predicate a family is gated by is [`crate::family_cfg`]'s.
//!
//! # What every region states today, and why it is `FallbackOnly`
//!
//! The approved region grammar has no syntax for naming an artifact family or a
//! [`NamedProfile`], so every invocation's tokens resolve to the same policy.
//! ADR 0053 makes that an explicit policy rather than an absence: "`FallbackOnly`
//! is an explicit valid policy and invokes no backend compiler". Saying it in the
//! type is what distinguishes it from a producer that assembled a selection and
//! forgot to put a family in it — which the driver rejects as `EmptySelection` —
//! so every expansion states `FallbackOnly` outright instead of leaving its
//! delivery unstated. [`DeliveryPlan::items_source`] emits nothing for it, so a
//! `FallbackOnly` expansion is token-for-token what it was before this module
//! could emit anything at all.
//!
//! Adding the syntax is a public-boundary decision rather than an omission here:
//! it publishes Apple family, deployment-minimum, and language-standard
//! vocabulary — or a profile name — on the consumer-facing region surface, which
//! ADR 0075 reserves to Tom. The machinery the syntax would drive is complete and
//! tested below; what is missing is only the way a consumer says which profile it
//! wants.
//!
//! # The delivery half
//!
//! [`DeliveryPlan`] is the pure function from *what the driver produced* to
//! *what the consumer compiles*. For each selected family it emits, under that
//! family's governed consumer-target `#[cfg]`, either the family's position in
//! the one embedded artifact or the retained toolchain diagnostic as a
//! `compile_error!`; and it emits the semantic fallback for every target
//! matching no selected family. Target-neutral failures never reach here: they
//! are [`crate::Refusal`]s and become unconditional `compile_error!`s at the
//! invocation span.
//!
//! **One envelope, N payloads** (Tom, 2026-07-25). A selection naming several
//! families produces one artifact carrying one payload per built family, so the
//! whole selection has one identity and a partial delivery is impossible by
//! construction. The consumer's `#[cfg]` therefore selects a payload *within* an
//! artifact it already holds, which is why the bytes are emitted once and
//! unconditionally while only the index is gated. The accepted cost is that a
//! consumer needing one family carries the bytes for all of them.
//!
//! # Why the selector is total
//!
//! Every emitted selector arm defines the same name, and the arms are a
//! partition: one arm per built family, plus a `not(any(…))` arm covering every
//! other target. Totality is what makes the two failure modes into build errors
//! in the consumer's own compilation rather than into wrong bytes — two families
//! whose predicates overlapped would define the name twice, and a gap would leave
//! it undefined. Neither can produce a silently wrong payload, which is the
//! outcome `docs/research/apple-targets/artifact-compatibility.md` forbids and
//! which nothing downstream would catch: an `air64-apple-ios16.0` metallib loads
//! and dispatches on the macOS host GPU without error.

use core::fmt;

use tiler_metal_aot::family::{
    ArtifactDeliveryPolicy, ArtifactFamilySelection, FamilyRequirement, FamilySelectionError,
    SelectedFamily,
};
use tiler_metal_aot::input::{ApplePlatform, DeploymentMinimum, MslVersion};

use crate::family_cfg::consumer_cfg;

/// The block-local name holding the one embedded artifact envelope.
///
/// Spelled like [`crate::REGION_FACTS_BINDING`] and for the same reason: block
/// scope stops it colliding with anything outside the expansion, and the spelling
/// stops it shadowing anything a consumer would write inside one.
const ARTIFACT_BINDING: &str = "__TILER_ARTIFACT";

/// The block-local name holding this consumer target's payload position, or
/// `None` when this target matches no built family and takes the fallback.
const SELECTED_PAYLOAD_BINDING: &str = "__TILER_SELECTED_PAYLOAD";

/// The Metal language standard the named profiles select.
///
/// MSL 3.1 is the standard `docs/research/apple-targets/` measured, and the
/// driver's own governed table is what fixes each profile's deployment minimum
/// to that standard's floor.
const PROFILE_MSL_VERSION: MslVersion = MslVersion::Metal3_1;

/// An ergonomic named delivery profile.
///
/// `docs/integration/frontends.md` permits one — "A frontend may offer an
/// ergonomic literal default profile, but the resolved selection is still
/// explicit compiler input" — and Q-ART-008 asks that named profiles expand to a
/// canonical [`ArtifactFamilySelection`]. That is exactly what [`Self::selection`]
/// does: a profile is a spelling, never a second encoder. Every deployment
/// minimum below is the governed floor for [`PROFILE_MSL_VERSION`] rather than a
/// number chosen here, so a profile excludes no OS version it could have
/// included and the driver would reject anything lower.
///
/// # Why Mac Catalyst is in no profile
///
/// The driver's governed table admits Mac Catalyst only at MSL 4.0, whose floor
/// is 26.0, and `docs/backends/metal.md` records that row as a bounded
/// compile-and-link measurement rather than a specification or runtime
/// qualification claim. A profile naming Catalyst would therefore have to raise
/// every other family in it to MSL 4.0 as well. A Catalyst consumer consequently
/// matches no profile's selected family and takes the semantic fallback, which is
/// the only correct outcome: `docs/backends/metal.md` forbids relabelling an
/// iOS-device or macOS payload as Catalyst-compatible.
#[allow(
    dead_code,
    reason = "the profiles are the resolved half of Q-ART-008 and are complete and tested. \
              Nothing constructs one during an expansion because the approved region grammar has \
              no syntax for stating a profile, and inventing that syntax is a consumer-visible \
              public boundary ADR 0075 reserves to Tom. The surface reserved is the profile names \
              and the exact selection each expands to."
)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NamedProfile {
    /// Build nothing; every consumer target uses the semantic fallback.
    FallbackOnly,
    /// macOS alone.
    MacOs,
    /// iOS device and the iOS simulator.
    ///
    /// Both, because a developer building for iOS builds for the simulator too,
    /// and a profile naming only the device would leave every simulator build
    /// silently on the fallback path.
    IOs,
    /// macOS, iOS device, and the iOS simulator.
    MacOsAndIOs,
}

#[allow(
    dead_code,
    reason = "see the reason on `NamedProfile` itself: the profiles resolve Q-ART-008 and are \
              reached only by their tests until a consumer can state one."
)]
impl NamedProfile {
    /// Every profile this frontend names.
    pub(crate) const ALL: [Self; 4] = [
        Self::FallbackOnly,
        Self::MacOs,
        Self::IOs,
        Self::MacOsAndIOs,
    ];

    /// Returns the stable name a consumer would state this profile by.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::FallbackOnly => "fallback-only",
            Self::MacOs => "macos",
            Self::IOs => "ios",
            Self::MacOsAndIOs => "macos-and-ios",
        }
    }

    /// Resolves a stated profile name, or nothing.
    ///
    /// Matched exactly, with no case folding and no prefixes: a name that is
    /// nearly a profile is a refusal rather than a guess, because guessing would
    /// pick which families a consumer's build compiles for.
    pub(crate) fn parse(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|profile| profile.as_str() == name)
    }

    /// Returns the delivery policy this profile spells.
    pub(crate) fn policy(self) -> ArtifactDeliveryPolicy {
        let families: Vec<SelectedFamily> = match self {
            Self::FallbackOnly => return ArtifactDeliveryPolicy::FallbackOnly,
            Self::MacOs => vec![selected(ApplePlatform::MacOs, 14, 0)],
            Self::IOs => vec![
                selected(ApplePlatform::IOsDevice, 17, 0),
                selected(ApplePlatform::IOsSimulator, 17, 0),
            ],
            Self::MacOsAndIOs => vec![
                selected(ApplePlatform::MacOs, 14, 0),
                selected(ApplePlatform::IOsDevice, 17, 0),
                selected(ApplePlatform::IOsSimulator, 17, 0),
            ],
        };
        ArtifactDeliveryPolicy::SelectedFamilies {
            families,
            requirement: FamilyRequirement::RequiredWhenTargetMatches,
        }
    }

    /// Expands this profile to the canonical selection it names.
    ///
    /// # Errors
    ///
    /// Returns whatever [`ArtifactFamilySelection::new`] returns. Every profile
    /// is a valid selection, so this is a propagated impossibility rather than a
    /// reachable failure — and it is propagated rather than unwrapped because the
    /// governed floors that make it impossible live in the driver, not here.
    pub(crate) fn selection(self) -> Result<ArtifactFamilySelection, FamilySelectionError> {
        ArtifactFamilySelection::new(self.policy())
    }
}

/// Builds one selected family at [`PROFILE_MSL_VERSION`].
fn selected(family: ApplePlatform, major: u16, minor: u16) -> SelectedFamily {
    SelectedFamily {
        family,
        deployment_minimum: DeploymentMinimum::new(major, minor),
        msl_version: PROFILE_MSL_VERSION,
    }
}

/// What the offline driver produced for one selected family.
///
/// Two outcomes and no third: ADR 0053 gives a matching consumer target either
/// the family's artifact or a compile error, never a quiet fallback. A variant
/// meaning "not attempted" would be the quiet fallback wearing a name.
#[allow(
    dead_code,
    reason = "an outcome is constructed by whoever ran the driver, and no expansion runs it yet: \
              every expansion states `FallbackOnly`, which ADR 0053 defines as invoking no \
              backend compiler, so every production plan is empty. The emission both variants \
              drive is complete and is what the slice that first compiles a selected family \
              consumes."
)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FamilyDelivery {
    /// The family built. Its payload is carried by the plan's one artifact, at
    /// the position implied by canonical family order.
    Payload,
    /// The family did not build, and this is the driver's retained diagnostic.
    Retained(String),
}

/// One selection, what the driver produced for it, and the tokens that deliver
/// it.
///
/// A verified product: its fields are private and it is reachable only through
/// [`DeliveryPlan::new`], which rejects a plan whose outcomes do not correspond
/// one-to-one with the selection's families and a plan whose artifact and
/// outcomes disagree about whether anything was built.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeliveryPlan {
    selection: ArtifactFamilySelection,
    artifact: Vec<u8>,
    deliveries: Vec<FamilyDelivery>,
}

impl DeliveryPlan {
    /// Validates one selection and its per-family outcomes into a plan.
    ///
    /// `deliveries` is positional against
    /// [`ArtifactFamilySelection::families`] — canonical family order, one entry
    /// each — rather than a map keyed by family. The selection already
    /// canonicalized that order and rejected duplicates, so a second keyed
    /// vocabulary here would be a second chance to disagree with it.
    ///
    /// # Errors
    ///
    /// Returns [`PlanRefusal::OutcomeCountMismatch`] when the outcomes do not
    /// cover the selection exactly, [`PlanRefusal::ArtifactMissing`] when a
    /// family built but no artifact carries it, and
    /// [`PlanRefusal::ArtifactUnused`] when an artifact is supplied although no
    /// family built.
    pub(crate) fn new(
        selection: ArtifactFamilySelection,
        artifact: Vec<u8>,
        deliveries: Vec<FamilyDelivery>,
    ) -> Result<Self, PlanRefusal> {
        let selected = selection.families().len();
        if deliveries.len() != selected {
            return Err(PlanRefusal::OutcomeCountMismatch {
                selected,
                supplied: deliveries.len(),
            });
        }
        let built = deliveries
            .iter()
            .filter(|delivery| matches!(delivery, FamilyDelivery::Payload))
            .count();
        if built > 0 && artifact.is_empty() {
            return Err(PlanRefusal::ArtifactMissing { built });
        }
        if built == 0 && !artifact.is_empty() {
            return Err(PlanRefusal::ArtifactUnused {
                bytes: artifact.len(),
            });
        }
        Ok(Self {
            selection,
            artifact,
            deliveries,
        })
    }

    /// Returns the Rust items this plan contributes to an expansion's block.
    ///
    /// Empty for `FallbackOnly`, which is what keeps a stated no-AOT expansion
    /// token-for-token unchanged.
    ///
    /// The order is fixed: retained diagnostics first, then the artifact and its
    /// selector. Leading with the diagnostics is what puts the actionable error
    /// first when a consumer's target matched a family that failed to build.
    pub(crate) fn items_source(&self) -> String {
        let families = self.selection.families();
        let mut items: Vec<String> = families
            .iter()
            .zip(&self.deliveries)
            .filter_map(|(selected, delivery)| match delivery {
                FamilyDelivery::Payload => None,
                // `{diagnostic:?}` is Rust's own string-literal escaping, so a
                // driver message containing a quote or a backslash cannot close
                // the literal early and turn a diagnostic into source.
                FamilyDelivery::Retained(diagnostic) => Some(format!(
                    "#[cfg({})]\nconst _: () = {{ ::core::compile_error!({diagnostic:?}); }};",
                    consumer_cfg(selected.family).predicate(),
                )),
            })
            .collect();

        let built: Vec<String> = families
            .iter()
            .zip(&self.deliveries)
            .filter(|(_, delivery)| matches!(delivery, FamilyDelivery::Payload))
            .map(|(selected, _)| consumer_cfg(selected.family).predicate())
            .collect();

        if !built.is_empty() {
            items.push(format!(
                "const {ARTIFACT_BINDING}: &[u8] = {};",
                byte_string_literal(&self.artifact),
            ));
            items.extend(built.iter().enumerate().map(|(position, predicate)| {
                format!(
                    "#[cfg({predicate})]\nconst {SELECTED_PAYLOAD_BINDING}: \
                     ::core::option::Option<usize> = \
                     ::core::option::Option::Some({position}usize);",
                )
            }));
            // The one arm that makes the selector total. A retained family's
            // predicate is deliberately absent from `any(…)`, so its target
            // lands here with a well-formed `None` *and* the `compile_error!`
            // above: one actionable diagnostic instead of that error plus an
            // undefined name. Nothing reaches the fallback on such a target,
            // because the build fails.
            items.push(format!(
                "#[cfg(not(any({})))]\nconst {SELECTED_PAYLOAD_BINDING}: \
                 ::core::option::Option<usize> = ::core::option::Option::None;",
                built.join(", "),
            ));
        }

        if items.is_empty() {
            return String::new();
        }
        let mut source = items.join("\n");
        source.push('\n');
        source
    }
}

/// Why one selection and its outcomes are not a deliverable plan.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a rejection vocabulary a
/// caller forwards rather than maps totally.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub(crate) enum PlanRefusal {
    /// The outcomes do not cover the selected families exactly.
    OutcomeCountMismatch {
        /// How many families the selection names.
        selected: usize,
        /// How many outcomes were supplied.
        supplied: usize,
    },
    /// A family built, but no artifact carries its payload.
    ArtifactMissing {
        /// How many families built.
        built: usize,
    },
    /// An artifact was supplied although no family built.
    ArtifactUnused {
        /// How many bytes were supplied.
        bytes: usize,
    },
}

impl fmt::Display for PlanRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutcomeCountMismatch { selected, supplied } => write!(
                formatter,
                "the artifact-family selection names {selected} families but {supplied} \
                 compilation outcomes were supplied; a plan must cover every selected family, \
                 because a family with no outcome is a family whose consumer target would compile \
                 the fallback it was owed an artifact for"
            ),
            Self::ArtifactMissing { built } => write!(
                formatter,
                "{built} artifact families built but no artifact carries their payloads; one \
                 selection produces one artifact carrying one payload per built family"
            ),
            Self::ArtifactUnused { bytes } => write!(
                formatter,
                "an artifact of {bytes} bytes was supplied although no artifact family built; \
                 embedding bytes no consumer target can select would ship an artifact with no \
                 payload reachable from any `#[cfg]`"
            ),
        }
    }
}

/// Renders bytes as a Rust byte-string literal.
///
/// Printable ASCII passes through so a reader can recognize a text payload, and
/// everything else becomes `\xNN`. `"` and `\` are escaped explicitly rather
/// than left to the printable range, which is exactly where a hand-written
/// escaper truncates a literal and produces source the macro cannot lex.
///
/// The hex digits are written out rather than formatted because this runs once
/// per byte of an embedded artifact, and `docs/correctness-and-testing.md`
/// measures bundles up to a megabyte: one allocation for the whole literal
/// instead of one per escaped byte.
fn byte_string_literal(bytes: &[u8]) -> String {
    const HEX: [u8; 16] = *b"0123456789abcdef";

    let mut literal = String::with_capacity(bytes.len() + 3);
    literal.push_str("b\"");
    for &byte in bytes {
        match byte {
            b'"' => literal.push_str("\\\""),
            b'\\' => literal.push_str("\\\\"),
            0x20..=0x7e => literal.push(char::from(byte)),
            _ => {
                literal.push_str("\\x");
                literal.push(char::from(HEX[usize::from(byte >> 4)]));
                literal.push(char::from(HEX[usize::from(byte & 0x0f)]));
            }
        }
    }
    literal.push('"');
    literal
}

/// The delivery policy an invocation's tokens resolve to.
///
/// Nullary rather than a function of the parsed region, and that is the honest
/// signature: the approved grammar admits neither a family statement nor a
/// [`NamedProfile`] name, so every region resolves to
/// [`ArtifactDeliveryPolicy::FallbackOnly`] and a parameter it ignored would
/// claim a dependence that does not exist. It becomes a function of the region
/// when Tom accepts the syntax a consumer would state a profile in.
pub(crate) const fn stated_policy() -> ArtifactDeliveryPolicy {
    ArtifactDeliveryPolicy::FallbackOnly
}

/// Why this expansion cannot deliver a stated policy.
///
/// Typed and non-erasing (ADR 0074 convention 1): the driver's own rejection is
/// carried rather than flattened into a message, so a caller can still tell an
/// empty selection from a duplicate family from an ungoverned target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DeliveryRefusal {
    /// The stated policy is not a valid artifact-family selection.
    InvalidSelection(FamilySelectionError),
    /// The selection is valid and requires backend compilation, which this
    /// expansion cannot perform yet.
    ///
    /// Refusing is the fail-closed half of ADR 0053: a selected family is
    /// *required* when the consumer target matches it, so an expansion that
    /// emitted its fallback anyway would silently turn a required artifact into
    /// fallback on exactly the target that was owed one.
    BackendCompilationUnavailable {
        /// The selected families' stable identifiers, in canonical order.
        families: Vec<&'static str>,
    },
    /// The selection and the outcomes claimed for it do not form a plan.
    MalformedPlan(PlanRefusal),
}

impl fmt::Display for DeliveryRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSelection(source) => write!(
                formatter,
                "`tiler::tensor!` cannot state its artifact-family delivery policy: {source}"
            ),
            Self::BackendCompilationUnavailable { families } => write!(
                formatter,
                "`tiler::tensor!` states an artifact-family selection naming {}, but this \
                 expansion performs no backend compilation yet and a selected family must not \
                 silently become fallback on a matching target; the family syntax that would \
                 state one is a public boundary, and the `#[cfg]`-gated delivery it drives is \
                 already implemented",
                families.join(", "),
            ),
            Self::MalformedPlan(source) => write!(
                formatter,
                "`tiler::tensor!` cannot deliver what it compiled; this is a defect in \
                 `tiler-macros`, not in the invocation: {source}"
            ),
        }
    }
}

/// Validates one stated policy into the canonical selection this expansion
/// delivers.
///
/// # Errors
///
/// Returns [`DeliveryRefusal::InvalidSelection`] when the policy is not a valid
/// selection, and [`DeliveryRefusal::BackendCompilationUnavailable`] when it is
/// valid but names families this expansion cannot yet build.
pub(crate) fn stated_delivery(
    policy: ArtifactDeliveryPolicy,
) -> Result<ArtifactFamilySelection, DeliveryRefusal> {
    let selection =
        ArtifactFamilySelection::new(policy).map_err(DeliveryRefusal::InvalidSelection)?;
    if selection.invokes_backend_compiler() {
        return Err(DeliveryRefusal::BackendCompilationUnavailable {
            families: selection
                .families()
                .iter()
                .map(|selected| selected.family.as_str())
                .collect(),
        });
    }
    Ok(selection)
}

/// Builds the plan one validated selection delivers.
///
/// # Errors
///
/// Returns [`DeliveryRefusal::MalformedPlan`] when the selection and outcomes do
/// not correspond. [`stated_delivery`] admits only `FallbackOnly`, whose plan
/// names no family and carries no artifact, so today this is a propagated
/// impossibility rather than a reachable failure.
pub(crate) fn stated_plan(
    selection: ArtifactFamilySelection,
) -> Result<DeliveryPlan, DeliveryRefusal> {
    DeliveryPlan::new(selection, Vec::new(), Vec::new()).map_err(DeliveryRefusal::MalformedPlan)
}

#[cfg(test)]
mod tests;
