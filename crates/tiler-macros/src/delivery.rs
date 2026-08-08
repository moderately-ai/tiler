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
//! # What a region states, and what it gets today
//!
//! Tom accepted the consumer-visible spelling on 2026-07-31 under
//! `accept-the-inline-artifact-family-profile-syntax`: a `deliver` statement in
//! the region's declaration block, naming either a [`NamedProfile`] or a list of
//! artifact families with their own deployment minimums. [`stated_policy`] is
//! where those tokens become an [`ArtifactDeliveryPolicy`], and it is the only
//! place either vocabulary is resolved.
//!
//! A region stating none resolves to `FallbackOnly`. ADR 0053 makes that an
//! explicit policy rather than an absence: "`FallbackOnly` is an explicit valid
//! policy and invokes no backend compiler". Saying it in the type is what
//! distinguishes it from a producer that assembled a selection and forgot to put
//! a family in it — which the driver rejects as `EmptySelection`. And
//! [`DeliveryPlan::items_source`] emits nothing for it, so a region with no
//! `deliver` statement is token-for-token what it was before this module could
//! emit anything at all.
//!
//! A region stating a selected family is *built*, by [`crate::aot`], which runs
//! the offline Metal driver at expansion time and returns the plan this module's
//! emission half turns into tokens. What it cannot build it refuses there, at
//! the `deliver` keyword, with a reason that is a fact about the one bound
//! compile declaration — never by emitting the semantic fallback anyway, which
//! is the one thing ADR 0053 forbids outright: a selected family is *required*
//! on a matching consumer target, and a quiet fallback there is a wrong answer
//! with no diagnostic.
//!
//! # The delivery half
//!
//! [`DeliveryPlan`] is the pure function from *what the driver produced* to
//! *what the consumer compiles*. For each selected family it emits, under that
//! family's governed consumer-target `#[cfg]`, either the family's position in
//! the one embedded artifact or the retained toolchain diagnostic through the
//! facade-owned diagnostic macro; and it emits the semantic fallback for every target
//! matching no selected family. Target-neutral failures never reach here: they
//! are [`crate::Refusal`]s and become unconditional `compile_error!`s at the
//! invocation span through the same facade-owned builtin re-export.
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
use tiler_metal_aot::input::{
    ApplePlatform, DeploymentMinimum, MetalTarget, MetalTargetError, MslVersion,
};

use crate::family_cfg::consumer_cfg;
use crate::grammar::{DeliverySyntax, FamilyMinimumSyntax, StatedDelivery};

/// The block-local name holding the one embedded artifact envelope.
///
/// Spelled like [`crate::REGION_FACTS_BINDING`] and for the same reason: block
/// scope stops it colliding with anything outside the expansion, and the spelling
/// stops it shadowing anything a consumer would write inside one.
pub(crate) const ARTIFACT_BINDING: &str = "__TILER_ARTIFACT";

/// The block-local name holding this consumer target's payload position, or
/// `None` when this target matches no built family and takes the fallback.
pub(crate) const SELECTED_PAYLOAD_BINDING: &str = "__TILER_SELECTED_PAYLOAD";

/// The Metal language standard a stated policy selects.
///
/// This is *"the Metal language standard Tiler compiles with"*, which is what
/// the accepted spelling's own definition of a profile is written in terms of,
/// and it is MSL 4.0 because that is the standard the one authoritative
/// compile-time declaration measures: `BoundMetalCompileDeclaration`'s ledger
/// row is `air64-apple-macos26.0` at `-std=metal4.0`, and its manifest records
/// that "the older MSL 3.1 / macOS 14.0 record would attribute these
/// measurements to a compilation that did not produce them".
///
/// It read `Metal3_1` while nothing compiled, which was harmless exactly
/// because nothing compiled: a stated policy that never reached the driver
/// could not deliver a consumer an artifact built for a standard other than the
/// one it named. Now that [`crate::aot`] compiles one, the two must agree or a
/// consumer stating `deliver macos;` would receive a payload requiring macOS
/// 26.0 under a policy that promised 14.0. The observable consequence — every
/// profile's governed floor moves to 26.0, and a family list stating a lower
/// one is refused at the version that stated it — is real and is listed in this
/// slice's boundary packet.
const PROFILE_MSL_VERSION: MslVersion = MslVersion::Metal4_0;

/// An artifact family a consumer may name, and the driver families it covers.
///
/// This is the vocabulary Tom accepted on the consumer-facing surface, and it is
/// deliberately *not* [`ApplePlatform`]: `ios` covers the iOS device and the iOS
/// simulator together, because a developer building for iOS builds for the
/// simulator too and a name covering only the device would leave every simulator
/// build silently on the fallback path. Publishing `ios-device` and
/// `ios-simulator` instead would put two driver identifiers on the region
/// surface to express the case that always wants both.
///
/// It is one vocabulary rather than two: [`NamedProfile`] names its families
/// through these values, so `deliver ios;` and `deliver ios 26.0;` cannot come
/// to disagree about which families `ios` means.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DeliveredFamily {
    /// macOS.
    MacOs,
    /// iOS device and the iOS simulator.
    IOs,
}

