use super::*;

/// The workload's own projection shape is refused by the *target*, not by
/// recognition.
///
/// Two claims, and they are different. A program carrying the pinned workload's
/// `[128, 1024] x [3072, 1024]` projection is now recognized, lowered, scheduled,
/// and assembled — the request boundary and the lowering registry both admit it.
/// What refuses is the governed baseline target profile, whose `GridAxisThreads`
/// bound is four: 393,216 output elements is a hard-feasibility refusal naming
/// that axis, and it is the same refusal the four-element pointwise fixtures in
/// this file would get at this size.
///
/// Asserting a recognition refusal here would now be fiction about which check
/// said no, which is exactly what this test guarded against in the other
/// direction before the direct path landed. The compiling case is
/// `tests/contraction_direct_path.rs`, at a shape the baseline admits.
#[test]
fn a_contraction_of_the_workload_shape_is_refused_by_the_target_not_by_recognition() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let activations = builder
        .input::<F32>(
            InputKey::new("activations").unwrap(),
            Shape::from_dims([128, 1024]),
        )
        .unwrap();
    let weights = builder
        .input::<F32>(
            InputKey::new("weights").unwrap(),
            Shape::from_dims([3072, 1024]),
        )
        .unwrap();
    // `td,od->to`, spelled with the frontend's own labels.
    let structure = ContractionIndexStructure::new(
        [
            [ContractionIndex::new(19), ContractionIndex::new(3)],
            [ContractionIndex::new(14), ContractionIndex::new(3)],
        ],
        [ContractionIndex::new(19), ContractionIndex::new(14)],
    )
    .unwrap();
    let projected =
        F32TensorContraction::apply(&mut builder, &structure, activations, weights).unwrap();
    builder
        .output(OutputKey::new("projected").unwrap(), projected)
        .unwrap();
    let semantic = builder.build().unwrap();
    assert_eq!(semantic.operation_count(), 1);

    let product = compile(CompilationRequest::governed(&semantic))
        .expect("recognition, lowering, and assembly all admit the projection");
    let Some(CompileError::Explained { source, .. }) = product.targets[0].failure() else {
        panic!("the baseline profile launches at most four threads");
    };
    assert_eq!(
        source.as_ref(),
        &CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(PhysicalError::Target {
            rule: "grid-axis",
            region: tiler_ir::schedule::RegionId::new(0),
            required: 393_216,
            available: 4,
        })),
        "the refusal is the target's launch bound, named as such",
    );
}

/// `activations[m, k] x weights[n, k] -> projected[m, n]`, the L3 profile's
/// index structure `td,od->to` at extents inside the governed baseline's
/// four-thread launch bound.
///
/// The labels are arbitrary so the renaming-invariant canonicalization is
/// exercised rather than assumed, exactly as
/// `tests/contraction_direct_path.rs` spells it.
fn projection_program(m: u64, n: u64, k: u64) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let activations = builder
        .input::<F32>(
            InputKey::new("activations").unwrap(),
            Shape::from_dims([m, k]),
        )
        .unwrap();
    let weights = builder
        .input::<F32>(InputKey::new("weights").unwrap(), Shape::from_dims([n, k]))
        .unwrap();
    let structure = ContractionIndexStructure::new(
        [
            [ContractionIndex::new(19), ContractionIndex::new(3)],
            [ContractionIndex::new(14), ContractionIndex::new(3)],
        ],
        [ContractionIndex::new(19), ContractionIndex::new(14)],
    )
    .unwrap();
    let projected =
        F32TensorContraction::apply(&mut builder, &structure, activations, weights).unwrap();
    builder
        .output(OutputKey::new("projected").unwrap(), projected)
        .unwrap();
    builder.build().unwrap()
}

/// Every strategy the governed provider considered for the contraction subject
/// and withheld, under one stated contract.
///
/// Returns the pairs rather than the whole frontier so the three tests below
/// compare the *same* observation under three contracts: what changes between
/// them is the contract and nothing else.
fn contraction_declines_under(
    contract: StrictF32NumericalContract,
) -> Vec<(&'static str, crate::frontier::StrategyDeclineCause)> {
    let semantic = projection_program(2, 2, 2);
    let verified = verify_planned_request(CompilationRequest::governed_under(&semantic, contract))
        .expect("the stated contract is admitted");
    let request = verified
        .for_target(verified.target_profiles()[0])
        .expect("the governed target resolves the stated contract");
    let members = request
        .contraction()
        .expect("the projection normalizes to a contraction")
        .members
        .clone();
    let subject = FrontierRegionSubject::new(
        "contraction",
        members,
        crate::physical::RegionWrite::ProgramOutput,
    );
    let providers: [&dyn PhysicalImplementationProvider; 1] = [&GovernedPhysicalProvider];
    let frontier = enumerate_frontier(
        &request,
        &subject,
        &providers,
        &crate::call_registry::OpaqueCallRegistry::new(),
        &crate::lowering::ResolvedLowering::unresolved_for_test(),
    )
    .expect("the governed provider emits well-formed proposals");
    // The direct fold is still offered; the declines are additive beside it.
    // Asserting this here means a decline set that grew by swallowing the offer
    // fails every test below rather than reading as a richer explanation.
    assert_eq!(
        frontier.admitted().len(),
        1,
        "the direct fold stopped being the one offered contraction realization"
    );
    let mut declined: Vec<_> = frontier
        .rejections()
        .iter()
        .filter_map(|rejection| match rejection {
            crate::frontier::FrontierRejection::StrategyDeclined {
                strategy, cause, ..
            } => Some((*strategy, *cause)),
            _ => None,
        })
        .collect();
    declined.sort_by_key(|(strategy, _)| *strategy);
    declined
}

