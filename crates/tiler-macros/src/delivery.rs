//! The frontend's statement of its artifact-family delivery policy.
//!
//! ADR 0049 requires every inline AOT compilation request to carry a canonical,
//! typed `ArtifactFamilySelection`, and requires that the proc macro not infer a
//! family from its host environment. This module is where this frontend states
//! one and reads it back. It states a policy; it does not discover Apple tools,
//! read the host, or decide a consumer `#[cfg]` predicate — the first belongs to
//! [`tiler_metal_aot::driver`] and the last to
//! `generate-cfg-gated-artifact-family-delivery`.
//!
//! There is exactly one canonical encoder for a selection and it lives in
//! [`tiler_metal_aot::family`]. Nothing here re-derives ordering, duplicate
//! rejection, per-family deployment minimums, language standards, or identity
//! bytes; [`ArtifactFamilySelection::new`] is the only way a value of that type
//! comes into being, on this side of the boundary as on the driver's.
//!
//! # What every region states, and why it is `FallbackOnly`
//!
//! The approved region grammar has no syntax for naming an artifact family, so
//! every invocation's tokens resolve to the same policy. ADR 0053 makes that an
//! explicit policy rather than an absence: "`FallbackOnly` is an explicit valid
//! policy and invokes no backend compiler". Saying it in the type is what
//! distinguishes it from a producer that assembled a selection and forgot to put
//! a family in it — which the driver rejects as `EmptySelection` — so every
//! expansion states `FallbackOnly` outright instead of leaving its delivery
//! unstated.
//!
//! Adding that syntax is a public-boundary decision rather than an omission
//! here: it publishes Apple family, deployment-minimum, and language-standard
//! vocabulary on the consumer-facing region surface, and the generated
//! `#[cfg]`-gated delivery it would drive is owned by
//! `generate-cfg-gated-artifact-family-delivery`, which depends on this
//! frontend.
//!
//! [`stated_delivery`] is deliberately a function of a *policy* rather than a
//! constant, so that ticket supplies a parsed policy without changing what
//! validates it, and so both refusal paths are exercised by the tests below
//! rather than only by the one policy every expansion states today.

use core::fmt;

use tiler_metal_aot::family::{
    ArtifactDeliveryPolicy, ArtifactFamilySelection, FamilySelectionError,
};

/// The delivery policy an invocation's tokens resolve to.
///
/// Nullary rather than a function of the parsed region, and that is the honest
/// signature: the approved grammar admits no family statement, so every region
/// resolves to [`ArtifactDeliveryPolicy::FallbackOnly`] and a parameter it
/// ignored would claim a dependence that does not exist. It becomes a function
/// of the region when `generate-cfg-gated-artifact-family-delivery` adds the
/// syntax and the `#[cfg]`-gated delivery that syntax would drive.
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
                 silently become fallback on a matching target; the family syntax and the \
                 `#[cfg]`-gated delivery half are owned by \
                 `generate-cfg-gated-artifact-family-delivery`",
                families.join(", "),
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

#[cfg(test)]
mod tests {
    use super::{DeliveryRefusal, stated_delivery, stated_policy};
    use tiler_metal_aot::family::{
        ArtifactDeliveryPolicy, ArtifactFamilySelection, FamilyRequirement, FamilySelectionError,
        SelectedFamily,
    };
    use tiler_metal_aot::input::{ApplePlatform, DeploymentMinimum, MetalTargetError, MslVersion};

    fn selected(family: ApplePlatform, major: u16, minor: u16) -> SelectedFamily {
        SelectedFamily {
            family,
            deployment_minimum: DeploymentMinimum::new(major, minor),
            msl_version: MslVersion::Metal3_1,
        }
    }

    fn policy(families: Vec<SelectedFamily>) -> ArtifactDeliveryPolicy {
        ArtifactDeliveryPolicy::SelectedFamilies {
            families,
            requirement: FamilyRequirement::RequiredWhenTargetMatches,
        }
    }

