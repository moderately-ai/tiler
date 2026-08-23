use super::support::{alternative, rule_counts, semantic};
use super::*;

/// A chain of `operations` output-reachable occurrences: one input, one hoisted
/// constant, and `operations - 1` multiplies against it.
///
/// The same family `spikes/program-planning/identity-growth` sweeps, so a wall
/// that moves there and a regression here name the same program. A pure
/// multiply chain rather than a mixed body because a region holding a multiply
/// beside an add is refused under the contraction-permitting contract, which
/// would make admissibility depend on the contract rather than on the width.
fn multiply_chain(operations: usize) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard registry binds");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("the input key is valid"),
            Shape::from_dims([4]),
        )
        .expect("the input binds");
    let scale = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).expect("the scale applies");
    let mut current = input;
    for _ in 1..operations {
        current = F32Multiply::apply(&mut builder, current, scale).expect("the product applies");
    }
    builder
        .output(
            OutputKey::new("result").expect("the output key is valid"),
            current,
        )
        .expect("the output binds");
    builder.build().expect("the chain verifies")
}

/// **A program inside every governed budget is not refused because explaining
/// it would not fit.**
///
/// Eleven occurrences over twelve values, against `semantic_operations = 62`
/// and `semantic_values = 80`. Cover enumeration reaches thousands of legal
/// covers here, and while the coverage-gap rule emitted one record per (cover,
/// region) pair it produced the 6,143 counted below — past both the explain
/// writer's `MAX_RECORDS` and its `MAX_CANONICAL_BYTES`, so the writer refused
/// the trace and the compilation was classed `InvalidCompilerOutput`: a class
/// the public documentation defines as "always a defect in Tiler rather than in
/// the caller's program", raised for a program that had nothing wrong with it.
///
/// The two numbers are the regression, and the relation between them is what
/// makes it one: **sixty-five records account for six thousand one hundred and
/// forty-three pairs**, because the record population is now the count of
/// regions nothing implemented and not the count of covers that placed one.
/// A change that reintroduced per-cover records would leave the second number
/// alone and multiply the first by about ninety-four.
#[test]
fn an_eleven_operation_chain_inside_every_budget_compiles() {
    let program = multiply_chain(11);
    let product = compile(CompilationRequest::governed(&program))
        .expect("a program inside every governed budget compiles");
    let trace = &product.targets[0].explain;

    let gaps: Vec<&crate::explain::ExplainRecord> = trace
        .records()
        .iter()
        .filter(|record| record.rule().key().as_str() == "selection.region-coverage.v1")
        .collect();
    let mut blocked = 0_u64;
    for gap in &gaps {
        let ExplainEvent::Check { assessment, .. } = gap.event() else {
            panic!("a coverage gap is a checked predicate");
        };
        let count = assessment
            .facts()
            .iter()
            .find(|fact| fact.key().as_str() == "blocked-covers")
            .map(crate::explain::ExplainFact::value)
            .expect("a coverage gap counts the covers it blocked");
        let crate::explain::FactValue::Count(count) = count else {
            panic!("a blocked-cover tally is a count");
        };
        blocked += *count;
    }
    assert_eq!(
        gaps.len(),
        65,
        "one coverage gap per region nothing implemented",
    );
    assert_eq!(
        blocked, 6_143,
        "the (cover, region) pairs those sixty-five records account for",
    );
}

