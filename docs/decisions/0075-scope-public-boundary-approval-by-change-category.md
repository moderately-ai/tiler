---
schema: "tiler-doc/v1"
id: "ADR-0075"
kind: "decision"
title: "Scope public-boundary approval by change category"
topics: ["governance", "api", "process", "review"]
catalog_group: "documentation-governance"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.architecture"]
evidence: ["tiler.research.workspace.prototype-crate-layout-and-msrv", "tiler.research.extensions.operation-extension-surface"]
depends_on: ["ADR-0074"]
ticket: "draft-public-boundary-approval-policy-adr"
---

# 0075: Scope public-boundary approval by change category

**Status:** accepted

## Context

**Fact — the standing obligation is unbounded.** `AGENTS.md` states that "Tom must review key public crate, module, trait, type, and call-site boundaries before they are accepted or merged." The sentence names five nouns and qualifies all of them with *key*, which the working contract never defines. The boundary is therefore resolved at dispatch time by whoever is holding the change, and the two readings that keep a worker safe — ask about everything, or ask about nothing that is not obviously public — are both consistent with the text. The cheap default is to ask, which spends a round trip on changes nobody would have altered.

**Fact — provenance of this decision.** Tom was asked this boundary directly on 2026-07-24 and chose the split recorded below over a tighter variant (also bring him every new public *type*) and a looser one (let the coordinator promote `pub(crate)` to `pub` unaided). The choice is recorded in `tickets/draft-public-boundary-approval-policy-adr.md` at commit `083ca5a`. This record writes down a decision that was made; it is not a survey of open options.

**Fact — the dependency is satisfied.** ADR 0074 was accepted at commit `4b1d828` on the same day. The no-approval half of this policy names "a record that conforms to the conventions", and before 0074 that phrase named nothing checkable. It now names seven stated conventions, each written so that conformance can be established by reading the item rather than by consulting a reviewer's memory of sibling modules.

**Claim from the originating ticket, not independently verified here.** The ticket states that over the period the compiler authorities landed, three changes went to Tom — a new crate, a new public module, and one public method plus its error type — that all three were approved essentially as designed, that four further changes never needed him because they landed as `pub(crate)` private drafts, and that the single substantive review catch was a missing `#[non_exhaustive]`. Ticket-to-change attribution is not recoverable from the tree, so this record does not adopt those counts. What the tree does show at `dc9990d` is the resulting shape: `crates/tiler-compiler/src/lib.rs` declares exactly two public modules — `capability` and `legality`, each of which documents itself as a reviewed *draft* boundary — and keeps `cover`, `explain`, `feasibility`, `frontier`, `fusion_legality`, and `selection` as private `mod` declarations whose items are `pub(crate)` without a single bare `pub` item among them. All six open with the module-level `#![allow(dead_code, reason = "…")]` that ADR 0074's convention 7 prescribes, and each reason names what the surface reserves and why it is not yet reachable; `fusion_legality`, `cover`, `frontier`, and `selection` additionally record in their module documentation that the surface stands "until Tom accepts the exact interface". That is six authority modules a coordinator could land without spending any public-surface commitment, staged in exactly the form the conventions prescribe. Six modules and four changes are not comparable quantities, and nothing below depends on either number.

**Inference.** Asking about a change that lands entirely behind `pub(crate)` cannot protect a public surface, because no public surface changes. The property worth gating on is the *category* of the change, which is mechanically visible in the diff — a new crate manifest, a new `pub mod`, a `pub trait` item, an altered public signature, a changed visibility keyword — rather than the reviewer's estimate of how important the change is.

### The first wording was narrower than the practice it was calibrated against

**Fact.** As first put to Tom, the always-ask list opened with "a new crate" and said nothing about introducing a new public module. A new `pub mod` is not a new crate, not a new public trait, not a change to an existing public signature, and not a promotion — its items are new rather than moved out of `pub(crate)` — so the literal wording exempted it.

**Fact.** Two such surfaces exist. `crates/tiler-ir/src/lib.rs` declares `pub mod schedule` over six files totalling 1,785 lines at `dc9990d`, and on branch `tkt/prototype-structured-kir-slice` it gains `pub mod kernel` over eight files totalling 4,616 lines — 3,436 excluding that module's 1,180-line test module — carrying a public builder, a verified product, an identity type, read views, and a large typed vocabulary. **Reported, not verifiable from the tree:** both went to Tom for review under the prior judgement-based rule. Review-to-change attribution is not recoverable from repository artifacts, so that half is recorded as reported.

