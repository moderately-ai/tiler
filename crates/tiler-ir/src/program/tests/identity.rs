//! Kernel-program identity: encoding, determinism, and what must change it.
//!
//! The domain separator, the published-output interface order, and the coverage/
//! refinement inputs `canonical_identity` folds — semantic graph, bound refinement,
//! coverage partition — each proven present and each proven not aliased by the others.

use super::super::{
    AllocationOwnership, CoveredOccurrence, KernelProgramBuildError, KernelProgramBuilder,
    KernelProgramDiagnostic, MaterializedOrigin, SemanticOccurrence, ValueRole,
};
use super::support::{
    OTHER_SCALE_BITS, SCALE_BITS, TwoStageShape, canonical_program, checked_coverage,
    complete_two_stage, coverage_range, declare_program_contract, device, diagnostic, fixture_abi,
    flush_contract, input_shape, occurrences, output_shape, pointwise_kernel, pointwise_region,
    program_input, publish_two_chain_keyed, read, reduction_kernel, serial_sum_program, two_chain,
    two_chain_program, two_chain_program_keyed, two_stage, value, wire_two_stage,
    wire_two_stage_storage, write_access,
};
use crate::semantic::{InputKey, OutputKey};

/// The separator is what distinguishes the reinterpreting steps, and only it.
///
/// `v7` and `v8` reinterpret retained bytes rather than adding any: `v7` reads
/// the same raw four-byte coverage ordinal as a canonical semantic occurrence,
/// and `v8` reads the same output records in interface order rather than sorted
/// by content. This fixture publishes exactly one output — asserted, because
/// that is the argument — so sorting its record list is the identity
/// permutation and its payload is byte-identical under all of those tags. A
/// `v6`, `v7`, or `v8` reader handed these bytes would recover records under a
/// meaning this layer no longer holds, which is why each tag stepped.
///
/// `v9`, `v10`, `v11`, and `v13` are a different kind of step and are included
/// here for the same reason: `v9` *adds* framed refinement evidence inside the
/// stage section, `v10` *adds* a publishing-copy declaration section, `v11`
/// *adds* a staged-realization declaration section, and `v13` *adds* the framed
/// shape-environment subject beside the graph, so the historical spellings below
/// are not merely reinterpretations of the current payload — they are shorter
/// encodings this test cannot reconstruct. What the loop still proves is the
/// property that matters at every step: no historical separator over these bytes
/// is the current identity.
///
/// The separators are not all the same length, so the spliced spelling is
/// compared by inequality alone where the lengths differ. Padding it back to a
/// common length would compare a byte string no encoder produces.
#[test]
fn the_program_domain_separator_is_what_distinguishes_the_reinterpreting_steps() {
    const V6: &[u8] = b"tiler.kernel-program.v6\0";
    const V7: &[u8] = b"tiler.kernel-program.v7\0";
    const V8: &[u8] = b"tiler.kernel-program.v8\0";
    const V9: &[u8] = b"tiler.kernel-program.v9\0";
    const V10: &[u8] = b"tiler.kernel-program.v10\0";
    const V11: &[u8] = b"tiler.kernel-program.v11\0";
    const V12: &[u8] = b"tiler.kernel-program.v12\0";
    const V13: &[u8] = b"tiler.kernel-program.v13\0";
    let semantic = serial_sum_program(SCALE_BITS);
    let program = canonical_program(&semantic);
    // One record: the v8 encoding and the v7 sort agree on this payload.
    assert_eq!(program.outputs().len(), 1);
    let current = program.canonical_identity().as_bytes();
    assert!(current.starts_with(V13));

    for historical in [V6, V7, V8, V9, V10, V11, V12] {
        let mut spelling = historical.to_vec();
        spelling.extend_from_slice(&current[V13.len()..]);
        assert_ne!(current, spelling.as_slice());
    }
    // The check can say no about the separator rather than about the length: the
    // current separator over the current payload *is* the current identity.
    let mut rebuilt = V13.to_vec();
    rebuilt.extend_from_slice(&current[V13.len()..]);
    assert_eq!(current, rebuilt.as_slice());
}