    /// The current expansion states `FallbackOnly`, and that is deliverable.
    ///
    /// This is the policy `tensor!` actually routes on today, so the anchor it
    /// emits is a *stated* no-AOT decision rather than an unstated one.
    #[test]
    fn the_current_expansion_states_a_deliverable_fallback_only_policy() {
        let selection = stated_delivery(stated_policy()).expect("FallbackOnly is deliverable");
        assert!(!selection.invokes_backend_compiler());
        assert!(selection.families().is_empty());
        assert_eq!(
            selection.policy(),
            &ArtifactDeliveryPolicy::FallbackOnly,
            "the frontend must state FallbackOnly rather than an empty family list",
        );
    }

    /// A valid selection naming families is refused rather than silently
    /// downgraded to fallback.
    ///
    /// The paired negative of the test above: without it, "`FallbackOnly` is
    /// deliverable" would also be what a function that accepted everything
    /// reported.
    #[test]
    fn a_selected_family_is_refused_rather_than_downgraded_to_fallback() {
        let refusal = stated_delivery(policy(vec![
            selected(ApplePlatform::IOsSimulator, 17, 0),
            selected(ApplePlatform::MacOs, 14, 0),
        ]))
        .expect_err("this expansion cannot build a selected family yet");
        assert_eq!(
            refusal,
            DeliveryRefusal::BackendCompilationUnavailable {
                families: vec!["ios-simulator", "macos"],
            },
            "the refusal names the families in canonical order, not declaration order",
        );
        let rendered = refusal.to_string();
        assert!(rendered.contains("ios-simulator"), "{rendered}");
        assert!(rendered.contains("macos"), "{rendered}");
    }

    /// The frontend gets the driver's empty-selection rejection, not its own.
    #[test]
    fn an_empty_family_list_is_refused_as_an_invalid_selection() {
        assert_eq!(
            stated_delivery(policy(Vec::new())).expect_err("an empty selection is invalid"),
            DeliveryRefusal::InvalidSelection(FamilySelectionError::EmptySelection),
        );
    }

    /// A repeated family is refused, and the refusal names it.
    #[test]
    fn a_repeated_family_is_refused_as_an_invalid_selection() {
        assert_eq!(
            stated_delivery(policy(vec![
                selected(ApplePlatform::MacOs, 14, 0),
                selected(ApplePlatform::MacOs, 15, 0),
            ]))
            .expect_err("a duplicate family is invalid"),
            DeliveryRefusal::InvalidSelection(FamilySelectionError::DuplicateFamily {
                family: ApplePlatform::MacOs,
            }),
        );
    }

    /// A deployment minimum below its language floor is refused with the
    /// target-level reason intact.
    ///
    /// The frontend forwards the driver's version check rather than restating a
    /// floor of its own, which is the point of there being one owner.
    #[test]
    fn a_deployment_minimum_below_its_language_floor_is_refused() {
        assert_eq!(
            stated_delivery(policy(vec![selected(ApplePlatform::MacOs, 13, 0)]))
                .expect_err("MSL 3.1 requires macOS 14.0"),
            DeliveryRefusal::InvalidSelection(FamilySelectionError::InvalidTarget {
                source: MetalTargetError::DeploymentMinimumTooLow {
                    platform: ApplePlatform::MacOs,
                    language: MslVersion::Metal3_1,
                    requested: DeploymentMinimum::new(13, 0),
                    required: DeploymentMinimum::new(14, 0),
                },
            }),
        );
    }

    /// The frontend reads one canonical value: declaration order is
    /// presentation, and the identity bytes are the driver's.
    ///
    /// Stating the same two families in either order has to yield one subject,
    /// or two invocations meaning the same thing would be two artifacts.
    #[test]
    fn declaration_order_does_not_change_what_the_frontend_states() {
        let forward = ArtifactFamilySelection::new(policy(vec![
            selected(ApplePlatform::MacOs, 14, 0),
            selected(ApplePlatform::IOsDevice, 17, 0),
        ]))
        .expect("the selection is valid");
        let reversed = ArtifactFamilySelection::new(policy(vec![
            selected(ApplePlatform::IOsDevice, 17, 0),
            selected(ApplePlatform::MacOs, 14, 0),
        ]))
        .expect("the selection is valid");
        assert_eq!(forward, reversed);
        assert_eq!(forward.canonical_bytes(), reversed.canonical_bytes());
        assert_eq!(
            forward
                .compile_targets()
                .expect("both families resolve")
                .len(),
            2,
            "two families remain two compilations after canonicalization",
        );
    }
}
