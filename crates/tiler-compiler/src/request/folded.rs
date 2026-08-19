//! The folding families, and the chains they anchor.
//!
//! A strict serial reduction, a binary tensor contraction, and any registered
//! family whose realization law spans a region sequence are the occurrences
//! whose result some region *materializes* rather than a consumer recomputes.
//! This module recognizes each of them, states the one rule bounding how deep a
//! recognized chain may run, and owns the producer entry every chain resolves
//! its far side through.

use super::*;

/// Returns whether one occurrence's result is a value some region *materializes*
/// rather than a value a consumer's own per-point body can recompute.
///
/// **The single statement of where a materialization edge may sit**, asked by
/// [`plan_elementwise`]'s folding discovery and by
/// [`recognize_staged_family`]'s operand walk. The two used to be one disjunct
/// written once; they are one function now because a second copy would be free
/// to disagree about which programs contain an edge at all, and the shape of
/// that disagreement is a walk naming a boundary the producer recognizer refuses
/// to build for.
///
/// A recognized *elementwise* family is deliberately not here. Its result is an
/// expression its consumer evaluates per point, which is the whole reason the
/// expression vocabulary exists; treating it as an edge would materialize a
/// value — and add the observable rounding boundary — the caller's program never
/// asked for.
pub(super) fn materializes_its_result(
    operation: &tiler_ir::semantic::OperationRef<'_>,
    laws: &FrozenIndexRealizationLawRegistry,
) -> bool {
    operation.key() == &strict_serial_sum_f32_op()
        || operation.key() == &tensor_contraction_f32_op()
        || laws.family_realizes_region_sequence(operation.key())
}

/// Recognizes the shape producing one materialized value.
///
/// The folding families and nothing else, which is exactly
/// [`materializes_its_result`]'s set. The refusal is not dead code standing in
/// for an impossible state: both callers gate on that predicate, and a family
/// added to it without a producer region here must refuse rather than acquire
/// one.
///
/// **The producer is at the far side of an edge, so it places none of its own**
/// — [`StagedOperandAdmission::NoEdge`] and
/// [`ReductionContributorAdmission::NoEdge`] below. This is the only site that
/// hands either value, and the two arms that can place an edge at all are the
/// ones it hands them to: a staged occurrence through its operand walk, and a
/// fold through its contributor walk. The contraction arm places none by its
/// own `contraction-operands` refusal, so the whole depth rule is reachable from
/// here. [`StagedOperandAdmission`] is where the rule is stated, including the
/// measured reason it stays and the neighbouring folded-value wall it is not.
pub(super) fn recognize_epilogue_producer(
    program: &SemanticProgram,
    staged: ValueId,
    output_key: OutputKey,
    laws: &FrozenIndexRealizationLawRegistry,
) -> Result<NormalizedOutput, RequestError> {
    let (member, root) = producer_for_value(program, staged)?;
    if root.key() == &strict_serial_sum_f32_op() {
        recognize_reduction(
            program,
            staged,
            output_key,
            member,
            &root,
            laws,
            // The far side of an edge places none of its own, which is the
            // reduction half of the same rule the staged arm below states. A
            // fold reached across an edge whose *own* contributor is
            // materialized is `reduction-contributor-depth`.
            ReductionContributorAdmission::NoEdge,
        )
        .map(NormalizedOutput::SerialSum)
    } else if root.key() == &tensor_contraction_f32_op() {
        normalize_contraction(program, staged, output_key)
            .map(|normalized| NormalizedOutput::Contraction(Box::new(normalized)))
    } else if laws.family_realizes_region_sequence(root.key()) {
        recognize_staged_family(
            program,
            laws,
            staged,
            output_key,
            member,
            &root,
            StagedOperandAdmission::NoEdge,
        )
        .map(|normalized| NormalizedOutput::Staged(Box::new(normalized)))
    } else {
        mismatch("operation-set")
    }
}