/// Byte offsets at which `needle` occurs in `haystack`.
fn byte_offsets_of(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    haystack
        .windows(needle.len())
        .enumerate()
        .filter(|(_, window)| *window == needle)
        .map(|(offset, _)| offset)
        .collect()
}

/// Identity folds the published outputs in the interface's order.
///
/// The fixture's interface order is the reverse of the order the `v7` sort
/// produced — `z_sum` is declared first and sorts second — so the two rules
/// disagree on this program and agree on every one-output program. Every place
/// a key appears must now agree on the order; under the sorted rule the output
/// section was transposed against the rest.
///
/// **The population is two, and it says which change moved it there.** A key
/// appears once inside the folded semantic graph identity, which has encoded
/// outputs in declaration order all along, and once in the program's own output
/// section. It used to appear a third time per coverage record, because every
/// record restated the whole bound graph identity; ADR 0104 replaced that
/// restatement with a fixed-width digest, and a digest of a key is not the key.
/// The coverage population is asserted non-empty beside the count for the reason
/// the count used to be derived from it — a program that covered nothing would
/// otherwise satisfy a literal `2` while proving nothing at all.
#[test]
fn published_output_interface_order_reaches_program_identity() {
    let semantic = two_chain_program_keyed(["z_sum", "a_sum"]);
    let program = publish_two_chain_keyed(two_chain(&semantic, true), ["z_sum", "a_sum"], false)
        .build()
        .expect("interface-ordered publication verifies");
    assert_eq!(program.outputs().len(), 2);
    assert_eq!(
        program
            .outputs()
            .map(|output| output.key().as_str().to_owned())
            .collect::<Vec<_>>(),
        vec!["z_sum".to_owned(), "a_sum".to_owned()],
    );

    let identity = program.canonical_identity().as_bytes();
    let declared_first = byte_offsets_of(identity, b"z_sum");
    let declared_second = byte_offsets_of(identity, b"a_sum");
    let coverage_records: usize = program.stages().map(|stage| stage.coverage().len()).sum();
    assert!(
        coverage_records > 0,
        "a program covering no occurrence proves nothing about what coverage restates",
    );
    let expected = 2;
    assert_eq!(
        declared_first.len(),
        expected,
        "the semantic fold and the output section, and no per-record graph restatement"
    );
    assert_eq!(declared_second.len(), expected);
    for (first, second) in declared_first.iter().zip(&declared_second) {
        assert!(
            first < second,
            "the output section holds the sorted order, not the interface order",
        );
    }

    // The check can say no about the order rather than about rebuilding:
    // re-declaring the same interface reproduces the bytes exactly.
    let rebuilt = publish_two_chain_keyed(two_chain(&semantic, true), ["z_sum", "a_sum"], false)
        .build()
        .expect("verified kernel program");
    assert_eq!(identity, rebuilt.canonical_identity().as_bytes());
}

/// Publishing the interface in any other order fails closed.
///
/// This is the neighbour that makes the identity claim above meaningful: the
/// permuted program is not a second identity to distinguish, it is not a
/// program. Rejecting it is what makes
/// [`VerifiedKernelProgram::outputs`]'s ordering claim true rather than a
/// convention every consumer would have to trust its producer to have kept.
#[test]
fn publishing_the_outputs_out_of_interface_order_is_rejected() {
    let semantic = two_chain_program_keyed(["z_sum", "a_sum"]);
    assert_eq!(
        diagnostic(publish_two_chain_keyed(
            two_chain(&semantic, true),
            ["z_sum", "a_sum"],
            true,
        )),
        KernelProgramDiagnostic::MisorderedNamedOutput { position: 0 },
    );
    // The lexicographic fixture reaches the same refusal, so the rule is the
    // interface order and not an incidental agreement with the sorted one.
    let sorted_interface = two_chain_program();
    assert_eq!(
        diagnostic(publish_two_chain_keyed(
            two_chain(&sorted_interface, true),
            ["sum_a", "sum_b"],
            true,
        )),
        KernelProgramDiagnostic::MisorderedNamedOutput { position: 0 },
    );
    assert_eq!(
        KernelProgramDiagnostic::MisorderedNamedOutput { position: 0 }.rule(),
        "misordered-named-output",
    );
}

