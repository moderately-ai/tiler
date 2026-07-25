---
id: correct-adr-0078-public-module-and-entry-point-facts
title: Correct ADR 0078's stale lib.rs and entry-point facts
status: done
priority: p2
dependencies: []
related: [propagate-extension-seam-classification-into-governed-contracts, prototype-public-compiler-api]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contracts, extensions]
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

## Outcome

Every source fact was re-derived at base `0f62737` rather than taken from this ticket. Six corrections landed in `docs/decisions/0078-name-the-intended-public-extension-seams.md`; two of them this ticket did not name, and one claim it did make about the record's own layout was wrong.

### The four this ticket named

1. **The module enumeration** — corrected in the Context section, not item 3. It read "declares exactly two public modules, `capability` and `legality`, and keeps [fifteen named modules] private"; three are public and the enumeration omitted `honourability`. Replaced by the invariant it existed to establish — `frontier` is a private module, so `frontier::PhysicalImplementationProvider` is `pub(crate)` while being cross-crate-ready — with `grep -n "^pub mod\|^mod " crates/tiler-compiler/src/lib.rs` given as the check and the count explicitly disclaimed as the thing that matters.
2. **Item 4's Fact** — "exports no entry point" is false; `pub mod session` exposes `compile_governed(&SemanticProgram, NumericalContract)`. Rewritten as "exports a compile entry point and no way to configure it", which is the asymmetry the item is about, with both reproduction greps stated. The `pub use` and reachability-assertion clauses were verified still true and kept.
3. **Item 4's Inference** — the middle clause "no public way … to reach `compile`" is false. Rewritten to separate reaching the compiler from configuring it and to say that only the first was promoted, which leaves the item's conclusion resting on the two clauses that remain true.
4. **Item 4's Proposal and open question 4** — the trigger fired without closing anything. Both now record that as a labelled **Fact**, and the trigger is restated as the half that remains: a public path admitting a caller-supplied `FrozenLoweringCapabilityRegistry`. The open question also notes that the cost of leaving `capability` public *rose* when `session` landed, because a reader who can now compile out of crate has one more reason to expect installation to work.

### Two this ticket did not name

5. **Two line-number citations in the Context section had drifted.** `pipeline.rs:827` and `pipeline.rs:924` were exact at `412ceae` — `git show 412ceae:crates/tiler-compiler/src/pipeline.rs | sed -n '827p;924p'` returns the `resolve_lowering` call and the one-element provider array — and are now lines 913 and 1010. This is the same snapshot-versus-invariant failure as the module count, so both were replaced by greps that return exactly one site each: `grep -n "resolve_lowering(" crates/tiler-compiler/src/pipeline.rs` (the `use` line has no parenthesis) and `grep -n "dyn PhysicalImplementationProvider; 1" crates/tiler-compiler/src/pipeline.rs`.
6. **The Implementation boundary asserted its own propagation had not happened.** It read "until it lands, neither contract states the classification", and [`propagate-extension-seam-classification-into-governed-contracts`](propagate-extension-seam-classification-into-governed-contracts.md) is `done`. `grep -n "0078" docs/operation-extensions.md docs/architecture.md` returns four citations. Corrected to state that acceptance released it and it landed, naming what each contract now carries; the conditional-on-acceptance rationale is preserved, because it explains why the propagation waited and is not itself stale. `implementation_status` stays `partial`, which items 4 and 5 still support, and the sentence justifying it was corrected to rest on those two rather than on the propagation.

### One claim in this ticket that was wrong

The ticket located the module enumeration in "item 3, the `pub`-keyword paragraph". It is in the **Context** section's *"Fact — the live inconsistency the ticket named is still live"*. Item 3's actual `pub`-keyword paragraph enumerates no modules and every one of its four claims was verified still true: `capability` is `pub` with nothing public able to install what it builds, `frontier::PhysicalImplementationProvider` is `pub(crate)` at `frontier.rs:574`, `ValueTypeMarker` is `pub` and unsealed at `semantic/registry.rs:85`, and `ShapeEvidence` and `ShapePredicate` are `pub` and `sealed::Sealed`. The private-module count is sixteen, not the fifteen the record listed; the ticket's "sixteen private ones" is right about the tree and was describing a list the record never contained in that form.

### Verified unchanged rather than assumed

Everything this ticket placed out of bounds was re-checked rather than skipped, and all of it holds at `0f62737`: all five seam trait paths in item 2's table; `crates/tiler-compiler/Cargo.toml` declaring `tiler-ir` under `[dependencies]` and `tiler-reference` under `[dev-dependencies]` and nothing else; `resolve_scalar_lowering` reachable only from its definition and `#[cfg(test)]` callers; `lowering.rs` resolving `resolve_index_access` alone; item 3's `LoweringCapabilityKey` four-field shape against `LoweringSelector`'s three; `ReservedProposalSeam` on the three `ProposalBody` variants; `FusionEvidenceClass`'s five unmerged classes; `proof_budget_stop`; the conformance block still last in `pipeline.rs` with `acme/external-multiply-lowering` at provider revision 3 and capability revision 7; and item 5's fact that `enumerate_frontier` has exactly one non-test call site with one in-crate `GovernedPhysicalProvider`. The rung table's `412ceae` pin was left alone, and nothing about the rungs moved anyway.

### The accepted-status hazard, checked by reading

`AGENTS.md` notes that a disclosure required while a decision is proposed becomes wrong on acceptance and that the gate stops applying at that moment. Read for it in both directions. The record's own status paragraph is a statement *about* acceptance and stays true. The one sentence whose truth depended on the pre-acceptance state is correction 6, and it was the propagation clause rather than a proposal disclosure. No governed contract carries a stale proposed-status disclosure about ADR 0078; `grep -rn "0078" docs/` outside `docs/decisions/0078-*` returns the two generated catalog lines in `docs/decisions/README.md`, the four contract citations, and ticket bodies.

### Split

[`correct-artifact-abi-reconstruction-ownership`](correct-artifact-abi-reconstruction-ownership.md). `docs/artifact-abi.md` names a `done` ticket as the present owner of the reconstruction question and records it as open in "Where the implemented profile is narrower than this contract". That file is `contracts/artifacts` and this ticket declares only `contracts/decisions`. ADR 0078 was checked for the same staleness and does not carry it: `grep -n -i "artifact\|decode\|envelope" docs/decisions/0078-name-the-intended-public-extension-seams.md` returns only "artifact plan", the compiler-side plan record, so nothing about the artifact ABI, the envelope codec, the proof sidecar, or the crate count appears in this record at all.

### No architectural consequence

The corrections change what a reader is *told*, not what a reader is *permitted* to do. Item 4's conclusion is unchanged — registration public, installation not — and the permission to call `session::compile_governed` was granted by Tom's approval under `prototype-public-compiler-api`, recorded there and in the governed contracts. This record is not the authority that granted it and does not become one by describing it.

`uv run --locked python scripts/docs.py render` and `uv run --locked python scripts/check_repository.py` pass.
