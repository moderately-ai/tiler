//! Canonical target-neutral lowering of a verified scheduled region.
//!
//! This is the authoritative structured realization of the bounded profile. It
//! makes every fact a backend needs explicit in the IR itself: the guarded
//! iteration predicate, the exact element-offset arithmetic that realizes the
//! scheduled logical access, the typed loads and stores with their schedule
//! witnesses, the named NaN-canonicalization conversion the numerical contract
//! requires, and — for a reduction — a bounded loop carrying the accumulator in
//! the exact scheduled contributor order. No backend has to consult the
//! semantic graph, re-derive an access relation, or infer a reduction order.
//!
//! The lowering constructs its kernel through the same public
//! [`KernelBuilder`] path an external producer uses, so it cannot bypass an
//! insertion-time invariant. [`super::verify`] re-derives this canonical body
//! and requires structural equality, which is what makes a producer-authored
//! kernel a proven refinement rather than a trusted one.

use crate::schedule::{
    Access, BoundsWitnessId, CanonicalScheduledRegionIdentity, LogicalAccess, NumericalRealization,
    OwnershipWitnessId, ReductionTopology, ResourceRequirements, ScalarProgram, ScheduledRegion,
    TensorRole, VerifiedScheduledRegion, contributor_count,
};
use crate::shape::Shape;

use super::builder::KernelBuilder;
use super::error::{KernelBuildError, KernelDiagnostic, KernelLoweringError};
use super::handles::{KernelBufferId, KernelValueId};
use super::model::{
    AddressSpace, BinaryOp, BufferAccess, BufferParameter, Builtin, CompareOp, ConvertOp,
    KernelConstant, KernelData, KernelType, SerialLoopSpec, VerifiedKernel,
};
use super::verify::{access_elements, boundary_accesses};

/// Which root index a linearization term extracts its coordinate from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OffsetRoot {
    /// The linear output coordinate carried by the global invocation index.
    Output,
    /// The linear contributor coordinate carried by the loop induction variable.
    Contributor,
}

/// One `stride * ((root / divisor) % modulus)` term of a linearized offset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OffsetTerm {
    root: OffsetRoot,
    divisor: u64,
    modulus: Option<u64>,
    stride: u64,
}

/// How the read access computes its element offset.
#[derive(Clone, Debug, Eq, PartialEq)]
enum ReadAddressing {
    /// One iteration coordinate addresses one linear element position.
    Identity,
    /// A reduction contributor position linearized over the input shape.
    Linearized(Vec<OffsetTerm>),
}

/// Everything the canonical emission needs, resolved before any operation.
#[derive(Clone, Debug)]
struct CanonicalPlan<'a> {
    scalar: &'a ScalarProgram,
    numerical: NumericalRealization,
    read_tensor: TensorRole,
    write_tensor: TensorRole,
    read_elements: u64,
    write_elements: u64,
    work_items: u64,
    read_bounds: BoundsWitnessId,
    write_bounds: BoundsWitnessId,
    ownership: OwnershipWitnessId,
    contributors: u64,
    addressing: ReadAddressing,
}

/// Lowers one verified scheduled region to its canonical verified kernel.
///
/// # Errors
///
/// Returns [`KernelLoweringError`] when the region is outside the lowered
/// structured-kernel profile, when an operation cannot be inserted, or when the
/// resulting kernel fails whole-kernel verification.
pub fn lower_scheduled_region(
    scheduled: &VerifiedScheduledRegion,
) -> Result<VerifiedKernel, KernelLoweringError> {
    let plan = plan(scheduled.region()).map_err(KernelLoweringError::Verification)?;
    let mut builder = KernelBuilder::new(scheduled)?;
    emit(&mut builder, &plan, scheduled.requirements())?;
    builder.build().map_err(|error| {
        KernelLoweringError::Verification(
            error
                .diagnostics()
                .first()
                .copied()
                .unwrap_or(KernelDiagnostic::BodyRefinement),
        )
    })
}

