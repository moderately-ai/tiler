//! Which physical authority produced each selected region, and what moves with it.
//!
//! Two families, and they are the two halves of one claim. The **invariance**
//! cases prove that a provider the plan never reached cannot move a published
//! byte, which is the ADR 0072 line; the **movement** cases prove that each of
//! the four subjects a selected row carries does move one, which is the
//! omission this run exists to close. Neither half is evidence without the
//! other: a run folded into nothing would pass every invariance case, and a run
//! that folded the whole offered environment would pass every movement case.
//!
//! Each movement case perturbs exactly one subject and leaves the other three
//! alone, so a failure names the subject that stopped being identity-bearing
//! rather than reporting that provenance in general does.

use super::super::model::PHYSICAL_SELECTION_KEY_DOMAIN;
use super::super::{
    ArtifactBuildError, ArtifactKeyKind, ArtifactProgramBuilder, CompilationEnvironment,
    MAX_ARTIFACT_IDENTITY_BYTES, MAX_PHYSICAL_SELECTION_IDENTITY_BYTES,
    MAX_SELECTED_PHYSICAL_IMPLEMENTATIONS, MAX_VARIANT_ENTRIES, PhysicalProposalKind,
    PhysicalRegionOccurrenceIdentity, SelectedPhysicalImplementation, VerifiedArtifactProgram,
};
use super::{
    SCALE_BITS, declare_realization, formulas, fused_program, lowering_provider, occurrence,
    offered_physical, partial_window_program, payload, physical_provider, physical_run,
    physical_selection, proposal, selection, semantic_program, variant,
};

/// The canonical identity bytes one packaged artifact carries.
///
/// The preimage rather than the published envelope, because the encoder lives
/// behind the codec's private module and its byte- and digest-level half of
/// these claims is proven there, in
/// `codec::tests::selected_physical_implementations`. Identity is the stronger
/// half to assert here anyway: envelope bytes are a canonical function of it,
/// so two artifacts with equal identity encode equally by construction.
fn identity_of(artifact: &VerifiedArtifactProgram) -> Vec<u8> {
    artifact.canonical_identity().as_bytes().to_vec()
}

/// Builds the canonical fixture with the two offered roles stated exactly.
///
/// Both roles are parameters so a case can widen one without widening the
/// other, which is the whole of what the independent-invariance cases need and
/// the reason no fixture here takes a single "environment".
fn artifact_with(
    offered_lowering: &[tiler_ir::semantic::ProviderIdentity],
    offered_physical_role: &[tiler_ir::semantic::ProviderIdentity],
    rows: Vec<SelectedPhysicalImplementation>,
) -> VerifiedArtifactProgram {
    build_with(offered_lowering, offered_physical_role, rows).expect("the fixture artifact builds")
}

fn build_with(
    offered_lowering: &[tiler_ir::semantic::ProviderIdentity],
    offered_physical_role: &[tiler_ir::semantic::ProviderIdentity],
    rows: Vec<SelectedPhysicalImplementation>,
) -> Result<VerifiedArtifactProgram, ArtifactBuildError> {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let environment = CompilationEnvironment::new(
        offered_lowering.iter().cloned(),
        offered_physical_role.iter().cloned(),
    )?;
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment)?;
    draft.select_lowering_provider(selection(lowering_provider(1)))?;
    let descriptor = draft.push_payload(payload(0xa1))?;
    let formulas = formulas(&mut draft);
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.selected_physical_implementations = rows;
    draft.push_variant(&program, spec)?;
    declare_realization(&mut draft, &program);
    Ok(draft.build().expect("the fixture artifact verifies"))
}

