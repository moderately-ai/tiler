---
id: retire-the-l4-zero-synchronization-ground-where-other-records-restate-it
title: Retire the L4 zero-synchronization ground where other records restate it
status: in-progress
priority: p2
dependencies: [reconcile-the-l4-records-self-contradicting-softmax-elimination-row]
related: []
scopes: [research/scheduling, research/program-planning, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, doc-drift, scheduling]
claimed_from: todo
assignee: worker-zerosync
lease_expires_at: 1787450840
---
## User-visible outcome

No record still eliminates threadgroup-cooperative softmax on a zero-synchronization ground, and none attributes that ground to the L4 record, which has withdrawn it.

## Why this exists

Filed 2026-08-22 when `reconcile-the-l4-records-self-contradicting-softmax-elimination-row` **withdrew** that elimination rather than re-grounding it. Both halves of the ground fell, and the coordinator verified the decisive ones:

- **The barrier is landed and Metal-proven.** `crates/tiler-build/src/metal_declaration.rs` declares the workgroup `ControlBarrier` subject at `SynchronizationSupport::Realized` from a production call site.
- **The surviving half is not a discriminator.** `tiler::softmax-f32@1` is registered and recognized; what it lacks is an installed lowering, so a softmax refuses under `UnsupportedCapability` rule `accuracy.elementary.no-installed-realization` — reached *before* a schedule is chosen, so it falls on every candidate topology alike and eliminates none relative to the others.

**The problem this ticket fixes is attribution.** Three documents restate the retired ground and **one quotes the L4 record by name**, so repairing L4 alone leaves the claim alive elsewhere *citing an authority that has withdrawn it*. Reported by that lane, unverified by the coordinator: `two-level-subgroup-workgroup-reduction.md` (2 sites), `multi-round-two-level-reduction-composition.md` (1), `autoregressive-state-and-kv-cache.md` (1).

Also reported: three stale claims in `crates/`, and one in the **`done`** ticket `implement-the-single-workgroup-synchronized-reduction-strategy` saying `metal_declaration.rs` declares no synchronization row — which the production `Realized` row falsifies in both halves. `make citations` reads only open tickets, so the gate has never seen that one.

## Required work

- Re-audit every site at your base with a per-Fact verdict; **re-derive the census yourself** and say which spellings you searched for and why that set is complete. A census is only as complete as its vocabulary — that is how a closed ticket shut green over live sites this week.
- Withdraw the ground where it appears, following L4's own precedent: the in-convention move is **withdrawal**, as the 2026-08-10 lane did three rows up in the same table for a structurally identical reason. Do not invent a replacement ground.
- Repair the attribution first where a record cites L4 by name — a claim borrowing a withdrawn authority is worse than a claim standing alone.
- **Preserve retired wording in dated corrections**; counts cannot shrink.
- The `crates/` sites and the terminal ticket are **out of this ticket's scopes** — report them rather than widening.

## A hazard this lane hit, worth inheriting

Its first draft asserted two false things about softmax, taken from a **stale doc comment** in `crates/tiler-compiler/src/request/recognize.rs` reading `carries no law at all`. An independent source sweep caught it before commit. **Do not source a claim from a doc comment without checking the code it describes** — that comment is still there.

## Non-goals

Re-deciding the softmax schedule set, which the L4 text explicitly leaves open; editing `crates/`; and the flash-class citation repairs, which are their own ticket.

## Closes when

No live record eliminates threadgroup-cooperative softmax on a zero-synchronization ground, no record attributes that ground to L4, every withdrawal preserves what it replaced, and the out-of-scope sites are reported with their owners named.

## Fact audit at base `3e6cc78ea56b54518e1b22c1fe076e523e201a1a`

Every Fact this ticket and its dispatching brief carried, re-read at this base. Four verified, two imprecise, none false.