**Inference.** The first wording was therefore looser than the practice it was meant to describe, which is the one direction a written rule must not drift: replacing judgement with a rule is only an improvement if the rule covers what the judgement covered. The omission reads as a drafting artifact rather than a considered narrowing, because `AGENTS.md` already lists "module" alongside crate, trait, type, and call site, and this record sharpens that sentence rather than replacing its noun list.

**Fact.** Tom was asked and accepted the reformulation, extending the category to *a new publicly reachable namespace* — a crate, or a `pub mod` in a crate root or an already-public module. He declined the alternative "a new public module with a substantial surface", because "substantial" reintroduces exactly the judgement term that makes the existing `AGENTS.md` sentence ambiguous. The accepted wording subsumes the crate category he originally chose rather than contradicting it, so the decision is extended along its own grain.

### The compatibility premise, considered and rejected

**Premise as the originating ticket states it.** The ticket justifies the always-ask items on compatibility grounds: a new public trait "is a durable compatibility commitment", and promotion to `pub` is "the moment a surface becomes externally load-bearing".

**Fact — Tiler has no external consumer and cannot currently acquire one.** The workspace `[workspace.package]` table sets `version = "0.0.0"` and `publish = false`, and all six library crates plus both prototype binaries inherit both through `publish.workspace = true`. No crate in this workspace is publishable. `docs/architecture.md` records the crate set as "an unstable prototype packaging profile, not the final published crate set"; `docs/vision.md` is still `contract_status: "proposed"`; and `tiler-compiler` does not export its own entry point at all, pinning `pipeline::compile` with a compile-time reachability assertion because the reviewed public facade does not exist yet.

**Fact — Tom rejected the compatibility framing on 2026-07-24.** Tiler is incomplete and nowhere near an alpha, so there is currently no such thing as a breaking change. Every consumer of every public item in this workspace is inside this workspace; reversing a public surface costs one internal edit, and `cargo check` enumerates the affected call sites exhaustively.

**Inference — the always-ask list survives on a different justification.** That justification is *design visibility*: Tom sees the public surface as it forms, while it is still cheap to reshape. The cost being controlled is not the cost of reversing a surface, which is near zero, but the cost of having built on the wrong shape — the dependent work that accumulates between a surface taking a shape and anyone noticing that the shape is wrong. Within that frame, a public namespace and a trait are still the two largest-grained architectural commitments available even though both are reversible. A crate fixes a dependency edge and a verifier-ownership boundary of exactly the kind ADR 0070 consolidated and `docs/architecture.md` records, and a public module fixes a region of the surface that later items are added to by default; in both cases everything added afterwards is placed relative to the structure, which is why the structure is the thing worth seeing early. A trait fixes a participation model, and each implementor written against it encodes that model further; `tiler.research.extensions.operation-extension-surface` is the standing evidence that a public extension seam is a statement about versioned identity, validation, capability-based participation, and what a transformation may assume when the seam is absent — none of which becomes cheaper to get wrong merely because it is reversible.

**Consequence for this record.** Nothing below is justified by compatibility, downstream breakage, semantic versioning, or a stability commitment, and a future reader who finds such an argument attached to this policy should treat it as reintroduced rather than inherited. When Tiler does acquire an external consumer, compatibility becomes an additional reason for the same list and a reason to re-derive its calibration. It is not the reason today.

## Decision

**Proposal.** Approval is decided by the category of the change, read from the diff, and not by an estimate of the change's importance.

**Always requires Tom's review before merge.**

- **A new publicly reachable namespace** — a new crate, or a new `pub mod` in a crate root or in an already-public module. A crate fixes a dependency edge and a verifier-ownership boundary in the profile `docs/architecture.md` records, and every crate admitted afterwards is placed relative to it; `tiler.research.workspace.prototype-crate-layout-and-msrv` is the evidence that the crate set is the mechanical enforcement of Tiler's layer separation rather than a packaging convenience. A new public module is the same act one level down: it names a region of the public surface that later items are added to by default. The test is mechanical — does the diff add a workspace member, or a `pub mod` reachable from a crate root — so no participant has to estimate how important the namespace is.
- **A new public trait** — an extension seam that something else implements. The trait fixes a participation model before its implementors exist, and each implementor written against it makes the model harder to see as a choice. Which traits are *intended* to become public seams at maturity is a separate and still unrecorded question owned by `draft-public-extension-seam-ownership-adr`; this policy says only that creating one is Tom's to review.
- **A breaking change to an existing public signature.** Breaking here means source-breaking for the in-workspace call sites, which is what the term can mean while nothing is published.
- **Promoting a module or type from `pub(crate)` to `pub`.** This is the scheduled moment at which a surface's shape becomes visible as a whole, and it is the last such moment that costs a reviewer one diff.