/// The refusal one candidate run draws, with the builder proven untouched.
///
/// Every insertion rule below goes through this, because "it refused" is only
/// half the transactional claim: the other half is that the draft a caller
/// still holds builds the same artifact it would have built had the refused
/// call never happened. Testing the refusal alone would pass on a builder that
/// committed the run and then errored.
fn refuse(rows: Vec<SelectedPhysicalImplementation>) -> ArtifactBuildError {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let environment = CompilationEnvironment::new([lowering_provider(1)], offered_physical())
        .expect("both roles compose");
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).expect("a draft");
    draft
        .select_lowering_provider(selection(lowering_provider(1)))
        .expect("the lowering row is offered");
    let descriptor = draft.push_payload(payload(0xa1)).expect("a payload");
    let formulas = formulas(&mut draft);

    let mut refused = variant(&formulas, descriptor, b"fused");
    refused.selected_physical_implementations = rows;
    let error = draft
        .push_variant(&program, refused)
        .expect_err("the candidate run is refused");

    // The draft is intact: the same variant a clean builder would accept is
    // still acceptable here, and the artifact it produces is byte-identical to
    // the one that never saw the refusal.
    draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .expect("a refused candidate leaves the builder able to accept a good one");
    declare_realization(&mut draft, &program);
    let after = draft.build().expect("the recovered draft verifies");
    let clean = artifact_with(
        &[lowering_provider(1)],
        &offered_physical(),
        physical_run(1),
    );
    assert_eq!(
        identity_of(&after),
        identity_of(&clean),
        "a refused physical run must leave no trace in what the builder publishes",
    );
    error
}

// -------------------------------------------------------------------------
// Invariance: an offered provider the plan never reached
// -------------------------------------------------------------------------

/// Widening the offered **lowering** set alone moves no published byte.
///
/// The physical set is held fixed, so a failure here cannot be explained by the
/// physical role having become identity-bearing. This is the half of ADR 0072
/// the lowering row already had; it is re-proven beside its sibling because the
/// two sets are now separate authorities and a union would break exactly one.
#[test]
fn an_unused_lowering_provider_moves_no_published_byte() {
    let narrow = artifact_with(
        &[lowering_provider(1)],
        &offered_physical(),
        physical_run(1),
    );
    let wide = artifact_with(
        &[lowering_provider(1), super::spare_provider(7)],
        &offered_physical(),
        physical_run(1),
    );
    assert_eq!(
        identity_of(&narrow),
        identity_of(&wide),
        "a lowering provider the plan never reached must not enter artifact identity",
    );
}

/// Widening the offered **physical** set alone moves no published byte.
///
/// The independent half, and the one this ticket adds. The lowering set is held
/// fixed, so this cannot pass because the environment is discarded wholesale —
/// the movement cases below prove the selected rows are folded, so an offered
/// set that also folded would fail here.
#[test]
fn an_unused_physical_provider_moves_no_published_byte() {
    let narrow = artifact_with(
        &[lowering_provider(1)],
        &offered_physical(),
        physical_run(1),
    );
    let wide = artifact_with(
        &[lowering_provider(1)],
        &[physical_provider(1), physical_provider(9)],
        physical_run(1),
    );
    assert_eq!(
        identity_of(&narrow),
        identity_of(&wide),
        "a physical provider the plan never reached must not enter artifact identity",
    );
}

// -------------------------------------------------------------------------
// Movement: each of the four subjects, independently
// -------------------------------------------------------------------------

/// Each subject of a selected row is identity-bearing on its own.
///
/// One perturbation per case, with the other three held: a single combined case
/// would go green while three of the four had silently stopped entering the
/// encoding. The offered sets are widened to admit each perturbed provider, so
/// what is under test is the *selection* rather than the environment — and the
/// invariance cases above are what prove that widening did not itself move the
/// bytes.
#[test]
fn every_selected_physical_subject_moves_identity_bytes_and_digest() {
    let offered = [physical_provider(1), physical_provider(2)];
    let baseline = artifact_with(&[lowering_provider(1)], &offered, physical_run(1));
    let published = identity_of(&baseline);

    let mut other_provider = physical_selection(0);
    other_provider.provider = physical_provider(2);

    let mut other_proposal = physical_selection(0);
    other_proposal.implementation_proposal = proposal(41);

    let mut other_occurrence = physical_selection(0);
    other_occurrence.region_occurrence = occurrence(41);

    let mut other_kind = physical_selection(0);
    other_kind.proposal_kind = PhysicalProposalKind::OpaqueCall;

    for (subject, row) in [
        ("selected provider identity", other_provider),
        ("implementation-proposal identity", other_proposal),
        ("occurrence association", other_occurrence),
        ("proposal kind", other_kind),
    ] {
        let perturbed = artifact_with(&[lowering_provider(1)], &offered, vec![row]);
        assert_ne!(
            published,
            identity_of(&perturbed),
            "{subject} must move canonical artifact identity",
        );
    }
}