impl DeliveredFamily {
    /// Every artifact family a `deliver` list may name.
    pub(crate) const ALL: [Self; 2] = [Self::MacOs, Self::IOs];

    /// Returns the stable name a consumer states this family by.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::MacOs => "macos",
            Self::IOs => "ios",
        }
    }

    /// Resolves a stated family name, or nothing.
    ///
    /// Matched exactly, for [`NamedProfile::parse`]'s reason: a name that is
    /// nearly a family decides which targets a consumer's build compiles for.
    pub(crate) fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|family| family.as_str() == name)
    }

    /// Returns the driver families this name covers, in the driver's own
    /// vocabulary.
    const fn platforms(self) -> &'static [ApplePlatform] {
        match self {
            Self::MacOs => &[ApplePlatform::MacOs],
            Self::IOs => &[ApplePlatform::IOsDevice, ApplePlatform::IOsSimulator],
        }
    }

    /// Returns the governed floor for [`PROFILE_MSL_VERSION`] on this family.
    ///
    /// The number a [`NamedProfile`] uses when a consumer states no floor of its
    /// own. It is the driver's table restated at one site rather than derived,
    /// because the driver publishes no accessor for a governed floor;
    /// `every_profile_family_sits_on_its_governed_language_floor` is what holds
    /// the two in agreement, by requiring one minor version lower to be refused
    /// by the driver.
    ///
    /// Both families sit at 26.0, and the shared arm is a fact about
    /// [`PROFILE_MSL_VERSION`] rather than about the families: MSL 4.0's
    /// governed floor is 26.0 on every family the driver admits it for. Under
    /// MSL 3.1 the same two rows read 14.0 and 17.0, so a standard change splits
    /// this arm rather than editing one number, and
    /// `every_profile_family_sits_on_its_governed_language_floor` is what fails
    /// if it is not split.
    const fn governed_minimum(self) -> DeploymentMinimum {
        match self {
            Self::MacOs | Self::IOs => DeploymentMinimum::new(26, 0),
        }
    }

    /// Expands this family to the driver's selected families at one minimum.
    fn selected(self, deployment_minimum: DeploymentMinimum) -> Vec<SelectedFamily> {
        self.platforms()
            .iter()
            .map(|platform| SelectedFamily {
                family: *platform,
                deployment_minimum,
                msl_version: PROFILE_MSL_VERSION,
            })
            .collect()
    }
}

/// An ergonomic named delivery profile.
///
/// `docs/integration/frontends.md` permits one — "A frontend may offer an
/// ergonomic literal default profile, but the resolved selection is still
/// explicit compiler input" — and it is the ergonomic half of what a consumer
/// writes: `deliver macos-and-ios;` states three families and no version at all.
/// A profile is a spelling, never a second encoder — [`Self::policy`] states a
/// policy, and [`stated_delivery`] validates it through
/// [`ArtifactFamilySelection::new`] like every other stated policy. Every
/// deployment minimum is [`DeliveredFamily::governed_minimum`], the governed
/// floor for [`PROFILE_MSL_VERSION`] rather than a number chosen here, so a
/// profile excludes no OS version it could have included and the driver would
/// reject anything lower.
///
/// A consumer needing a *higher* floor states the family list instead; that is
/// the escape hatch the profile vocabulary is affordable because of.
///
/// # Why Mac Catalyst is in no profile
///
/// Not because the standard excludes it — [`PROFILE_MSL_VERSION`] is MSL 4.0
/// and the driver's governed table admits Catalyst there — but because
/// `docs/backends/metal.md` records the Catalyst row as a bounded
/// compile-and-link measurement rather than a specification or runtime
/// qualification claim, and admitting a spelling for it is `Q-ART-012` in
/// `docs/open-questions.md`, deferred until an explicit trigger. A Catalyst
/// consumer consequently matches no profile's selected family and takes the
/// semantic fallback, which is the only correct outcome:
/// `docs/backends/metal.md` forbids relabelling an iOS-device or macOS payload
/// as Catalyst-compatible. The family list cannot reach it either, because
/// [`DeliveredFamily`] publishes no Catalyst spelling.
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

    /// Returns the artifact families this profile names.
    const fn families(self) -> &'static [DeliveredFamily] {
        match self {
            Self::FallbackOnly => &[],
            Self::MacOs => &[DeliveredFamily::MacOs],
            Self::IOs => &[DeliveredFamily::IOs],
            Self::MacOsAndIOs => &[DeliveredFamily::MacOs, DeliveredFamily::IOs],
        }
    }

    /// Returns the delivery policy this profile spells.
    ///
    /// A policy rather than an [`ArtifactFamilySelection`], because a profile is
    /// a *spelling* and never a second encoder: the one canonical constructor
    /// validates it, on the production path through [`stated_delivery`] like any
    /// other stated policy, so no profile can reach a selection the driver would
    /// have refused.
    pub(crate) fn policy(self) -> ArtifactDeliveryPolicy {
        let families: Vec<SelectedFamily> = self
            .families()
            .iter()
            .flat_map(|family| family.selected(family.governed_minimum()))
            .collect();
        if families.is_empty() {
            return ArtifactDeliveryPolicy::FallbackOnly;
        }
        ArtifactDeliveryPolicy::SelectedFamilies {
            families,
            requirement: FamilyRequirement::RequiredWhenTargetMatches,
        }
    }
}