**The accepted cost of the namespace category.** A two-item `pub mod` now requires review exactly as a crate does. That is a real cost and it was accepted rather than overlooked: such modules are expected to be rare, and the only wording that avoids the cost — "a new public module with a *substantial* surface" — reintroduces precisely the judgement term that makes `AGENTS.md`'s "key public … boundaries" ambiguous today. Paying review on an occasional trivial module is the cheaper of the two, because the ambiguity it avoids is paid on every change rather than on the rare small one.

**No approval required; the coordinator may merge.**

- **A new compiler-internal authority introduced as a `pub(crate)` draft**, in the form ADR 0074's convention 7 already prescribes: a private `mod` whose items are `pub(crate)`, carrying the module-level `#![allow(dead_code, reason = "…")]` whose reason names what the surface reserves, and adding nothing to its crate's exports. This record defers to that convention rather than restating a second definition of the same shape, so the category cannot drift away from the rule it depends on.
- **Additive growth of a type already marked `#[non_exhaustive]`**, which is the growth that attribute exists to make additive.
- **A new public error, provenance, or identity *record* that conforms to ADR 0074.** Conformance is the load-bearing word: the conventions state what such a record must look like precisely enough that a deviation is visible by reading the item, so a conforming record carries no design choice a reviewer would have changed.
- **Tests.**
- **Documentation.**

**The coordinator's terminal-merge authority is conditional.** It exists only when all four of the following hold: `uv run --locked python scripts/check_repository.py` is green; `ticketsplease guard` reports no scope escape; the change stays inside its ticket's declared scopes; and the coordinator has reviewed the actual diff rather than an agent's summary of it. If any of the four fails, the change returns to Tom regardless of its category. The no-approval list is a statement about which categories are cheap to trust *given* those checks. It is never a statement that they may be skipped, and a category match does not substitute for any of them.

**Phase scope.** This policy is calibrated to Tiler as it is: pre-alpha, incomplete, with no external consumer and no publishable crate.

**Reconsideration trigger.** Revisit the calibration when Tiler acquires an external consumer, when any crate becomes publishable, or when a release is approached. At that point compatibility becomes a real cost rather than a rejected premise, and the split must be re-derived rather than inherited: a permissive autonomous half would then be wrong for precisely the reason it is acceptable now.

**What this record does not change.** `AGENTS.md` remains the operative working contract and still states the unbounded "key public crate, module, trait, type, and call-site boundaries" obligation. Propagating this policy into it is a deliberate follow-up conditional on acceptance, so that a proposed ADR never becomes the operative rule by default.

## Consequences

- A change's category is readable from its diff before the work is dispatched, so the approval question is answered when the ticket is written instead of when the branch is ready.
- Tom's attention concentrates on the acts that create or reshape public structure — opening a namespace, opening a participation seam, changing an existing signature, promoting a draft — rather than on conformance that a written convention already covers and a reader can check.
- The coordinator gains no discretion. Every no-approval category remains gated on four objective checks, and a failure in any of them returns the change regardless of category, so the policy cannot be used to argue that a red gate is acceptable for a documentation-only change.
- The policy is deliberately blind to size and risk. A three-thousand-line internal authority merges without approval while a five-line `pub trait` or a two-item `pub mod` does not. That is intended — size is not the property being controlled, and every wording that would make it the property reintroduces a judgement term — so a reviewer who wants to see a large internal change should ask for it explicitly rather than expect this policy to route it.
- Because nothing here rests on compatibility, none of these consequences changes when a public surface is later reshaped. They change when Tiler acquires an external consumer, which is the recorded trigger.
- The no-approval half is only as safe as ADR 0074. If those conventions were weakened or withdrawn, "conforms to the conventions" would stop naming anything checkable and the record category would have to return to Tom.

## Implementation boundary

**This record changes nothing operative.** `implementation_status` is `not-started` because no governed contract states this split today and `AGENTS.md` still states the unbounded obligation it sharpens. That is the intended state for a proposed policy.

The four gates the policy conditions on already exist and are already run. `scripts/check_repository.py` is the canonical complete contributor and CI gate; `ticketsplease guard` is the scope check; and `docs/work-tracking.md` already requires the ticket's tests, `tkt lint`, `git diff --check`, and `tkt guard` against the true base before integration. What this record adds is the requirement to read the actual diff rather than an agent's summary, which no gate can check, and the conditional-authority clause that makes all four preconditions of merge authority rather than items on a recommended checklist. That distinction — an existing checklist versus a stated precondition on authority — is the only genuinely new mechanism here, and it is unimplemented until the policy is accepted and propagated.

