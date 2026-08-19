//! The recognition walk's entry, and the obligations it states over the whole
//! program.
//!
//! One walk per ordered named output, dispatched by the occurrence that produces
//! it, plus the program-wide properties no single walk can decide: one
//! recognized arithmetic throughout, walks that partition the program's
//! occurrences, and every declared input read by some walk. The per-shape
//! recognizers are `elementwise`, `structural`, and `folded`.

use super::*;

/// Recognizes one verified semantic program, or explains what it could not
/// recognize.
///
/// # What generalized, and what the generalization rests on
///
/// This is **not** a match against whole-program templates. The program-wide
/// properties every recognized program shares — at least one declared input, and
/// one recognized arithmetic type throughout — are checked once and each names
/// its own rule, and the
/// program's shape is then decided per declared output by *the occurrence that
/// produces it*, walked outward through the occurrences that feed it. A program
/// whose exact shape nothing here was taught is admitted when every occurrence
/// it contains is one the physical layer can realize and they compose into a
/// region chain it can assemble; nothing asks whether the whole graph matches a
/// spelling.
///
/// Concretely, the elementwise dimension is now the general
/// [`PointwiseF32Expression`] vocabulary rather than a leaf count. A reduction
/// over `(a * b) + c` with three declared inputs, a whole-program
/// `((a * 2.0) + b) * c` over two, and a chain sharing one subexpression at
/// several places are all admitted by the same walk that admits the scale-bias
/// program the old template spelled, and none of them was a shape this boundary
/// had been taught.
///
/// # What is still refused, and where the wall actually is
///
/// Recognition may only admit what the physical layer can express, so the walls
/// below this boundary are refused *at* it, each under its own rule:
///
/// - **An operation the region vocabulary cannot spell** (`operation-set`). A
///   family whose per-point body no [`PointwiseF32Expression`] node composes and
///   whose access relation no [`tiler_ir::schedule::LogicalAccess`] denotes has
///   no region to be built into, and a region for it is a `tiler-ir` widening
///   rather than a projection this boundary could make.
///
///   **A family realized as a region *sequence* is the one shape that leaves
///   this rule without being expressible per point, and the admitting fact is a
///   registry row.** An occurrence whose registered
///   [`tiler_ir::index::IndexRealizationLaw`] realizes a sequence computes
///   several regions' worth of work, so no single per-point body was ever going
///   to spell it; what makes it recognizable is that the law says how many
///   regions there are and region formation enumerates one candidate per stage.
///   [`recognize_staged_family`] is that arm and it names no operation key, so
///   `tiler::rms-norm-f32@1` and any family registered after it are recognized
///   by the same statement. `tiler::softmax-f32@1` still refuses here because it
///   carries no law at all — `crates/tiler-compiler/tests/softmax_recognizer_boundary.rs`
///   measures that — and a recognized staged family still stops one layer down,
///   where no [`tiler_ir::schedule::ScalarProgram`] spells a stage's work
///   ([`crate::physical::RegionVocabularyWall::StagedFamilyUnspellable`]).
///
///   **Two families used to be named here and no longer are, for two different
///   reasons, and both distinctions are worth keeping.** `tiler::silu-f32@1` has
///   no node of its own, but its per-point body is expressible in the node
///   vocabulary, so the boundary *projects* it — admissibly, because the
///   projection is not written here:
///   [`crate::elementary::silu_point_body`] is the one statement of the
///   composition and the governed index-access lowering drives the same
///   function, so occurrence refinement's proof that the emitted region realizes
///   the occurrence covers the projection. `tiler::reindex-f32@1` and
///   `tiler::broadcast-f32@1` project no body at all; they were refused because
///   `LogicalAccess` spelled neither access relation, and
///   `admit-the-structural-families-into-the-scheduled-region-vocabulary` landed
///   `LogicalAccess::ReindexBijection` and
///   `LogicalAccess::BroadcastReplication`, so each is now recognized by
///   [`recognize_structural_read`] as a *mapped read* contributing addressing and
///   no arithmetic.
/// - **An elementwise stage reading a materialized intermediate**
///   (`operation-set` from the contraction cover, `elementwise-shape` or
///   `operation-set` from the elementwise walk). Every elementwise region this
///   profile builds reads declared input tensors and nothing else, so a
///   contraction or a reduction feeding an elementwise epilogue is a *chain*
///   rather than a refusal.
///
///   **The wall was in the schedule vocabulary rather than in this crate, and it
///   is gone in all three of its rows.** The paragraph that used to stand here
///   reasoned from `tiler_ir::schedule::TensorRole::Intermediate` being a
///   per-region role to the conclusion that nothing in `tiler-ir` forbade the
///   chain. The role is indeed per-region; what forbade the chain was the
///   *access contract* each scalar-program family declares around it.
///   At that step `verify_pointwise_region` required read access `i` to be an
///   input carrying declared ordinal `i` at every position, so no
///   `ScalarProgram::PointwiseF32` region could read an intermediate at all —
///   `admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary`
///   separated the access position from the declared input the role named. The
///   role is fieldless now; the compiler projects each exact access through the
///   retained checked request subject, and a pointwise region may read one
///   materialized intermediate alongside declared inputs.
///   `verify_access_and_semantics` then
///   admitted a fold only when its owning write targeted `TensorRole::Output`,
///   and `admit-a-strict-serial-fold-that-writes-a-materialized-intermediate`
///   replaced that with a cover-assigned obligation at every committing pass. A
///   contraction could already write one.
///   `crates/tiler-compiler/tests/materialized_intermediate_epilogue_wall.rs`
///   measures all three rows.
///
///   [`recognize_epilogue`] is what builds the chain from a recognized program:
///   the elementwise walk *names* the value a folding family produced instead of
///   stopping at it, the producer is recognized as its own shape, and the cover
///   search places the materialization edge between them. What is still refused
///   under `operation-set` is a walk that reaches a *second, different* folded
///   value, and that is a rule about chain **width** rather than depth: the two
///   folds would feed one region two `TensorRole::Intermediate` reads, which is
///   the same unordinalled-role fact [`record_leaf`] refuses for one staged value
///   read twice.
///   [`admit-a-scheduled-region-that-reads-two-materialization-edges`](../../../tickets/admit-a-scheduled-region-that-reads-two-materialization-edges.md)
///   owns the region vocabulary underneath both. The chain-*depth* rule is
///   [`recognize_staged_family`]'s `staged-operand-depth`, stated once at
///   [`StagedOperandAdmission`], which also names the third folded-value wall —
///   a fold whose prologue is itself a chain — and its separate owner.
/// - **A staged family reading a materialized intermediate is admitted**, which
///   is where the last of this rule's rows moved.
///   [`admit-a-staged-family-that-reads-a-materialized-intermediate`](../../../tickets/admit-a-staged-family-that-reads-a-materialized-intermediate.md)
///   gave the recognized staged shape a per-operand [`BoundaryRead`] and the
///   producer whose regions write the edge, so `rms_norm(matmul(a, b), w)` is one
///   output's partition rather than a `staged-operand` refusal. It stops one
///   layer down instead: the consuming stage would read that edge *and* the value
///   the producing stage handed it, which is two `TensorRole::Intermediate`
///   accesses, so [`crate::physical::staged_plan`] declines the occurrence and
///   `crates/tiler-compiler/tests/staged_family_over_a_materialized_intermediate.rs`
///   measures where that leaves it.
///
/// **A reduction reading a declared input directly was the third wall here, and
/// it is gone.** `sum(x)` was refused under `reduction-prologue` because
/// `verify_access_and_semantics` required a `ScalarProgram::StrictSerialSum`
/// region's contributor access to read `TensorRole::Intermediate`;
/// `admit-a-reduction-over-a-declared-input-tensor` widened that arm to the fold's
/// *declared contributor domain*, which is the input tensor the program folds
/// directly or an intermediate when a prologue region wrote it.
/// [`recognize_reduction`] therefore admits the shape with no prologue at all, and
/// the rule name no longer exists.
///
/// Which refusal a rejected program reports is settled by the occurrence it
/// actually ends in rather than by enumeration order: a program whose output is
/// a reduction gets the reduction's reason, one whose output is a contraction
/// gets the contraction's, and any other gets the elementwise walk's. With
/// several declared outputs the walks run in declaration order and the first
/// one that cannot be recognized reports, so the rule names a property of the
/// caller's own interface rather than of a traversal it cannot see.
///
/// # Ordered multi-output programs are admitted, and the arity guard is gone
///
/// This function used to open with `output_count() != 1`, refusing every
/// multi-output program under `output-arity` before any occurrence was
/// classified. That refusal is gone in both of the places it stood — here and
/// `verify_artifact_refinements`'s `semantic-output-coverage` arity check — and
/// nothing replaced it with a narrower cardinality rule: a program declaring
/// several ordered named outputs is now recognized, covered, planned, and
/// assembled like any other.
///
/// What made it removable is that every layer it was standing in for now answers
/// for itself. [`recognize_program_outputs`] walks each declared output and
/// [`check_output_cover`] requires the walks to *partition* the occurrences, so
/// every occurrence is claimed exactly once and every published value has one
/// region that owns its write. The cover carries which named result each region
/// retains, so `CoverAssembly::from_plan` attributes each declared output to its
/// publishing region *by value* rather than by execution order — the pairing
/// that made the guard load-bearing after recognition had already been widened.
///
/// **What a multi-output program is still refused for is the shape of its
/// outputs, never their number.** Two outputs whose walks share an occurrence
/// refuse under `output-partition-overlap`, which is the branch where one
/// region's owning write would have to serve both a materialization edge and a
/// publication: two keys naming one value, and a published intermediate that is
/// also consumed. [`tiler_ir::program::ValueRole`] is exclusive and a region
/// writes one owning tensor, so a cover this boundary let through would die
/// mid-pipeline instead.
///
/// **One spelling of the overlap is now admitted, and the four walls that
/// blocked it came down together.** A published-and-consumed intermediate is
/// realized as two dispatches of the region that computed it — one staging the
/// value the fold reads across, one publishing a copy of it — so the shape is
/// no longer refused here. [`check_output_cover`] owns that rule and states the
/// measured order the four walls fell in; it is not restated here, because two
/// derivations of one measurement are what drift.
/// `crate::pipeline::conformance`'s
/// `a_published_and_consumed_intermediate_compiles_and_agrees` is the compiling
/// assertion, `an_output_key_pair_naming_one_value_still_refuses_by_name` is the
/// neighbour that must keep refusing, and
/// `crates/tiler-compiler/tests/multi_output_boundary.rs` holds the evidence for
/// where the boundary now is.
pub(super) fn select_supported_strategy(
    program: &SemanticProgram,
    laws: &FrozenIndexRealizationLawRegistry,
) -> Result<NormalizedProgram, RequestError> {
    // Program-wide properties first, each under the rule that names it. A
    // program failing one of these fails it for every shape below, so reporting
    // it here is both the more specific statement and the only one that does not
    // depend on which occurrence happens to produce the output.
    if program.input_count() == 0 {
        return mismatch("input-arity");
    }
    let arithmetic = recognized_program_arithmetic(program)?;
    recognize_program_outputs(program, laws, arithmetic)
}

