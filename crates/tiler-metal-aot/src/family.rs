//! Explicit Apple artifact-family selection: which families a request builds.
//!
//! ADR 0049 requires that "every inline AOT compilation request contains a
//! canonical, typed `ArtifactFamilySelection`" and that "the proc macro does not
//! infer a family from its host environment". This module is that request field.
//! It names the governed families to build, each with its own deployment minimum
//! and language standard, and resolves them to the exact compile targets the
//! driver will invoke.
//!
//! # The half of ADR 0053 this module deliberately does not implement
//!
//! ADR 0053 pairs the selection with a delivery policy whose *other* half is
//! generated Rust: "Generated Rust gates the payload or diagnostic by the
//! family's versioned consumer-target `#[cfg]` predicate. A matching target
//! requires the selected artifact and sees `compile_error!` on build failure; a
//! nonmatching target uses the semantic fallback."
//!
//! None of that is here, and the omission is a boundary rather than an
//! oversight. ADR 0077 item 1 states that this crate "does not emit MSL, does
//! not assemble the target-neutral artifact bundle, and does not implement the
//! expansion cache or the **proc-macro layer**", and
//! `docs/architecture.md`'s packaging profile assigns "emit artifact plus
//! runtime/fallback tokens" to the frontend proc-macro crate. A family's
//! consumer-target `#[cfg]` predicate is a fact about a *Rust* target — the
//! probe records macOS as `target_os = "macos"`, iOS device as `target_os =
//! "ios"` with an empty `target_abi`, and the iOS simulator as `target_abi =
//! "sim"` — and this crate knows only about `xcrun`. Putting versioned
//! generated-code data here would give the driver a second responsibility that
//! an accepted packaging profile places elsewhere.
//!
//! What this module owns is therefore exactly the driver-side question: **given
//! a selection, which compilations happen?** `prototype-artifact-family-delivery`
//! records the split and the follow-up that owns the generated half.
//!
//! # Why `FallbackOnly` is here even though it compiles nothing
//!
//! ADR 0053 makes `FallbackOnly` "an explicit valid policy" that "invokes no
//! backend compiler". Stating it as a variant rather than as an empty family
//! list is what makes "this request deliberately performs no AOT work"
//! distinguishable from "a producer assembled a selection and forgot to put a
//! family in it". The second is [`FamilySelectionError::EmptySelection`], and
//! rejecting it is what leaves `FallbackOnly` the only spelling of the first.

#![allow(
    dead_code,
    reason = "the family selection is landed ahead of its production caller (ADR 0074 convention 7). It reserves the canonical `ArtifactFamilySelection` that ADR 0049 requires every inline AOT compilation request to carry, and its first non-test caller is the frontend proc-macro crate, which emits the `#[cfg]`-gated delivery half. That crate does not exist: `prototype-inline-proc-macro-frontend` depends on `prototype-public-compiler-api`, whose closing condition is Tom's acceptance of a public boundary rather than further engineering, and `record-that-the-frontend-axis-is-review-gated` records that the axis is gated on that review."
)]

use core::fmt;

use crate::identity::{push_len, push_str};
use crate::input::{ApplePlatform, DeploymentMinimum, MetalTarget, MetalTargetError, MslVersion};

/// Versioned domain tag opening the canonical selection bytes.
///
/// ADR 0074 convention 3: bytes produced for one subject can never be mistaken
/// for another subject's.
const SELECTION_DOMAIN: &[u8] = b"tiler.metal-aot.artifact-family-selection.v1\0";

/// One artifact family a request selects, with the target facts it selects it at.
///
/// The deployment minimum and language standard are *per family* rather than
/// shared across a selection. The macro-environment research contract requires
/// it — "each selected Apple family has an explicit platform, SDK identity,
/// deployment minimum, Metal language standard, compiler flags, and payload" —
/// and the facts genuinely differ: a macOS 13.0 minimum and an iOS 13.0 minimum
/// are unrelated version lines, so one shared field would silently apply one
/// family's floor to another.
///
/// A caller-constructed leaf value record, so its fields are visible
/// (ADR 0074 convention 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SelectedFamily {
    /// The artifact family to build.
    pub(crate) family: ApplePlatform,
    /// The deployment minimum this family is built at.
    pub(crate) deployment_minimum: DeploymentMinimum,
    /// The MSL standard this family is compiled under.
    pub(crate) msl_version: MslVersion,
}