## Open questions

These are recorded unresolved on purpose. None is settled by this ADR.

- **Where this policy becomes normative.** `applies_to` names `tiler.contract.architecture` because the two structural always-ask items — a new publicly reachable namespace, and promoting a module out of crate-private draft — change what its "Component ownership" and "Accepted prototype packaging profile" sections record, and because ADR 0074 routed the same crate-private staging rule to the same contract. That edge under-describes the destination and must not be read as a claim that `docs/architecture.md` will state the whole policy. The policy's real operative home is `AGENTS.md`, which is not a governed record at all (`scripts/docs.py` governs the root `README.md`, `docs/**`, and `spikes/**/README.md`), and secondarily `docs/work-tracking.md`, which is a portal and therefore not a legal `applies_to` target. The metadata schema has no relation that expresses "this decision governs the working contract", and `catalog_group: "documentation-governance"` is likewise the closest available bucket for a project-process decision rather than an exact fit. Whether the schema should grow such an edge, or whether the working contract should become a governed record, is unowned. The parallel propagation problem for ADR 0074 is owned by `propagate-accepted-api-conventions-into-governed-contracts`.
- **Whether adding a variant to a recognized `#[non_exhaustive]` enum is additive growth.** ADR 0074 records that marking a recognized enum non-exhaustive forces its consumers to grow an explicit reject-unknown arm. Adding a variant to such an enum still compiles at every consumer outside the defining crate, because `#[non_exhaustive]` obliged each of them to carry that arm, but it silently changes their behaviour from accept to reject for the new variant; inside the defining crate the attribute has no effect and exhaustive matches fail to compile, which is the intended fail-closed half. The split above does not say whether the cross-crate half is the "additive `#[non_exhaustive]` growth" a coordinator may merge or a semantic change that returns to Tom, and the honest answer probably depends on whether the new variant is reachable by the paths those consumers guard. A dedicated follow-up ticket owns this question; it is referenced here as unresolved and is deliberately not settled by this record.
- **Whether adding a defaulted method to an existing public trait is covered.** It is neither a new public trait nor a breaking change to an existing public signature, yet it extends a participation model in the way the trait rule exists to make visible. The same question applies to adding a supertrait bound satisfied by every current implementor.

## Alternatives considered

**The tighter variant — also bring Tom every new public type.** Declined by Tom on 2026-07-24. The reasoning that supports declining it is ADR 0074's: conventions 1, 2, and 5 make a public error, provenance, or identity record checkable by reading it, so a conforming record contains no design choice a reviewer would change, and the tighter variant would spend a round trip per record confirming conformance that a written convention already states. This argument is contingent on ADR 0074 remaining accepted, which is why the record category is stated as conditional on conformance rather than as a blanket exemption for records.

**The looser variant — let the coordinator promote `pub(crate)` to `pub` unaided.** Declined by Tom on the same day. The originating ticket attributes the refusal to promotion being "the moment a surface becomes externally load-bearing", which is the premise rejected above and does not support the conclusion: nothing in this workspace is externally load-bearing, and promotion commits nothing that one internal edit could not undo. The reasoning that does support it is scheduling. Promotion is the last cheap and reliably scheduled moment at which a surface's shape is visible as a whole, before other work is written against it, and the reviewer's cost at that moment is one diff. This is the alternative most likely to become correct first: `docs/correctness-and-testing.md` records that the optimizer conformance owner must drive an external operation through the ordinary capability and refinement path before the public compiler facade is accepted, and once a seam has been exercised end to end the promotion decision has evidence behind it instead of judgement.

**Leaving `AGENTS.md`'s "key ... boundaries" sentence as the whole rule.** This is the status quo, and it is not wrong — only unbounded. It resolves to whatever the person holding the change believes *key* means, which is the same failure mode ADR 0074 was written to stop for API shape: a rule that propagates by imitation and gives a reviewer nothing to cite.

**Making the coordinator's terminal authority unconditional.** Rejected. The conditional form is what makes the no-approval list safe: those categories are cheap to trust only because a green complete gate, a clean guard, scope conformance, and a full read of the diff have already established that the change is what it claims to be. An unconditional authority would make the category list the whole check, and a miscategorized change would then merge with nothing behind it.