/// Whether one staged occurrence may place a materialization edge of its own.
///
/// **This is the single statement of the recognized chain's depth rule**, and
/// every other site that mentions depth points here rather than restating it.
///
/// **A depth counter would be the wrong shape.** What bounds the recognized
/// chain is not a number of levels but a rule about *sides*: a recognized shape
/// admits at most one edge of its own, and a shape reached across an edge admits
/// none. Two variants say exactly that, and a reader can refute the rule by
/// checking the two call sites rather than by reasoning about arithmetic.
///
/// # The rule has two guards, and one neighbour that is not either
///
/// The `NoEdge` arm of [`recognize_staged_family`]'s operand walk —
/// `staged-operand-depth` — is the staged half.
/// [`ReductionContributorAdmission`]'s `NoEdge` arm —
/// `reduction-contributor-depth` — is the reduction half, and the two are the
/// same rule about sides written for the two shapes that can place an edge.
/// [`recognize_epilogue_producer`] is the one function reached across an edge
/// and the only site that passes either `NoEdge`, and of the three shapes it
/// recognizes the contraction is the one that can place no edge at all:
/// [`normalize_contraction`] refuses a non-declared operand under
/// `contraction-operands`.
///
/// One neighbouring refusal also fires on a folded value and states a
/// *different* rule. Reading it as this one is what would make a widener delete
/// the wrong guard, so it is named with the shape that separates it:
/// [`plan_elementwise`]'s `leaves.staged.is_none()` guard refuses one walk that
/// reaches a *second, different* folded value — `sum(a, 1) * sum(b, 1)`. That is
/// one region reading two materialization edges, which is a rule about chain
/// *width*: the walk is still one boundary deep, and what it lacks is the
/// ordinal [`TensorRole::Intermediate`] does not carry.
/// [`admit-a-scheduled-region-that-reads-two-materialization-edges`](../../../tickets/admit-a-scheduled-region-that-reads-two-materialization-edges.md)
/// owns the region vocabulary and
/// [`admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region`](../../../tickets/admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region.md)
/// owns the one-value-twice spelling of it.
///
/// # Why `NoEdge` stays, measured rather than argued
///
/// Widening is
/// [`admit-a-recognized-chain-more-than-one-materialization-boundary-deep`](../../../tickets/admit-a-recognized-chain-more-than-one-materialization-boundary-deep.md)'s,
/// and it measured that the widening buys no program today. Every program this
/// guard refuses contains a staged occurrence whose operand is an edge, and
/// [`crate::physical::staged_plan`] has no region for one: its only law arm
/// destructures two [`BoundaryRead::Input`] operands, so such an occurrence is
/// [`crate::physical::RegionVocabularyWall::StagedFamilyUnspellable`] however
/// deep the chain around it is. Handing `OneEdge` here therefore recognizes the
/// chain — the nested shape is well formed, and only the assertion of the
/// refusal itself moves — and then refuses it as a target rejection instead of a
/// named program property. `crates/tiler-compiler/tests/recognized_chain_depth_boundary.rs`
/// holds the measurement and the trigger that reopens it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StagedOperandAdmission {
    /// One operand may be a value another region materializes.
    OneEdge,
    /// Every operand must be a declared input, because this occurrence is
    /// already at the far side of an edge.
    NoEdge,
}

/// Whether one recognized serial reduction may consume contributors another
/// region materializes.
///
/// **The reduction half of [`StagedOperandAdmission`]'s rule, in the same shape
/// and for the same reason.** What bounds the recognized chain is a rule about
/// *sides* rather than a number of levels: a recognized shape admits at most one
/// edge of its own, and a shape reached across an edge admits none. A depth
/// counter would be the wrong shape here for the reason it is wrong there, and a
/// reader can refute the rule by checking the two call sites.
///
/// The recognition entry for a declared output passes `OneEdge` — that fold is at
/// the near side of every edge this walk may place. [`recognize_epilogue_producer`]
/// passes `NoEdge`, because a fold it recognizes is already the far side of one.
/// The call graph is therefore at most
/// `recognize_reduction(OneEdge)` → `recognize_epilogue_producer` →
/// `recognize_reduction(NoEdge)`, with no further retain, so the host stack this
/// recognition consumes is constant in program depth — the same bound the staged
/// rule keeps, and the reason neither is a worklist.
///
/// # What `NoEdge` refuses, and what would lift it
///
/// `sum(sum(sum(x) * 2) * 2)`, `sum(sum(contract(a, b)) * 2)`, and an epilogue
/// over a produced sum such as `(sum(sum(x) * 2)) * 3` are all well-formed trees
/// of already-admitted shapes, and all three refuse `reduction-contributor-depth`.
/// Admitting them needs a worklist rewrite of the whole producer walk — the
/// recognizer, the subject encoder, the member partition, and the physical
/// spelling all recurse over `Box<NormalizedOutput>` — under a named
/// deterministic budget, which is the recorded reversal path for this rule and
/// not a guard to delete on its own. Shipping unbounded recursion to buy those
/// subjects would turn an input property into a crash rather than a refusal,
/// which is what [`plan_elementwise`]'s own worklist exists to prevent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ReductionContributorAdmission {
    /// This fold's contributors may be a value another region materializes.
    OneEdge,
    /// This fold's contributors must be declared inputs or a pointwise
    /// expression over them, because the fold is already at the far side of an
    /// edge.
    NoEdge,
}