/// Derives the canonical structured body of one scheduled region.
///
/// This is the reference the refinement gate compares a producer's kernel
/// against; it deliberately stops before whole-kernel verification so the gate
/// cannot recurse into itself.
pub(super) fn derive_canonical(
    schedule: &ScheduledRegion,
    schedule_identity: &CanonicalScheduledRegionIdentity,
    requirements: ResourceRequirements,
) -> Result<KernelData, KernelDiagnostic> {
    let plan = plan(schedule)?;
    let mut builder =
        KernelBuilder::from_parts(schedule.clone(), schedule_identity.clone(), requirements)
            .map_err(|_| KernelDiagnostic::BodyRefinement)?;
    emit(&mut builder, &plan, requirements).map_err(|error| match error {
        KernelLoweringError::Verification(diagnostic) => diagnostic,
        KernelLoweringError::Construction(_) | KernelLoweringError::UnsupportedRegion { .. } => {
            KernelDiagnostic::BodyRefinement
        }
    })?;
    builder.into_data()
}

fn plan(schedule: &ScheduledRegion) -> Result<CanonicalPlan<'_>, KernelDiagnostic> {
    let (read, write) = boundary_accesses(schedule)?;
    let contributors = match &schedule.schedule.reduction {
        ReductionTopology::None => 0,
        ReductionTopology::Serial { axes, .. } => {
            contributor_count(axes, &read.map).map_err(|_| KernelDiagnostic::ContributorDomain)?
        }
    };
    Ok(CanonicalPlan {
        scalar: &schedule.index.scalar_program,
        numerical: schedule.index.numerical,
        read_tensor: read.tensor,
        write_tensor: write.tensor,
        read_elements: access_elements(read, schedule)?,
        write_elements: access_elements(write, schedule)?,
        work_items: schedule.schedule.work_items,
        read_bounds: read.bounds,
        write_bounds: write.bounds,
        ownership: schedule.schedule.output_owner,
        contributors,
        addressing: addressing(read)?,
    })
}

/// Resolves how the read access computes its element offset.
fn addressing(read: &Access) -> Result<ReadAddressing, KernelDiagnostic> {
    match &read.map {
        LogicalAccess::LinearIdentity => Ok(ReadAddressing::Identity),
        LogicalAccess::ReductionContributor {
            input_shape, axes, ..
        } => {
            let reduced: Vec<usize> = axes
                .iter()
                .map(|axis| usize::try_from(axis.get()).unwrap_or(usize::MAX))
                .collect();
            if reduced.iter().any(|axis| *axis >= input_shape.rank()) {
                return Err(KernelDiagnostic::ContributorDomain);
            }
            Ok(ReadAddressing::Linearized(linearize(input_shape, &reduced)))
        }
    }
}

/// Builds the ordered row-major linearization terms of a contributor access.
///
/// Each input axis contributes one term. A kept axis extracts its coordinate
/// from the linear output index and a reduced axis from the linear contributor
/// index, each using the suffix products of its own sub-shape. A term whose
/// extent is one, or whose divisor, modulus, or stride is zero, is dropped: the
/// coordinate is then constantly zero, or the whole iteration domain is empty
/// and the guarded block never executes.
fn linearize(input_shape: &Shape, reduced: &[usize]) -> Vec<OffsetTerm> {
    let extents: Vec<u64> = input_shape
        .extents()
        .iter()
        .map(|extent| extent.get())
        .collect();
    let strides = suffix_products(&extents);
    let kept: Vec<usize> = (0..extents.len())
        .filter(|axis| !reduced.contains(axis))
        .collect();
    let kept_extents: Vec<u64> = kept.iter().map(|axis| extents[*axis]).collect();
    let reduced_extents: Vec<u64> = reduced.iter().map(|axis| extents[*axis]).collect();
    let kept_suffix = suffix_products(&kept_extents);
    let reduced_suffix = suffix_products(&reduced_extents);

    let mut terms = Vec::new();
    for (axis, extent) in extents.iter().copied().enumerate() {
        let (root, position, sub_extents, sub_suffix) =
            if let Some(position) = reduced.iter().position(|reduced| *reduced == axis) {
                (
                    OffsetRoot::Contributor,
                    position,
                    &reduced_extents,
                    &reduced_suffix,
                )
            } else if let Some(position) = kept.iter().position(|kept| *kept == axis) {
                (OffsetRoot::Output, position, &kept_extents, &kept_suffix)
            } else {
                continue;
            };
        let divisor = sub_suffix[position];
        let modulus = (position > 0).then(|| sub_extents[position]);
        let stride = strides[axis];
        if extent == 1 || divisor == 0 || modulus == Some(0) || stride == 0 {
            continue;
        }
        terms.push(OffsetTerm {
            root,
            divisor,
            modulus,
            stride,
        });
    }
    terms
}