/// Every draft authority the conformance gate wires must speak the explain
/// vocabulary; a silent authority cannot be audited.
#[test]
fn every_wired_authority_emits_its_typed_explain_records() {
    let semantic = semantic(false);
    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let trace = &product.targets[0].explain;
    // The exhaustive snapshot: every rule the wired compile path emits, and
    // exactly how many records each contributes. A new authority that stays
    // explain-silent, or one that becomes chatty, fails here.
    assert_eq!(
        rule_counts(trace),
        BTreeMap::from([
            ("compile.request.general-boundary", 1),
            ("region.formation.v1", 1),
            ("region.candidate.v1", 17),
            // One resolution and one refinement per recognized occurrence.
            ("capability.index-access-resolution.v1", 5),
            ("kernel.index-region-refinement.v1", 5),
            ("cover.enumeration.v1", 1),
            ("fusion.legality.v1", 12),
            ("fusion.strict-f32-equivalence", 1),
            // Two summary records per region subject: admitted count and
            // rejected count. Typed per-opaque-rejection detail records accompany
            // them when present; this governed compile fixture has no opaque
            // rejection, so its **seventeen** region subjects contribute
            // thirty-four.
            //
            // Seventeen rather than the four this census used to record, and
            // the difference is the explain half of the region-general
            // provider: the frontier record is keyed by each region's canonical
            // occurrence rather than by its role, so the fourteen subjects that
            // share the role `unrecognized` are fourteen records instead of one
            // record and thirteen silences.
            ("frontier.enumeration.v1", 34),
            // Sixteen: the two parallel reduction strategies, plus the serial
            // baseline withheld once for each of the fourteen region subjects
            // this schedule vocabulary cannot spell.
            //
            // The two are the multi-pass split and the single-workgroup tree,
            // both at the reduction subject and both for the same reason — this
            // fixture compiles under the strict contract, and each *is* a
            // reassociation of the declared contributor sequence. No other
            // subject reaches either strategy, so a seventeenth record would
            // mean one was being considered somewhere it does not apply.
            ("frontier.strategy-decline.v1", 16),
            ("selection.complete-plan.v1", 1),
            // One per unimplemented region, each carrying the number of covers
            // it blocked. Fourteen are the `unrecognized` region subjects this
            // fixture's schedule vocabulary cannot spell; their counts sum to
            // the thirty-eight (cover, region) pairs the rule used to emit one
            // record apiece for.
            ("selection.region-coverage.v1", 14),
            ("compile.region.verified", 3),
            ("compile.plan.boundary", 2),
            ("schedule.plan-regions", 2),
            ("kernel.plan-refinement", 2),
            ("program.plan-determinism.v1", 2),
            ("program.plan-verified", 2),
            ("artifact.plan-construction", 2),
            ("target.buffer-bindings", 3),
            ("target.device-memory", 3),
            ("target.grid-axis", 3),
            ("target.index-arithmetic-u64", 3),
            ("target.local-memory-bytes", 3),
            // One honourability record per realized dimension per region:
            // three regions each report which behaviour was assessed and by
            // what means. Materialization rounding stays off this census until
            // a region can record it.
            ("target.numerics.approximate-intrinsics", 3),
            ("target.numerics.contraction", 3),
            ("target.numerics.infinity-assumptions", 3),
            ("target.numerics.input-subnormals", 3),
            ("target.numerics.nan-assumptions", 3),
            ("target.numerics.permutation", 3),
            ("target.numerics.reassociation", 3),
            ("target.numerics.reciprocal-transform", 3),
            ("target.numerics.result-subnormals", 3),
            ("target.numerics.signed-zero", 3),
            ("target.threads-per-workgroup", 3),
            // Two retained plans, two records each: exact terms share one
            // checked-invariant assessment, while the memory-traffic bound
            // shares one assumption assessment. Grouping by evidence class is
            // the mechanism that moved this census: one assessment has one
            // basis, so mixing the terms would make one class lie.
            ("tiler.cost.analytical.v1", 4),
            // One record per legal cover the partition search's own estimate
            // beat, naming the pruned cover and the cover that beat it. Sixteen
            // covers are enumerated for this program and fifteen of them are
            // dominated by the fused one, which crosses no boundary at all — so
            // the count is a fact about this program's cover space rather than
            // an arbitrary number. It is deliberately a *cost* record: those
            // fifteen covers are legal, and only a refused one is a rejection.
            ("tiler.cost.partition-structural.v1", 15),
            ("tiler.cost.structural.v1", 2),
            ("tiler.selection.structural-pareto.v1", 2),
        ])
    );
    assert!(
        trace.records().iter().all(|record| {
            record.rule().key().as_str() != "target.barriers"
                && !matches!(
                    record.event(),
                    ExplainEvent::Feasibility { predicate, .. }
                        if predicate.as_str() == "barriers"
                )
        }),
        "a zero-synchronization program emitted an invented barrier capability fact"
    );
    // The same absence one layer up. The retired barrier-count axis is gone, and
    // the *realization* record that replaced it must not appear either: a program
    // with no synchronization point derives no requirement, so the authority
    // consults no fact and there is no check to report. A record saying
    // "undeclared" would be the manufactured zero in a new spelling — it would
    // read as a target limitation rather than as a question never asked.
    assert!(
        trace.records().iter().all(|record| {
            !record
                .rule()
                .key()
                .as_str()
                .starts_with("target.synchronization")
                && !matches!(
                    record.event(),
                    ExplainEvent::SynchronizationRealization { .. }
                )
        }),
        "a zero-synchronization program emitted a synchronization-realization record"
    );
    assert!(
        trace.records().iter().all(|record| {
            record.rule().key().as_str() != "target.device-address-bits"
                && !matches!(
                    record.event(),
                    ExplainEvent::Feasibility { predicate, .. }
                        if predicate.as_str() == "device-address-bits"
                )
        }),
        "a program with no address-width requirement emitted an address-width fact"
    );
    let analytical = trace
        .records()
        .iter()
        .filter(|record| record.rule().key().as_str() == ANALYTICAL_MODEL_KEY)
        .collect::<Vec<_>>();
    assert_eq!(
        analytical
            .iter()
            .filter(|record| matches!(
                record.event(),
                ExplainEvent::CostAssessment {
                    basis: EvidenceBasis::CheckedInvariant,
                    terms,
                    disposition: CostDisposition::Reported,
                    ..
                } if terms.len() == 7
            ))
            .count(),
        2,
        "each plan reports six exact components and its exact unknown count"
    );
    assert_eq!(
        analytical
            .iter()
            .filter(|record| matches!(
                record.event(),
                ExplainEvent::CostAssessment {
                    basis: EvidenceBasis::Assumption,
                    terms,
                    disposition: CostDisposition::Reported,
                    ..
                } if terms.len() == 2
            ))
            .count(),
        2,
        "each plan reports both endpoints of its modelled memory bound"
    );
    assert!(
        analytical
            .iter()
            .all(|record| record.event().disposition() == ExplainDisposition::Reported)
    );
    let rendered = trace.render();
    for typed_term in [
        "cost.memory-traffic.bounded.low:bytes=",
        "cost.indexing.exact:operations=",
        "cost.dispatch.exact:count=",
        "cost.threadgroup-memory.exact:bytes=",
    ] {
        assert!(
            rendered.contains(typed_term),
            "missing typed analytical term {typed_term}"
        );
    }
    for (rule, fact_key, expected) in [
        ("normalize.semantics.v1", "rewrite-count", 0),
        ("region.formation.v1", "candidate-count", 17),
        ("region.formation.v1", "operation-count", 5),
        ("cover.enumeration.v1", "cover-count", 16),
        ("selection.complete-plan.v1", "plan-count", 2),
    ] {
        let records = if rule == "normalize.semantics.v1" {
            product.targets[0].selection_explain.records()
        } else {
            trace.records()
        };
        let record = records
            .iter()
            .find(|record| record.rule().key().as_str() == rule)
            .unwrap_or_else(|| panic!("missing typed count emitter {rule}"));
        let ExplainEvent::Check { assessment, .. } = record.event() else {
            panic!("typed count emitter {rule} must be a checked assertion");
        };
        assert!(assessment.predicate().as_str().contains('.'));
        let actual = assessment
            .facts()
            .iter()
            .find(|fact| fact.key().as_str() == fact_key)
            .map(|fact| fact.value().clone());
        assert_eq!(
            actual,
            Some(FactValue::Count(expected)),
            "{rule}/{fact_key}"
        );
    }
    // Every recognized occurrence resolved a lowering capability and carries
    // exhaustive finite refinement evidence attributed to the same provider.
    for (rule, stage, basis) in [
        (
            "capability.index-access-resolution.v1",
            ExplainStage::CapabilityResolution,
            EvidenceBasis::CheckedInvariant,
        ),
        (
            "kernel.index-region-refinement.v1",
            ExplainStage::KernelRefinement,
            EvidenceBasis::ExhaustiveFinite,
        ),
    ] {
        let records: Vec<_> = trace
            .records()
            .iter()
            .filter(|record| record.rule().key().as_str() == rule)
            .collect();
        assert_eq!(records.len(), 5, "{rule}");
        for record in records {
            assert_eq!(record.event().disposition(), ExplainDisposition::Admitted);
            assert_eq!(record.event().stage(), stage);
            let ExplainEvent::Check { assessment, .. } = record.event() else {
                panic!("{rule} must be a checked assertion");
            };
            assert_eq!(assessment.basis(), &basis);
            // Attribution is the resolved lowering provider, never the
            // compiler: an out-of-crate provider owns this claim.
            assert_ne!(record.rule().provider(), &ProviderRef::builtin());
        }
    }
    // Fusion legality is attributed to the capability provider that declared
    // the member operations' roles, never to the compiler itself.
    let legality = trace
        .records()
        .iter()
        .find(|record| record.rule().key().as_str() == "fusion.legality.v1")
        .expect("a fusion-legality record");
    assert_eq!(legality.event().disposition(), ExplainDisposition::Admitted);
    assert!(trace.render().starts_with("tiler-explain-v10 request="));
}