/// Recognizes one occurrence of a registered family realized as a region
/// sequence.
///
/// **The admitting fact is the registered law, and no operation key appears
/// here.** The caller has already asked
/// [`FrozenIndexRealizationLawRegistry::family_realizes_region_sequence`], so
/// every family the law authority carries a multi-region law for reaches this
/// function and a family registered tomorrow reaches it unchanged. What this
/// function decides is whether *this occurrence* of such a family is one the
/// boundary can describe, and it names each property it cannot.
///
/// **What it does not do is describe the realization.** The stage count, each
/// stage's reads, and the handed values are the law's, read by
/// [`crate::region::RegionGraph::with_realizations`] when it enumerates one
/// region candidate per stage. Re-deriving them here would put a second account
/// of one law in the boundary, which is the drift
/// [`recognize_elementwise_output`](super::recognize::recognize_elementwise_output)'s own doc argues against for the same
/// reason.
///
/// **One operand may be a value another region materializes, and that is this
/// function's own admission rather than a later stage's derivation.** The
/// recognized shape carries a [`BoundaryRead`] per operand and the producer's
/// recognized shape beside them, so `rms_norm(matmul(a, b), w)` is a partition
/// this output owns end to end — the producer's occurrence included, which is
/// what [`check_output_cover`](super::recognize::check_output_cover) requires and what makes the *producing* region
/// spellable from a shape this partition holds. The alternative considered and
/// rejected was deriving each operand's source from the cover's materialization
/// edges: it keeps one authority for the stage split and moves a recognition-time
/// property to a stage that can only report it as a cover it could not assemble.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] naming the exact property
/// that was not recognized:
///
/// - `staged-result-arity` for an occurrence whose results are not exactly the
///   recognized value. A staged realization's final stage writes the
///   occurrence's results and every earlier stage publishes one handed value, so
///   a second result would need a second write this boundary cannot attribute.
/// - `staged-operand` for an operand that is neither a declared program input
///   nor a value some region materializes — an elementwise expression feeding
///   the family directly, whose result [`materializes_its_result`] says is no
///   materialization edge at all. Admitting it here would be a second account of
///   where an edge may sit, disagreeing with the one
///   [`plan_elementwise`]'s folding discovery reads.
/// - `staged-operand-conflict` for a second operand supplied by a
///   materialization edge, whether that is one staged value read twice or two
///   different ones. [`TensorRole::Intermediate`] carries no ordinal, so nothing
///   says which edge each read binds; it is the same unattributable pair
///   [`record_leaf`](super::elementwise::record_leaf) refuses for an epilogue's leaves.
/// - `staged-operand-depth` for a staged operand of an occurrence that is
///   *itself* at the far side of an edge. That is a recognized chain more than
///   one materialization boundary deep, and it is the one guard the depth rule
///   has; [`StagedOperandAdmission`] states the rule, the measured reason it
///   stays, and the neighbouring refusals that are about chain width and about a
///   fold's chained prologue instead.
/// - `staged-attributes` for an attribute record the canonical encoder cannot
///   write. The record is part of the occurrence's meaning, so a subject that
///   could not carry it whole must refuse rather than bind a partial one.
/// - `input-handle`/`output-handle` for a value the program holds no shape for,
///   and every rule [`declared_ordinal`] and [`recognize_epilogue_producer`]
///   report.
///
/// Returns [`RequestError::ShapeProductOverflow`] for a domain whose extents do
/// not multiply into a `u64`.
pub(super) fn recognize_staged_family(
    program: &SemanticProgram,
    laws: &FrozenIndexRealizationLawRegistry,
    result: ValueId,
    output_key: OutputKey,
    member: u32,
    operation: &tiler_ir::semantic::OperationRef<'_>,
    admission: StagedOperandAdmission,
) -> Result<NormalizedStaged, RequestError> {
    if operation.results().collect::<Vec<_>>() != [result] {
        return mismatch("staged-result-arity");
    }
    let declared: Vec<ValueId> = program.inputs().map(|input| input.value()).collect();
    let operands: Vec<ValueId> = operation.operands().collect();
    let mut operand_reads = Vec::with_capacity(operands.len());
    let mut operand_shapes = Vec::with_capacity(operands.len());
    let mut operand_elements = Vec::with_capacity(operands.len());
    let mut producer = None;
    for operand in &operands {
        let read = if declared.contains(operand) {
            BoundaryRead::Input(declared_ordinal(&declared, *operand)?)
        } else {
            // The operand is computed. Whether that makes it a *materialization
            // edge* is `materializes_its_result`'s answer and not this walk's,
            // which is what keeps one statement of where an edge may sit; the
            // producer's own recognition is `recognize_epilogue_producer`'s, so
            // this arm decides only that there is one edge and that this
            // occurrence is allowed to place it.
            let (_, root) = producer_for_value(program, *operand)?;
            if !materializes_its_result(&root, laws) {
                return mismatch("staged-operand");
            }
            if producer.is_some() {
                return mismatch("staged-operand-conflict");
            }
            if admission == StagedOperandAdmission::NoEdge {
                return mismatch("staged-operand-depth");
            }
            producer = Some(Box::new(recognize_epilogue_producer(
                program,
                *operand,
                output_key.clone(),
                laws,
            )?));
            BoundaryRead::Staged
        };
        operand_reads.push(read);
        let shape = static_shape(program, *operand, "input-handle")?;
        operand_elements.push(element_count_u64(&shape, "input")?);
        operand_shapes.push(shape);
    }
    let output_shape = static_shape(program, result, "output-handle")?;
    let output_elements = element_count_u64(&output_shape, "output")?;
    let mut attributes = Vec::new();
    crate::region::encode_attributes(&mut attributes, operation.attributes()).map_err(|_| {
        RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "staged-attributes",
        }
    })?;
    // The same registry row the caller's admission read. `None` is unreachable
    // through that admission — a family with no law realizes no region sequence —
    // and is refused by name rather than unwrapped, because this function is the
    // one that would otherwise carry an invented law into every later stage.
    let law = laws
        .family_realization_law(operation.key())
        .ok_or(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "staged-law",
        })?
        .clone();
    Ok(NormalizedStaged {
        producer,
        operation: operation.key().clone(),
        law,
        attribute_record: operation.attributes().clone(),
        attributes: attributes.into_boxed_slice(),
        input_keys: program.inputs().map(|input| input.key().clone()).collect(),
        output_key,
        operand_reads,
        operand_shapes,
        output_shape,
        member: SemanticMemberId(member),
        inputs: declared,
        output: result,
        operand_elements,
        output_elements,
    })
}