/// What the offline driver produced for one selected family.
///
/// Two outcomes and no third: ADR 0053 gives a matching consumer target either
/// the family's artifact or a compile error, never a quiet fallback. A variant
/// meaning "not attempted" would be the quiet fallback wearing a name.
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
                // the literal early and turn a diagnostic into source. The
                // facade-owned re-export resolves compiler `compile_error!`
                // while the facade is built, outside the consumer's Cargo
                // dependency namespace.
                FamilyDelivery::Retained(diagnostic) => Some(format!(
                    "#[cfg({})]\n::tiler::__private::__tiler_compile_error!({diagnostic:?});",
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
pub(crate) fn byte_string_literal(bytes: &[u8]) -> String {
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

/// Why a `deliver` statement names no policy this frontend can state.
///
/// Every variant carries the span of the token that caused it, because the
/// statement's tokens are the only thing a consumer can act on: an unknown name
/// is reported at the name, and a floor the driver's governed table refuses is
/// reported at the version that stated it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StatementRefusal<S> {
    /// The statement names a profile this frontend does not publish.
    UnknownProfile {
        /// The name as written.
        name: String,
        /// The token.
        span: S,
    },
    /// A family list names a family this frontend does not publish.
    UnknownFamily {
        /// The name as written.
        name: String,
        /// The token.
        span: S,
    },
    /// A family list names one family twice.
    RepeatedFamily {
        /// The family's stable name.
        name: &'static str,
        /// The repeated token.
        span: S,
    },
    /// A stated family and deployment minimum are not a governed target.
    ///
    /// The driver's own rejection is carried rather than flattened (ADR 0074
    /// convention 1), so a floor below the governed minimum still names the
    /// minimum it needed.
    UngovernedTarget {
        /// The driver's target-level reason.
        source: MetalTargetError,
        /// The deployment-minimum token.
        span: S,
    },
}

impl<S> fmt::Display for StatementRefusal<S> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownProfile { name, .. } => write!(
                formatter,
                "`{name}` is not a delivery profile; this frontend names {}, or state a family \
                 list such as `deliver macos 26.0, ios 26.0;` to choose a deployment minimum of \
                 your own",
                rendered_profiles(),
            ),
            Self::UnknownFamily { name, .. } => write!(
                formatter,
                "`{name}` is not an artifact family a `deliver` list may name; this frontend names \
                 {}",
                rendered_families(),
            ),
            Self::RepeatedFamily { name, .. } => write!(
                formatter,
                "the `{name}` artifact family is stated twice; one `deliver` statement states each \
                 family once, because two entries for one family disagree about its deployment \
                 minimum whenever they differ"
            ),
            Self::UngovernedTarget { source, .. } => write!(
                formatter,
                "this artifact family and deployment minimum are not a governed Metal target: \
                 {source}"
            ),
        }
    }
}

impl<S> StatementRefusal<S> {
    /// Returns the span this refusal must be reported at.
    pub(crate) const fn span(&self) -> &S {
        match self {
            Self::UnknownProfile { span, .. }
            | Self::UnknownFamily { span, .. }
            | Self::RepeatedFamily { span, .. }
            | Self::UngovernedTarget { span, .. } => span,
        }
    }
}

/// Renders the profile vocabulary a diagnostic offers.
fn rendered_profiles() -> String {
    rendered_names(NamedProfile::ALL.map(NamedProfile::as_str).as_slice())
}

/// Renders the artifact-family vocabulary a diagnostic offers.
fn rendered_families() -> String {
    rendered_names(DeliveredFamily::ALL.map(DeliveredFamily::as_str).as_slice())
}