/// Asserts the honourability half of the end-to-end explain conformance.
///
/// The numerical dimensions left the quantitative predicate space when
/// `strict-f32` was retired, so they are counted through their own typed
/// record. Each names the dimension, the behaviour the resolved contract
/// required, the means the profile declares, and the declaring profile — and
/// the admitted trace asserts the *means*, because a proven verdict alone
/// would not distinguish native support from emulation.
fn assert_honoured_dimensions_are_exhaustive(trace: &crate::explain::VerifiedExplainTrace) {
    let mut honoured = BTreeMap::new();
    for record in trace.records() {
        let ExplainEvent::NumericalHonourability {
            dimension,
            arithmetic,
            required,
            outcome,
            profile,
            resolved_type,
        } = record.event()
        else {
            continue;
        };
        assert_eq!(
            outcome,
            &crate::explain::HonourabilityOutcome::Honoured {
                means: crate::explain::ReasonCode::new("supported-exactly").unwrap(),
            }
        );
        assert_eq!(
            profile.as_str(),
            "tiler.prototype-target-neutral-baseline.v1"
        );
        assert_eq!(*arithmetic, tiler_ir::schedule::ArithmeticType::F32);
        assert_eq!(resolved_type, &tiler_ir::semantic::F32::resolved_type());
        *honoured
            .entry((dimension.as_str(), required.as_str()))
            .or_insert(0_usize) += 1;
    }
    assert_eq!(
        honoured,
        BTreeMap::from([
            (
                ("numerics.approximate-intrinsics", "approximation.forbidden"),
                3,
            ),
            (("numerics.contraction", "forbidden"), 3),
            (("numerics.infinity-assumptions", "make-no-assumption"), 3),
            (("numerics.input-subnormals", "preserve"), 3),
            (("numerics.nan-assumptions", "make-no-assumption"), 3),
            (("numerics.permutation", "forbidden"), 3),
            (("numerics.reassociation", "forbidden"), 3),
            (("numerics.reciprocal-transform", "forbidden"), 3),
            (("numerics.result-subnormals", "preserve"), 3),
            (("numerics.signed-zero", "forbidden"), 3),
        ])
    );
    assert!(trace.render().contains(
            "honourability:numerics.input-subnormals:tiler::f32@1:preserve:honoured:supported-exactly:profile=tiler.prototype-target-neutral-baseline.v1"
        ));
}

