---
id: step-the-refinement-subject-identity-to-carry-its-shape-environment
title: Step the refinement subject identity to carry its shape environment
status: done
priority: p0
dependencies: []
related: [decide-whether-the-refinement-subject-identity-should-carry-its-environment]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [identity, indexing]
---
## User-visible outcome

`IndexRefinementSubject`'s canonical identity carries the identity of the shape environment its symbolic extents resolve against, so two subjects that differ only in environment mint different bytes and `ResolvedIndexRealization` can no longer report two resolutions equal while each realizes a region the other's verifier refuses.

## Why this exists

The decision is already taken and evidenced: [`decide-whether-the-refinement-subject-identity-should-carry-its-environment`](decide-whether-the-refinement-subject-identity-should-carry-its-environment.md) enumerated the options at `9b61b563`, eliminated the documented exclusion, and constructed the collision from the governed standard registry alone. Read that ticket's Fact audit and Measurement sections before starting; do not re-derive the decision, and do re-audit its Facts at your own base.

This is a separate ticket because folding the environment in steps an identity domain, and an identity step needs its own coherent change across the owning version, the domain ledger, and every nested pin — which the decision ticket was explicitly scoped out of.

## Facts, each to be re-audited at your base

**Fact — verified at `9b61b563`.** `encode_subject_identity_with` in `crates/tiler-ir/src/index/refinement/identity.rs` writes the graph subject, anchored `push_slice(&mut bytes, subject.graph.as_bytes());`, and never reads the environment: `grep -c "environment" crates/tiler-ir/src/index/refinement/identity.rs` returns **0**.

**Fact — verified at `9b61b563`.** The domain to step is anchored `tiler.ir.index-refinement-subject.v2` in `crates/tiler-ir/src/index/refinement/mod.rs`, and `crates/tiler-ir/src/domains.rs` pins both `v1` and `v2` in its ledger. `v3` is added there; `v1` and `v2` stay, on the reasoning that file already records for keeping a superseded spelling beside its successor.

**Fact — verified at `9b61b563`, and it bounds the blast radius much more tightly than the previous subject step did.** `encode_receipt_identity` nests the subject bytes with `push_slice(&mut bytes, &subject.identity);` and `encode_resolution_identity` nests them too, so both `tiler.ir.index-refinement-receipt.v1` and `tiler.ir.index-realization-resolution.v1` move **in value**. Neither grammar changes, because both length-frame the complete stepped key including its separator — the same argument [`canonicalize-index-refinement-occurrence-ordinals`](canonicalize-index-refinement-occurrence-ordinals.md) made for `tiler.artifact-program.v14`. Confirm that argument yourself rather than inheriting it.

**Fact — verified at `9b61b563`.** `encode_executable_coverage_identity` does **not** read `subject.identity`; it writes the graph digest, occurrence, numerical contract, realization identity, semantic authority projections, and law row. Kernel-program identity folds the *coverage* identity, not the receipt identity — `crates/tiler-ir/src/program/model.rs` states this under the anchor `reached-only projection, never`. So kernel-program and artifact identities do **not** move, and this step is not the workspace-wide re-pin the previous one was. Verify this before deciding not to touch the artifact scope; `implementation/artifact` is deliberately not declared on this ticket on the strength of it, and if it turns out to be wrong the scope must be added before editing.

**Fact — verified at `9b61b563`.** `crates/tiler-compiler/src/legality.rs` nests `push_slice(&mut bytes, receipt.identity().as_bytes());` into its own `IndexRefinementIdentity`, so compiler-side refinement identity bytes move in value. `implementation/compiler` is declared for that reason; if no compiler test pins those bytes, say so and leave the scope unused rather than editing to justify it.

## Required work

- Fold the environment identity into `encode_subject_identity_with`, and step `SUBJECT_IDENTITY_TAG` to `tiler.ir.index-refinement-subject.v3`. Decide and record whether the field is written as a presence tag plus framed identity, the way `encode_region` does it — the site is anchored `push_slice(&mut out, sources.environment_identity().as_bytes());` in `crates/tiler-ir/src/index/builder/identity.rs` — or as the total fifth subject the semantic layer uses, where a program with no environment reports the empty environment's identity, anchored `the fifth semantic subject is` in `crates/tiler-ir/src/semantic/program.rs`. These are not equivalent: the second makes "declares no symbols" and "has an empty environment" one fact with one spelling, which is the property that layer chose deliberately. Pick one, say why, and make the encoder agree with `SubjectEnvironment`'s `PartialEq`.
- Add `tiler.ir.index-refinement-subject.v3` to the ledger in `crates/tiler-ir/src/domains.rs` and update the prose beside `v1`/`v2` there. Recheck the no-prefix argument that file carries: a `v3` spelling must be checkably non-prefixing against the admitted set, and the existing test population is what establishes it.
- Repair the pin in the refinement suite that asserts a derived subject identity `starts_with(SUBJECT_IDENTITY_TAG)` and not `LEGACY_SUBJECT_IDENTITY_TAG`. A single legacy constant can no longer name the whole superseded set; decide whether it becomes a set and make a reverted domain still redden the gate, which is the property [`pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate`](pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate.md) exists to hold.
- Land the regression. The probe below is the decision ticket's evidence verbatim at `9b61b563`; convert it into an assertion that the subject identity bytes now **differ**, that `ResolvedIndexRealization` no longer compares equal, and that the receipt identities still differ. Perturb the subject, not the assertion: revert the encoder line and quote the failure text.
- Recompute every generated value, golden, and identity pin on the merged tree, and confirm by reading which ones did not move as well as which did.