/// Renders one vocabulary the way a diagnostic lists it.
fn rendered_names(names: &[&str]) -> String {
    names
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The delivery policy one region's tokens resolve to.
///
/// A function of the parsed region since Tom accepted the `deliver` statement on
/// 2026-07-31; it was a constant `FallbackOnly` while no region could say
/// otherwise. `None` is a region that states no `deliver` statement, and it
/// resolves to [`ArtifactDeliveryPolicy::FallbackOnly`] — the policy every region
/// stated before the statement existed, which is what keeps a region written
/// without one expanding to the tokens it always did.
///
/// # Errors
///
/// Returns the [`StatementRefusal`] carrying the span of the token that caused
/// it. The policy is *stated* here and validated by [`stated_delivery`]; the
/// per-family check below runs early only so a refused floor lands on the
/// version that stated it rather than on the statement as a whole, and it calls
/// the driver's own [`MetalTarget::new`] rather than restating a floor.
pub(crate) fn stated_policy<S: Copy>(
    stated: Option<&DeliverySyntax<S>>,
) -> Result<ArtifactDeliveryPolicy, StatementRefusal<S>> {
    let Some(delivery) = stated else {
        return Ok(ArtifactDeliveryPolicy::FallbackOnly);
    };
    match &delivery.stated {
        StatedDelivery::Profile(name) => NamedProfile::parse(&name.text)
            .map(NamedProfile::policy)
            .ok_or_else(|| StatementRefusal::UnknownProfile {
                name: name.text.clone(),
                span: name.span,
            }),
        StatedDelivery::Families(stated) => stated_family_list(stated),
    }
}

/// Resolves one `deliver` family list into the policy it states.
fn stated_family_list<S: Copy>(
    stated: &[FamilyMinimumSyntax<S>],
) -> Result<ArtifactDeliveryPolicy, StatementRefusal<S>> {
    let mut named: Vec<DeliveredFamily> = Vec::with_capacity(stated.len());
    let mut families: Vec<SelectedFamily> = Vec::new();
    for entry in stated {
        let family = DeliveredFamily::parse(&entry.name.text).ok_or_else(|| {
            StatementRefusal::UnknownFamily {
                name: entry.name.text.clone(),
                span: entry.name.span,
            }
        })?;
        // The driver rejects a duplicate too, at the family it canonicalized to.
        // Refusing the repeated *spelling* here is what puts the diagnostic on
        // the second `ios` a consumer wrote rather than on the invocation.
        if named.contains(&family) {
            return Err(StatementRefusal::RepeatedFamily {
                name: family.as_str(),
                span: entry.name.span,
            });
        }
        named.push(family);

        let minimum = DeploymentMinimum::new(entry.minimum.major, entry.minimum.minor);
        for selected in family.selected(minimum) {
            MetalTarget::new(selected.family, minimum, selected.msl_version).map_err(|source| {
                StatementRefusal::UngovernedTarget {
                    source,
                    span: entry.minimum.span,
                }
            })?;
            families.push(selected);
        }
    }
    // The grammar admits no empty list, so `EmptySelection` is unreachable from
    // a stated one; `stated_delivery` remains the authority that says so.
    Ok(ArtifactDeliveryPolicy::SelectedFamilies {
        families,
        requirement: FamilyRequirement::RequiredWhenTargetMatches,
    })
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
/// It no longer refuses a selection that invokes the backend compiler: since
/// `prototype-inline-aot-integration-proof`, [`crate::aot`] runs the offline
/// driver for one, and refusing here would refuse the very thing the statement
/// asks for. What the frontend still cannot build refuses in [`crate::aot`]
/// instead, where the reason is a fact about the one bound compile declaration
/// rather than about the policy vocabulary.
///
/// # Errors
///
/// Returns [`DeliveryRefusal::InvalidSelection`] when the policy is not a valid
/// selection.
pub(crate) fn stated_delivery(
    policy: ArtifactDeliveryPolicy,
) -> Result<ArtifactFamilySelection, DeliveryRefusal> {
    ArtifactFamilySelection::new(policy).map_err(DeliveryRefusal::InvalidSelection)
}

/// Builds the plan a selection naming no family delivers.
///
/// `FallbackOnly`'s plan, and only its: a selection that invokes the backend
/// compiler goes through [`crate::aot::deliver`], which has an artifact
/// and per-family outcomes to supply.
///
/// # Errors
///
/// Returns [`DeliveryRefusal::MalformedPlan`] when the selection and outcomes do
/// not correspond, which for an empty selection is a propagated impossibility
/// rather than a reachable failure.
pub(crate) fn fallback_plan(
    selection: ArtifactFamilySelection,
) -> Result<DeliveryPlan, DeliveryRefusal> {
    DeliveryPlan::new(selection, Vec::new(), Vec::new()).map_err(DeliveryRefusal::MalformedPlan)
}

#[cfg(test)]
mod tests;
