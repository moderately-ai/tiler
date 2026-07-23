---
id: close-remaining-adr-status-drift
title: Close the remaining ADR status drift
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions]
claimed_from: todo
assignee: agent-close-remaining-adr-status-drift
lease_expires_at: 1784828687
---
The coherence sweep bumped ADR 0006, ADR 0018, and numerical-semantics.md from `not-started` to `partial` after reading their implementations, and deliberately stopped there rather than silently widening accepted-ADR maturity metadata beyond the audited set. Two further ADRs carry the same class of drift and were reported with evidence:

- ADR 0009 (resolve numerical typing before semantic optimization) remains `not-started` although per-value `ResolvedValueType` with registry-resolved operation signatures is implemented and tested in `tiler-ir`;
- ADR 0024 (round-to-nearest ties-to-even for initial arithmetic) remains `not-started` although `binary32-round-to-nearest-ties-even` is registered as durable identity on the standard f32 arithmetic operations in `crates/tiler-ir/src/semantic/registry.rs`, with reference-evaluator coverage.

Read each implementation before bumping; do not bump on this ticket's say-so. While in this scope, audit every remaining accepted ADR's `implementation_status` against the crates so the corpus stops drifting one audit at a time, and report any ADR whose status cannot be justified either direction.

## Outcome

- Audited all 71 accepted ADRs against `crates/` rather than against ADR prose, this ticket's claims, or sibling ADR status. Ten `implementation_status` values changed; no ADR body, `decision_status`, or typed edge was touched.
- The two reported ADRs were confirmed by reading the implementation and bumped `not-started` to `partial`: ADR 0009 (`ResolvedValueType` stored per value in `crates/tiler-ir/src/semantic/operation.rs:1309`, `ValueFact` operand/result signatures resolved and revalidated by `FrozenSemanticRegistry::infer_operation` at `crates/tiler-ir/src/semantic/registry.rs:1276`) and ADR 0024 (`binary32-round-to-nearest-ties-even` as durable operation-definition identity at `crates/tiler-ir/src/semantic/registry.rs:1911`, applied to Add and Multiply, with round-to-nearest reference evaluation and NaN canonicalization at `crates/tiler-reference/src/lib.rs:1536`).
- Six further ADRs were bumped after full reads: ADR 0026 and ADR 0027 (`not-started` to `partial`); ADR 0007, ADR 0043, ADR 0046, ADR 0048, and ADR 0069 (`spike-only` to `partial`, because each now has real production code in `crates/`, not only a spike).
- ADR 0067 moved `partial` to `implemented`. Every clause of its decision is closed: the dependent-array family at `crates/tiler-ir/src/shape/evidence.rs:63`, exactly the three named features and none of the forbidden ones at `crates/tiler-ir/src/lib.rs:1`, builder- and program-owned refinement at `crates/tiler-ir/src/semantic/program.rs:358`, the `nightly-2026-07-19` pin with no stable `rust-version`, and the retained conformance harness executed by the repository gate at `scripts/check_rust.py:389`. The commit that implemented the shaped-value API updated the ADR body but left the frontmatter behind; that is the drift this ticket names.
- Deliberately left unchanged despite partial machinery, because the decision's central mechanism does not exist and a bump would misreport a type-system reservation or an architectural seam as implemented support: ADR 0011 and ADR 0019 (single-variant permission and subnormal enums that can never resolve anything), ADR 0012 and ADR 0014 (physical reduction topology exists, but no typed semantic order-contract vocabulary), ADR 0020, ADR 0022, ADR 0025, and ADR 0051 (a checked one-way routing policy exists in the compiler-produced plan at `crates/tiler-compiler/src/program.rs:511`, but every clause of the decision is a runtime-launcher behaviour and no runtime exists).
- ADR 0034 is recorded as undetermined rather than changed: `NormativeDefinitionRef` is mandatory on every registered type definition and enters durable identity, but it is a validated string that cannot separately represent the authority, document, revision, and format the decision requires, and no same-format owner check runs before minting a built-in key.
- No accepted ADR was found over-claimed. ADR 0073 was examined for promotion to `implemented` and deliberately left at `partial`: `crates/tiler-compiler/src/explain.rs:1` declares itself a private draft with reserved unused views, and its trace verification is `#[cfg(test)]` only.
- `docs/decisions/README.md` is generated but `implementation_status` is not a catalog input; `scripts/docs.py render` confirmed it unchanged.
