//! A decoded binding's target, component, and access type are re-proven.

use super::super::super::model::BindingTargetData;
use super::super::error::{ArtifactCodecError, OrderedSubject};
use super::support::reject_forged;
use tiler_ir::kernel::KernelType;
use tiler_ir::program::StorageScalar;
use tiler_ir::semantic::{InputKey, OutputKey};

/// A binding target naming an interface entry the artifact does not declare is refused.
///
/// Framing, every digest, and the re-derived identity all still agree here — the
/// forged name is folded into the identity the decoder recomputes — so this is
/// the check that catches it, and it is the reason a name is validated even
/// though the correspondence behind it cannot be.
#[test]
fn a_binding_target_naming_an_undeclared_interface_entry_is_rejected() {
    assert_eq!(
        reject_forged(|envelope| {
            envelope.variants[0].entries[0].bindings[0].target =
                BindingTargetData::ProgramInput(InputKey::new("absent").unwrap());
        }),
        ArtifactCodecError::UnknownBindingTargetKey {
            key: "absent".to_owned(),
            input: true,
        },
    );
    assert_eq!(
        reject_forged(|envelope| {
            envelope.variants[0].entries[0].bindings[1].target =
                BindingTargetData::ProgramOutput(vec![OutputKey::new("absent").unwrap()]);
        }),
        ArtifactCodecError::UnknownBindingTargetKey {
            key: "absent".to_owned(),
            input: false,
        },
    );
}

/// A binding addressing output storage under no name at all is refused.
#[test]
fn a_binding_target_that_names_no_output_is_rejected() {
    assert_eq!(
        reject_forged(|envelope| {
            envelope.variants[0].entries[0].bindings[1].target =
                BindingTargetData::ProgramOutput(Vec::new());
        }),
        ArtifactCodecError::EmptyBindingTarget,
    );
}

/// A binding target's output names are a set, and a repeat is refused.
#[test]
fn a_repeated_binding_target_name_is_rejected() {
    let result = OutputKey::new("result").unwrap();
    assert_eq!(
        reject_forged(|envelope| {
            envelope.variants[0].entries[0].bindings[1].target =
                BindingTargetData::ProgramOutput(vec![result.clone(), result.clone()]);
        }),
        ArtifactCodecError::DuplicateItem {
            subject: OrderedSubject::BindingTargetKey,
        },
    );
}

/// A self-consistent binding still cannot contradict the physical component it targets.
#[test]
fn a_binding_with_the_wrong_target_component_storage_is_rejected() {
    assert_eq!(
        reject_forged(|envelope| {
            let binding = &mut envelope.variants[0].entries[0].bindings[0];
            binding.storage_scalar = StorageScalar::U8;
            binding.access_type = KernelType::U8;
        }),
        ArtifactCodecError::BindingComponentMismatch,
    );
}

/// A binding cannot ask a kernel to access an unpacked carrier through another type.
#[test]
fn an_incompatible_binding_access_type_is_rejected() {
    assert_eq!(
        reject_forged(|envelope| {
            envelope.variants[0].entries[0].bindings[0].access_type = KernelType::Bool;
        }),
        ArtifactCodecError::BindingAccessTypeMismatch,
    );
}

/// The two-byte carrier is refused against the four-byte access type.
///
/// This is the misread `check_binding_access` exists to stop, stated at the
/// widest gap the carrier vocabulary now admits: a `Bf16` carrier read as `F32`
/// addresses twice the bytes the interface provides, and every framing, digest,
/// and identity check passes on the way there.
///
/// The second case is what makes the first about the *pair* rather than about
/// `Bf16` being refused wherever it appears. Pairing `Bf16` with `Bf16` clears
/// this check and is stopped one step later by the component it contradicts, so
/// reaching `BindingComponentMismatch` is the observation that the access check
/// admitted the matched pair.
#[test]
fn a_bf16_carrier_is_refused_against_a_wider_access_type_and_admitted_against_its_own() {
    assert_eq!(
        reject_forged(|envelope| {
            envelope.variants[0].entries[0].bindings[0].storage_scalar = StorageScalar::Bf16;
        }),
        ArtifactCodecError::BindingAccessTypeMismatch,
    );
    assert_eq!(
        reject_forged(|envelope| {
            let binding = &mut envelope.variants[0].entries[0].bindings[0];
            binding.storage_scalar = StorageScalar::Bf16;
            binding.access_type = KernelType::Bf16;
        }),
        ArtifactCodecError::BindingComponentMismatch,
    );
}
