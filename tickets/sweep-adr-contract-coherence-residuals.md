---
id: sweep-adr-contract-coherence-residuals
title: Sweep ADR and contract coherence residuals
status: todo
priority: p1
dependencies: [reconcile-scalar-broadcast-contract]
related: []
scopes: [contracts/decisions, contracts/foundation, contracts/numerics, contracts/optimizer, research/semantic-graph]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions]
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