/// Recognizes a strict serial reduction and whatever elementwise expression
/// feeds it.
///
/// The prologue is recognized by the same general walk a whole-program
/// elementwise expression is, so what composes with the reduction is bounded by
/// the expression vocabulary rather than by the scale-then-bias shape the
/// superseded template spelled.
///
/// **A fold whose operand is a declared input has no prologue at all, and that is
/// recognized rather than refused.** The walk below is run for it too, so the
/// obligations it states — every read at the contributor domain — are discharged
/// for this shape by the same authority and under the same rules. What differs is
/// what the walk returns: a bare input leaf claiming no occurrence, which is the
/// fold's own contributor read and not an expression a region computes. Recording
/// [`SerialSumContributor::DeclaredInput`] is therefore the fact, and it is what
/// makes the fold's contributor access bind the input tensor directly instead of
/// an intermediate a synthesized identity prologue would have had to
/// materialize.
///
/// **A fold whose contributors another region materializes is retained rather
/// than flattened**, which is what [`SerialSumContributor::Materialized`] is for.
/// The walk raises [`ElementwiseRefusal::Folded`] naming the produced value; this
/// function then re-plans the contributor with that value as the walk's staged
/// leaf — the numbering [`recognize_epilogue`] already states — and recognizes the
/// producer through [`recognize_epilogue_producer`], so `sum(sum(x) * 2)` is one
/// output's partition end to end. The finding is *never* mapped through the
/// elementwise refusal's flattening once a retain is attempted: falling back to a
/// rule after a failed admission would be an unstated policy, so a producer this
/// function cannot recognize reports the producer's own refusal.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] naming the unrecognized
/// property: `sum-signature`, `sum-output`, `sum-shape`, `sum-axes*`, and
/// `input-rank` for the reduction itself, every rule the contributor walk
/// reports — `operation-set` among them, which is what a walk reaching a
/// *second* folded value reports — `reduction-contributor-depth` for a
/// materialized contributor this fold may not consume because it is itself
/// across an edge, and every rule [`recognize_epilogue_producer`] reports for the
/// producing half.
pub(super) fn recognize_reduction(
    program: &SemanticProgram,
    result: ValueId,
    output_key: OutputKey,
    sum_member: u32,
    sum: &tiler_ir::semantic::OperationRef<'_>,
    laws: &FrozenIndexRealizationLawRegistry,
    admission: ReductionContributorAdmission,
) -> Result<NormalizedSerialSum, RequestError> {
    let declared: Vec<ValueId> = program.inputs().map(|input| input.value()).collect();
    let sum_operands: Vec<_> = sum.operands().collect();
    let [contributor] = sum_operands.as_slice() else {
        return mismatch("sum-signature");
    };
    if sum.results().collect::<Vec<_>>() != [result] {
        return mismatch("sum-output");
    }
    let axes = reduction_axes(sum.attributes())?;
    let input_shape = static_shape(program, *contributor, "input-handle")?;
    if input_shape.rank() == 0 {
        return mismatch("input-rank");
    }
    check_canonical_reduction_axes(&axes, input_shape.rank())?;
    let output_shape = input_shape.without_axes(&axes);
    if static_shape_ref(program, result) != Some(&output_shape) {
        return mismatch("sum-shape");
    }

    // `f32` is the fold family's own, not the enclosing program's: the caller
    // reached this function by matching `tiler::strict-serial-sum-f32@1`, so the
    // contributor tensor and every occurrence feeding it are binary32 or the
    // program is mixed-width and `recognized_program_arithmetic` already refused
    // it. Stating the width at the call site is what lets
    // [`SerialSumContributor`] stay typed as the one vocabulary a fold's
    // contributor regions can carry.
    let (contributor_source, pointwise_members) = match recognize_elementwise(
        program,
        *contributor,
        &declared,
        &input_shape,
        laws,
        ArithmeticType::F32,
    ) {
        Ok(RecognizedElementwise {
            expression,
            members,
            reads,
        }) => {
            // The walk claims an occurrence for every leaf and node it mints
            // except one: a declared input contributes the leaf that reads it
            // and nothing else. So a fold straight over a declared input arrives
            // here with an empty member set and a bare input leaf, and that leaf
            // is the fold's own contributor read rather than a prologue any
            // region computes — which is why the condition tested is the operand
            // itself and not the emptiness that follows from it.
            //
            // The read list belongs to the prologue *region*, so a fold that has
            // none states none: the walk still returns the fold's own
            // contributor read, and recording it as a prologue region's would
            // describe a region no cover places.
            if declared.contains(contributor) {
                (
                    SerialSumContributor::DeclaredInput(declared_ordinal(&declared, *contributor)?),
                    Vec::new(),
                )
            } else {
                (
                    SerialSumContributor::PointwisePrologue {
                        expression: expression.into_f32()?,
                        reads,
                    },
                    members,
                )
            }
        }
        // The contributor walk reached a value some region materializes. Whether
        // this fold may consume it is the sides rule's answer and not this
        // walk's; where it may, the boundary is *retained* rather than reported,
        // which is the whole of what separates `sum(sum(x) * 2)` from the
        // refusal it used to be.
        Err(ElementwiseRefusal::Folded(staged)) => {
            if admission == ReductionContributorAdmission::NoEdge {
                return mismatch("reduction-contributor-depth");
            }
            (
                SerialSumContributor::Materialized(Box::new(recognize_materialized_contributor(
                    program,
                    *contributor,
                    staged,
                    &declared,
                    &input_shape,
                    output_key.clone(),
                    laws,
                )?)),
                // The continuation's occurrences are its own part, carried on
                // the continuation rather than folded in here; see
                // [`RecognizedSerialSumMembers`], which states why the pointwise
                // part must stay the declared-input prologue's.
                Vec::new(),
            )
        }
        Err(ElementwiseRefusal::Refused(error)) => return Err(error),
    };
    let members = RecognizedSerialSumMembers::new(pointwise_members, sum_member);

    let input_elements = element_count_u64(&input_shape, "input")?;
    let output_elements = element_count_u64(&output_shape, "output")?;
    Ok(NormalizedSerialSum {
        input_keys: program.inputs().map(|input| input.key().clone()).collect(),
        output_key,
        input_shape,
        output_shape,
        reduction_axes: axes,
        contributor: contributor_source,
        members,
        inputs: declared,
        pointwise_result: *contributor,
        output: result,
        input_elements,
        output_elements,
    })
}