impl SelectedFamily {
    /// Returns the fully specified compile target this selection requires.
    ///
    /// # Errors
    ///
    /// Returns [`FamilySelectionError::InvalidTarget`] when the selected facts
    /// do not form a governed target.
    fn compile_target(self) -> Result<MetalTarget, FamilySelectionError> {
        MetalTarget::new(self.family, self.deployment_minimum, self.msl_version)
            .map_err(|source| FamilySelectionError::InvalidTarget { source })
    }

    /// Returns this family's canonical order key.
    ///
    /// The family's stable lowercase identifier, never its declaration ordinal
    /// or its discriminant (ADR 0074 convention 3).
    fn canonical_key(self) -> &'static str {
        self.family.as_str()
    }
}

/// What a request asks the Apple offline toolchain to build.
///
/// The grammar is the one `docs/integration/frontends.md` states:
///
/// ```text
/// ArtifactDeliveryPolicy =
///     SelectedFamilies([AppleArtifactFamily], RequiredWhenTargetMatches)
///   | FallbackOnly
/// ```
///
/// Deliberately **not** `#[non_exhaustive]`. This is an ADR 0074 convention 5b
/// type: [`ArtifactFamilySelection::canonical_bytes`] matches it totally to
/// produce an identity tag, and a wildcard arm there could only invent a tag
/// that the variant alone determines — which would let two selections meaning
/// different things share identity bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ArtifactDeliveryPolicy {
    /// Build each named family. A consumer target matching a selected family
    /// requires that family's artifact and must not silently receive another's.
    ///
    /// `RequiredWhenTargetMatches` is the requirement mode the contract names,
    /// and today it is the only one. It is a field rather than an implied
    /// property because `docs/integration/frontends.md` reserves a second — "a
    /// frontend may expose a separate explicit 'acceleration required' policy" —
    /// and a mode that is not stated cannot later be distinguished in identity
    /// from one that is.
    SelectedFamilies {
        /// The selected families, in canonical family order.
        families: Vec<SelectedFamily>,
        /// What a matching consumer target is owed.
        requirement: FamilyRequirement,
    },
    /// Build nothing. Every consumer target uses the semantic fallback.
    ///
    /// ADR 0053: "`FallbackOnly` is an explicit valid policy and invokes no
    /// backend compiler."
    FallbackOnly,
}

/// What a consumer target that matches a selected family is owed.
///
/// A one-variant enum rather than an implied constant, so the mode reaches
/// canonical bytes and a second mode becomes an explicit identity change instead
/// of a silent behavioural one. `canonical_bytes` destructures it irrefutably,
/// which makes adding a variant a compile error at the encoder
/// (ADR 0074 convention 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum FamilyRequirement {
    /// The family's artifact is required; a build failure is a compile error on
    /// the matching target and never a silent fallback.
    RequiredWhenTargetMatches,
}

/// The canonical, typed artifact-family selection of one compilation request.
///
/// A verified product: its field is private and it is reachable only through
/// [`ArtifactFamilySelection::new`], which rejects a duplicate family, an empty
/// selected list, and a family no governed SDK produces
/// (ADR 0074 conventions 4 and 6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArtifactFamilySelection {
    policy: ArtifactDeliveryPolicy,
}

