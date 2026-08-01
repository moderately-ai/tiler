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
    OwnershipWitnessId, PointwiseF32Expression, PointwiseF32Node, ReductionPass, ReductionTopology,
    ResourceRequirements, ScalarProgram, ScheduledRegion, TensorRole, VerifiedScheduledRegion,
    contributor_count,
};
use crate::shape::Shape;

use super::builder::KernelBuilder;
use super::error::{KernelBuildError, KernelDiagnostic, KernelLoweringError};
use super::handles::{KernelBufferId, KernelValueId};
use super::model::{
    AddressSpace, BinaryOp, BufferAccess, BufferParameter, Builtin, CompareOp, ConvertOp,
    KernelConstant, KernelData, KernelType, PackedExtractOp, SerialLoopSpec, UnaryOp,
    VerifiedKernel,
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
    /// A partitioned contributor position of one pass of a split reduction.
    ///
    /// The invocation index carries the output coordinate *and* the partition
    /// ordinal, because the partial pass runs one invocation per
    /// (output, partition) pair. Splitting them here is what keeps the shared
    /// linearization below unchanged: it still receives one linear output
    /// coordinate and one linear contributor coordinate.
    Partitioned {
        /// Row-major terms over the read tensor's own shape.
        terms: Vec<OffsetTerm>,
        /// Partial values per output position.
        partitions: u64,
        /// Contributors each partition combines.
        contributors_per_partition: u64,
    },
}

/// Everything the canonical emission needs, resolved before any operation.
///
/// `reads`, `read_elements`, and `addressing` are parallel and complete: a
/// pointwise region declares one buffer per read and a contraction two with
/// *different* coordinate maps, so a plan carrying only the first read's facts
/// would emit a signature narrower than the region it lowers and address the
/// second operand by the first's relation. `contributors` is the fold length one
/// invocation performs, and is zero for the families that fold nothing.
#[derive(Clone, Debug)]
struct CanonicalPlan<'a> {
    scalar: &'a ScalarProgram,
    reads: &'a [Access],
    write: &'a Access,
    numerical: NumericalRealization,
    write_tensor: TensorRole,
    read_elements: Vec<u64>,
    write_elements: u64,
    work_items: u64,
    write_bounds: BoundsWitnessId,
    ownership: OwnershipWitnessId,
    contributors: u64,
    addressing: Vec<ReadAddressing>,
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
    let (reads, write) = boundary_accesses(schedule)?;
    let read = reads.first().ok_or(KernelDiagnostic::ScheduleAccessCount)?;
    // The contributors *one invocation* combines. For a partial pass that is
    // its own partition's share, not the whole reduction's sequence, which is
    // exactly the difference the split exists to create.
    let contributors = match &schedule.schedule.reduction {
        ReductionTopology::None => 0,
        ReductionTopology::Serial { axes, .. }
        | ReductionTopology::MultiPass {
            pass: ReductionPass::Final,
            axes,
            ..
        } => contributor_count(axes, &read.map).map_err(|_| KernelDiagnostic::ContributorDomain)?,
        ReductionTopology::MultiPass {
            pass: ReductionPass::Partial,
            partition,
            ..
        } => partition.contributors_per_partition,
        // The contracted index space, which the topology states because no
        // single operand's map determines it.
        ReductionTopology::Contraction {
            contracted_shape, ..
        } => crate::schedule::element_count(contracted_shape)
            .map_err(|_| KernelDiagnostic::ElementCountOverflow)?,
        // No canonical body exists for a cooperative tile, and refusing here is
        // what keeps that true. Emitting one would mean emitting the staged
        // handoff its phases describe, which is correct only when something
        // orders the producing phase before the consuming one -- and the
        // barrier vocabulary is refused intrinsically, so nothing can. A body
        // that staged and re-read without that ordering would be a race this
        // lowering had authored, so the region is refused before any operation
        // is inserted rather than lowered into one.
        ReductionTopology::CooperativeWorkgroup { .. } => {
            return Err(KernelDiagnostic::UndischargedVisibility);
        }
    };
    // The strict-affine decode addresses its three role-scoped components by the
    // invocation index directly, so it consults no coordinate map.
    let addressing = if matches!(
        &schedule.index.scalar_program,
        ScalarProgram::StrictAffineU4Dequantize { .. }
    ) {
        vec![ReadAddressing::Identity; reads.len()]
    } else {
        reads
            .iter()
            .map(|read| addressing(read, &schedule.schedule.reduction))
            .collect::<Result<Vec<_>, _>>()?
    };
    Ok(CanonicalPlan {
        scalar: &schedule.index.scalar_program,
        reads,
        write,
        numerical: schedule.index.numerical,
        write_tensor: write.tensor,
        read_elements: reads
            .iter()
            .map(|read| access_elements(read, schedule))
            .collect::<Result<Vec<_>, _>>()?,
        write_elements: access_elements(write, schedule)?,
        work_items: schedule.schedule.work_items,
        write_bounds: write.bounds,
        ownership: schedule.schedule.output_owner,
        contributors,
        addressing,
    })
}

