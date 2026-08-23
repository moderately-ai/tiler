---
id: reconcile-the-l4-records-self-contradicting-softmax-elimination-row
title: Reconcile the L4 record's self-contradicting softmax elimination row
status: in-progress
priority: p2
dependencies: []
related: [plan-the-materialized-attention-decomposition]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, doc-drift, scheduling]
claimed_from: todo
assignee: worker-l4row2
lease_expires_at: 1787449582
---

The L4 attention-planning record, [`first-attention-program-vertical.md`](../docs/research/program-planning/first-attention-program-vertical.md), contradicted itself about threadgroup-cooperative softmax within one file. Its elimination table rejected the candidate on a zero-synchronization barrier ground, while a dated 2026-08-10 correction lower in the same document already superseded that ground. A reader landing on the table got one answer and a reader reaching the correction got another, and neither knew the other existed.

## Fact audit at base `f69829143a387a8e117858dbcaad416715f7e788`

The ticket carried no stated Facts — its body was a single placeholder character — so the audited claims are the dispatching brief's.

- **Verified.** The elimination row exists, at anchor `Threadgroup-cooperative softmax`, with ground `A SIMD-group-cooperative row reduction survives; anything wider does not.`
- **Verified.** A dated correction lower in the same file supersedes that ground, at anchor `the zero-synchronization profile above is proposal-era history`.
- **Imprecise, and understated.** The brief described one contradicting row. There are **two** uncorrected sites: the elimination row, and the Softmax (18) row of the legal-candidate table at anchor `has no synchronization construct to be built from`. Two further zero-synchronization sites already carried dated 2026-08-10 corrections and needed no repair.
- **Verified at source, and the landed position is stronger than the record's own correction records.** The 2026-08-10 correction says a cooperative schedule must state and prove its own point and that "whether a target can realize an exact point remains a separate atomic target-feasibility question". That question is now answered for this subject: `crates/tiler-build/src/metal_declaration.rs` declares the workgroup-scoped `ControlBarrier` subject at `SynchronizationSupport::Realized` from a production call site, so the realization is target-proven rather than undeclared.

## The elimination's conclusion does not survive, and the verdict is withdrawn rather than re-grounded

Both halves of the row's ground have moved.

**Fact — the barrier half is gone.** `ReductionTopology::CooperativeWorkgroup` carries a `CooperativeTile` owning identity-bearing `SynchronizationPoint` declarations, and `SynchronizationKind::ControlBarrier` is the one admitted kind. The kernel verifier's barrier elimination did not disappear but became *conditional*: it refuses a barrier only in a schedule whose topology carries no cooperative tile, so a schedule that carries one is verified rather than refused.

**Fact — the surviving half is no longer a discriminator.** `tiler::softmax-f32@1` is registered and recognized and carries the realization law `IndexRealizationLaw::StagedSoftmaxF32`. What it lacks is an installed lowering for that law's stages: `physical::staged_plan` has one arm, for `StagedRootMeanSquareScaleF32`, and every other law falls through to `_ => None`. A softmax program therefore refuses at compile under `UnsupportedCapability` with rule `accuracy.elementary.no-installed-realization`, uniformly across all five numerical contracts, measured by `crates/tiler-compiler/tests/softmax_recognizer_boundary.rs` at `a_softmax_program_is_refused_for_want_of_an_installed_lowering` against a control program that compiles under the same request.

**Inference.** A refusal reached before a schedule is chosen falls on candidates (a) through (d) of the softmax row alike. It grounds no elimination *between* them, and cannot leave a SIMD-group-cooperative row standing while a wider one falls. The row eliminated a candidate on a ground that no longer holds, and no replacement ground eliminates it, so the repair withdraws the verdict instead of supplying a new one. What is true today is narrower than either reading of the row, and narrower than "a barrier is unavailable": the barrier is available and target-proven, and no softmax realization of any topology is installed.

Nothing downstream moves. The delivered feasibility predicates are unchanged — the barrier ground was never one of the six. D-A, D-B, and D-C are untouched, because none rests on this row: D-C falls on distributivity and the exponential elementary-function identity, and D-B's two-stage handoff is an ordinary `Data` dependency between dispatches, which `crates/tiler-ir/src/program/model.rs` still records at anchor `pass boundary`. [Q-PLAN-004](../docs/open-questions.md#q-plan-004--coexisting-reductions-in-one-kernel) remains open, so the delivered softmax form stays a two-stage subprogram regardless of topology.

The precedent for withdrawing rather than re-grounding is in the same table: the `Whole-block single kernel` row was moved to `No physical realization delivered` by the 2026-08-10 lane for the structurally identical reason.

## Repair

Two rows marked in place and two dated correction paragraphs added, each directly beneath the table it corrects, so a reader landing on a table meets the correction without leaving it.

- The Softmax (18) row's ground column keeps its retired text and gains a dated supersession marker.
- The elimination row's verdict cell moves from a bare `No` to `Verdict withdrawn 2026-08-22`; its retired ground stays in place, marked superseded.
- Both corrections state the landed vocabulary, the conditional barrier rule, the target-proven Metal row, and the installed-lowering refusal, and state explicitly that the softmax schedule set is not reopened here.

