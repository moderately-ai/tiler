---
id: correct-roadmap-capability-wiring-claims
title: Correct the roadmap's stale claim that no capability registry caller exists
status: done
priority: p1
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, roadmap, capability]
---
Two passages in `docs/roadmap.md` state that the lowering-capability registry has no production caller and that no governed provider registers a capability. Both were true when written and are false at `412ceae`.

**Fact — the stale text.** The Milestone 6 preconditions paragraph (`docs/roadmap.md`, the "Fact — a matmul cannot currently be presented to the compiler at all" paragraph) says of `capability.rs` and `legality.rs`: "neither is reached from `pipeline::compile`; both are `pub mod` draft authorities with no in-crate production caller, and no governed provider registers an index-access or scalar-lowering capability". The support-matrix closing paragraph ("Two structural limits bound every rung above R4") repeats the second half: "the lowering-capability registry ... is an implemented mechanism with no in-crate production caller, and no governed provider registers a built-in index-access or scalar-lowering capability today".

**Fact — what the tree does.** `crates/tiler-compiler/src/pipeline.rs:827` calls `crate::lowering::resolve_lowering`, which resolves an index-access capability for every recognized occurrence through the frozen registry the request carries and then drives the resolved provider through `crate::legality::refine_index_region`. `crates/tiler-compiler/src/governed.rs` registers four governed `IndexAccessLoweringProvider` capabilities — `tiler.constant-f32`, `tiler.multiply-f32`, `tiler.add-f32`, `tiler.strict-serial-sum-f32` — and `request::CompilerCapabilitySnapshot::governed` installs them. `wire-capability-and-refinement-into-compile-path` is the ticket that landed it, and it is `done`.

Correct both passages against the tree rather than deleting them: the surrounding arguments (a matmul still cannot be presented to the compiler because `normalize_serial_sum` recognizes one program shape; the four admitted operations are still not compilable in arbitrary combinations) remain correct and load-bearing, and only the capability-wiring clause is wrong. Check the whole file for the same claim under other wording before concluding two sites are all of them.

Scope note: `propagate-lowering-capability-wiring-into-contracts` owns the same correction for `contracts/optimizer`, `contracts/numerics`, and `contracts/foundation`. `docs/roadmap.md` is `contracts/navigation` and falls outside it, which is why this is separate. Coordinate so the two do not restate each other.

Run `uv run --locked python scripts/docs.py render` and `uv run --locked python scripts/check_repository.py` before completion.

## Outcome

Done by the coordinator at merge time rather than dispatched: `contracts/navigation` was free once `draft-public-extension-seam-ownership-adr` merged, and both corrections are factual edits with the evidence already in hand.

**Both sites corrected, not one.** The ticket named two and both were live on `main`. The claim at the Milestone 6 framing said `capability.rs` and `legality.rs` are "`pub mod` draft authorities with no in-crate production caller" and that "no governed provider registers an index-access or scalar-lowering capability"; the claim in the support matrix's structural-limits paragraph said the same of the registry. Verified false before editing: `crates/tiler-compiler/src/pipeline.rs:827` calls `resolve_lowering` on the ordinary path, and `crates/tiler-compiler/src/governed.rs` registers four index-access providers.

Each correction is marked as an after-the-fact correction naming what the text previously read, rather than silently rewritten, so a reader tracing the claim can see what changed and why. Both were true when written and were falsified by `wire-capability-and-refinement-into-compile-path`.

**The surrounding claims survive and were preserved deliberately.** At the Milestone 6 site, the reason a contraction still cannot be presented is *not* that the authorities are unwired — it is that no capability exists for a contraction occurrence, so resolution fails closed rather than lowering one. That is a stronger statement than the one it replaces. In the support matrix, the operation-family rungs are unaffected: capability resolution constrains which programs compile, not how far any operation family was built.

This is the third stale-claim class found today, after the `select_alternative` citation and three rotted verbatim quotations of `docs/compiler/optimizer.md`. `detect-stale-cross-document-quotations` owns the mechanical question of whether the gate can catch the quotation form; this one was a paraphrase of implementation state rather than a quotation, so that check would not have caught it.