/// Resolves how the read access computes its element offset.
fn addressing(
    read: &Access,
    reduction: &ReductionTopology,
) -> Result<ReadAddressing, KernelDiagnostic> {
    match &read.map {
        LogicalAccess::LinearIdentity => Ok(ReadAddressing::Identity),
        LogicalAccess::ScalarBroadcast | LogicalAccess::PackedU4LsbZeroTail { .. } => {
            Err(KernelDiagnostic::BodyRefinement)
        }
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
            let terms = linearize(input_shape, &reduced);
            // Only the partial pass splits its invocation index. A final pass
            // runs one invocation per output, exactly as a serial reduction
            // does, so it uses the unsplit form.
            match reduction {
                ReductionTopology::MultiPass {
                    pass: ReductionPass::Partial,
                    partition,
                    ..
                } => Ok(ReadAddressing::Partitioned {
                    terms,
                    partitions: partition.partitions,
                    contributors_per_partition: partition.contributors_per_partition,
                }),
                // The cooperative arm is unreachable from `plan`, which
                // refuses the topology before resolving any addressing, and it
                // is spelled rather than wildcarded so a later reachable path
                // is a build error here instead of silently addressing a tile
                // by the unsplit relation.
                ReductionTopology::CooperativeWorkgroup { .. } => {
                    Err(KernelDiagnostic::UndischargedVisibility)
                }
                ReductionTopology::None
                | ReductionTopology::Serial { .. }
                | ReductionTopology::Contraction { .. }
                | ReductionTopology::MultiPass { .. } => Ok(ReadAddressing::Linearized(terms)),
            }
        }
        LogicalAccess::ContractionOperand {
            operand_shape,
            output_shape,
            contracted_shape,
            sources,
            ..
        } => Ok(ReadAddressing::Linearized(linearize_contraction_operand(
            operand_shape,
            output_shape,
            contracted_shape,
            sources,
        )?)),
    }
}