/// The one arithmetic type every value of a recognizable program is stated in.
///
/// **This replaced a `dtype-f32` gate, and the two refusals it splits into are
/// different findings.** The gate refused every program carrying a non-`f32`
/// value, which conflated "this build states no per-point vocabulary for that
/// width" with "this program mixes two widths and therefore has no single scalar
/// program at all". Both still refuse, and each now names the property it found:
///
/// - `dtype-recognized` for a value whose resolved type is neither of the two
///   widths [`RecognizedPointwise`] can spell. Every conversion family is in this
///   arm, which is correct rather than incidental — a program that converts
///   between widths has no *one* arithmetic and a region carrying one realization
///   record cannot realize it.
/// - `dtype-uniform` for a program whose values are two recognized widths at
///   once. A scheduled region carries one [`ArithmeticType`] worth of numerical
///   realization and one scalar-program vocabulary, so a mixed-width program is
///   refused here rather than compiled under whichever width happened to be
///   first.
///
/// **What it deliberately does not decide is whether the width can be
/// dispatched or its contract honoured.** Those are the target profile's and the
/// numerical contract's, they run before this function, and each reports its own
/// typed refusal: [`require_compile_profile_dispatch`] for a width the profile
/// names no dispatch fact for, [`resolve_numerical_contract`] for a contract no
/// stated entry resolves, and
/// [`RequestError::NoApplicableNumericalContract`] for a preference no entry of
/// which is about this program's arithmetic at all.
pub(super) fn recognized_program_arithmetic(
    program: &SemanticProgram,
) -> Result<ArithmeticType, RequestError> {
    let mut recognized: Option<ArithmeticType> = None;
    for value in program.values() {
        let Some(arithmetic) = recognized_arithmetic(value.resolved_type()) else {
            return mismatch("dtype-recognized");
        };
        match recognized {
            Some(seen) if seen != arithmetic => return mismatch("dtype-uniform"),
            Some(_) => {}
            None => recognized = Some(arithmetic),
        }
    }
    // Unreachable through the caller, which has already refused a program
    // declaring no input, and refused by name rather than defaulted: a width
    // nothing derived is not a width this build may compile under.
    recognized.ok_or(RequestError::UnsupportedCapability {
        phase: "strategy",
        rule: "dtype-recognized",
    })
}

