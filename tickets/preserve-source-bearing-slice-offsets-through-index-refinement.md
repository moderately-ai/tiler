---
id: preserve-source-bearing-slice-offsets-through-index-refinement
title: Preserve source-bearing Slice offsets through index refinement
status: in-progress
priority: p1
dependencies: [admit-source-bearing-slice-selection-semantics]
related: [admit-a-position-selecting-slice-for-the-rotary-table, admit-live-extent-operands-to-payload-indexing]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, slice, symbolic-extents, indexing, compiler]
claimed_from: todo
assignee: worker-source-bearing-slice
lease_expires_at: 1786647017
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