impl ArtifactFamilySelection {
    /// Validates one delivery policy into a canonical selection.
    ///
    /// The selected families are reordered into canonical family order, so a
    /// selection is a function of *which* families it names rather than of the
    /// order a producer happened to list them in. Duplicates are rejected rather
    /// than deduplicated: two entries for one family disagree about that
    /// family's deployment minimum or language standard whenever they differ,
    /// and silently keeping one would drop a stated compilation input.
    ///
    /// # Errors
    ///
    /// Returns [`FamilySelectionError::EmptySelection`] for
    /// `SelectedFamilies` naming no family,
    /// [`FamilySelectionError::DuplicateFamily`] for a repeated family, or
    /// [`FamilySelectionError::InvalidTarget`] when one family's platform,
    /// minimum, and language revision are not a governed target.
    pub(crate) fn new(policy: ArtifactDeliveryPolicy) -> Result<Self, FamilySelectionError> {
        let policy = match policy {
            ArtifactDeliveryPolicy::FallbackOnly => ArtifactDeliveryPolicy::FallbackOnly,
            ArtifactDeliveryPolicy::SelectedFamilies {
                mut families,
                requirement,
            } => {
                if families.is_empty() {
                    return Err(FamilySelectionError::EmptySelection);
                }
                families.sort_unstable_by_key(|selected| selected.canonical_key());
                for pair in families.windows(2) {
                    if pair[0].family == pair[1].family {
                        return Err(FamilySelectionError::DuplicateFamily {
                            family: pair[0].family,
                        });
                    }
                }
                for selected in &families {
                    selected.compile_target()?;
                }
                ArtifactDeliveryPolicy::SelectedFamilies {
                    families,
                    requirement,
                }
            }
        };
        Ok(Self { policy })
    }

    /// Returns the validated delivery policy.
    pub(crate) const fn policy(&self) -> &ArtifactDeliveryPolicy {
        &self.policy
    }

    /// Returns the selected families in canonical order, empty under
    /// `FallbackOnly`.
    pub(crate) fn families(&self) -> &[SelectedFamily] {
        match &self.policy {
            ArtifactDeliveryPolicy::SelectedFamilies { families, .. } => families,
            ArtifactDeliveryPolicy::FallbackOnly => &[],
        }
    }

    /// Returns whether this selection invokes the backend compiler at all.
    ///
    /// Stated as a method because "no compile targets" is the *consequence* a
    /// caller acts on and `FallbackOnly` is the *reason*; a caller that inferred
    /// the reason from an empty target list would also infer it from a selection
    /// this constructor rejects.
    pub(crate) const fn invokes_backend_compiler(&self) -> bool {
        match &self.policy {
            ArtifactDeliveryPolicy::SelectedFamilies { .. } => true,
            ArtifactDeliveryPolicy::FallbackOnly => false,
        }
    }

    /// Returns the exact compile targets this selection requires, in canonical
    /// family order.
    ///
    /// One target per selected family, never fewer: two families are two
    /// compilations producing two independently identified payloads, and a
    /// selection is not satisfied by compiling a subset of it.
    ///
    /// # Errors
    ///
    /// Returns [`FamilySelectionError::InvalidTarget`]. The constructor already
    /// rejects that case, so this is a propagated impossibility rather than a
    /// reachable failure.
    pub(crate) fn compile_targets(&self) -> Result<Vec<MetalTarget>, FamilySelectionError> {
        self.families()
            .iter()
            .map(|selected| selected.compile_target())
            .collect()
    }

    /// Returns the canonical bytes identifying this selection.
    ///
    /// ADR 0049 requires the selection to "fully participate in explain output
    /// and content identity". This is the subject that participation is over:
    /// domain-separated, length-prefixed, free of arena and declaration
    /// ordinals, and matched exhaustively (ADR 0074 convention 3).
    ///
    /// It is deliberately **bytes and not a digest**. This crate's dependency
    /// closure is empty by decision (ADR 0077 item 2), so it owns no hash
    /// function, and the governed artifact digest is
    /// `tiler.digest.sha-256.v1` in `tiler-artifact`. A local digest here would
    /// be a second identity authority over the same subject; a caller that
    /// already owns the governed algorithm digests these bytes instead
    /// (ADR 0074 convention 2).
    ///
    /// The framing is [`crate::identity::push_len`], this crate's sole admitted
    /// copy of the workspace's canonical eight-byte big-endian prefix. It is not
    /// merely shared for tidiness: this function previously carried a private
    /// four-byte `u32` framing while `identity.rs` framed the compilation
    /// subject in eight, so one crate held two widths under one name, and a
    /// reader comparing the two encoders had nothing but the `u32` literal to
    /// tell them apart.
    pub(crate) fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(SELECTION_DOMAIN);
        match &self.policy {
            ArtifactDeliveryPolicy::FallbackOnly => bytes.push(0x01),
            ArtifactDeliveryPolicy::SelectedFamilies {
                families,
                requirement,
            } => {
                bytes.push(0x02);
                let FamilyRequirement::RequiredWhenTargetMatches = requirement;
                bytes.push(0x01);
                push_len(&mut bytes, families.len());
                for selected in families {
                    push_str(&mut bytes, selected.family.as_str());
                    bytes.extend_from_slice(&selected.deployment_minimum.major().to_be_bytes());
                    bytes.extend_from_slice(&selected.deployment_minimum.minor().to_be_bytes());
                    push_str(&mut bytes, selected.msl_version.semantic_name());
                }
            }
        }
        bytes
    }
}