/// The arithmetic type one resolved value type names, when this build states a
/// per-point vocabulary for it.
///
/// **The single statement of which widths recognition admits.** Every authority
/// that needs the set asks this rather than restating it —
/// [`recognized_program_arithmetic`] derives a program's width from it, and
/// [`crate::program::verify_semantic_output_type`] checks a declared output
/// against it — because two lists would be free to disagree about which
/// programs the compiler claims it can plan, and the disagreement's shape is a
/// program admitted by one and refused by the other after a plan exists.
pub(crate) fn recognized_arithmetic(resolved: &ResolvedValueType) -> Option<ArithmeticType> {
    if resolved == &F32::resolved_type() {
        Some(ArithmeticType::F32)
    } else if resolved == &Bf16::resolved_type() {
        Some(ArithmeticType::Bf16)
    } else {
        None
    }
}

/// Recognizes every ordered named output of one verified program.
///
/// **One walk per declared output, and the outputs are recognized in declaration
/// order** — the order is identity rather than presentation, and the recognized
/// list preserves it so that the request subject, the cover's named-output
/// attribution, and the assembled program's interface all speak about the same
/// ordering the caller declared.
///
/// The per-output walk is exactly the one a single-output program has always
/// taken: the occurrence producing the output decides the shape, and the
/// occurrences feeding it are walked outward. What changed is where the
/// whole-program obligation lives. Each recognizer used to end by demanding that
/// its own walk cover the program exactly; that demand is now
/// [`check_output_cover`]'s, stated over the walks together, and it is strictly
/// the same requirement when there is one output.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] under `missing-output` for a
/// program declaring none, every rule the per-output recognizers report, and the
/// two [`check_output_cover`] states: `operation-set` for an occurrence no
/// walk claimed, and `output-partition-overlap` for one claimed twice.
pub(super) fn recognize_program_outputs(
    program: &SemanticProgram,
    laws: &FrozenIndexRealizationLawRegistry,
    arithmetic: ArithmeticType,
) -> Result<NormalizedProgram, RequestError> {
    if program.output_count() == 0 {
        return unsupported("strategy", "missing-output");
    }
    let mut outputs = Vec::with_capacity(program.output_count());
    for output in program.outputs() {
        outputs.push(recognize_output(program, &output, laws, arithmetic)?);
    }
    check_output_cover(program, &outputs)?;
    Ok(NormalizedProgram { outputs })
}

