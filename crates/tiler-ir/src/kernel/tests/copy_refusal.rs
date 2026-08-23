//! The kernel-layer refusal wall in front of the partitioned-copy region.
//!
//! A verified partitioned-copy region is constructible — the intrinsic
//! schedule verifier accepts it as a carrier — while its guarded-store body,
//! bit-preserving evidence, and identity rows are a separate accepted boundary
//! (`lower-the-partitioned-copy-region-through-kernel-ir`). Two typed refusals
//! keep it out of kernel IR, and both are watched here, in the idiom the
//! schedule builder's `partitioned-copy-*` rule tests use: drive the subject
//! in and assert the stable rule identifier one for one.
//!
//! Two further raise sites of the same diagnostic are deliberately **not**
//! tested, because neither is reachable and a test over an unreachable check
//! reads as coverage while providing none:
//!
//! - the `LogicalAccess::PartitionedCopySource` arm of `lower::addressing`.
//!   `addressing` has exactly one call site, inside `plan`, downstream of the
//!   `RegionProgram::Numerical` binding this module's first test watches — so
//!   no copy region reaches it. An arithmetic region cannot carry the map
//!   either: every scalar family binds its reads to one named relation, and
//!   none of them is this one — the pointwise admissibility gate answers
//!   `false` for `LogicalAccess::PartitionedCopySource`, and the reduction,
//!   contraction, and strict-affine gates each require their own variant by
//!   name. Reaching the arm would take a second caller of `addressing` outside
//!   `plan`, or an arithmetic family admitting the copy-source read.
//! - `model::push_requirements`' `RegionNumericalRequirements` binding. Kernel
//!   identity is encoded only from `KernelBuilder::build`, and only after
//!   `verify_kernel` returned `Ok` — which this module's second test shows a
//!   copy region cannot do. Nor can an arithmetic region smuggle the copy arm
//!   in through its *declared* requirements: `verify_kernel` proves the
//!   declaration equals the region's derived requirements, and an arithmetic
//!   region derives `FloatingPoint`, so the mismatch is refused as
//!   `resource-requirements` before any byte is encoded. Reaching it would
//!   take an encoding path that does not run whole-kernel verification first,
//!   or a region deriving the copy arm that `verify_signature` admits.

use super::super::{
    AddressSpace, BufferAccess, BufferParameter, KernelBuilder, KernelType, lower_scheduled_region,
};
use super::support::{linear_schedule, numerical};
use crate::schedule::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, BoundsWitnessId, CopyElement,
    CopyMember, LogicalAccess, OwnershipProof, OwnershipProofKind, OwnershipWitnessId,
    PartitionedCopyProgram, RegionId, RegionNumericalRequirements, RegionProgram,
    ScheduledRegionBuilder, TensorRole, VerifiedScheduledRegion,
};
use crate::shape::{Axis, Shape};

/// The stable rule identifier both refusals below must name.
const RULE: &str = "unlowered-region-program";

/// A verified arity-2 partitioned copy: `concat(a, b)` on axis 0 of `[4, 5]`.
///
/// The kernel-layer twin of the schedule builder's own copy fixture. Two reads
/// carrying the fieldless copy-source map with `LinearRange` proofs of their
/// member-derived source element counts, and one owning `LinearIdentity` write.
fn partitioned_copy_region() -> VerifiedScheduledRegion {
    let shape = Shape::from_dims([4, 5]);
    let elements = 20;
    let members = [(0_u32, 1_u64), (1, 3)];
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
    builder.iteration_shape(shape).unwrap();
    for (source, _) in members {
        builder
            .push_access(Access {
                tensor: TensorRole::Input,
                component_role: None,
                mode: AccessMode::Read,
                map: LogicalAccess::PartitionedCopySource,
                bounds: BoundsWitnessId::new(source),
                ownership: None,
            })
            .unwrap();
    }
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(2),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    for (source, extent) in members {
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(source),
                tensor: TensorRole::Input,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    // The member's own slab of the concatenated domain: its
                    // axis-0 extent against the unpartitioned axis-1 extent.
                    element_count: extent * 5,
                },
            })
            .unwrap();
    }
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(2),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: elements,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: elements,
            },
        })
        .unwrap();
    builder
        .program(RegionProgram::PartitionedCopy(PartitionedCopyProgram {
            element: CopyElement::F32,
            axis: Axis::new(0),
            members: members
                .iter()
                .map(|(source, extent)| CopyMember {
                    source: AccessOrdinal::new(*source),
                    extent: *extent,
                })
                .collect(),
        }))
        .unwrap();
    builder
        .schedule(linear_schedule(elements, OwnershipWitnessId::new(0)))
        .unwrap();
    builder.build().expect("the arity-2 copy fixture verifies")
}

/// The fixture is the subject the two refusals below are supposed to meet: a
/// region intrinsic verification accepted, carrying the copy program and the
/// proved-absent numerical requirement.
#[test]
fn the_fixture_is_a_verified_partitioned_copy_region() {
    let scheduled = partitioned_copy_region();
    assert!(matches!(
        scheduled.region().index.program,
        RegionProgram::PartitionedCopy(_)
    ));
    assert_eq!(
        scheduled.requirements().numerical,
        RegionNumericalRequirements::BitPreservingCopy
    );
}

/// Canonical lowering refuses the copy region by name, before any body.
///
/// `plan` is the derivation `lower_scheduled_region`, `derive_canonical`, and
/// the refinement gate all share, so this one refusal covers all three.
#[test]
fn canonical_lowering_refuses_the_copy_region_program() {
    let scheduled = partitioned_copy_region();
    // The success value is discarded before `expect_err` formats it: a whole
    // verified kernel in a panic message buries the failure text a perturbation
    // run is read for.
    let error = lower_scheduled_region(&scheduled)
        .map(|_| ())
        .expect_err("the copy region has no body");
    assert_eq!(error.rule(), RULE, "{error:?}");
}

/// Whole-kernel verification refuses a producer kernel opened against the copy
/// region, before any buffer is compared.
///
/// The signature is otherwise well formed — one buffer per read plus the write,
/// which is the count `buffer-contract` demands — so the diagnostic is the
/// region-program refusal and not a signature-width defect standing in for it.
#[test]
fn whole_kernel_verification_refuses_the_copy_region_program() {
    let scheduled = partitioned_copy_region();
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    for (access, element_count) in [
        (BufferAccess::Read, 5),
        (BufferAccess::Read, 15),
        (BufferAccess::Write, 20),
    ] {
        builder
            .declare_buffer(BufferParameter {
                tensor: if matches!(access, BufferAccess::Write) {
                    TensorRole::Output
                } else {
                    TensorRole::Input
                },
                component_role: None,
                element_type: KernelType::F32,
                address_space: AddressSpace::Device,
                access,
                element_count,
            })
            .unwrap();
    }
    builder.numerical(numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    let diagnostics = builder.build().map(|_| ()).unwrap_err().into_parts().1;
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].rule(), RULE, "{diagnostics:?}");
}