#[test]
fn end_to_end_explain_emitter_has_exhaustive_typed_conformance() {
    let semantic = semantic(false);
    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let trace = &product.targets[0].explain;

    let mut target_predicates = BTreeMap::new();
    for record in trace.records() {
        let ExplainEvent::Feasibility {
            predicate,
            outcome: crate::explain::FeasibilityOutcome::Admitted,
            required,
            available,
        } = record.event()
        else {
            continue;
        };
        let unit_is_exact = match predicate.as_str() {
            "grid-axis" => {
                matches!(
                    (required, available),
                    (Quantity::Threads(_), Quantity::Threads(_))
                )
            }
            "buffer-bindings" => matches!(
                (required, available),
                (Quantity::Bindings(_), Quantity::Bindings(_))
            ),
            "local-memory-bytes" => {
                matches!(
                    (required, available),
                    (Quantity::Bytes(_), Quantity::Bytes(_))
                )
            }
            "index-arithmetic-u64" | "device-memory" => {
                matches!(
                    (required, available),
                    (Quantity::Count(_), Quantity::Count(_))
                )
            }
            "device-address-bits" => {
                matches!(
                    (required, available),
                    (Quantity::Bits(_), Quantity::Bits(_))
                )
            }
            other => panic!("unexpected target predicate {other}"),
        };
        assert!(unit_is_exact);
        *target_predicates
            .entry(predicate.as_str())
            .or_insert(0_usize) += 1;
    }
    assert_eq!(
        target_predicates,
        BTreeMap::from([
            ("buffer-bindings", 3),
            ("device-memory", 3),
            ("grid-axis", 3),
            ("index-arithmetic-u64", 3),
            ("local-memory-bytes", 3),
        ])
    );

    let materialized = alternative(&product, ProgramAlternativeKind::Materialized);
    let fused = alternative(&product, ProgramAlternativeKind::Fused);
    let mut deferred_subjects = BTreeMap::new();
    for record in trace.records() {
        let ExplainEvent::DeferredTargetRequirement {
            entry,
            predicate,
            required,
            requirement,
        } = record.event()
        else {
            continue;
        };
        assert_eq!(predicate.as_str(), "threads-per-workgroup");
        assert_eq!(*required, Quantity::Threads(1));
        assert_eq!(requirement.required(), 1);
        assert_eq!(
            requirement.relation(),
            TargetPropertyRequirementRelation::ObservedAtLeastRequired
        );
        let query = requirement.query();
        assert_eq!(
            query.key().as_str(),
            "tiler.target.prepared-entry.max-threads-per-workgroup.v1"
        );
        assert_eq!(
            query.available_at(),
            AvailabilityPhase::PreparedKernelPreflight
        );
        assert_eq!(query.provider().namespace(), "tiler");
        assert_eq!(query.provider().name(), "prepared-entry-properties");
        assert_eq!(query.provider().revision(), 1);
        assert_eq!(record.subjects().len(), 1);
        assert_eq!(
            deferred_subjects.insert(
                (record.subjects()[0].key().as_str().to_owned(), *entry,),
                1_usize,
            ),
            None,
            "each exact alternative/region/entry subject is reported once"
        );
    }
    let expected_deferred_subjects = [materialized, fused]
        .into_iter()
        .flat_map(|alternative| {
            alternative
                .scheduled_regions
                .iter()
                .enumerate()
                .map(move |(entry, scheduled)| {
                    (
                        (
                            format!(
                                "{}/region:{}",
                                alternative.stable_id,
                                scheduled.region().index.id.get()
                            ),
                            u32::try_from(entry).unwrap(),
                        ),
                        1_usize,
                    )
                })
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(deferred_subjects, expected_deferred_subjects);
    assert_eq!(deferred_subjects.len(), 3);

    assert_honoured_dimensions_are_exhaustive(trace);

    let selections = trace
        .records()
        .iter()
        .filter_map(|record| match record.event() {
            ExplainEvent::Selection { outcome, .. } => {
                Some((record.subjects()[0].key().as_str().to_owned(), *outcome))
            }
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        selections.get(&materialized.stable_id),
        Some(&SelectionOutcome::Dominated)
    );
    assert_eq!(
        selections.get(&fused.stable_id),
        Some(&SelectionOutcome::Selected)
    );
}
