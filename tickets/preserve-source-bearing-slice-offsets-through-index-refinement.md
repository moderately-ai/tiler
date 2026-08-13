---
id: preserve-source-bearing-slice-offsets-through-index-refinement
title: Preserve source-bearing Slice offsets through index refinement
status: done
priority: p1
dependencies: [admit-source-bearing-slice-selection-semantics]
related: [admit-a-position-selecting-slice-for-the-rotary-table, admit-live-extent-operands-to-payload-indexing, accept-the-source-bearing-slice-realization]
scopes: [implementation/ir, implementation/compiler, contracts/foundation, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, slice, symbolic-extents, indexing, compiler]
---
# Preserve source-bearing Slice offsets through index refinement

## Goal

A source-bearing Slice offset reaches the exact index relation `t + C` without rebinding or specializing `C`, and total-access verification discharges the symbolic bound from the same retained `ShapeEnv` authority before the occurrence can become a physical candidate.

## Work

- Re-audit the accepted Slice decision, current law/lowering contexts, `IndexRefinementSubject`, region construction, symbolic interval propagation, and physical subject binding at the exact base before editing.
- Carry the exact source environment through Slice realization and refinement. Build source-aware regions and expose the existing sourced linear-combination authority rather than reconstructing a literal or caller-provided scalar.
- Extend the Slice law and governed lowering to spell `t + C` through the accepted `SourcedIndexInteger` coefficient/addend vocabulary while keeping old literal law bytes and behavior unchanged where possible.
- Add a checked total-access proof derived from the same environment or a compiler-minted proof subject that is identity-bound to it. Syntax alone is insufficient; an unproved symbolic coefficient remains a typed refusal.
- Audit law/provider revisions, refinement/request/explain identities, compiler budgets, failure vocabulary, docs, and pins. Do not add a second source environment, backend convention, specialization by live value, or artifact/runtime carrier here.

## Acceptance

- The canonical relation for a window at `C` contains the source-bearing `C * 1` term and no duplicated cursor input.
- Static/literal neighbours retain their existing bytes and realizations unless an explicitly justified provider revision moves their provenance.
- Foreign environment, wrong symbol, missing source, insufficient interval proof, overflow, and an intentionally removed bound check each fail at their named layer with watched failure text.
- A valid source-bearing Slice reaches a verified index region but remains non-executable until the live-extent payload carrier is present.
- Complete Fact verdict, identity blast radius, targeted IR/compiler tests, rustdoc, Clippy, `tkt lint`, `git diff --check`, `tkt guard`, and the required repository gate are recorded.

## Fact audit at `b0aa7d6ec892ec8eca45b4e982babafe5fccff9e`

The ticket has no numbered Facts section. These are the load-bearing claims in Work, Acceptance, the accepted decision, and the parent Outcome, re-read at this exact base.

- **Verified — a window offset is already `SourcedExtent`.** `crates/tiler-ir/src/semantic/slice.rs`, anchors `pub enum SliceAxisSelection` and `offset: SourcedExtent`. Construction is shape-independent. `SliceSelection::apply` proves `offset + extent <= available_axis` against the program's exact `ShapeEnv`.
- **Verified — the Slice law and governed lowering still refused a symbolic offset.** `realize_slice` and `GovernedSliceF32::lower`, anchor `unsupported("slice-symbolic-offset")` / `occurrence_error("slice-symbolic-offset")`, reconstructed only `SourcedExtent::Static` through `linear_combination`. Parent Outcome: "A symbolic offset is `unsupported("slice-symbolic-offset")` there."
- **Verified — `t + C` is already expressible and interval-provable.** `IndexRegionBuilder::sourced_linear_combination` admits a `SourcedIndexInteger` addend; a symbolic addend normalizes to the term `C * 1`. `interval_linear` bounds a symbolic coefficient from `ExtentSources::interval`. A `ShapeEnv` holds no values; `encode_region` folds the environment identity.
- **Verified — the environment is attached only for parametric broadcast.** `IndexRealizationLaw::realize` and `occurrence_needs_shape_environment`, anchors `broadcast_subject_is_parametric` and `if subject.operation() != &broadcast_f32_op()`. A neighbour in a program with an environment keeps the environment-free builder.
- **Verified — `IndexRefinementSubject` already retains the program environment.** `derive`, anchor `environment: SubjectEnvironment(program.extent_sources()...)`. `shape_environment()` returns that exact `Arc<ShapeEnv>`.
- **Verified — total access is not syntax.** An unproved symbolic coefficient retains `IndexDomainUnknownReason::InsufficientFacts`. `ResolvedIndexRealization::verify_sequence` returns `Pending` when residual obligations remain. `refine_index_region` does not treat Pending as Refined.
- **Verified — live-extent payload indexing is not this ticket.** `admit-live-extent-operands-to-payload-indexing` is `review` on a sibling branch. `tiler::slice-f32@1` remains in `UNPLANNED_OPERATIONS`. Compile of a source-bearing slice program is `compile.unsupported.strategy.operation-set`.
- **Verified — coordinator-unverified authorities were current, not drifted.** Purpose unchanged. Scopes added before editing: `contracts/foundation` for `docs/ir.md`; `contracts/navigation` for the glossary and support-matrix repairs.