- **Verified — the barrier is landed, target-proven, and reached from production.** `crates/tiler-build/src/metal_declaration.rs` declares the workgroup-scoped `ControlBarrier` subject at `SynchronizationSupport::Realized` in the file-scope `const FIRST_MACOS_APPLE9: LedgerRows`, which is not test-gated (the file's only `#[cfg(test)] mod tests` begins at line 1602, after it). It reaches a target profile through the non-test `BoundMetalCompileDeclaration::declare`, whose `declare_synchronization_realization` call passes those rows; the public `first_macos_apple9()` constructor is re-exported from `crates/tiler-build/src/lib.rs` and called from `crates/tiler-macros/src/aot.rs` in `pub(crate) fn deliver`, which the proc-macro expansion invokes. Production call site confirmed, not merely a public constructor.
- **Verified — and stronger than "the barrier exists".** The kernel verifier's barrier rule is *conditional* exactly as the L4 correction states. `verify_synchronization` in `crates/tiler-ir/src/kernel/verify.rs` opens by binding `cooperative_tile(&schedule.schedule.reduction)`; only on `None` does a body barrier return `KernelDiagnostic::UnexpectedSynchronization`. `cooperative_tile` (`crates/tiler-ir/src/schedule/model.rs`) returns `Some` for `CooperativeWorkgroup` and `CooperativeContraction` and `None` for the other five variants. So a schedule carrying a cooperative tile is verified rather than refused, and the verifier never consults a subgroup width or subgroup target fact at this gate.
- **Verified — the surviving half discriminates nothing.** `accuracy.elementary.no-installed-realization` is the rule asserted by `crates/tiler-compiler/tests/softmax_recognizer_boundary.rs`, and the refusal is reached before a schedule is chosen, so it falls on every candidate topology alike.
- **Verified — the reported site counts are exact.** `two-level-subgroup-workgroup-reduction.md` 2, `multi-round-two-level-reduction-composition.md` 1, `autoregressive-state-and-kv-cache.md` 1. Four sites in three documents, and the re-derived census below found no fifth.
- **Imprecise — "one quotes the L4 record by name" undercounts the attribution problem by three.** Three of the four sites attribute to L4, not one. `two-level:337` quotes the withdrawn sentence verbatim and introduces it with "The attention vertical records that"; `two-level:426` asserts "The attention vertical's admissibility statement is unchanged by this record"; `autoregressive:219` says "unchanged from L4". Only `multi-round:282` carries no direct attribution — it inherits the clause from the two-level record, so its attribution is one hop away rather than absent. This is the dangerous direction: the brief's count would have left two attributions standing.
- **Imprecise — `make citations` reads three populations, not one.** The ticket says it "reads only open tickets". `AGENTS.md` at this base says it resolves local markdown links in "an open ticket, a live document, or a retained spike record", the `spikes/**` population having been added on 2026-08-22. The ticket's *conclusion* is unaffected and correct: a `done` ticket is in none of those populations, so the terminal ticket stays invisible to the gate and needs hand-verification.
- **Verified — the stale doc comment is still there.** `crates/tiler-compiler/src/request/recognize.rs:57` still reads `carries no law at all`. I did not source any claim from it; see the out-of-scope report below.

## The re-derived census, and why its vocabulary is complete

The census was rebuilt from scratch rather than inherited. The retired ground is a conjunction of three separable things, and any restatement must spell at least one of them, so the vocabulary is organized by axis rather than by phrase. Counts are of **matching lines**, and every anchor below was grepped against the file it names before being used.

| Axis | Spellings searched (case-insensitive, `grep -rniE`) |
| --- | --- |
| A — the profile's name | `zero.synchroni` — one regex covering `zero-synchronization`, `zero synchronization`, and any other single separator |
| B — barrier absence | `no barrier`, `admits no`, `barrier is (un)?available`, `without a barrier`, `barrier-free` |
| C — the topology bound | `threadgroup.cooperative`, `threadgroup.wide`, `spans? more than one SIMD`, `multi.SIMD.group`, `wider than (one\|a) SIMD` |
| D — attribution, independent of ground | `first-attention-program-vertical`, `attention vertical`, `\bL4\b` |
| E — residual phrasings | `synchronization construct`, `anything wider`, `once a barrier is admitted`, `barrier is admitted`, `no synchronization`, `strategy 4`, `schedule profile` |

**Why this set is complete.** Axes A, B, and C exhaust the ground's own content: a record cannot restate "threadgroup-cooperative softmax is eliminated because no barrier is admitted under the zero-synchronization profile" without naming the profile, the barrier absence, or the topology. Axis D is the load-bearing addition and is deliberately *independent* of A–C: a site that restated the ground in wording none of A, B, C, or E anticipated would still be caught if it cited L4, which is the failure mode this ticket exists for. Axis E was added after A–D as an adversarial pass for phrasings that carry the ground without its vocabulary. E returned no site that A–D had not already returned, which is the evidence that the vocabulary had converged rather than merely stopped.

**What it found, over `docs/**` and `spikes/**`, excluding the frozen `docs/research/documentation/ticket-audit-2026-08-10/` corpus:** four sites in three documents, matching the reported counts exactly. Axis B alone returns roughly 120 lines and is nearly all noise — "admits no boolean dtype", "admits no CFG" — which is why it was read rather than counted. `docs/artifact-abi.md` names *nonzero* synchronization as current state and is clean. `docs/ir.md` states the implemented profile declares and verifies its synchronization and is current. `docs/research/scheduling/subgroup-execution-tier.md` says a subgroup form has "no workgroup memory, no barrier, no visibility edge", which is a positive claim about what that form buys, not a restatement of the ground; it is clean.

**The negative result worth recording:** no site outside these three documents restates the ground, and no site inside them was missed by the reported census. The lane that reported it was right on the sites and wrong only on how many carried an attribution.

## Repairs, attribution first

Four dated corrections, each placed directly beneath the text it corrects so a reader landing on the claim meets the correction without leaving the section. Every retired phrase is left in place and marked superseded; all eight tracked phrases still return 1.

- **`two-level-subgroup-workgroup-reduction.md`, the `none of the three is admissible` site.** Attribution repaired first: the quoted sentence is L4's, L4 withdrew it, and the quotation is retained only so a reader can see what was borrowed. The verdict then narrows from three forms to **two**, on a ground this record already derives rather than an invented one. B's exclusion is withdrawn — `ReductionTopology::CooperativeWorkgroup` is landed and the barrier rule is conditional on tile presence. A and C stay unavailable because `ReductionTopology` carries exactly seven variants and none is a subgroup combine or this composition, which is this record's own "It is a third `ReductionTopology` variant" conclusion and ADR 0094's sibling reservation, both still reservations. The block-level ceiling is separately identified as the uninstalled softmax lowering, which falls on all three alike.
- **`two-level-subgroup-workgroup-reduction.md`, the measurement-boundary bullet.** Two of its four clauses stand and are re-verified, two do not. The zero-synchronization clause is withdrawn, and its comparison "exactly as the workgroup tile is" is flagged as now pointing at a landed, verified, target-proven construct rather than an excluded one. The attribution clause is withdrawn at its source.
- **`multi-round-two-level-reduction-composition.md`.** Same repair, plus the consequence specific to this record: §5 prices the composition in **barriers**, so the `2R − 1` count is a derived count of a construct that exists and is target-proven rather than one the vocabulary lacks. The comparison to the one-round composition survives, because both are unavailable for the same surviving reason. The relayed authority is traced back through the two-level record to L4.
- **`autoregressive-state-and-kv-cache.md`.** The `unchanged from L4` attribution is retired first and explicitly independently of whether the conclusion holds. The conclusion survives on a different ground the record already states: a decode step is a sequence of dispatched stages ordered by an ordinary `Data` dependency. What is withdrawn is the "and nothing else" — no cooperative schedule is *selected* inside a step today, but no longer because the vocabulary lacks a barrier.

## What survived on other grounds, re-evidenced rather than assumed

The brief's instruction not to invent a replacement ground made one check decisive, because two of the four sites rest their surviving conclusion on **"No profile declares a subgroup width"** — and had that also fallen, this lane would have replaced a false claim with a different false claim, which is the failure `AGENTS.md` names. It has not fallen, and the evidence is unusually strong:

- The only subgroup-width declaration is `BoundMetalSubgroupDeclaration` in `crates/tiler-build/src/metal_subgroup_declaration.rs`. Its entry points are `pub(crate)`; `crates/tiler-build/src/lib.rs` declares the module with no `pub use`, unlike every sibling.
- The module carries a `#![cfg_attr(not(test), expect(dead_code, ...))]`. That is **compiler-enforced unreachability**: any non-test caller would fire `unfulfilled_lint_expectation` and fail the build. This is a check that can say *no*, unlike a doc comment.
- `the_standard_declaration_stays_subgroup_silent` pins that the production `first_macos_apple9` profile resolves `SubgroupRealizationResolution::Unknown` and its descriptor carries neither the subgroup-realization nor the subgroup-width-query key.
- `crates/tiler-metal/src/target.rs` contains no occurrence of `subgroup`, `simd`, or `thread_execution` at all.

The asymmetry is the point: the **synchronization** row is on the production profile and reaches the proc-macro AOT path, while the **subgroup** row is a crate-private fixture reached only by its own tests. That is exactly why the barrier half of the L4 ground fell and the subgroup-width half did not.

## Out of scope, reported with owners named

Not edited. Each is reported rather than widened.

- **`crates/**` — three stale claims, owner: a `crates/`-scoped lane.** `crates/tiler-compiler/src/request/recognize.rs` anchor `carries no law at all`, contradicted by the registered `StagedSoftmaxF32`; `crates/tiler-compiler/tests/softmax_recognizer_boundary.rs`, whose module header names rule `missing-capability` while its own test asserts `accuracy.elementary.no-installed-realization`; and `crates/tiler-compiler/src/region.rs` anchor `registered law spells a multi-reader chain yet`. **Anchor warning for whoever takes it:** the `///` comment in `region.rs` wraps after `no`, so the fuller `no registered law spells a multi-reader chain yet` greps to 0 while the fragment above returns 1.
- **[`implement-the-single-workgroup-synchronized-reduction-strategy`](implement-the-single-workgroup-synchronized-reduction-strategy.md) — terminal, owner: the coordinator.** It states that `metal_declaration.rs` "still declares no synchronization row, so a cooperative region remains `Unknown` against the macOS profile". Both halves are falsified by the production `SynchronizationSupport::Realized` row audited above. Verified present at this base by hand. **`make citations` cannot see it** — the gate's populations are open tickets, live documents, and retained spike records, and a `done` ticket is in none of them — so this needs hand-verification whenever it is repaired, and its repair will not be policed by the gate.
- **`docs/research/program-planning/flash-class-capability-set.md` — owner: [`repair-the-flash-class-records-falsified-supplied-greps`](repair-the-flash-class-records-falsified-supplied-greps.md).** Named as a non-goal here and left alone. Noted only because its subject touches this one: its falsified `SubgroupWidth` grep is about *schedule vocabulary existing*, which is compatible with the finding above that no *profile* declares a width. A repairer should not read this lane's result as contradicting that record's surviving conclusion.

## Scope note

`research/runtime` was added to this ticket's scopes. It was required by the ticket's own Required work, which names `docs/research/runtime/autoregressive-state-and-kv-cache.md`, while the dispatched scopes covered only `research/scheduling` and `research/program-planning`; `ticketsplease.toml` maps `docs/research/runtime/**` to `research/runtime`. Under `AGENTS.md` this is scheduling metadata for authorized work rather than an outcome expansion, so it was added and explained rather than escalated. No file under `research/program-planning` was edited — the L4 record was read as evidence and left as its own lane repaired it.

## Checks

`tkt lint`, `make citations`, `git diff --check`, and `tkt guard --base 3e6cc78e` against the true base. A docs-and-tickets delta touching only `docs/research/scheduling/two-level-subgroup-workgroup-reduction.md`, `docs/research/scheduling/multi-round-two-level-reduction-composition.md`, `docs/research/runtime/autoregressive-state-and-kv-cache.md`, and this ticket file carries the last green gate: it touches none of `crates/`, `prototypes/`, `Cargo.toml`, `Cargo.lock`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, `deps.sh`, or `check-citations.sh`.