#[test]
fn a_verified_program_binds_its_refinements_coverage_and_named_outputs() {
    let semantic = serial_sum_program(SCALE_BITS);
    let program = canonical_program(&semantic);

    assert_eq!(program.stages().len(), 2);
    assert_eq!(program.values().len(), 3);
    assert_eq!(program.allocations().len(), 3);
    assert_eq!(program.views().len(), 3);
    assert_eq!(program.dependencies().len(), 1);
    assert_eq!(program.outputs().len(), 1);
    assert_eq!(
        program.semantic_graph_identity(),
        semantic.semantic_identity().graph()
    );

    // The stage DAG is ordered by its typed dependency, not by insertion.
    let order: Vec<_> = program
        .execution_order()
        .map(|stage| stage.coverage().to_vec())
        .collect();
    assert_eq!(
        order,
        vec![occurrences(&semantic, 0..4), occurrences(&semantic, 4..5)]
    );

    // Each stage retains the exact structured kernel it dispatches, which in
    // turn retains the exact scheduled region that kernel refines.
    let pointwise = program.stages().next().expect("pointwise stage");
    assert_eq!(
        pointwise.kernel().canonical_identity(),
        pointwise_kernel(0, SCALE_BITS).canonical_identity()
    );
    assert_eq!(
        pointwise.kernel().scheduled_region_identity(),
        pointwise_region(0, SCALE_BITS).canonical_identity()
    );
    assert_eq!(pointwise.accesses().len(), 2);

    // The temporary is defined by the pointwise stage and lives in its own
    // program-owned allocation.
    let temporary = program
        .values()
        .find(|value| value.role() == ValueRole::Temporary)
        .expect("one temporary");
    assert_eq!(temporary.required_bytes(), 24);
    assert_eq!(temporary.shape(), &input_shape());
    assert_eq!(temporary.definition(), Some(pointwise));
    assert_eq!(
        temporary.allocation().ownership(),
        AllocationOwnership::Program
    );
    assert_eq!(temporary.allocation().values().count(), 1);

    // The input is externally bound and has no defining stage.
    let source = program
        .values()
        .find(|value| value.role() == ValueRole::Input)
        .expect("one input");
    assert_eq!(source.definition(), None);
    assert_eq!(
        source.origin(),
        &MaterializedOrigin::ProgramInput {
            key: InputKey::new("input").expect("key"),
        }
    );

    let output = program.outputs().next().expect("one output");
    assert_eq!(output.key().as_str(), "result");
    assert_eq!(output.value().role(), ValueRole::Output);
    assert_eq!(output.value().required_bytes(), 8);
}

#[test]
fn identity_is_deterministic_and_independent_of_declaration_order() {
    let semantic = serial_sum_program(SCALE_BITS);
    let first = canonical_program(&semantic);
    let second = canonical_program(&semantic);
    assert_eq!(
        first.canonical_identity().as_bytes(),
        second.canonical_identity().as_bytes()
    );

    let reordered = complete_two_stage(two_stage(&semantic, TwoStageShape::ReversedDeclaration))
        .build()
        .expect("verified kernel program");
    assert_eq!(
        first.canonical_identity().as_bytes(),
        reordered.canonical_identity().as_bytes()
    );
    assert_eq!(first, reordered);
}

#[test]
fn identity_excludes_the_transient_planning_region_ordinal() {
    let semantic = serial_sum_program(SCALE_BITS);
    // The same schedules planned under different `RegionId` ordinals.
    let renumbered_pointwise = pointwise_kernel(41, SCALE_BITS);
    let renumbered_reduction = reduction_kernel(97);
    assert_ne!(
        renumbered_pointwise.scheduled_region(),
        pointwise_kernel(0, SCALE_BITS).scheduled_region()
    );
    assert_eq!(
        renumbered_pointwise.canonical_identity(),
        pointwise_kernel(0, SCALE_BITS).canonical_identity()
    );

    let renumbered = complete_two_stage(wire_two_stage(
        &semantic,
        &renumbered_pointwise,
        &renumbered_reduction,
        TwoStageShape::Canonical,
    ))
    .build()
    .expect("verified kernel program");
    assert_eq!(
        canonical_program(&semantic).canonical_identity().as_bytes(),
        renumbered.canonical_identity().as_bytes()
    );
}