/// Returns the product of every later extent, saturating an overflow to zero.
///
/// An overflowing suffix product can only occur when some extent is zero, which
/// makes the whole domain empty; zero then drops the affected term.
fn suffix_products(extents: &[u64]) -> Vec<u64> {
    let mut products = vec![1_u64; extents.len()];
    for index in (0..extents.len()).rev() {
        let next = products.get(index + 1).copied().unwrap_or(1);
        let extent = extents.get(index + 1).copied().unwrap_or(1);
        products[index] = next.checked_mul(extent).unwrap_or(0);
    }
    products
}

fn emit(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    requirements: ResourceRequirements,
) -> Result<(), KernelLoweringError> {
    let read_buffer = builder.declare_buffer(BufferParameter {
        tensor: plan.read_tensor,
        element_type: KernelType::F32,
        address_space: AddressSpace::Device,
        access: BufferAccess::Read,
        element_count: plan.read_elements,
    })?;
    let write_buffer = builder.declare_buffer(BufferParameter {
        tensor: plan.write_tensor,
        element_type: KernelType::F32,
        address_space: AddressSpace::Device,
        access: BufferAccess::Write,
        element_count: plan.write_elements,
    })?;
    builder.admit_builtin(Builtin::GlobalInvocationIndex)?;
    builder.numerical(plan.numerical)?;
    builder.requirements(requirements)?;

    let invocation = builder.builtin(Builtin::GlobalInvocationIndex)?;
    let extent = builder.constant(KernelConstant::Index(plan.work_items))?;
    let active = builder.compare(CompareOp::IndexLessThan, invocation, extent)?;
    builder.predicated(active, |builder| {
        emit_guarded(builder, plan, read_buffer, write_buffer, invocation)
    })?;
    Ok(())
}

fn emit_guarded(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    read_buffer: KernelBufferId,
    write_buffer: KernelBufferId,
    invocation: KernelValueId,
) -> Result<(), KernelBuildError> {
    match plan.scalar {
        ScalarProgram::MultiplyThenAdd {
            scale_bits,
            bias_bits,
            ..
        } => {
            let loaded = builder.load(read_buffer, invocation, plan.read_bounds)?;
            let mapped = emit_scale_bias(builder, loaded, *scale_bits, *bias_bits)?;
            builder.store(
                write_buffer,
                invocation,
                mapped,
                plan.write_bounds,
                plan.ownership,
            )
        }
        ScalarProgram::StrictSerialSum {
            empty_identity_bits,
            ..
        } => emit_reduction(
            builder,
            plan,
            read_buffer,
            write_buffer,
            invocation,
            *empty_identity_bits,
            None,
        ),
        ScalarProgram::FusedMultiplyAddSerialSum {
            scale_bits,
            bias_bits,
            empty_identity_bits,
            ..
        } => emit_reduction(
            builder,
            plan,
            read_buffer,
            write_buffer,
            invocation,
            *empty_identity_bits,
            Some((*scale_bits, *bias_bits)),
        ),
    }
}

fn emit_scale_bias(
    builder: &mut KernelBuilder,
    value: KernelValueId,
    scale_bits: u32,
    bias_bits: u32,
) -> Result<KernelValueId, KernelBuildError> {
    let scale = builder.constant(KernelConstant::F32Bits(scale_bits))?;
    let product = builder.binary(BinaryOp::F32Multiply, value, scale)?;
    let product = builder.convert(ConvertOp::CanonicalizeF32Nan, product)?;
    let bias = builder.constant(KernelConstant::F32Bits(bias_bits))?;
    let biased = builder.binary(BinaryOp::F32Add, product, bias)?;
    builder.convert(ConvertOp::CanonicalizeF32Nan, biased)
}