## The construction, verbatim from the decision lane

Appended to `crates/tiler-ir/src/index/refinement/tests.rs` at `9b61b563`, where the surrounding `use super::*;` and the module's existing imports already supply every name it uses. It exits by `panic!` because it was a probe; the regression form asserts instead.

```rust
fn probe_env(axis: u32, relations: &[ExtentRelation]) -> Arc<crate::shape::ShapeEnv> {
    let symbol = ShapeSymbol::new(SymbolScope::new("probe/0").unwrap(), "n").unwrap();
    let mut draft = ShapeEnvBuilder::new();
    draft.declare(symbol.clone()).unwrap();
    draft
        .bind(
            &symbol,
            RootBinding::new(
                BindingSource::InputDimension {
                    input: InputKey::new("rows").unwrap(),
                    axis: crate::shape::Axis::new(axis),
                },
                EXTENT_PHASE_CEILING,
                FactProvenance::RuntimeValidated,
            )
            .unwrap(),
        )
        .unwrap();
    for relation in relations {
        draft
            .require(SemanticInputConstraint::new(
                relation.clone(),
                FactProvenance::FrontendRequired,
            ))
            .unwrap();
    }
    Arc::new(draft.build().unwrap())
}

fn probe_subject(environment: Arc<crate::shape::ShapeEnv>) -> IndexRefinementSubject {
    let symbol = ShapeSymbol::new(SymbolScope::new("probe/0").unwrap(), "n").unwrap();
    let mut program =
        SemanticProgramBuilder::try_standard_with_shape_environment(environment).unwrap();
    let input = program
        .input_sourced::<F32>(
            InputKey::new("rows").unwrap(),
            vec![SourcedExtent::Symbol(symbol)],
        )
        .unwrap();
    let result = F32Multiply::apply(&mut program, input, input).unwrap();
    program
        .output(OutputKey::new("output").unwrap(), result)
        .unwrap();
    let program = program.build().unwrap();
    IndexRefinementSubject::derive(
        &program,
        program.operations().next().unwrap().id(),
        test_contract(),
    )
    .unwrap()
}

#[test]
fn probe_two_environments_one_subject_identity() {
    let first = probe_subject(probe_env(0, &[]));
    let by_axis = probe_subject(probe_env(1, &[]));
    let symbol = ShapeSymbol::new(SymbolScope::new("probe/0").unwrap(), "n").unwrap();
    let constrained = probe_subject(probe_env(
        0,
        &[ExtentRelation::interval(ExtentTerm::Symbol(symbol), 1, 8).unwrap()],
    ));

    let scalars = FrozenScalarRegistry::standard().unwrap();
    let semantic = FrozenSemanticRegistry::standard().unwrap();
    let laws =
        FrozenIndexRealizationLawRegistry::from_semantic(semantic.clone(), scalars.clone()).unwrap();

    for (label, other) in [("binding axis", &by_axis), ("constraint", &constrained)] {
        println!(
            "{label}: environment identities differ = {}",
            first.shape_environment().unwrap().identity()
                != other.shape_environment().unwrap().identity()
        );
        println!(
            "{label}: graph identities equal = {}",
            first.graph() == other.graph()
        );
        println!(
            "{label}: SUBJECT IDENTITY BYTES EQUAL = {}",
            first.identity == other.identity
        );
        println!(
            "{label}: subject PartialEq says equal = {}",
            first == *other
        );
        let left = super::super::IndexRealizationLaw::multiply_f32().realize(&first, &scalars);
        let right = super::super::IndexRealizationLaw::multiply_f32().realize(other, &scalars);
        match (&left, &right) {
            (Ok(left_region), Ok(right_region)) => println!(
                "{label}: REGION IDENTITIES EQUAL = {}",
                left_region.canonical_identity() == right_region.canonical_identity()
            ),
            _ => println!("{label}: realize refused: {left:?} / {right:?}"),
        }
        let left_resolution = laws.resolve(&first).unwrap();
        let right_resolution = laws.resolve(other).unwrap();
        println!(
            "{label}: RESOLUTION IDENTITIES EQUAL = {}",
            left_resolution.identity == right_resolution.identity
        );
        println!(
            "{label}: ResolvedIndexRealization PartialEq says equal = {}",
            left_resolution == right_resolution
        );
        let authority = IndexRealizationAuthority::admit(
            &semantic,
            &scalars,
            first.operation().clone(),
            first.signature().clone(),
            &[super::super::multiply_f32_scalar_op()],
        )
        .unwrap();
        let left_region = left.unwrap();
        let right_region = right.unwrap();
        let cross = left_resolution.verify(&authority, &right_region);
        println!(
            "{label}: resolution A accepts region B = {}",
            cross.is_ok()
        );
        if let Err(error) = &cross {
            println!(
                "{label}: cross refusal kind = {}",
                match error {
                    IndexRefinementVerificationError::SemanticRealizationMismatch { .. } =>
                        "SemanticRealizationMismatch",
                    _ => "other",
                }
            );
        }
        println!(
            "{label}: residual obligations A/B = {} / {}",
            left_region.unknown_index_domain_predicates().len(),
            right_region.unknown_index_domain_predicates().len()
        );
        let IndexRefinementVerificationOutcome::Verified(left_receipt) =
            left_resolution.verify(&authority, &left_region).unwrap()
        else {
            panic!("no residual obligation expected")
        };
        let IndexRefinementVerificationOutcome::Verified(right_receipt) =
            right_resolution.verify(&authority, &right_region).unwrap()
        else {
            panic!("no residual obligation expected")
        };
        println!(
            "{label}: RECEIPT IDENTITIES EQUAL = {}",
            left_receipt.identity() == right_receipt.identity()
        );
        println!(
            "{label}: COVERAGE IDENTITIES EQUAL = {}",
            left_receipt.executable_coverage_identity()
                == right_receipt.executable_coverage_identity()
        );
    }
    panic!("probe output above");
}
```