#[test]
fn identity_changes_when_the_semantic_graph_layer_changes() {
    // Identical bound implementations, coverage, and structure over two graphs
    // that differ only in one constant: only the ADR 0072 semantic-graph layer
    // moves, and program identity must move with it.
    let first = serial_sum_program(SCALE_BITS);
    let second = serial_sum_program(OTHER_SCALE_BITS);
    assert_ne!(
        first.semantic_identity().graph(),
        second.semantic_identity().graph()
    );

    let over_first = canonical_program(&first);
    let over_second = canonical_program(&second);
    assert_ne!(
        over_first.canonical_identity().as_bytes(),
        over_second.canonical_identity().as_bytes()
    );
    assert_ne!(over_first, over_second);
}

#[test]
fn identity_changes_when_a_bound_refinement_changes() {
    // One semantic graph, one coverage split, one structure: only the selected
    // pointwise refinement differs.
    let semantic = serial_sum_program(SCALE_BITS);
    let selected = pointwise_kernel(0, SCALE_BITS);
    let alternative = pointwise_kernel(0, OTHER_SCALE_BITS);
    assert_ne!(
        selected.canonical_identity(),
        alternative.canonical_identity()
    );

    let first = complete_two_stage(wire_two_stage(
        &semantic,
        &selected,
        &reduction_kernel(1),
        TwoStageShape::Canonical,
    ))
    .build()
    .expect("verified kernel program");
    let second = complete_two_stage(wire_two_stage(
        &semantic,
        &alternative,
        &reduction_kernel(1),
        TwoStageShape::Canonical,
    ))
    .build()
    .expect("verified kernel program");
    assert_ne!(
        first.canonical_identity().as_bytes(),
        second.canonical_identity().as_bytes()
    );
}

/// Evidence is identity, not decoration.
///
/// The two programs agree on the semantic graph, the bound kernels, the
/// coverage partition, and every covered occurrence — asserted, because those
/// agreements are what make the remaining difference the refinement evidence
/// and nothing else. The receipts were minted under two governed numerical
/// contracts, which is a real difference in what was proved and not a fixture
/// trick: a contract is folded into executable coverage and is deliberately
/// absent from semantic graph meaning.
#[test]
fn identity_changes_when_only_the_refinement_evidence_changes() {
    let semantic = serial_sum_program(SCALE_BITS);
    let strict = canonical_program(&semantic);
    let alternative = complete_two_stage(two_stage(
        &semantic,
        TwoStageShape::AlternateRefinementEvidence,
    ))
    .build()
    .expect("verified kernel program over alternate refinement evidence");

    assert_eq!(
        strict.semantic_graph_identity(),
        alternative.semantic_graph_identity()
    );
    let paired = || strict.stages().zip(alternative.stages());
    assert!(paired().all(|(left, right)| left.kernel() == right.kernel()));
    assert!(paired().all(|(left, right)| {
        left.coverage()
            .iter()
            .map(CoveredOccurrence::occurrence)
            .eq(right.coverage().iter().map(CoveredOccurrence::occurrence))
    }));
    assert!(paired().any(|(left, right)| {
        left.coverage()
            .iter()
            .zip(right.coverage())
            .any(|(left, right)| left.refinement() != right.refinement())
    }));

    assert_ne!(
        strict.canonical_identity().as_bytes(),
        alternative.canonical_identity().as_bytes(),
    );
}

/// A receipt from another graph is refused before it can stand in for one here.
///
/// The foreign graph has the same five operations at the same canonical
/// ordinals, so nothing about the occurrence itself would catch the
/// substitution — only the retained graph does.
#[test]
fn coverage_proved_against_another_graph_is_rejected_at_insertion() {
    let semantic = serial_sum_program(SCALE_BITS);
    let foreign = serial_sum_program(OTHER_SCALE_BITS);
    assert_ne!(
        semantic.semantic_identity().graph(),
        foreign.semantic_identity().graph()
    );
    let mut builder = KernelProgramBuilder::new(&semantic).expect("builder");
    let abi = fixture_abi(&mut builder);
    let storage = wire_two_stage_storage(&mut builder, TwoStageShape::Canonical);
    assert_eq!(
        builder
            .push_stage(
                &pointwise_kernel(0, SCALE_BITS),
                &occurrences(&foreign, 0..4),
                &[
                    read(storage.source_view, abi.input_bytes),
                    write_access(storage.temporary_view, abi.input_bytes),
                ],
                abi.pointwise_launch(),
            )
            .expect_err("a receipt minted against another graph is not evidence here"),
        KernelProgramBuildError::ForeignCoverageGraph {
            occurrence: SemanticOccurrence::new(0),
        }
    );
}