/// Builds the row-major linearization terms of one contraction operand access.
///
/// Each operand axis contributes one term. The axis's coordinate is decoded from
/// whichever linear index the schedule verifier proved it names — the invocation
/// index for an output coordinate, the loop induction variable for a contracted
/// one — using the suffix products of *that* space, and is scaled by the
/// operand's own row-major stride. The leading position of a space needs no
/// wrap, because the linear index is already below the product of every extent
/// in it. A term whose extent is one, or whose divisor, modulus, or stride is
/// zero, is dropped: the coordinate is then constantly zero, or the domain is
/// empty and the guarded block never executes.
fn linearize_contraction_operand(
    operand_shape: &Shape,
    output_shape: &Shape,
    contracted_shape: &Shape,
    sources: &[crate::schedule::ContractionAxisSource],
) -> Result<Vec<OffsetTerm>, KernelDiagnostic> {
    let extents_of =
        |shape: &Shape| -> Vec<u64> { shape.extents().iter().map(|extent| extent.get()).collect() };
    let operand_extents = extents_of(operand_shape);
    let operand_strides = suffix_products(&operand_extents);
    let output_extents = extents_of(output_shape);
    let output_suffix = suffix_products(&output_extents);
    let contracted_extents = extents_of(contracted_shape);
    let contracted_suffix = suffix_products(&contracted_extents);
    if sources.len() != operand_extents.len() {
        return Err(KernelDiagnostic::ContributorDomain);
    }

    let mut terms = Vec::with_capacity(sources.len());
    for (axis, source) in sources.iter().enumerate() {
        let (root, position, sub_extents, sub_suffix) = match source {
            crate::schedule::ContractionAxisSource::Output { position } => (
                OffsetRoot::Output,
                *position,
                &output_extents,
                &output_suffix,
            ),
            crate::schedule::ContractionAxisSource::Contracted { position } => (
                OffsetRoot::Contributor,
                *position,
                &contracted_extents,
                &contracted_suffix,
            ),
        };
        let position =
            usize::try_from(position).map_err(|_| KernelDiagnostic::ContributorDomain)?;
        let (Some(divisor), Some(sub_extent)) =
            (sub_suffix.get(position), sub_extents.get(position))
        else {
            return Err(KernelDiagnostic::ContributorDomain);
        };
        let modulus = (position > 0).then_some(*sub_extent);
        let stride = operand_strides[axis];
        if operand_extents[axis] == 1 || *divisor == 0 || modulus == Some(0) || stride == 0 {
            continue;
        }
        terms.push(OffsetTerm {
            root,
            divisor: *divisor,
            modulus,
            stride,
        });
    }
    Ok(terms)
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
    if matches!(plan.scalar, ScalarProgram::StrictAffineU4Dequantize { .. }) {
        return emit_strict_affine_u4_dequantize(builder, plan, requirements);
    }
    // One buffer per read, in access order. The component role stays `None`
    // rather than being copied from the access: these families read dense
    // values, and `verify_signature` compares the two, so copying it would make
    // that comparison agree with itself instead of checking anything.
    let mut read_buffers = Vec::with_capacity(plan.reads.len());
    for (read, elements) in plan.reads.iter().zip(&plan.read_elements) {
        read_buffers.push(builder.declare_buffer(BufferParameter {
            tensor: read.tensor,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: *elements,
        })?);
    }
    let write_buffer = builder.declare_buffer(BufferParameter {
        tensor: plan.write_tensor,
        component_role: None,
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
        emit_guarded(builder, plan, &read_buffers, write_buffer, invocation)
    })?;
    Ok(())
}

fn emit_guarded(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    read_buffers: &[KernelBufferId],
    write_buffer: KernelBufferId,
    invocation: KernelValueId,
) -> Result<(), KernelBuildError> {
    // The reduction families read exactly one tensor, which the schedule
    // verifier proved before this plan existed. Resolving that here as a typed
    // handle error rather than by indexing keeps a widened region from lowering
    // against whichever buffer happened to be first.
    let sole_read_buffer = || {
        let [buffer] = read_buffers else {
            return Err(KernelBuildError::InvalidHandle {
                entity: super::error::KernelEntityKind::Buffer,
            });
        };
        Ok(*buffer)
    };
    let sole_read = || {
        let [read] = plan.reads else {
            return Err(KernelBuildError::InvalidHandle {
                entity: super::error::KernelEntityKind::Buffer,
            });
        };
        Ok(read.bounds)
    };
    match plan.scalar {
        ScalarProgram::PointwiseF32(expression) => {
            let mut inputs = Vec::with_capacity(read_buffers.len());
            for (buffer, read) in read_buffers.iter().zip(plan.reads) {
                inputs.push(builder.load(*buffer, invocation, read.bounds)?);
            }
            let mapped = emit_pointwise(builder, expression, &inputs)?;
            builder.store(
                write_buffer,
                invocation,
                mapped,
                plan.write_bounds,
                plan.ownership,
            )
        }
        ScalarProgram::StrictAffineU4Dequantize { .. } => {
            unreachable!("strict-affine lowering uses its role-addressed signature")
        }
        ScalarProgram::StrictSerialSum {
            empty_identity_bits,
            ..
        } => emit_reduction(
            builder,
            plan,
            (sole_read_buffer()?, sole_read()?),
            write_buffer,
            invocation,
            *empty_identity_bits,
            ReductionPrologue::None,
        ),
        ScalarProgram::FusedMultiplyAddSerialSum {
            scale_bits,
            bias_bits,
            empty_identity_bits,
            ..
        } => emit_reduction(
            builder,
            plan,
            (sole_read_buffer()?, sole_read()?),
            write_buffer,
            invocation,
            *empty_identity_bits,
            ReductionPrologue::ScaleBias {
                scale_bits: *scale_bits,
                bias_bits: *bias_bits,
            },
        ),
        ScalarProgram::SquaredSerialSum {
            empty_identity_bits,
            ..
        } => emit_reduction(
            builder,
            plan,
            (sole_read_buffer()?, sole_read()?),
            write_buffer,
            invocation,
            *empty_identity_bits,
            ReductionPrologue::Square,
        ),
        ScalarProgram::StrictTensorContraction { .. } => {
            let ([left, right], [left_read, right_read]) = (read_buffers, plan.reads) else {
                return Err(KernelBuildError::InvalidHandle {
                    entity: super::error::KernelEntityKind::Buffer,
                });
            };
            emit_contraction(
                builder,
                plan,
                [(*left, left_read.bounds), (*right, right_read.bounds)],
                write_buffer,
                invocation,
            )
        }
    }
}