/// Recognizes the region partition implementing one ordered named output.
fn recognize_output(
    program: &SemanticProgram,
    output: &tiler_ir::semantic::ProgramOutputRef<'_>,
    laws: &FrozenIndexRealizationLawRegistry,
    arithmetic: ArithmeticType,
) -> Result<NormalizedOutput, RequestError> {
    // An output that *is* a declared input computes nothing: it names no
    // operation for any region to realize. The property that was not recognized
    // is its operation set, so it is reported under that rule rather than as the
    // missing producer a bare graph walk would report.
    if program
        .inputs()
        .any(|input| input.value() == output.value())
    {
        return mismatch("operation-set");
    }
    let (member, root) = producer_for_value(program, output.value())?;
    if root.key() == &strict_serial_sum_f32_op() {
        recognize_reduction(
            program,
            output.value(),
            output.key().clone(),
            member,
            &root,
            laws,
        )
        .map(NormalizedOutput::SerialSum)
    } else if root.key() == &tensor_contraction_f32_op() {
        normalize_contraction(program, output.value(), output.key().clone())
            .map(|normalized| NormalizedOutput::Contraction(Box::new(normalized)))
    } else if laws.family_realizes_region_sequence(root.key()) {
        recognize_staged_family(
            program,
            laws,
            output.value(),
            output.key().clone(),
            member,
            &root,
            // The declared output's own occurrence is at the near side of every
            // materialization edge this walk may place, so it is the one that
            // may read one. See [`recognize_staged_family`]'s
            // `staged-operand-depth` refusal for the far side.
            StagedOperandAdmission::OneEdge,
        )
        .map(|normalized| NormalizedOutput::Staged(Box::new(normalized)))
    } else {
        recognize_elementwise_output(program, output, laws, arithmetic)
    }
}