/// Multiplicity across distinct occurrences survives build and both read views.
///
/// The property an artifact-global provider set would lose: two occurrences
/// implemented by one provider through two proposals are two rows, and reading
/// them back must still find two.
#[test]
fn multiplicity_across_distinct_occurrences_is_preserved() {
    let semantic = semantic_program();
    let program = partial_window_program(&semantic);
    let environment = CompilationEnvironment::new([lowering_provider(1)], offered_physical())
        .expect("both roles compose");
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).expect("a draft");
    draft
        .select_lowering_provider(selection(lowering_provider(1)))
        .expect("the lowering row is offered");
    let descriptor = draft.push_payload(payload(0xa1)).expect("a payload");
    let mut spec = super::super::tests::support::artifacts::partial_window_variant(descriptor);
    // One provider, two occurrences, two proposals: legal repetition that a
    // provider-keyed collapse would reduce to one row.
    let mut second = physical_selection(1);
    second.provider = physical_provider(1);
    spec.selected_physical_implementations = vec![physical_selection(0), second];
    draft
        .push_variant(&program, spec)
        .expect("two rows over two entries are admitted");
    declare_realization(&mut draft, &program);
    let artifact = draft.build().expect("the artifact verifies");

    let variant = artifact.variants().next().expect("one packaged variant");
    let rows = variant.selected_physical_implementations();
    assert_eq!(rows.len(), 2, "both occurrences are retained");
    assert_eq!(rows[0].region_occurrence, occurrence(0));
    assert_eq!(rows[1].region_occurrence, occurrence(1));
    assert_eq!(
        rows[0].provider, rows[1].provider,
        "one provider across two occurrences is legal repetition, not a duplicate",
    );
}

// -------------------------------------------------------------------------
// Insertion rules, each transactional
// -------------------------------------------------------------------------

/// An empty run is refused rather than read as "this variant selected nothing".
#[test]
fn an_empty_physical_run_is_refused() {
    assert!(matches!(
        refuse(Vec::new()),
        ArtifactBuildError::EmptySelectedPhysicalImplementations
    ));
}

/// One occurrence may carry exactly one selected implementation.
///
/// Stated over the **two-entry** fixture deliberately. The specified refusal
/// precedence puts the `rows <= entries` rule ahead of the order rules, so a
/// two-row run over a one-entry variant is a cardinality refusal and would
/// never reach the check under test — a case this suite got wrong once and the
/// precedence caught.
#[test]
fn a_repeated_occurrence_is_refused() {
    let mut second = physical_selection(0);
    second.implementation_proposal = proposal(99);
    let error = refuse_two_entry(vec![physical_selection(0), second]);
    assert!(
        matches!(
            error,
            ArtifactBuildError::DuplicatePhysicalRegionOccurrence { .. }
        ),
        "one occurrence carries exactly one implementation, got {error:?}",
    );
}

/// A descending run is refused rather than sorted into canonical order.
///
/// Sorting would silently repair a statement that contradicts the compiler that
/// minted it, which is the one repair this boundary must not make.
#[test]
fn a_descending_occurrence_run_is_refused_rather_than_sorted() {
    let error = refuse_two_entry(vec![physical_selection(1), physical_selection(0)]);
    assert!(
        matches!(
            error,
            ArtifactBuildError::NoncanonicalPhysicalRegionOccurrenceOrder { .. }
        ),
        "a descending run is refused, not reordered, got {error:?}",
    );
}

/// The refusal one candidate run draws on the **two-entry** fixture.
///
/// Separate from [`refuse`] because the order rules sit behind the cardinality
/// rule in the specified precedence, so proving them needs a variant with room
/// for two rows.
fn refuse_two_entry(rows: Vec<SelectedPhysicalImplementation>) -> ArtifactBuildError {
    let semantic = semantic_program();
    let program = partial_window_program(&semantic);
    let environment = CompilationEnvironment::new([lowering_provider(1)], offered_physical())
        .expect("both roles compose");
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).expect("a draft");
    draft
        .select_lowering_provider(selection(lowering_provider(1)))
        .expect("the lowering row is offered");
    let descriptor = draft.push_payload(payload(0xa1)).expect("a payload");
    let mut spec = super::super::tests::support::artifacts::partial_window_variant(descriptor);
    spec.selected_physical_implementations = rows;
    draft
        .push_variant(&program, spec)
        .expect_err("the candidate run is refused")
}

