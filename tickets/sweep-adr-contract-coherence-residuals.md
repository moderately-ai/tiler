---
id: sweep-adr-contract-coherence-residuals
title: Sweep ADR and contract coherence residuals
status: in-progress
priority: p1
dependencies: [reconcile-scalar-broadcast-contract]
related: []
scopes: [contracts/decisions, contracts/foundation, contracts/numerics, contracts/optimizer, research/semantic-graph]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions]
claimed_from: todo
assignee: agent-sweep-adr-contract-coherence-residuals
lease_expires_at: 1784825414
---
Adversarially verified coherence residuals across accepted ADRs and contracts (2026-07-23 audit), none owned by an open ticket. Sequenced after reconcile-scalar-broadcast-contract because both edit ir.md/numerical-semantics.md.

- ADR 0058 still orders a `reference` namespace in tiler-ir, contradicting ADR 0065 and the crate; annotate the superseded sentence.
- operation-extensions.md evidence frontmatter omits the adopted semantic-foundation-api-v2 correction it was rebuilt on.
- Textually declared refinement chains carry no `refines` frontmatter: 0025 to 0022, 0042 to 0016, 0061 to 0059, 0062 to 0059/0060.
- ADR 0016's Decision still says the accuracy vocabulary "remains open" although ADR 0042 fixed it; add the forward annotation used elsewhere in the corpus.
- numerical-semantics.md's ownership section claims "the accepted decisions are ADRs 0009-0042" while 0055/0059/0060/0062/0066 also declare it in applies_to.
- ADR 0061's consequences retain a future-tense conformance-spike gate for an implementation its own Decision records as landed.
- ADR 0052's Decision body spells the canonical type `CanonicalAttrValue` while its Implementation boundary and the code use `CanonicalValue`; reconcile the spelling.
- ADR 0056's supersession wording names only the compiler-to-artifact edge; explicitly retire the stale "tiler-artifact may use lockstep internal IR types" consequence under ADR 0070/0071.
- ADR 0069's general compilation boundary is unrepresented in docs/compiler/optimizer.md; represent the boundary and its five failure classes.
- fusion-and-scheduling.md uses `StorageHandoff` as the producer-to-consumer visibility edge, contradicting the program-planning research that defines it as the allocation-reuse ordering edge; correct the mechanism named.
- numerical-semantics.md announces "three machine-checkable parts" but its heading structure nests only one; restore the three-part structure.
- Status frontmatter drift: ADR 0006, ADR 0018, and numerical-semantics.md remain implementation_status "not-started" although their bounded-f32 cores are implemented and tested (siblings 0005/0052 were bumped to partial in the replan).
- docs/research/semantic-graph/contract-memo.md records a dated accepted identity-normalization decision (merge identical pure invocations before computation identity) that no contract or ADR represents and that ADR 0064 appears to contradict. **Resolution decided by Tom on 2026-07-23: relocate, do not supersede.** ADR 0064 did not reject that decision; it moved common-subexpression elimination out of commitment compaction into the later layers ("Semantic rewrites, common-subexpression elimination, and physical planning remain in their existing later layers"). Annotate the memo's item 2 in place to record that the merge obligation belongs to the deterministic normalization stage rather than to commitment, and cross-reference ADR 0064. Do not weaken or delete the original decision text, and do not supersede ADR 0064. The matching implementation obligation is recorded on `prototype-semantic-normalization`.

Run the full documentation gate before completion.

## Outcome

All thirteen residuals were re-verified against their cited files and the crates before editing, then closed. No generated catalog block changed: contract `evidence`, decision `refines`, and `implementation_status` are not inputs to any `docs.py` catalog, and `docs.py render` left `docs/decisions/README.md`, `docs/research/README.md`, and `spikes/README.md` untouched.