/// Why one artifact-family selection is not a valid compilation request.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a rejection vocabulary a
/// caller forwards or partially classifies, which no crate maps totally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum FamilySelectionError {
    /// `SelectedFamilies` named no family. `FallbackOnly` is the explicit
    /// spelling of a request that deliberately builds nothing.
    EmptySelection,
    /// One artifact family was selected more than once.
    DuplicateFamily {
        /// The repeated family.
        family: ApplePlatform,
    },
    /// One selected family does not form a governed compiler target.
    InvalidTarget {
        /// The target-level reason.
        source: MetalTargetError,
    },
}

impl fmt::Display for FamilySelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySelection => formatter.write_str(
                "a selection naming no family is not FallbackOnly; state FallbackOnly explicitly",
            ),
            Self::DuplicateFamily { family } => {
                write!(
                    formatter,
                    "the {} family is selected twice",
                    family.as_str()
                )
            }
            Self::InvalidTarget { source } => {
                write!(formatter, "invalid selected family: {source}")
            }
        }
    }
}

impl std::error::Error for FamilySelectionError {}

#[cfg(test)]
mod tests {
    use super::{
        ArtifactDeliveryPolicy, ArtifactFamilySelection, FamilyRequirement, FamilySelectionError,
        SelectedFamily,
    };
    use crate::input::{ApplePlatform, AppleSdk, DeploymentMinimum, MslVersion};

    fn selected(family: ApplePlatform, major: u16, minor: u16) -> SelectedFamily {
        SelectedFamily {
            family,
            deployment_minimum: DeploymentMinimum::new(major, minor),
            msl_version: MslVersion::Metal3_1,
        }
    }

    fn selection(families: Vec<SelectedFamily>) -> ArtifactFamilySelection {
        ArtifactFamilySelection::new(ArtifactDeliveryPolicy::SelectedFamilies {
            families,
            requirement: FamilyRequirement::RequiredWhenTargetMatches,
        })
        .expect("the selection is valid")
    }

    /// Each selected family compiles for its own triple, and for no other's.
    ///
    /// This is the property the ticket names: a nonmatching target must not
    /// silently receive incompatible bytes. At this layer that means a selection
    /// naming three families produces three *distinct* compilations, so no two
    /// families can be satisfied by one payload.
    #[test]
    fn each_selected_family_compiles_for_its_own_target() {
        let selection = selection(vec![
            selected(ApplePlatform::MacOs, 14, 0),
            selected(ApplePlatform::IOsDevice, 17, 0),
            selected(ApplePlatform::IOsSimulator, 17, 0),
        ]);
        let targets = selection.compile_targets().expect("every family resolves");
        assert_eq!(targets.len(), 3);

        let mut triples: Vec<String> = targets.iter().map(|target| target.triple()).collect();
        triples.sort();
        assert_eq!(
            triples,
            [
                "air64-apple-ios17.0",
                "air64-apple-ios17.0-simulator",
                "air64-apple-macos14.0",
            ],
        );

        for (target, expected) in targets.iter().zip(selection.families()) {
            assert_eq!(
                target.platform(),
                expected.family,
                "a compilation must produce the family that selected it",
            );
        }
    }