/// More selected rows than executable entries cannot be the accepted population.
///
/// Refused as the **relational** rule and deliberately not as a structural
/// limit: the absolute ceiling is the entry table's own 4,096, so a fixed
/// producer-policy count here would refuse artifacts the entry table admits.
#[test]
fn more_rows_than_entries_is_a_cardinality_refusal_not_a_structural_limit() {
    let error = refuse(vec![physical_selection(0), physical_selection(1)]);
    assert!(
        matches!(
            error,
            ArtifactBuildError::PhysicalSelectionCardinality {
                selected: 2,
                entries: 1
            }
        ),
        "two rows over a one-entry variant is a cardinality refusal, got {error:?}",
    );
    assert!(
        !matches!(error, ArtifactBuildError::StructuralLimit { .. }),
        "the relational rule must not be reported as an exhausted absolute budget",
    );
}

/// The absolute ceiling follows the entry table rather than a compiler policy.
///
/// What it would take for this to say *no*: give
/// `MAX_SELECTED_PHYSICAL_IMPLEMENTATIONS` a literal of its own — twelve, say,
/// from the current governed request budget — instead of deriving it. That is
/// the exact substitution the packet's candidate A proposed and this pins
/// against, because a fixed twelve would refuse a structurally consistent
/// thirteen-entry artifact the entry table already admits.
#[test]
fn the_selected_row_ceiling_is_the_entry_ceiling() {
    assert_eq!(
        MAX_SELECTED_PHYSICAL_IMPLEMENTATIONS, MAX_VARIANT_ENTRIES,
        "the selected run is bounded by the executable-entry table it is associated with",
    );
    const { assert!(MAX_SELECTED_PHYSICAL_IMPLEMENTATIONS > 12) }
}

/// Cross-role authority: the two offered sets are never consulted as a union.
///
/// Both directions, because a union would admit both and only one of them would
/// be noticed by a test that checked one direction.
#[test]
fn cross_role_offered_authority_is_refused_in_both_directions() {
    // The physical implementer offered *only* in the lowering role.
    let error = build_with(
        &[lowering_provider(1), physical_provider(1)],
        &[],
        physical_run(1),
    )
    .expect_err("a physical selection needs physical authority");
    assert!(
        matches!(error, ArtifactBuildError::PhysicalProviderNotOffered { .. }),
        "a provider offered only to lower must not be able to implement, got {error:?}",
    );

    // The lowering provider offered *only* in the physical role.
    let error = build_with(
        &[],
        &[physical_provider(1), lowering_provider(1)],
        physical_run(1),
    )
    .expect_err("a lowering selection needs lowering authority");
    assert!(
        matches!(error, ArtifactBuildError::LoweringProviderNotOffered { .. }),
        "a provider offered only to implement must not be able to lower, got {error:?}",
    );
}

/// A wholly absent physical role refuses every selected row.
#[test]
fn an_absent_physical_role_refuses_the_run() {
    let error = build_with(&[lowering_provider(1)], &[], physical_run(1))
        .expect_err("no physical authority was granted at all");
    assert!(matches!(
        error,
        ArtifactBuildError::PhysicalProviderNotOffered { .. }
    ));
}

/// Both identity wrappers reject empty and oversize bytes by their own subject.
///
/// The oversize arm allocates its 64 MiB candidate deliberately: the bound is
/// defined as the whole-identity limit, so a cheaper synthetic ceiling would be
/// testing a different constant from the one the boundary enforces.
#[test]
fn a_physical_identity_is_bounded_by_the_whole_identity_limit() {
    assert_eq!(
        MAX_PHYSICAL_SELECTION_IDENTITY_BYTES, MAX_ARTIFACT_IDENTITY_BYTES,
        "a received identity is bounded by the identity it is a subset of",
    );
    assert!(matches!(
        PhysicalRegionOccurrenceIdentity::from_bytes([]),
        Err(ArtifactBuildError::EmptyKey {
            kind: ArtifactKeyKind::PhysicalRegionOccurrence
        })
    ));
    assert!(matches!(
        super::super::PhysicalImplementationProposalIdentity::from_bytes([]),
        Err(ArtifactBuildError::EmptyKey {
            kind: ArtifactKeyKind::PhysicalImplementationProposal
        })
    ));
    let oversize = vec![0x5a; MAX_PHYSICAL_SELECTION_IDENTITY_BYTES + 1];
    assert!(matches!(
        PhysicalRegionOccurrenceIdentity::from_bytes(&oversize),
        Err(ArtifactBuildError::KeyTooLong {
            kind: ArtifactKeyKind::PhysicalRegionOccurrence,
            ..
        })
    ));
}