fn emit_reduction(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    read_buffer: KernelBufferId,
    write_buffer: KernelBufferId,
    invocation: KernelValueId,
    empty_identity_bits: u32,
    prologue: Option<(u32, u32)>,
) -> Result<(), KernelBuildError> {
    if plan.contributors == 0 {
        let identity = builder.constant(KernelConstant::F32Bits(empty_identity_bits))?;
        return builder.store(
            write_buffer,
            invocation,
            identity,
            plan.write_bounds,
            plan.ownership,
        );
    }
    let first_offset = emit_offset(builder, plan, invocation, None)?;
    let first = builder.load(read_buffer, first_offset, plan.read_bounds)?;
    let seed = match prologue {
        Some((scale_bits, bias_bits)) => emit_scale_bias(builder, first, scale_bits, bias_bits)?,
        None => first,
    };
    // A single contributor supplies the whole strict-serial value, but the
    // reduction still canonicalizes at its result boundary: ADR 0055 and the
    // numerical contract both require that boundary rule "even when the
    // contributor sequence is a singleton", so an uncombined input payload
    // cannot leak its NaN bits through an arithmetic reduction.
    //
    // The conversion is what realizes the rule here. Emitting a loop would need
    // an empty iteration range, and combining with the reduction identity would
    // change the observable sign of a negative zero, whereas canonicalization
    // rewrites a NaN and leaves every other payload — including `-0.0` — alone.
    //
    // It is emitted exactly where the boundary value is an uncombined input,
    // which is the leak the rule names. The fold already applies the conversion
    // after each combine, and a prologue already applies it to the scaled seed,
    // so those boundaries are canonical without a second one.
    let total = if plan.contributors == 1 {
        match prologue {
            Some(_) => seed,
            None => builder.convert(ConvertOp::CanonicalizeF32Nan, seed)?,
        }
    } else {
        let results = builder.serial_loop(
            SerialLoopSpec {
                start: 1,
                end: plan.contributors,
            },
            &[seed],
            |builder, parameters| {
                let induction = parameters.induction();
                let accumulator = parameters
                    .accumulator(0)
                    .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
                let offset = emit_offset(builder, plan, invocation, Some(induction))?;
                let loaded = builder.load(read_buffer, offset, plan.read_bounds)?;
                let contributor = match prologue {
                    Some((scale_bits, bias_bits)) => {
                        emit_scale_bias(builder, loaded, scale_bits, bias_bits)?
                    }
                    None => loaded,
                };
                let sum = builder.binary(BinaryOp::F32Add, accumulator, contributor)?;
                let sum = builder.convert(ConvertOp::CanonicalizeF32Nan, sum)?;
                Ok(vec![sum])
            },
        )?;
        results
            .get(0)
            .ok_or(KernelBuildError::EmptyLoopAccumulators)?
    };
    builder.store(
        write_buffer,
        invocation,
        total,
        plan.write_bounds,
        plan.ownership,
    )
}

/// Emits the element offset of one read access.
///
/// `contributor` is `None` for the seed load, whose contributor coordinate is
/// zero; every contributor term then vanishes exactly.
fn emit_offset(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    invocation: KernelValueId,
    contributor: Option<KernelValueId>,
) -> Result<KernelValueId, KernelBuildError> {
    let terms = match &plan.addressing {
        ReadAddressing::Identity => return Ok(invocation),
        ReadAddressing::Linearized(terms) => terms,
    };
    let mut total: Option<KernelValueId> = None;
    for term in terms {
        let root = match term.root {
            OffsetRoot::Output => invocation,
            OffsetRoot::Contributor => match contributor {
                Some(value) => value,
                None => continue,
            },
        };
        let mut value = root;
        if term.divisor > 1 {
            let divisor = builder.constant(KernelConstant::Index(term.divisor))?;
            value = builder.binary(BinaryOp::IndexDivide, value, divisor)?;
        }
        if let Some(modulus) = term.modulus {
            let modulus = builder.constant(KernelConstant::Index(modulus))?;
            value = builder.binary(BinaryOp::IndexModulo, value, modulus)?;
        }
        if term.stride > 1 {
            let stride = builder.constant(KernelConstant::Index(term.stride))?;
            value = builder.binary(BinaryOp::IndexMultiply, value, stride)?;
        }
        total = Some(match total {
            Some(accumulated) => builder.binary(BinaryOp::IndexAdd, accumulated, value)?,
            None => value,
        });
    }
    match total {
        Some(value) => Ok(value),
        None => builder.constant(KernelConstant::Index(0)),
    }
}
