---
id: implement-index-domain-predicates
title: Implement typed index-domain predicates and proof exchange
status: todo
priority: p1
dependencies: [implement-shapeenv-index-bindings]
related: [prototype-canonical-index-region-slice]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, proof, mature-product]
---
Add the accepted bounded typed predicate language, semantic obligations, durable proof evidence, and sound Unknown outcomes to verified index regions. Extend bounds and write-ownership proving beyond the static structural and finite fallback profile without converting semantic predicates into physical guards.


## Scoped — split at the Unknown boundary (2026-07-27)

Thirteen lines naming four deliverables at once. Reading `crates/tiler-ir/src/index/builder/proof.rs` (691 lines) shows the four are not peers, and that one of them is largely built.

**The three-valued logic already exists and is already sound.** `interval_verdict` returns `IntervalVerdict { interval_proved, definitely_outside }`, and its own documentation states the invariant this ticket would otherwise have to establish from scratch: "the two answers are independent and neither implies the other's negation: an interval that overlaps a boundary proves nothing either way, while one lying wholly outside refutes the access." A symbolic axis is compared "against the side of its own interval that makes each answer sound" — lower bound to prove, upper bound to refute. Where the environment bounds an axis nowhere, both flags stay false: "nothing about it is provable and nothing about it is refutable either." That is `Unknown`, computed correctly, today.

**What is absent is durability, not the logic.** An unproved obligation currently has nowhere to live. When the exhaustive fallback exceeds `MAX_EXHAUSTIVE_PROOF_CELLS` / `MAX_EXHAUSTIVE_PROOF_BYTES`, the budget refusal is pushed as a *diagnostic* (`proof.rs:336`) — the region is refused rather than verified-with-an-obligation. So `Unknown` is computed internally and then collapsed to rejection at the boundary. The four deliverables reduce to: a predicate language able to *state* an obligation, a place in the verified region to *carry* it, and a discharge protocol; the sound `Unknown` outcome is mostly the existing verdict stopping being flattened.

**The constraint that orders the split, and it is the whole reason the ticket exists.** The body says the extension must not convert semantic predicates into physical guards. That is precisely the tempting resolution: a region whose bounds cannot be proved could be admitted with a runtime bounds check, and it would work. It would also move a semantic obligation into the physical layer where the optimizer can no longer reason about it, and make "unproved" indistinguishable from "proved, then guarded anyway". An `Unknown` must remain an obligation the compiler carries and reports, never a guard it silently inserts.

**Suggested split, in dependency order.** Each is landable alone and the first is the only one that needs a design decision this ticket cannot supply from the code:

1. **The bounded typed predicate language.** The vocabulary an obligation is stated in. Everything below is expressed in it, so it comes first. *This is the piece the ticket's phrase "the accepted bounded typed predicate language" refers to, and no accepted design is linked from here or findable under `docs/decisions/`; locate or write it before starting.*
2. **Durable proof evidence.** Where a discharged obligation's evidence lives in the verified region and how it enters identity. Must keep `SoundProof`, exhaustive finite evidence, empirical evidence, and `Unknown` as distinct classes rather than a confidence scalar.
3. **Sound `Unknown` outcomes.** Stop flattening the budget refusal and the unbounded-axis case into diagnostics; carry them as stated obligations. The pin: a region that is refused today for exceeding the proof budget must instead verify carrying an explicit obligation, and a test must confirm no physical guard was inserted for it.
4. **Semantic obligations and discharge.** Who discharges a carried obligation, when, and what happens if nobody does — which must be an explainable refusal at a named stage, not an admitted program.

Do not start at 3 or 4. Without 1 there is no vocabulary to state an obligation in, and an obligation stated ad hoc is the physical-guard failure in a different spelling.