    /// Two families are two compilations; neither is dropped or merged.
    #[test]
    fn a_two_family_selection_produces_two_distinct_compilations() {
        let selection = selection(vec![
            selected(ApplePlatform::IOsDevice, 17, 0),
            selected(ApplePlatform::IOsSimulator, 17, 0),
        ]);
        let targets = selection.compile_targets().expect("every family resolves");
        assert_eq!(targets.len(), 2);
        assert_ne!(
            targets[0].triple(),
            targets[1].triple(),
            "the simulator environment must not collapse onto the device triple",
        );
        assert_ne!(targets[0].sdk().selector(), targets[1].sdk().selector());
    }

    /// A deployment minimum is per family, not shared across the selection.
    ///
    /// Without this, one family's version floor could silently be applied to
    /// another's unrelated version line.
    #[test]
    fn each_family_keeps_its_own_deployment_minimum() {
        let selection = selection(vec![
            selected(ApplePlatform::MacOs, 14, 0),
            selected(ApplePlatform::IOsDevice, 17, 2),
        ]);
        let mut triples: Vec<String> = selection
            .compile_targets()
            .expect("every family resolves")
            .iter()
            .map(|target| target.triple())
            .collect();
        triples.sort();
        assert_eq!(triples, ["air64-apple-ios17.2", "air64-apple-macos14.0"]);
    }

    /// `FallbackOnly` invokes no backend compiler.
    ///
    /// ADR 0053 states this as a property of the policy, so both halves are
    /// asserted: no compile target is produced, and the reason is legible
    /// without inferring it from the empty list.
    #[test]
    fn fallback_only_invokes_no_backend_compiler() {
        let selection = ArtifactFamilySelection::new(ArtifactDeliveryPolicy::FallbackOnly)
            .expect("FallbackOnly is always valid");
        assert!(!selection.invokes_backend_compiler());
        assert!(selection.families().is_empty());
        assert!(
            selection
                .compile_targets()
                .expect("no family to resolve")
                .is_empty()
        );
    }

    /// A selection naming families does invoke the compiler.
    ///
    /// The negative case above is only meaningful paired with this one.
    #[test]
    fn a_selected_family_invokes_the_backend_compiler() {
        let selection = selection(vec![selected(ApplePlatform::MacOs, 14, 0)]);
        assert!(selection.invokes_backend_compiler());
        assert_eq!(selection.compile_targets().expect("it resolves").len(), 1);
    }

    /// An empty family list is rejected rather than treated as `FallbackOnly`.
    ///
    /// Collapsing the two would make a producer's omission indistinguishable
    /// from a deliberate decision to perform no AOT work, and the contract
    /// requires the second to be explicit.
    #[test]
    fn an_empty_selected_family_list_is_not_fallback_only() {
        let error = ArtifactFamilySelection::new(ArtifactDeliveryPolicy::SelectedFamilies {
            families: Vec::new(),
            requirement: FamilyRequirement::RequiredWhenTargetMatches,
        })
        .expect_err("an empty selection is rejected");
        assert_eq!(error, FamilySelectionError::EmptySelection);
    }

    /// A repeated family is rejected, not silently deduplicated.
    ///
    /// The two entries disagree about the family's deployment minimum, and
    /// keeping either one would drop a stated compilation input.
    #[test]
    fn a_repeated_family_is_rejected_rather_than_deduplicated() {
        let error = ArtifactFamilySelection::new(ArtifactDeliveryPolicy::SelectedFamilies {
            families: vec![
                selected(ApplePlatform::MacOs, 14, 0),
                selected(ApplePlatform::MacOs, 15, 0),
            ],
            requirement: FamilyRequirement::RequiredWhenTargetMatches,
        })
        .expect_err("a duplicate family is rejected");
        assert_eq!(
            error,
            FamilySelectionError::DuplicateFamily {
                family: ApplePlatform::MacOs,
            },
        );
    }