/// **The ticket's core claim:** each uncovered contraction realization is
/// declined under its own ground, and no two of them share an answer.
///
/// The L3 elimination measured six realizations and this build offers exactly
/// one, the direct fold. A caller asking why gets three named alternatives back
/// rather than an absence — and told only "a permission was refused" could not
/// tell a contiguous split from a strided one from a matrix instruction, which
/// is why the three grounds are asserted individually rather than counted.
///
/// The grounds are the ones the record states, and the cause variant is ADR
/// 0014's two-fact rule naming *which* fact is missing: the contiguous split
/// consumes reassociation, which this family grants and the strict contract
/// withholds, so the failing source is the caller's; the strided split
/// additionally permutes leaves and the matrix instruction fuses a multiply into
/// an add, and for both the failing source is the operation's own declared
/// maximum.
#[test]
fn each_uncovered_contraction_realization_is_declined_under_its_own_ground() {
    let declined = contraction_declines_under(StrictF32NumericalContract::governed());
    assert_eq!(
        declined,
        vec![
            (
                "tiler.contraction.contiguous-k-split",
                crate::frontier::StrategyDeclineCause::NumericalPermissionRefused {
                    dimension: "numerics.reassociation",
                },
            ),
            (
                "tiler.contraction.simdgroup-matrix",
                crate::frontier::StrategyDeclineCause::AlgebraicCapabilityUnsupported {
                    dimension: "numerics.contraction",
                },
            ),
            (
                "tiler.contraction.strided-k-split",
                crate::frontier::StrategyDeclineCause::AlgebraicCapabilityUnsupported {
                    dimension: "numerics.permutation",
                },
            ),
        ],
        "the strict contract left an uncovered contraction realization unexplained"
    );
    // Three distinct answers, not one wearing three names. A caller separating
    // a split from a matrix instruction reads the *cause*, so two equal causes
    // would leave it unable to however the strategy names differ.
    for (left, right) in [(0, 1), (0, 2), (1, 2)] {
        assert_ne!(
            declined[left].1, declined[right].1,
            "two contraction declines collapsed into one answer: {declined:?}"
        );
    }
}

/// The contiguous split's decline is **a function of the contract**; the other
/// two are not, and that asymmetry is the measured distinction.
///
/// `REASSOCIATE_F32` grants ordered regrouping and nothing else. The contiguous
/// split consumes exactly that, so its decline disappears — whether the split is
/// then *offered* belongs to
/// `admit-reassociated-contraction-schedule-alternatives` and deliberately does
/// not happen here. The strided split's leaves are reordered, and granting
/// reassociation never grants permutation, so its decline stands unchanged. That
/// is the L3 record's `ksplit_strided` result — the measured demonstration that
/// the two splits are different plans and not one — expressed as an outcome a
/// caller can read.
///
/// Without this control the declines could be constants: a set that named the
/// same three grounds under every contract would be reporting the code rather
/// than the request.
#[test]
fn granting_reassociation_retires_only_the_contiguous_split_decline() {
    let strict = contraction_declines_under(StrictF32NumericalContract::governed());
    let reassociating =
        contraction_declines_under(StrictF32NumericalContract::governed_reassociating());
    assert!(
        strict
            .iter()
            .any(|(strategy, _)| *strategy == "tiler.contraction.contiguous-k-split"),
        "the strict contract did not decline the contiguous split at all"
    );
    assert!(
        !reassociating
            .iter()
            .any(|(strategy, _)| *strategy == "tiler.contraction.contiguous-k-split"),
        "a reassociating contract still refused the contiguous split a permission it granted: \
         {reassociating:?}"
    );
    assert_eq!(
        reassociating,
        vec![
            (
                "tiler.contraction.simdgroup-matrix",
                crate::frontier::StrategyDeclineCause::AlgebraicCapabilityUnsupported {
                    dimension: "numerics.contraction",
                },
            ),
            (
                "tiler.contraction.strided-k-split",
                crate::frontier::StrategyDeclineCause::AlgebraicCapabilityUnsupported {
                    dimension: "numerics.permutation",
                },
            ),
        ],
        "granting reassociation changed a decline it does not reach"
    );
}