/// Recognizes an output whose producing occurrence is elementwise.
///
/// **Two shapes share this entry, and which one a program is depends on a fact
/// only the walk can report.** An elementwise expression over declared inputs is
/// one region; the same expression over a *folded* value is two, because no
/// per-point body spells a fold. The walk is run once and its answer decides:
/// a completed plan is the whole-program shape, and a plan that stopped at a
/// folding family names the value the chain materializes.
///
/// Deciding it this way rather than by pre-scanning the graph is deliberate. A
/// pre-scan would be a second classifier of the same operand DAG, and the two
/// would have to agree about constants, structural occurrences, shapes, and
/// arity for the answer to mean anything — which is exactly the drift a single
/// authority exists to prevent.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] under `output-handle` for an
/// output the program holds no shape for, `elementwise-rank` for a rank-zero
/// domain no region iterates, and every rule [`plan_elementwise`],
/// [`mint_elementwise`], and [`recognize_epilogue`] report.
pub(super) fn recognize_elementwise_output(
    program: &SemanticProgram,
    output: &tiler_ir::semantic::ProgramOutputRef<'_>,
    laws: &FrozenIndexRealizationLawRegistry,
    arithmetic: ArithmeticType,
) -> Result<NormalizedOutput, RequestError> {
    let declared: Vec<ValueId> = program.inputs().map(|input| input.value()).collect();
    let shape = sourced_shape(program, output.value(), "output-handle")?;
    if shape.rank() == 0 {
        return mismatch("elementwise-rank");
    }
    let leaves = ElementwiseLeaves {
        declared: &declared,
        staged: None,
    };
    match plan_elementwise(program, output.value(), &leaves, &shape, laws, arithmetic) {
        Ok(plan) => recognize_pointwise(program, output, &declared, shape, plan, arithmetic)
            .map(NormalizedOutput::Pointwise),
        Err(ElementwiseRefusal::Folded(staged)) => {
            let Some(static_shape) = shape.as_static() else {
                return Err(unsupported_symbolic_extent(program, output.value(), &shape));
            };
            recognize_epilogue(
                program,
                output,
                &declared,
                static_shape.clone(),
                staged,
                laws,
                arithmetic,
            )
            .map(|chain| NormalizedOutput::Epilogue(Box::new(chain)))
        }
        Err(ElementwiseRefusal::Refused(error)) => Err(error),
    }
}