/// A run whose own bytes already prove the identity oversize is refused at insertion.
///
/// The boundary is exercised from both sides in one case, because the claim is
/// that the check sits exactly at the existing whole-identity limit rather than
/// somewhere near it. The passing side is the evidence that this adds no second
/// budget: a candidate whose proved minimum is *exactly* the limit is admitted
/// here and left to the reachable whole-artifact `IdentityLimit` at `build`.
///
/// What it would take for this to say *no*: drop the `+ 1` from the lower
/// bound, and the failing side is admitted; drop the running-total commit in
/// `push_variant`, and the arithmetic stops depending on retained rows at all.
#[test]
fn a_physical_subset_that_proves_the_identity_oversize_is_refused_at_insertion() {
    // Sized so the run's exact contribution plus the one mandatory nonphysical
    // byte lands exactly on the limit. Derived from the row grammar rather than
    // guessed, so a grammar change moves this with it instead of silently
    // testing an interior point.
    let probe = physical_selection(0);
    let overhead = probe.canonical_key_bytes() - probe.region_occurrence.as_bytes().len();
    let run_overhead = 1 + 8 + 8 + overhead;
    let exact = MAX_ARTIFACT_IDENTITY_BYTES - 1 - run_overhead;

    let at_boundary = SelectedPhysicalImplementation {
        region_occurrence: PhysicalRegionOccurrenceIdentity::from_bytes(vec![0x5a; exact])
            .expect("the boundary identity is inside the per-value bound"),
        ..physical_selection(0)
    };
    let error = refuse_admitting(vec![at_boundary]);
    assert!(
        error.is_none(),
        "a candidate whose proved minimum is exactly the limit must pass insertion, got {error:?}",
    );

    let past_boundary = SelectedPhysicalImplementation {
        region_occurrence: PhysicalRegionOccurrenceIdentity::from_bytes(vec![0x5a; exact + 1])
            .expect("one byte more is still inside the per-value bound"),
        ..physical_selection(0)
    };
    let error = refuse_admitting(vec![past_boundary]).expect("one byte more is refused");
    assert!(
        matches!(
            error,
            ArtifactBuildError::IdentityLowerBound {
                minimum_bytes,
                limit
            } if limit == MAX_ARTIFACT_IDENTITY_BYTES && minimum_bytes == limit + 1
        ),
        "the refusal states the proved minimum against the existing limit, got {error:?}",
    );
}

/// The retained total is a fact about the builder, not about one candidate.
///
/// Two variants whose runs are individually admissible and jointly are not. The
/// second push can only refuse by consulting what the first one retained, so
/// this is the case that makes the private running total load-bearing — with a
/// per-call check the second candidate looks exactly like the first and passes.
///
/// What it would take for this to say *no*: drop the
/// `self.physical_identity_bytes = physical_bytes;` commit in `push_variant`,
/// and the second variant is admitted because the builder has forgotten the
/// first. The single-variant boundary case above cannot see that, because a
/// total that is never accumulated and a total that starts at zero agree when
/// there is only one contribution.
#[test]
fn the_retained_total_accumulates_across_variants() {
    let semantic = semantic_program();
    let first = fused_program(&semantic, SCALE_BITS);
    let second = fused_program(&semantic, super::OTHER_SCALE_BITS);
    let environment = CompilationEnvironment::new([lowering_provider(1)], offered_physical())
        .expect("both roles compose");
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).expect("a draft");
    draft
        .select_lowering_provider(selection(lowering_provider(1)))
        .expect("the lowering row is offered");
    let descriptor = draft.push_payload(payload(0xa1)).expect("a payload");
    let formulas = formulas(&mut draft);

    // Each run is a little over half the whole-identity limit, so either alone
    // is admissible and the pair cannot be.
    let half = MAX_ARTIFACT_IDENTITY_BYTES / 2;
    let run = |ordinal: u16| {
        vec![SelectedPhysicalImplementation {
            region_occurrence: PhysicalRegionOccurrenceIdentity::from_bytes(vec![0x5a; half])
                .expect("half the limit is inside the per-value bound"),
            ..physical_selection(ordinal)
        }]
    };

    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.selected_physical_implementations = run(0);
    draft
        .push_variant(&first, spec)
        .expect("one run of half the limit is admissible on its own");

    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.selected_physical_implementations = run(1);
    let error = draft
        .push_variant(&second, spec)
        .expect_err("the second run is refused against what the first retained");
    assert!(
        matches!(error, ArtifactBuildError::IdentityLowerBound { .. }),
        "the refusal reads the retained total, got {error:?}",
    );
}

