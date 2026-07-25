---
id: correct-adr-0078-public-module-and-entry-point-facts
title: Correct ADR 0078's stale lib.rs and entry-point facts
status: in-progress
priority: p2
dependencies: []
related: [propagate-extension-seam-classification-into-governed-contracts, prototype-public-compiler-api]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contracts, extensions]
claimed_from: todo
assignee: agent-adr0078
lease_expires_at: 1785041166
---
ADR 0078 was drafted against base `412ceae` and accepted unamended on 2026-07-25. Four of its source facts about `crates/tiler-compiler/src/lib.rs` were true at that base and are false at `c173ed3`, because `pub mod session` landed in between and Tom approved it on the same day the record was accepted. The classification the record decides is unaffected; what is stale is the evidence it cites for item 4, which is exactly the item a reader consults to learn whether the lowering seam is reachable.

Found while propagating the record into the governed contracts under [`propagate-extension-seam-classification-into-governed-contracts`](propagate-extension-seam-classification-into-governed-contracts.md), which corrected the same claims where the contracts stated them. It could not correct the record itself: `docs/decisions/` is `contracts/decisions` and that ticket declares only `contracts/foundation`.

**Fact — reproduce each in one line at `c173ed3`.** `grep -n "^pub mod\|^mod " crates/tiler-compiler/src/lib.rs` lists three public modules — `capability`, `legality`, `session` — and sixteen private ones including `honourability`. `grep -n "pub fn compile_governed" crates/tiler-compiler/src/session.rs` returns the public compile entry point. `grep -n "pub(crate) struct CompilationRequest\|pub(crate) capabilities" crates/tiler-compiler/src/request.rs` shows the request and its installed-capability field are still crate-private.

**What is stale, item by item.**

- **Item 3, the `pub`-keyword paragraph.** "`crates/tiler-compiler/src/lib.rs` declares exactly two public modules, `capability` and `legality`, and keeps [sixteen named modules] private." Three are public and the private list omits `honourability`. This is the count-versus-invariant failure `AGENTS.md` names: the sentence goes stale on any module addition, and what it exists to establish — that a `pub` keyword is neither necessary nor sufficient for an intended seam — needs no enumeration at all.
- **Item 4, the Fact.** "`crates/tiler-compiler/src/lib.rs` exports no entry point." It exports `session::compile_governed`. The rest of the sentence — no `pub use`, `pipeline::compile` pinned behind a compile-time reachability assertion — still holds.
- **Item 4, the Inference.** An out-of-crate consumer "has no public way to install it, to reach `compile`, or to reuse the governed capabilities it would compose beside." The middle clause is false; the first and third are true and are what the item's conclusion actually rests on.
- **Item 4, the Proposal, and open question 4.** Both close on `prototype-public-compiler-api` landing "the reviewed compiler facade". The facade landed and Tom approved `pub mod session` on 2026-07-25, so the stated trigger has fired — and installation is still unreachable, so the question it was meant to close is still live. The trigger needs sharpening to the half that remains: a public path that lets an out-of-crate caller supply its own `FrozenLoweringCapabilityRegistry` to a compilation, which is the request half that ticket still carries as its stated gap.

**What must not change.** The classification, the seam inventory, the maturity rungs, item 1's admission test, item 3's negative space, item 5's deferral, and item 6's internal list are all decided and unaffected — nothing here reopens an accepted decision. The item-4 asymmetry itself also survives: registration is public, installation is not, and the corrected evidence supports that conclusion more directly than the stale evidence did.

**Prefer an invariant to a snapshot.** Where the record needs to say what `lib.rs` exposes, state the property that decides the point rather than a list that a later module invalidates. The rung table's `412ceae` pin is a measurement tied to a commit and is correct as written; the prose facts are not pinned and should not acquire a pin instead of an invariant.

Run `uv run --locked python scripts/docs.py render` and `uv run --locked python scripts/check_repository.py` before completion.