Retired wording is preserved in place, so grep counts grow rather than shrink. Each of `A SIMD-group-cooperative row reduction survives; anything wider does not.`, `The kernel verifier admits no barrier under the implemented zero-synchronization schedule profile`, and `has no synchronization construct to be built from` still returns 1 against the record after the repair.

**A first draft of this repair asserted that softmax "carries no registered `IndexRealizationLaw`" and "refuses at request recognition".** Both are false and were caught before commit by an independent source sweep. The claim was taken from a doc comment in `crates/tiler-compiler/src/request/recognize.rs` reading `still refuses here because it carries no law at all`, which is itself stale — `register-the-softmax-realization-law` landed `StagedSoftmaxF32` and the recognizer now admits it. This is the hazard `AGENTS.md` names in treating comments as claims about current behaviour rather than authority, and it is recorded here so the next reader of that comment does not repeat it.

## Sibling scan

**Clean, and the clean result is informative.** Across all eleven documents under `docs/research/program-planning/` and every record under `spikes/program-planning/`, the L4 record is the only file carrying zero-synchronization language. Its own remaining barrier statement, that D-B's inter-stage handoff needs none, verifies against `crates/tiler-ir/src/program/model.rs` and correctly stays. [IR](../docs/ir.md) is current and needed no finding.

**One finding, same scope and different subject.** [`flash-class-capability-set.md`](../docs/research/program-planning/flash-class-capability-set.md) carries five stale citations whose conclusions all survive. Two are falsified supplied greps, which fail in the dangerous direction because a reader re-running them today would conclude the row had reversed:

| Stated check | Stated result | Result at this base | Conclusion |
| --- | --- | --- | --- |
| `grep -rn 'SubgroupWidth\|lane_identity\|SubgroupThenWorkgroup' crates/` | returns nothing | **69 lines**, including `pub struct SubgroupWidth(u32)` in `crates/tiler-ir/src/schedule/subgroup.rs` | survives — the hits are schedule vocabulary; `MetalTargetFacts` still has exactly five fields and none is a subgroup width |
| `grep -rni 'simdgroup' crates/` | returns five lines | **21 lines** | survives — no matrix construct among them |

Three line pins in the same section also drifted: `crates/tiler-compiler/src/target/feasibility.rs:211` is now 241 and the enum is `pub(crate)` rather than `pub`; `crates/tiler-metal/src/target.rs:755` is now 871; `component_cost.rs:619` is now 629. Repairing that record is a different subject from this ticket and is left to the coordinator to schedule rather than folded in as outcome expansion.

**Out of scope, and reported rather than edited.** Three documents outside `research/program-planning` restate the retired L4 ground, one quoting this record by name, so repairing L4 alone leaves the stale claim alive elsewhere with an attribution to L4:

- `docs/research/scheduling/two-level-subgroup-workgroup-reduction.md`, two sites, one quoting the L4 row verbatim at anchor `none of the three is admissible in the attention block today`.
- `docs/research/scheduling/multi-round-two-level-reduction-composition.md`, one site.
- `docs/research/runtime/autoregressive-state-and-kv-cache.md`, one site, at anchor `synchronization inside a step is dispatch ordering`.

`docs/artifact-abi.md` mentions nonzero synchronization as current state and is clean.

**Three stale claims in `crates/` and one in a closed ticket, all out of scope and reported for a separate lane.**

- `crates/tiler-compiler/src/request/recognize.rs`, anchor `carries no law at all`, contradicts `crates/tiler-ir/src/index/law.rs`, which registers `StagedSoftmaxF32`.
- `crates/tiler-compiler/tests/softmax_recognizer_boundary.rs` has a module header saying the refusal rule is `missing-capability` while its own test in the same file asserts `accuracy.elementary.no-installed-realization`. A doc-versus-test disagreement about where the ceiling sits.
- `crates/tiler-compiler/src/region.rs`, anchor `registered law spells a multi-reader chain yet` — the surrounding comment reads as though softmax's law were still outstanding, which sits awkwardly beside the registered `StagedSoftmaxF32`. Flagged rather than asserted stale: whether that law spells a multi-reader chain was not read here, and the sentence may be about the chain rather than the registration. Note the anchor: the `///` comment wraps after `no`, so the fuller `no registered law spells a multi-reader chain yet` greps to 0 while the fragment above returns 1.
- [`implement-the-single-workgroup-synchronized-reduction-strategy`](implement-the-single-workgroup-synchronized-reduction-strategy.md), which is `done`, states that `metal_declaration.rs` declares no synchronization row and that a cooperative region stays `Unknown` against the macOS profile. The production ledger row at `SynchronizationSupport::Realized` falsifies both halves.

## Checks

`tkt lint`, `make citations`, `git diff --check`, and `tkt guard --base f6982914` against the true base. A docs-and-tickets delta touching only `docs/research/program-planning/first-attention-program-vertical.md` and this ticket file carries the last green gate: it touches none of `crates/`, `prototypes/`, `Cargo.toml`, `Cargo.lock`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, `deps.sh`, or `check-citations.sh`.