/// Recognizes the producer of one fold's materialized contributors, and whatever
/// expression stands between the two.
///
/// **The producer is recognized before the continuation is minted**, so a
/// producer this boundary cannot describe reports its own refusal rather than
/// one about the continuation that would have consumed it.
///
/// The continuation is recorded as an absence when the fold's contributor *is*
/// the produced value — `sum(rms_norm(x, w))`, `sum(contract(a, b))`. Minting an
/// identity expression for it would spell a copy region, and its rounding
/// boundary, that the caller's program never asked for, which is the reason
/// [`SerialSumContributor::DeclaredInput`] carries no prologue either.
///
/// # Errors
///
/// Returns every rule [`recognize_epilogue_producer`] reports for the producing
/// half and every rule [`recognize_staged_elementwise`] reports for the
/// continuation — `operation-set` among them, which is what the re-planned walk
/// reports when it reaches a *second*, different materialized value.
fn recognize_materialized_contributor(
    program: &SemanticProgram,
    contributor: ValueId,
    staged: ValueId,
    declared: &[ValueId],
    input_shape: &Shape,
    output_key: OutputKey,
    laws: &FrozenIndexRealizationLawRegistry,
) -> Result<MaterializedContributor, RequestError> {
    let producer = recognize_epilogue_producer(program, staged, output_key, laws)?;
    let continuation = if contributor == staged {
        None
    } else {
        let recognized = recognize_staged_elementwise(
            program,
            contributor,
            declared,
            input_shape,
            staged,
            laws,
            ArithmeticType::F32,
        )?;
        Some(ContributorContinuation {
            expression: recognized.expression,
            reads: recognized.reads,
            members: recognized.members,
        })
    };
    Ok(MaterializedContributor {
        producer,
        continuation,
    })
}