/// Requires the recognized walks to partition the program's occurrences and to
/// read every declared input between them.
///
/// **Three obligations, and they are separate claims about different failures.**
///
/// *Every occurrence is claimed by some walk* (`operation-set`). A built program
/// retains only output-reachable operations, so an unclaimed one is work no
/// region would compute and the assembled program would silently drop. This is
/// the widened form of the check each recognizer used to make alone — with one
/// declared output the union is that output's own member set and the rule is
/// unchanged, which is why widening it rather than removing it is what keeps the
/// uncovered case refused.
///
/// *Every occurrence claimed twice is claimed by the one admitted overlap*
/// (`output-partition-overlap`). Two outputs whose walks share an occurrence are
/// the shape where one value is both published and consumed, and exactly one
/// spelling of it is admitted — [`published_and_consumed_overlap`] is the
/// predicate and states what it proves. Everything else still refuses here,
/// including two output keys naming one value, because
/// [`tiler_ir::program::ValueRole`] is exclusive and a dispatch owns one write,
/// so a cover the boundary let through would die mid-pipeline instead.
///
/// **What lifted the admitted case was four walls, and only the last was a crate
/// down.** Measured by disabling each in turn against a governed spelling of the
/// fixture: this rule; then [`crate::program`]'s `cover-named-output-attribution`
/// and its `internal-unwritten`, both widenings *here*, because the cover legally
/// places one region as both the edge's producer and the publication's retainer
/// and that region needs a *second dispatch* to write the publication; and only
/// then `tiler_ir::program`'s `UncoveringStage`, which its publishing-copy
/// declaration now accounts for. `crate::pipeline::conformance`'s
/// `a_published_and_consumed_intermediate_compiles_and_agrees` is the compiling
/// assertion and `an_output_key_pair_naming_one_value_still_refuses_by_name` is
/// the neighbour that must keep refusing.
///
/// *Every declared input is read by some walk* (`input-set`). This is the
/// obligation [`canonical_input_reads`] used to state per walk, under
/// `elementwise-reads`, and it was the same requirement while a program had one
/// declared output: that walk's read set was the program's. With several
/// outputs the walks split the declared inputs between them, so the per-walk
/// form refused a program whose *union* was complete —
/// `admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs` is
/// what moved it here rather than deleting it. What it protects is unchanged:
/// a declared input no region reads is a buffer the caller binds, the ABI
/// declares, and no kernel loads.
///
/// **It is defence in depth here, and the derivation says so rather than the
/// check pretending otherwise.** `SemanticProgramBuilder` freezes only
/// output-reachable values, so a retained declared input is an operand of some
/// retained occurrence; the `operation-set` obligation above claims every
/// retained occurrence for some walk; and every way a walk consumes an operand
/// — an elementwise node, a structural occurrence, a fold's contributor, a
/// contraction's operand — records a read of it. So no program the public
/// builder can construct reaches this refusal, and `tiler_ir::program`'s
/// `verify_usage` refuses the same shape a layer down under `unused-value`.
/// Stating it here is what makes the boundary report the program property
/// instead of letting an assembled program die naming a different authority.
///
/// Claimed counts are taken over the deduplicated per-output member sets, so one
/// constant shared by two operands of the *same* walk contributes one member
/// rather than two — the normalized spelling of one program, not a duplicate.
pub(super) fn check_output_cover(
    program: &SemanticProgram,
    outputs: &[NormalizedOutput],
) -> Result<(), RequestError> {
    let claimed: Vec<Vec<SemanticStage>> = outputs.iter().map(NormalizedOutput::members).collect();
    let total: usize = claimed.iter().map(Vec::len).sum();
    let mut distinct: Vec<SemanticStage> = claimed.iter().flatten().copied().collect();
    distinct.sort_unstable();
    distinct.dedup();
    if total != distinct.len()
        && published_and_consumed_overlap(program, outputs, &claimed).is_none()
    {
        return mismatch("output-partition-overlap");
    }
    if program.operation_count() != distinct.len() {
        return mismatch("operation-set");
    }
    for position in 0..program.input_count() {
        let Ok(ordinal) = u32::try_from(position) else {
            return mismatch("input-ordinal");
        };
        if !outputs
            .iter()
            .any(|output| output.reads_declared_input(DeclaredInputOrdinal::new(ordinal)))
        {
            return mismatch("input-set");
        }
    }
    Ok(())
}