- **ADR 0058 `reference` namespace.** `crates/tiler-ir/src/lib.rs` exposes only `index`, `semantic`, and `shape`. Added a paragraph after the namespace rule recording that ADR 0065 supersedes the `reference` namespace and that the remaining rules stand.
- **operation-extensions.md evidence.** `semantic-foundation-api-v2` declares `informs: [… tiler.contract.operation-extensions]` with no reciprocal `evidence` entry. Added it.
- **Refinement chains.** Added `refines` to ADR 0025 (`ADR-0022`), ADR 0042 (`ADR-0016`), ADR 0061 (`ADR-0059`), and ADR 0062 (`ADR-0059`, `ADR-0060`), each backed by explicit prose in the refined or refining decision. The relation graph remains acyclic.
- **ADR 0016 forward annotation.** Added the ADR 0022-style note after the Status line pointing at ADR 0042's four discriminated forms; the original Decision text is unchanged.
- **numerical-semantics.md ownership.** Enumerating `applies_to` across `docs/decisions/[0-9]*.md` confirms ADRs 0009–0042 plus 0055, 0059, 0060, 0062, and 0066. The ownership sentence now names them.
- **ADR 0061 conformance gate.** `scripts/check_rust.py` runs `spikes/shapes/nightly-dependent-static-shapes/check.sh` on every gate invocation, so the consequence now records a standing gate instead of a pending landing.
- **ADR 0052 spelling.** `CanonicalValue` is the implemented Rust type; `CanonicalAttrValue` survived only in documentation. Reconciled ADR 0052's Decision body and Implementation boundary, `docs/ir.md`, and `docs/operation-extensions.md` onto `CanonicalValue`.
- **ADR 0056 lockstep consequence.** `tiler-artifact` depends on `tiler-ir` alone. Marked the consequence **Retired** in place under ADR 0070's IR ownership and ADR 0071's requirement that artifact decoding reconstruct values through the shared checked builders, and extended the Status line. The original text and the retained prohibition on invoking compiler passes are preserved.
- **ADR 0069 in optimizer.md.** ADR 0069 lists `tiler.contract.optimizer` in `applies_to` but only `docs/architecture.md` represented it. Added a `Compilation boundary and failure classes` section naming the general boundary and the five classes, and added the adopted research to the contract's `evidence`.
- **fusion-and-scheduling.md `StorageHandoff`.** The program-planning research defines `StorageHandoff(AllocationId)` as the allocation-reuse ordering edge and `Data(MaterializedValueId)` as the producer-to-consumer edge. The multi-pass reduction paragraph now names `Data` and states what `StorageHandoff` actually orders.
- **Three-part structure.** Commit `1340ec4` inserted `## Resolved numerical typing` as a sibling of `## Three parts of the contract`, orphaning two of the three parts. Demoting it and `## Value assumptions and validation` to `####` restores exactly three `###` parts — operation semantics, optimization permissions, execution guarantees — without reordering any prose. The intro now names them.
- **Status drift.** Bumped ADR 0006, ADR 0018, and numerical-semantics.md to `partial` after confirming the implementations: `SemanticProgram` with operations, values, ordered named outputs, and output-reachable canonical identity; `CANONICAL_F32_ARITHMETIC_NAN_BITS = 0x7fc0_0000` applied by the reference evaluator with tests covering arithmetic canonicalization and constant payload preservation.
- **Contract memo identity normalization.** Annotated item 2 in place as **relocated, not superseded**, per Tom's 2026-07-23 decision. The 2026-07-18 text is untouched, ADR 0064 is not superseded, and the annotation points at `prototype-semantic-normalization` as the implementation owner. No contract or ADR restates the obligation.

Beyond the ticket's list: ADR 0009 and ADR 0024 carry the same `not-started` drift. Both are implemented in part — per-value `ResolvedValueType` with registry-resolved signatures, and the `binary32-round-to-nearest-ties-even` fact registered as durable identity on the standard f32 arithmetic operations in `crates/tiler-ir/src/semantic/registry.rs`. They were left unchanged because the audit did not list them; they are a candidate follow-up.

`uv run --locked python scripts/check_repository.py` fails at `scripts/docs.py validate` with `tickets/prototype-typed-explain-infrastructure.md: done ticket requires ## Outcome`. That breakage predates this branch: base commit `e29fa19` marked that ticket done without an Outcome, and it is present on `main` and `origin/main`. It belongs to `prototype-typed-explain-infrastructure`, not here. With a temporary local Outcome stub in place the complete gate — docs validate, CI contract, Ruff, pytest, ShellCheck, `tkt lint`, and the Rust gate — passed; the stub was reverted and is not committed.