    /// Declaration order is presentation; the canonical bytes are a function of
    /// which families are named.
    ///
    /// Artifact identity folds these bytes, so two requests meaning the same
    /// thing must not be two different artifacts.
    #[test]
    fn declaration_order_does_not_change_the_canonical_bytes() {
        let forward = selection(vec![
            selected(ApplePlatform::MacOs, 14, 0),
            selected(ApplePlatform::IOsSimulator, 17, 0),
        ]);
        let reversed = selection(vec![
            selected(ApplePlatform::IOsSimulator, 17, 0),
            selected(ApplePlatform::MacOs, 14, 0),
        ]);
        assert_eq!(forward.canonical_bytes(), reversed.canonical_bytes());
        assert_eq!(forward, reversed);
    }

    /// Every facet a compilation depends on moves the canonical bytes.
    ///
    /// A facet that did not would let two genuinely different requests share an
    /// identity, which is the failure ADR 0074 convention 3 exists to prevent.
    #[test]
    fn every_selection_facet_reaches_the_canonical_bytes() {
        let baseline = selection(vec![selected(ApplePlatform::MacOs, 14, 0)]).canonical_bytes();

        let other_family =
            selection(vec![selected(ApplePlatform::IOsDevice, 17, 0)]).canonical_bytes();
        assert_ne!(baseline, other_family, "the family must reach identity");

        let other_minor = selection(vec![selected(ApplePlatform::MacOs, 14, 1)]).canonical_bytes();
        assert_ne!(baseline, other_minor, "the minor version must reach it");

        let other_major = selection(vec![selected(ApplePlatform::MacOs, 15, 0)]).canonical_bytes();
        assert_ne!(baseline, other_major, "the major version must reach it");

        let other_language = selection(vec![SelectedFamily {
            msl_version: MslVersion::Metal3_0,
            ..selected(ApplePlatform::MacOs, 14, 0)
        }])
        .canonical_bytes();
        assert_ne!(baseline, other_language, "the standard must reach it");

        let two_families = selection(vec![
            selected(ApplePlatform::MacOs, 14, 0),
            selected(ApplePlatform::IOsDevice, 17, 0),
        ])
        .canonical_bytes();
        assert_ne!(baseline, two_families, "the family count must reach it");
    }

    /// `FallbackOnly` and a one-family selection are different subjects.
    ///
    /// They perform different work, so an identity that could not tell them
    /// apart would let a cache serve one for the other.
    #[test]
    fn fallback_only_has_its_own_identity() {
        let fallback = ArtifactFamilySelection::new(ArtifactDeliveryPolicy::FallbackOnly)
            .expect("FallbackOnly is always valid")
            .canonical_bytes();
        let macos = selection(vec![selected(ApplePlatform::MacOs, 14, 0)]).canonical_bytes();
        assert_ne!(fallback, macos);
        assert!(fallback.starts_with(super::SELECTION_DOMAIN));
        assert!(macos.starts_with(super::SELECTION_DOMAIN));
    }

    /// The family count is framed in the workspace's eight bytes, not four.
    ///
    /// The prefix is spelled out here rather than produced by
    /// `crate::identity::push_len`, because checking the encoder with the
    /// encoder's own helper is exactly what cannot catch the width changing.
    /// This crate carried a private four-byte `u32` framing in this file while
    /// framing its compilation subject in eight, so the two encoders in one
    /// crate disagreed on the one thing a canonical form has to fix. Nothing
    /// consumed these bytes yet, so consolidating them moved no identity — this
    /// test is what makes the width a stated property rather than a byte a
    /// reader has to derive from a literal.
    #[test]
    fn the_family_count_carries_the_eight_byte_framing() {
        let bytes = selection(vec![selected(ApplePlatform::MacOs, 14, 0)]).canonical_bytes();
        // The policy tag and its requirement mode precede the count.
        let count_at = super::SELECTION_DOMAIN.len() + 2;
        assert_eq!(
            &bytes[count_at..count_at + 8],
            &[0, 0, 0, 0, 0, 0, 0, 1],
            "one selected family must be framed as eight big-endian bytes",
        );
    }