/// Recognizes the one overlap between two recognized walks this boundary admits,
/// as `(published output, consuming output)` declaration positions.
///
/// **The predicate is not "any overlap", and each conjunct is load-bearing.**
///
/// *Exactly one pair of walks overlaps.* A value published and consumed by two
/// different downstream outputs, or two independent published-and-consumed
/// values, would need a cover shape nothing below here expresses — the first
/// because `cover-region-multiple-materializations` refuses a region producing
/// two edges, the second because both would have to be the same region's second
/// dispatch. They are unsupported cases that reject explicitly rather than being
/// approximated.
///
/// *One walk's member set is a strict subset of the other's.* Two walks that
/// merely intersect share work neither wholly owns, which is the shape where one
/// region's write would have to serve two publications.
///
/// *The shorter walk is one whole **part** of the longer walk's recognized
/// partition*, asked through [`NormalizedOutput::owns_region_members`] — the same
/// authority [`crate::physical::spell_region`] resolves a region against. A
/// subset that is not a part has no scheduled region of its own, so nothing could
/// publish it without splitting a region the recognizer did not split.
///
/// *The published value is the one crossing the part boundary*: some occurrence
/// of the longer walk **outside** the part reads it. That is what makes the
/// publication and the materialization edge the same value, and it is the
/// conjunct that distinguishes this from a subset walk publishing some *other*
/// value the part happens to compute.
///
/// What it does not prove is that a cover placing the part as one region exists;
/// that is the cover search's answer, and a program admitted here whose cover
/// cannot be assembled is refused by name at the assembler.
pub(super) fn published_and_consumed_overlap(
    program: &SemanticProgram,
    outputs: &[NormalizedOutput],
    claimed: &[Vec<SemanticStage>],
) -> Option<(usize, usize)> {
    let mut overlapping = None;
    for short in 0..claimed.len() {
        for long in (short + 1)..claimed.len() {
            if !claimed[short]
                .iter()
                .any(|member| claimed[long].contains(member))
            {
                continue;
            }
            if overlapping.is_some() {
                return None;
            }
            overlapping = Some((short, long));
        }
    }
    let (first, second) = overlapping?;
    // Orient the pair by containment rather than by declaration order: which
    // output publishes the consumed value is a fact about the walks, and the
    // caller may declare them either way round.
    let (published, consuming) = if claimed[first].len() < claimed[second].len() {
        (first, second)
    } else {
        (second, first)
    };
    if claimed[published].len() >= claimed[consuming].len()
        || !claimed[published]
            .iter()
            .all(|member| claimed[consuming].contains(member))
    {
        return None;
    }
    if !outputs[consuming].owns_region_members(&claimed[published]) {
        return None;
    }
    let staged = program.outputs().nth(published)?.value();
    let crosses = program
        .operations()
        .enumerate()
        .filter(|(ordinal, _)| {
            u32::try_from(*ordinal).is_ok_and(|ordinal| {
                !claimed[published]
                    .iter()
                    .any(|atom| atom.member().0 == ordinal)
            })
        })
        .any(|(_, operation)| operation.operands().any(|operand| operand == staged));
    crosses.then_some((published, consuming))
}