## Outcome

A source-bearing `tiler::slice-f32@1` window now realizes as `t + C` through the accepted sourced addend vocabulary. The Slice law and governed lowering attach the subject's exact `ShapeEnv` only when `SliceSelection::names_a_symbol()` is true, emit `sourced_linear_combination(C, [(1, t)])`, and keep the literal `d + offset` path environment-free. The canonical relation stores constant `0` and the terms `C * 1` and `t * 1`. There is no second cursor input and no resolved-value rewrite.

Total-access verification discharges the symbolic bound from that retained environment (`BoundsProofView::Interval { facts: ShapeEnvironment }`). An unproved interval remains `InsufficientFacts` and does not mint a verified receipt. A window that is outside every model is `CoordinateOutOfBounds`. Foreign, wrong, and missing sources refuse at `ExtentSources::admit` as `sourced-extent.undeclared-symbol`.

A valid source-bearing occurrence refines to a verified one-region realization and remains non-executable: `compile(CompilationRequest::governed)` refuses `compile.unsupported.strategy.operation-set`. Live-extent payload indexing stays with its owning ticket.

**Identity blast radius.** `tiler.slice-selection.v1`, `tiler::slice-f32@1`, law encoding tag 13, slice law revision 1, `tiler.ir.index-realization-law-registry.v1`, standard semantic provider revision 8, and `governed_provider("slice-f32")` revision 1 are unchanged. Literal law-row digest `2a352358c72d1d4c…` and every older row stay byte-identical. Literal slice realizations keep the environment-free builder, so their region identities do not move. No artifact or manifest schema step. Explain-request and lowering-registry pins that fold those unchanged identities stay put.

**Quoted perturbation failures.**

- Foreign environment (`ghost` against an environment that declares `c`): `sourced-extent.undeclared-symbol: slice/0::ghost is not declared by this program's shape environment`
- Wrong symbol (`d` against an environment that declares `c`): `sourced-extent.undeclared-symbol: slice/0::d is not declared by this program's shape environment`
- Missing source (no environment): `sourced-extent.undeclared-symbol: slice/0::c is not declared by this program's shape environment`
- Insufficient interval (`C` in `[0, 64]` against a 64-extent axis and extent 6): `IndexDomainUnknownReason::InsufficientFacts`
- Overflow (`C` pinned at 64 against a 64-extent axis): `IndexRegionDiagnostic::CoordinateOutOfBounds`
- Intentionally removed bound check (`C` in `[0, 100]`, syntax present, no interval proof): the region builds, the read has no `bounds_proof`, and the residual `LessThanExtent` obligation remains
- Non-executable after a verified region: `compile.unsupported.strategy.operation-set: no installed capability can compile this valid semantic program`

**Public surface.** `SliceSelection::names_a_symbol` is an additive query on the existing labelled-draft selection type. `IndexAccessLoweringContext::sourced_linear_combination` is crate-internal, matching `sourced_dimension`. No second source environment, backend convention, live-value specialization, or artifact/runtime carrier was added.

**Checks.** `cargo test -p tiler-ir -p tiler-compiler` green, including lib, integration, trybuild, and doc-tests. `cargo clippy -p tiler-ir -p tiler-compiler --all-targets -- -D warnings` clean. `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-ir -p tiler-compiler --no-deps` clean. `tkt lint`, `git diff --check`, and `tkt guard --base main --format json` recorded at commit time. Coordinator `make full` on `f903da13`: 3465 passed. Merged to `main` at `0742e22dc5e3cc1e24b017f4bc2d4b0f0fde9c03`. Tom accepted the widened Slice-law interpretation on 2026-08-13 in [`accept-the-source-bearing-slice-realization`](accept-the-source-bearing-slice-realization.md), without a law or provider revision bump.