#[test]
fn identity_changes_when_complete_coverage_is_partitioned_differently() {
    // One semantic graph and one pair of bound implementations; two different
    // complete and disjoint coverage partitions.
    let semantic = serial_sum_program(SCALE_BITS);
    let canonical = canonical_program(&semantic);
    let shifted = complete_two_stage(two_stage(&semantic, TwoStageShape::ShiftedCoverage))
        .build()
        .expect("verified kernel program");
    assert_ne!(
        canonical.canonical_identity().as_bytes(),
        shifted.canonical_identity().as_bytes()
    );
}

#[test]
fn incomplete_coverage_of_the_bound_graph_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut builder = KernelProgramBuilder::new(&semantic).expect("builder");
    let abi = fixture_abi(&mut builder);
    let external = builder
        .push_allocation(device(24, AllocationOwnership::External))
        .expect("external allocation");
    let owned = builder
        .push_allocation(device(24, AllocationOwnership::Program))
        .expect("temporary allocation");
    let output_allocation = builder
        .push_allocation(device(8, AllocationOwnership::Program))
        .expect("output allocation");
    let source = builder
        .push_value(
            value(program_input("input"), ValueRole::Input, input_shape()),
            external,
        )
        .expect("input value");
    let temporary = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Temporary,
                input_shape(),
            ),
            owned,
        )
        .expect("temporary value");
    let output = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                output_shape(),
            ),
            output_allocation,
        )
        .expect("output value");
    let source_view = builder.push_whole_view(source).expect("input view");
    let temporary_view = builder.push_whole_view(temporary).expect("temporary view");
    let output_view = builder.push_whole_view(output).expect("output view");
    let pointwise = builder
        .push_stage(
            &pointwise_kernel(0, SCALE_BITS),
            // One graph operation is left uncovered.
            &occurrences(&semantic, 0..3),
            &[
                read(source_view, abi.input_bytes),
                write_access(temporary_view, abi.input_bytes),
            ],
            abi.pointwise_launch(),
        )
        .expect("pointwise stage");
    let reduction = builder
        .push_stage(
            &reduction_kernel(1),
            &occurrences(&semantic, 3..4),
            &[
                read(temporary_view, abi.input_bytes),
                write_access(output_view, abi.output_bytes),
            ],
            abi.reduction_launch(),
        )
        .expect("reduction stage");
    builder
        .push_data_dependency(pointwise, reduction, temporary)
        .expect("data dependency");
    builder
        .push_output(OutputKey::new("result").expect("key"), output)
        .expect("named output");
    declare_program_contract(&mut builder);

    assert_eq!(
        diagnostic(builder),
        KernelProgramDiagnostic::IncompleteCoverage {
            covered: 4,
            required: 5,
        }
    );
}

#[test]
fn covering_one_occurrence_twice_is_rejected_at_insertion() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    // Coverage the two wired stages already claim, but proved under another
    // numerical contract. The refusal is therefore about the occurrence being
    // claimed twice and not about a record repeating byte for byte — the case
    // that matters, because two *different* proofs of one occurrence are the
    // ambiguity this binding exists to make impossible.
    let conflicting = coverage_range(&checked_coverage(&semantic, &flush_contract()), 3..5);
    assert_eq!(
        wired
            .builder
            .push_stage(
                &pointwise_kernel(2, OTHER_SCALE_BITS),
                &conflicting,
                &[
                    read(wired.source_view, wired.abi.input_bytes),
                    write_access(wired.temporary_view, wired.abi.input_bytes),
                ],
                wired.abi.pointwise_launch(),
            )
            .expect_err("repeated coverage is rejected"),
        KernelProgramBuildError::DuplicateCoverage {
            occurrence: SemanticOccurrence::new(3),
        }
    );
}