    /// The canonical bytes open with their versioned domain tag.
    #[test]
    fn the_canonical_bytes_are_domain_separated() {
        let bytes = selection(vec![selected(ApplePlatform::MacOs, 14, 0)]).canonical_bytes();
        assert!(bytes.starts_with(super::SELECTION_DOMAIN));
        assert!(
            bytes.len() > super::SELECTION_DOMAIN.len(),
            "the domain tag must precede content rather than be the whole subject",
        );
    }

    /// Two families whose names share a prefix cannot share identity bytes.
    ///
    /// The length prefixes are what prevent it; without them `ios-device` and
    /// `ios-simulator` concatenated with their versions could align.
    #[test]
    fn length_prefixes_keep_adjacent_runs_unambiguous() {
        let device_then_simulator = selection(vec![
            selected(ApplePlatform::IOsDevice, 17, 0),
            selected(ApplePlatform::IOsSimulator, 17, 0),
        ])
        .canonical_bytes();
        let simulator_only =
            selection(vec![selected(ApplePlatform::IOsSimulator, 17, 0)]).canonical_bytes();
        assert_ne!(device_then_simulator, simulator_only);
        assert!(
            !device_then_simulator.ends_with(&simulator_only[super::SELECTION_DOMAIN.len()..]),
            "a selection's bytes must not be a suffix of a larger selection's",
        );
    }

    /// Every represented family derives its own SDK without conflating SDK and
    /// artifact identity.
    #[test]
    fn every_represented_family_derives_its_sdk() {
        for (family, sdk) in [
            (ApplePlatform::MacOs, AppleSdk::MacOs),
            (ApplePlatform::MacCatalyst, AppleSdk::MacOs),
            (ApplePlatform::IOsDevice, AppleSdk::IPhoneOs),
            (ApplePlatform::IOsSimulator, AppleSdk::IPhoneSimulator),
            (ApplePlatform::TvOsDevice, AppleSdk::AppleTvOs),
            (ApplePlatform::TvOsSimulator, AppleSdk::AppleTvSimulator),
            (ApplePlatform::VisionOsDevice, AppleSdk::XrOs),
            (ApplePlatform::VisionOsSimulator, AppleSdk::XrSimulator),
            (ApplePlatform::WatchOsDevice, AppleSdk::WatchOs),
            (ApplePlatform::WatchOsSimulator, AppleSdk::WatchSimulator),
        ] {
            let expected = match family {
                ApplePlatform::MacOs | ApplePlatform::MacCatalyst => AppleSdk::MacOs,
                ApplePlatform::IOsDevice => AppleSdk::IPhoneOs,
                ApplePlatform::IOsSimulator => AppleSdk::IPhoneSimulator,
                ApplePlatform::TvOsDevice => AppleSdk::AppleTvOs,
                ApplePlatform::TvOsSimulator => AppleSdk::AppleTvSimulator,
                ApplePlatform::VisionOsDevice => AppleSdk::XrOs,
                ApplePlatform::VisionOsSimulator => AppleSdk::XrSimulator,
                ApplePlatform::WatchOsDevice => AppleSdk::WatchOs,
                ApplePlatform::WatchOsSimulator => AppleSdk::WatchSimulator,
            };
            assert_eq!(sdk, expected, "{}", family.as_str());
            assert_eq!(family.sdk(), sdk, "{}", family.as_str());
        }
    }

    /// The validated policy is readable, and its requirement mode survives.
    #[test]
    fn the_validated_policy_retains_its_requirement_mode() {
        let selection = selection(vec![selected(ApplePlatform::MacOs, 14, 0)]);
        match selection.policy() {
            ArtifactDeliveryPolicy::SelectedFamilies {
                families,
                requirement,
            } => {
                assert_eq!(families.len(), 1);
                assert_eq!(*requirement, FamilyRequirement::RequiredWhenTargetMatches);
            }
            ArtifactDeliveryPolicy::FallbackOnly => {
                panic!("a selected-family policy must not become FallbackOnly")
            }
        }
    }
}