/// Permitting ADR 0015 contraction at the *ceiling* still does not reach the
/// matrix realization, because the fact that withholds it is the operation's.
///
/// `RELAXED_F32` permits contraction and reassociation. The contiguous split's
/// decline retires with the reassociation grant, exactly as above; the matrix
/// realization's does not, and it keeps the algebraic cause rather than
/// downgrading to a numerical one. This is the half of ADR 0014's rule that a
/// single-source report would lose: a caller reading "numerical permission
/// refused" here would go looking for a contract to widen, and no contract
/// exists that reaches it.
#[test]
fn a_contraction_permitting_ceiling_still_withholds_the_matrix_realization() {
    let relaxed = contraction_declines_under(StrictF32NumericalContract::governed_relaxed());
    assert_eq!(
        relaxed,
        vec![
            (
                "tiler.contraction.simdgroup-matrix",
                crate::frontier::StrategyDeclineCause::AlgebraicCapabilityUnsupported {
                    dimension: "numerics.contraction",
                },
            ),
            (
                "tiler.contraction.strided-k-split",
                crate::frontier::StrategyDeclineCause::AlgebraicCapabilityUnsupported {
                    dimension: "numerics.permutation",
                },
            ),
        ],
        "a contraction-permitting ceiling changed which fact withholds the matrix realization"
    );
}

/// The algebraic maxima [`crate::frontier`]'s contraction declines assume are
/// the ones this build actually registered.
///
/// The declines carry those maxima as constants because a
/// `VerifiedTargetRequest` holds no frozen semantic registry to decode from, so
/// nothing else in the compile path would notice a registration that stopped
/// agreeing with them. This is the tie: a later key generation that declared
/// fold permutation, or admitted ADR 0015 fusion, fails here rather than leaving
/// a decline standing that names a freedom the operation now grants.
///
/// **What it takes for this to say *no*, because two of these assertions cannot
/// fail on their own and saying otherwise would overstate the check.**
/// `ContractionF32ReductionDescriptor`'s decoder returns `Unsupported` for
/// permutation and signed-zero as literals rather than from the decoded row, and
/// `arithmetic_contraction_supported` and `distributivity_supported` are
/// `const fn`s answering `false`, so no edit to the registered facts alone can
/// move them. Both reachable routes were driven while this landed. Widening the
/// registered permutation row to `permission-gated` and nothing else reddens the
/// `expect` below rather than an assertion, because the decoder fail-closes
/// first: `InvalidGovernedContractionDescriptor { source: UnsupportedValue {
/// field: Reduction(AttributeFieldId(5)) } }`. Widening the decoder to admit and
/// return that row as well — which is exactly what admitting fold permutation
/// would require — then reddens the assertion itself, `left: PermissionGated,
/// right: Unsupported`. So the decode is the load-bearing guard today and the
/// assertions become live the moment the decoder widens, which is the moment
/// they matter.
#[test]
fn the_algebraic_maxima_these_declines_assume_are_the_registered_ones() {
    let semantic = projection_program(2, 2, 2);
    let descriptor = tiler_ir::semantic::tensor_contraction_f32_reduction_descriptor(
        semantic.semantic_registry(),
    )
    .expect("the governed contraction registers a typed reduction descriptor");
    assert_eq!(
        descriptor.permutation(),
        tiler_ir::semantic::ContractionF32OrderFreedom::Unsupported,
        "fold permutation became supported, so the strided split's decline is now stale"
    );
    assert!(
        !descriptor.arithmetic_contraction_supported(),
        "ADR 0015 fusion became supported, so the matrix realization's decline is now stale"
    );
    // Distributivity is the third explanation the L3 record names, and no
    // decline above carries it: nothing among these three realizations consumes
    // it — a contraction-*order* rewrite would, and this build enumerates none.
    // It is asserted here so that admitting one finds this note rather than
    // reading the absent decline as an oversight.
    assert!(
        !descriptor.distributivity_supported(),
        "distributivity became supported, so a contraction-order rewrite is now expressible \
         and owes a decline of its own"
    );
    // The reassociation maximum is the other half of the same rule: the
    // contiguous split's decline is numerical *because* the operation grants
    // the freedom and only the ceiling withholds it. A maximum of `unsupported`
    // here would make that decline name the wrong source.
    assert_eq!(
        descriptor.reassociation(),
        tiler_ir::semantic::ContractionF32OrderFreedom::PermissionGated,
        "reassociation stopped being permission-gated, so the contiguous split's decline \
         names the wrong one of the two facts"
    );
}