Its output at `9b61b563` is recorded in the decision ticket. The two lines that matter are `SUBJECT IDENTITY BYTES EQUAL = true` beside `subject PartialEq says equal = false`, and `RESOLUTION IDENTITIES EQUAL = true` beside `REGION IDENTITIES EQUAL = false`.

## Non-goals

Changing `encode_region`'s treatment of environment, which is settled and reasoned. Folding the environment into `SemanticGraphIdentity`, which `crates/tiler-ir/src/semantic/identity.rs` rejects with a recorded reason under the anchor `A separate subject rather than part of`. Widening the environment-aware operation population. Any public surface change beyond the identity bytes themselves.

## Outcome

Implemented at `d45f8f36d41cc6326798d783558d2b1265fd4948` and integrated without rewriting that reviewed commit at `c454eccc`. The live subject domain is `tiler.ir.index-refinement-subject.v3`; v1 and v2 remain reconstructible test grammars and remain pinned in the IR domain ledger.

The encoder appends one framed total `ShapeEnvIdentity`. It deliberately gives an absent environment object and an explicitly empty environment the same identity, matching the semantic layer's existing authority and `SubjectEnvironment::PartialEq`. An exhaustive destructure names every subject field except the self-derived identity, so adding a future field is a compile error at the encoder until its identity policy is explicit.

The merged tests confirm that binding-axis and constraint-only environment changes move subject, resolution, receipt, and compiler refinement values, while executable coverage, `CoveredOccurrence`, kernel-program identity, and artifact identity do not move. No public API, artifact schema, or downstream domain required a step.

The subject perturbation was rerun in the detached review worktree by removing the environment fold and executing:

```sh
cargo nextest run -p tiler-ir environment_only_changes_separate_subjects_resolutions_and_receipts
```

It failed at the intended subject with `binding axis: environment-only subjects minted equal identity bytes`. Restoring the fold returned the reviewed worktree to a clean `d45f8f36`.

The independent review found no defects and reran the complete `tiler-ir` package tests, selected identity tests, doctests, Clippy with warnings denied, rustdoc with warnings denied, ticket lint, exact-base guard, and `git diff --check`. On the merged tree, `make full` passed: 4,083 workspace tests, 1,354 release tests, formatting, workspace checks, Clippy, doctests, public and private rustdoc with warnings denied, citations, ticket lint, and shellcheck.

## Closes when

The subject identity carries the environment under a stepped domain, the ledger and every pin are coherent on the merged tree, the regression fails when the fold is reverted with its failure text quoted, and the claim that kernel-program and artifact identities do not move is either confirmed by reading or corrected with the scope added.