/// Pushes one candidate run and returns only whether insertion refused it.
///
/// Separate from [`refuse`] because the boundary case above must be able to
/// *pass* insertion, and `refuse` asserts a refusal happened.
fn refuse_admitting(rows: Vec<SelectedPhysicalImplementation>) -> Option<ArtifactBuildError> {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let environment = CompilationEnvironment::new([lowering_provider(1)], offered_physical())
        .expect("both roles compose");
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).expect("a draft");
    draft
        .select_lowering_provider(selection(lowering_provider(1)))
        .expect("the lowering row is offered");
    let descriptor = draft.push_payload(payload(0xa1)).expect("a payload");
    let formulas = formulas(&mut draft);
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.selected_physical_implementations = rows;
    draft.push_variant(&program, spec).err()
}

// -------------------------------------------------------------------------
// The closed kind vocabulary
// -------------------------------------------------------------------------

/// The proposal-kind table is size-derived, injective, and total on its inverse.
///
/// The population comes from `variant_count`, so a widened enum is a build error
/// at `ALL` rather than a census that quietly stops covering its domain. The
/// reserved `0x04` is asserted refused by name: `tiler-compiler` has a fourth
/// kind today and rejects its body before selection, so admitting the tag would
/// let a forged manifest assert a selected state no compiler can produce.
#[test]
fn the_proposal_kind_table_is_injective_and_reserves_the_view_tag() {
    assert_eq!(
        PhysicalProposalKind::ALL.len(),
        std::mem::variant_count::<PhysicalProposalKind>(),
        "ALL must name every admitted proposal kind",
    );
    let mut seen = std::collections::HashMap::new();
    for kind in PhysicalProposalKind::ALL {
        let tag = kind.tag();
        assert!(
            seen.insert(tag, kind).is_none(),
            "tag {tag:#04x} is shared by two proposal kinds",
        );
        assert_eq!(
            PhysicalProposalKind::from_tag(tag),
            Some(kind),
            "tag {tag:#04x} does not round-trip",
        );
    }
    for tag in u8::MIN..=u8::MAX {
        if seen.contains_key(&tag) {
            continue;
        }
        assert_eq!(
            PhysicalProposalKind::from_tag(tag),
            None,
            "unclaimed tag {tag:#04x} must be refused",
        );
    }
    assert_eq!(
        PhysicalProposalKind::from_tag(0x04),
        None,
        "0x04 is reserved for a future reviewed `View` and is never admitted",
    );
}

/// The row grammar is exactly the seven fields the accepted surface names.
///
/// Pinned as a byte layout rather than described, because every consumer of
/// these bytes — the identity encoder, the manifest encoder, and the decoder's
/// nested cursor — has to agree on it, and prose cannot hold three encoders
/// together.
#[test]
fn the_row_key_is_its_exact_seven_field_grammar() {
    let row = physical_selection(0);
    let key = row.canonical_key();
    assert!(
        key.starts_with(PHYSICAL_SELECTION_KEY_DOMAIN),
        "the row key is self-describing and opens with its own domain",
    );
    assert_eq!(
        key.len(),
        row.canonical_key_bytes(),
        "the arithmetic sizing and the written bytes are one definition",
    );

    let mut expected = PHYSICAL_SELECTION_KEY_DOMAIN.to_vec();
    for field in [
        row.region_occurrence.as_bytes(),
        row.implementation_proposal.as_bytes(),
        row.provider.namespace().as_bytes(),
        row.provider.name().as_bytes(),
    ] {
        expected.extend_from_slice(&u64::try_from(field.len()).unwrap().to_be_bytes());
        expected.extend_from_slice(field);
    }
    // The revision is fixed-width and deliberately not length-framed.
    expected.extend_from_slice(&row.provider.revision().to_be_bytes());
    expected.push(row.proposal_kind.tag());
    assert_eq!(
        key, expected,
        "the row grammar is exactly these seven fields"
    );
}