/// Recognizes a two-input binary tensor contraction over `f32`.
///
/// The admitted set is *every* well-formed binary index structure the semantic
/// registry validates, not one hard-coded matmul spelling. That is not a
/// widening for its own sake: the physical realization addresses each operand
/// axis by whichever output or contracted coordinate the structure binds it to,
/// so a structure whose contracted index sits at a different axis of each
/// operand costs this recognizer nothing extra and refusing it would be a check
/// with no correctness content behind it. What stays narrow is everything else —
/// exactly two operands, exactly one contraction operation reachable, `f32`
/// throughout, and no attribute beyond the index structure.
pub(super) fn normalize_contraction(
    program: &SemanticProgram,
    result: ValueId,
    output_key: OutputKey,
) -> Result<NormalizedContraction, RequestError> {
    // An elementwise epilogue over a contraction result is a two-region chain
    // this profile assembles as a two-region chain, and this normalization is
    // the producer half of it: [`recognize_epilogue`] reaches here with the
    // contraction's own result value rather than a declared program output, and
    // `contraction_region` writes whichever tensor the cover assigns.
    let (ordinal, operation) = producer(program, result, &tensor_contraction_f32_op())?;
    if operation.results().collect::<Vec<_>>() != [result] {
        return mismatch("contraction-output");
    }
    // Exactly the index structure. An attribute this normalization does not
    // carry forward is a semantic fact it would silently drop.
    let [field] = operation.attributes().fields() else {
        return mismatch("contraction-attributes");
    };
    if field.id() != CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE {
        return mismatch("contraction-attributes");
    }
    let structure =
        ContractionIndexStructure::from_canonical_value(field.value()).map_err(|_| {
            RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "contraction-structure",
            }
        })?;
    if structure.operand_count() != 2 {
        return mismatch("contraction-operand-count");
    }

    // Each structure operand must be one distinct declared input. The complete
    // declaration may be wider: a sibling output or a later stage can read an
    // input this contraction does not. Each read therefore carries both the
    // program ordinal the ABI binds and the structure operand position it
    // supplies, then the pair is canonicalized by ascending program ordinal.
    let operands: [ValueId; 2] = operation
        .operands()
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "contraction-operand-count",
        })?;
    let declared: Vec<ValueId> = program.inputs().map(|input| input.value()).collect();
    let shape_of = |value: ValueId| static_shape(program, value, "input-handle");
    let mut reads = Vec::with_capacity(2);
    for (position, operand) in operands.into_iter().enumerate() {
        let Some(declaration) = declared.iter().position(|declared| *declared == operand) else {
            return mismatch("contraction-operands");
        };
        let input_ordinal = u32::try_from(declaration)
            .map(DeclaredInputOrdinal::new)
            .map_err(|_| RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "input-ordinal",
            })?;
        let shape = shape_of(operand)?;
        let elements = element_count_u64(&shape, "input")?;
        reads.push(NormalizedContractionRead {
            input_ordinal,
            shape,
            elements,
            value: operand,
            operand_position: position,
        });
    }
    reads.sort_by_key(|read| read.input_ordinal);
    let reads: [NormalizedContractionRead; 2] =
        reads
            .try_into()
            .map_err(|_| RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "contraction-operand-count",
            })?;
    if reads[0].input_ordinal >= reads[1].input_ordinal {
        return mismatch("contraction-operands");
    }

    // One extent per index, bound by the first operand axis naming it. The
    // semantic inferencer already proved agreement at construction, so a
    // disagreement here is invalid state and is refused rather than preferred
    // one way.
    let mut extents: Vec<(ContractionIndex, Extent)> = Vec::new();
    for read in &reads {
        let tuple = structure.operand(read.operand_position).ok_or(
            RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "contraction-structure",
            },
        )?;
        if read.shape.rank() != tuple.len() {
            return mismatch("contraction-rank");
        }
        for (axis, index) in tuple.iter().enumerate() {
            let extent = read.shape.extents()[axis];
            match extents.iter().find(|(bound, _)| bound == index) {
                Some((_, bound)) if *bound != extent => return mismatch("contraction-extent"),
                Some(_) => {}
                None => extents.push((*index, extent)),
            }
        }
    }
    let extent_of = |index: &ContractionIndex| -> Result<Extent, RequestError> {
        extents
            .iter()
            .find(|(bound, _)| bound == index)
            .map(|(_, extent)| *extent)
            .ok_or(RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "contraction-extent",
            })
    };
    let shape_over = |indices: &[ContractionIndex]| -> Result<Shape, RequestError> {
        Shape::try_new(
            indices
                .iter()
                .map(&extent_of)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .map_err(|_| RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "contraction-shape",
        })
    };
    let output_shape = shape_over(structure.output())?;
    let contracted_shape = shape_over(structure.contracted())?;
    if static_shape_ref(program, result) != Some(&output_shape) {
        return mismatch("contraction-output-shape");
    }

    let output_elements = element_count_u64(&output_shape, "output")?;
    let contracted_elements = element_count_u64(&contracted_shape, "input")?;
    // `direct`'s one precondition, and its only one. The semantic inferencer
    // refuses a zero contracted extent at construction, so this is unreachable
    // through a built program; it is kept because it is the *stated* precondition
    // of this realization and a reader must be able to find it here rather than
    // infer it from an inferencer three crates away.
    if contracted_elements == 0 {
        return mismatch("contraction-empty-domain");
    }

    Ok(NormalizedContraction {
        input_keys: program.inputs().map(|input| input.key().clone()).collect(),
        output_key,
        reads,
        output_shape,
        contracted_shape,
        structure,
        members: vec![SemanticStage::first(SemanticMemberId(ordinal))],
        output: result,
        output_elements,
        contracted_elements,
    })
}