/// Emits the guarded body of one strict tensor contraction.
///
/// One thread folds its own output element in ascending contracted order. The
/// accumulator is seeded at the *first product* rather than at `+0.0`: the two
/// differ observably where every product is `-0.0`, and the registered family
/// declares no seed, so an identity-seeded fold would compute a contraction
/// carrying an explicit `initial` — a different operation.
///
/// **The fold is deliberately three separate structured operations per step**: a
/// multiply, a NaN canonicalization, and an add. The canonicalization is the
/// declared `after-every-combine-and-at-the-result-boundary` rule reaching the
/// product, and it is also what makes a fused multiply-add unformable — the
/// backend sees a call between the two arithmetic operations, not an adjacent
/// pair. That matters because the governed contracts forbid ADR 0015 contraction
/// and the measured Apple row shows `-ffp-contract=off` is no defence against a
/// *fused instruction the source asks for*; here the source cannot ask for one.
///
/// **No result-boundary conversion is emitted, and its absence is derived.** The
/// serial sum needs one when its contributor sequence is a singleton, because
/// its seed is a raw load no combine has canonicalized. A contraction's seed is
/// a *product*, which this emission canonicalizes, so every path out of the fold
/// already carries the canonical payload and a second conversion would be a
/// provable identity in a body the refinement gate compares structurally.
fn emit_contraction(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    reads: [(KernelBufferId, BoundsWitnessId); 2],
    write_buffer: KernelBufferId,
    invocation: KernelValueId,
) -> Result<(), KernelBuildError> {
    let seed = emit_contraction_product(builder, plan, reads, invocation, None)?;
    let total = if plan.contributors <= 1 {
        seed
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
                let product =
                    emit_contraction_product(builder, plan, reads, invocation, Some(induction))?;
                let sum = builder.binary(BinaryOp::F32Add, accumulator, product)?;
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

/// Emits one contracted point's separately rounded, canonicalized product.
fn emit_contraction_product(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    reads: [(KernelBufferId, BoundsWitnessId); 2],
    invocation: KernelValueId,
    contributor: Option<KernelValueId>,
) -> Result<KernelValueId, KernelBuildError> {
    let mut loaded = [None, None];
    for (position, (buffer, bounds)) in reads.into_iter().enumerate() {
        let addressing = plan
            .addressing
            .get(position)
            .ok_or(KernelBuildError::InvalidHandle {
                entity: super::error::KernelEntityKind::Buffer,
            })?;
        let offset = emit_offset(builder, addressing, invocation, contributor)?;
        loaded[position] = Some(builder.load(buffer, offset, bounds)?);
    }
    let [Some(left), Some(right)] = loaded else {
        return Err(KernelBuildError::InvalidHandle {
            entity: super::error::KernelEntityKind::Value,
        });
    };
    let product = builder.binary(BinaryOp::F32Multiply, left, right)?;
    builder.convert(ConvertOp::CanonicalizeF32Nan, product)
}

/// The elementwise expression applied to each contributor before the fold.
///
/// A typed enum rather than an `Option<(u32, u32)>`, because there are now two
/// prologues and they are not two constant choices of one shape: the scale-bias
/// form is affine in the contributor and the squaring form is quadratic, so no
/// pair of constants makes one express the other. The exhaustive match at the
/// emission site is what forces a third prologue to state its own arithmetic
/// rather than borrowing whichever of these two it resembles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReductionPrologue {
    /// The contributor enters the fold unchanged.
    None,
    /// `scale * x + bias`, two roundings per contributor.
    ScaleBias {
        /// Scale constant bit pattern.
        scale_bits: u32,
        /// Bias constant bit pattern.
        bias_bits: u32,
    },
    /// `x * x`, one rounding per contributor.
    ///
    /// Emitted as a multiplication of the loaded value by *itself* rather than by
    /// a second load of the same address: one load, one product, and no
    /// assumption that two reads of one element agree.
    Square,
}

fn emit_strict_affine_u4_dequantize(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    requirements: ResourceRequirements,
) -> Result<(), KernelLoweringError> {
    let [codes, scale, zero_point] = plan.reads else {
        return Err(KernelLoweringError::UnsupportedRegion {
            rule: "strict-affine-u4-access-count",
        });
    };
    let codes_buffer = builder.declare_buffer(BufferParameter {
        tensor: codes.tensor,
        component_role: codes.component_role,
        element_type: KernelType::U8,
        address_space: AddressSpace::Device,
        access: BufferAccess::Read,
        element_count: plan.read_elements.first().copied().unwrap_or(0),
    })?;
    let scale_buffer = builder.declare_buffer(BufferParameter {
        tensor: scale.tensor,
        component_role: scale.component_role,
        element_type: KernelType::F32,
        address_space: AddressSpace::Device,
        access: BufferAccess::Read,
        element_count: 1,
    })?;
    let zero_buffer = builder.declare_buffer(BufferParameter {
        tensor: zero_point.tensor,
        component_role: zero_point.component_role,
        element_type: KernelType::U8,
        address_space: AddressSpace::Device,
        access: BufferAccess::Read,
        element_count: 1,
    })?;
    let output_buffer = builder.declare_buffer(BufferParameter {
        tensor: plan.write.tensor,
        component_role: plan.write.component_role,
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
        let two = builder.constant(KernelConstant::Index(2))?;
        let carrier_index = builder.binary(BinaryOp::IndexDivide, invocation, two)?;
        let carrier = builder.load(codes_buffer, carrier_index, codes.bounds)?;
        let code = builder.packed_extract(PackedExtractOp::U4LsbZeroTail, carrier, invocation)?;
        let zero_index = builder.constant(KernelConstant::Index(0))?;
        let scale_value = builder.load(scale_buffer, zero_index, scale.bounds)?;
        let zero = builder.load(zero_buffer, zero_index, zero_point.bounds)?;
        let code = builder.convert(ConvertOp::U8ToI32, code)?;
        let zero = builder.convert(ConvertOp::U8ToI32, zero)?;
        let difference = builder.binary(BinaryOp::I32Subtract, code, zero)?;
        let difference = builder.convert(ConvertOp::I32ToF32, difference)?;
        let result = builder.binary(BinaryOp::F32Multiply, difference, scale_value)?;
        builder.store(
            output_buffer,
            invocation,
            result,
            plan.write.bounds,
            plan.ownership,
        )
    })?;
    Ok(())
}

/// Emits the scalar body of a pointwise expression over its loaded inputs.
///
/// `inputs` is indexed by the leaf's own ordinal, not by the order the leaves
/// appear: canonicalization orders nodes by root-first discovery, so a leaf's
/// position among the nodes says nothing about which tensor it reads. An ordinal
/// with no loaded value is a region whose reads and expression disagree, which
/// the schedule verifier rejects — this reports it as an invalid handle rather
/// than reading whichever value sits at that index.
fn emit_pointwise(
    builder: &mut KernelBuilder,
    expression: &PointwiseF32Expression,
    inputs: &[KernelValueId],
) -> Result<KernelValueId, KernelBuildError> {
    let mut values = Vec::with_capacity(expression.nodes().len());
    for node in expression.nodes() {
        let value = match node {
            PointwiseF32Node::Input { ordinal } => usize::try_from(ordinal.get())
                .ok()
                .and_then(|ordinal| inputs.get(ordinal).copied())
                .ok_or(KernelBuildError::InvalidHandle {
                    entity: super::error::KernelEntityKind::Buffer,
                })?,
            PointwiseF32Node::Constant { bits } => {
                builder.constant(KernelConstant::F32Bits(*bits))?
            }
            PointwiseF32Node::Add { lhs, rhs } => {
                let lhs = pointwise_value(&values, *lhs)?;
                let rhs = pointwise_value(&values, *rhs)?;
                let result = builder.binary(BinaryOp::F32Add, lhs, rhs)?;
                builder.convert(ConvertOp::CanonicalizeF32Nan, result)?
            }
            PointwiseF32Node::Divide { lhs, rhs } => {
                let lhs = pointwise_value(&values, *lhs)?;
                let rhs = pointwise_value(&values, *rhs)?;
                let result = builder.binary(BinaryOp::F32Divide, lhs, rhs)?;
                builder.convert(ConvertOp::CanonicalizeF32Nan, result)?
            }
            // The exponential's result is canonicalized on the same rule every
            // other arithmetic result is: the numerical realization installs one
            // canonical arithmetic NaN payload, and an elementary function that
            // skipped it would deliver a payload the contract does not name.
            PointwiseF32Node::Exp { argument } => {
                let argument = pointwise_value(&values, *argument)?;
                let result = builder.unary(UnaryOp::F32Exp, argument)?;
                builder.convert(ConvertOp::CanonicalizeF32Nan, result)?
            }
            PointwiseF32Node::Rsqrt { argument } => {
                let argument = pointwise_value(&values, *argument)?;
                let result = builder.unary(UnaryOp::F32Rsqrt, argument)?;
                builder.convert(ConvertOp::CanonicalizeF32Nan, result)?
            }
            PointwiseF32Node::Multiply { lhs, rhs } => {
                let lhs = pointwise_value(&values, *lhs)?;
                let rhs = pointwise_value(&values, *rhs)?;
                let result = builder.binary(BinaryOp::F32Multiply, lhs, rhs)?;
                builder.convert(ConvertOp::CanonicalizeF32Nan, result)?
            }
        };
        values.push(value);
    }
    pointwise_value(&values, expression.root())
}

fn pointwise_value(
    values: &[KernelValueId],
    node: crate::schedule::PointwiseF32NodeId,
) -> Result<KernelValueId, KernelBuildError> {
    usize::try_from(node.index())
        .ok()
        .and_then(|index| values.get(index).copied())
        .ok_or(KernelBuildError::InvalidHandle {
            entity: super::error::KernelEntityKind::Value,
        })
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

/// Emits one contributor's prologue expression, or the contributor unchanged.
///
/// The match is exhaustive over a crate-private enum, so a third prologue is a
/// build error here rather than a silent reuse of whichever arm it resembles.
fn emit_prologue(
    builder: &mut KernelBuilder,
    value: KernelValueId,
    prologue: ReductionPrologue,
) -> Result<KernelValueId, KernelBuildError> {
    match prologue {
        ReductionPrologue::None => Ok(value),
        ReductionPrologue::ScaleBias {
            scale_bits,
            bias_bits,
        } => emit_scale_bias(builder, value, scale_bits, bias_bits),
        // The loaded value multiplied by itself, so the square rests on one read
        // rather than on two reads agreeing. One rounding, which is what the
        // semantic reference states for `q_i = x_i * x_i`.
        ReductionPrologue::Square => {
            let square = builder.binary(BinaryOp::F32Multiply, value, value)?;
            builder.convert(ConvertOp::CanonicalizeF32Nan, square)
        }
    }
}

fn emit_reduction(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    read: (KernelBufferId, BoundsWitnessId),
    write_buffer: KernelBufferId,
    invocation: KernelValueId,
    empty_identity_bits: u32,
    prologue: ReductionPrologue,
) -> Result<(), KernelBuildError> {
    let (read_buffer, read_bounds) = read;
    let addressing = plan
        .addressing
        .first()
        .ok_or(KernelBuildError::InvalidHandle {
            entity: super::error::KernelEntityKind::Buffer,
        })?;
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
    let first_offset = emit_offset(builder, addressing, invocation, None)?;
    let first = builder.load(read_buffer, first_offset, read_bounds)?;
    let seed = emit_prologue(builder, first, prologue)?;
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
            ReductionPrologue::ScaleBias { .. } | ReductionPrologue::Square => seed,
            ReductionPrologue::None => builder.convert(ConvertOp::CanonicalizeF32Nan, seed)?,
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
                let offset = emit_offset(builder, addressing, invocation, Some(induction))?;
                let loaded = builder.load(read_buffer, offset, read_bounds)?;
                let contributor = emit_prologue(builder, loaded, prologue)?;
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

/// Splits a partial pass's invocation index into its output and partition parts.
///
/// One invocation covers one (output, partition) pair, laid out so the
/// partition ordinal is the innermost coordinate — which is also what makes the
/// partial tensor's linear write index equal to the invocation index. Returns
/// the linear output coordinate and the partition's first contributor ordinal.
///
/// A single partition needs neither operation: the output coordinate is the
/// invocation and the partition ordinal is constantly zero, so emitting the
/// division and remainder would put two provably identity operations into the
/// canonical body a refinement gate compares against.
fn split_partitioned_invocation(
    builder: &mut KernelBuilder,
    invocation: KernelValueId,
    partitions: u64,
) -> Result<(KernelValueId, Option<KernelValueId>), KernelBuildError> {
    if partitions <= 1 {
        return Ok((invocation, None));
    }
    let extent = builder.constant(KernelConstant::Index(partitions))?;
    let output = builder.binary(BinaryOp::IndexDivide, invocation, extent)?;
    let partition = builder.binary(BinaryOp::IndexModulo, invocation, extent)?;
    Ok((output, Some(partition)))
}

/// Emits the contributor ordinal one partitioned load addresses.
///
/// The ordinal is `partition * contributors_per_partition + within`, which is
/// the contiguous range this partition owns in the region's declared
/// contributor order. `within` is `None` for the seed load, whose position
/// inside the partition is zero.
///
/// `None` comes back only when the whole ordinal is provably zero — a single
/// partition seeding at its first contributor — so the caller drops every
/// contributor term exactly as the unsplit lowering does.
fn emit_partition_contributor(
    builder: &mut KernelBuilder,
    partition: Option<KernelValueId>,
    within: Option<KernelValueId>,
    contributors_per_partition: u64,
) -> Result<Option<KernelValueId>, KernelBuildError> {
    let base = match partition {
        None => None,
        Some(partition) => {
            if contributors_per_partition <= 1 {
                Some(partition)
            } else {
                let stride = builder.constant(KernelConstant::Index(contributors_per_partition))?;
                Some(builder.binary(BinaryOp::IndexMultiply, partition, stride)?)
            }
        }
    };
    Ok(match (base, within) {
        (None, within) => within,
        (Some(base), None) => Some(base),
        (Some(base), Some(within)) => Some(builder.binary(BinaryOp::IndexAdd, base, within)?),
    })
}

/// Emits the element offset of one read access.
///
/// `contributor` is `None` for the seed load, whose contributor coordinate is
/// zero; every contributor term then vanishes exactly.
fn emit_offset(
    builder: &mut KernelBuilder,
    addressing: &ReadAddressing,
    invocation: KernelValueId,
    contributor: Option<KernelValueId>,
) -> Result<KernelValueId, KernelBuildError> {
    let (terms, output, contributor) = match addressing {
        ReadAddressing::Identity => return Ok(invocation),
        ReadAddressing::Linearized(terms) => (terms, invocation, contributor),
        ReadAddressing::Partitioned {
            terms,
            partitions,
            contributors_per_partition,
        } => {
            let (output, base) = split_partitioned_invocation(builder, invocation, *partitions)?;
            let contributor = emit_partition_contributor(
                builder,
                base,
                contributor,
                *contributors_per_partition,
            )?;
            (terms, output, contributor)
        }
    };
    let mut total: Option<KernelValueId> = None;
    for term in terms {
        let root = match term.root {
            OffsetRoot::Output => output,
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
