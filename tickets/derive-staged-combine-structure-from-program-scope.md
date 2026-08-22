---
id: derive-staged-combine-structure-from-program-scope
title: Derive staged combine structure from program scope
status: in-progress
priority: p1
dependencies: []
related: [accept-the-exact-composed-reference-session-and-event-surface, encode-identity-bearing-staged-combine-structure]
scopes: [research/reference]
shared_scopes: [project/tickets]
paths: []
tags: [research, spike, reference, conformance, numerics]
claimed_from: todo
assignee: worker-stagedspike
lease_expires_at: 1787428751
---
## User-visible outcome

A bounded, reproducible answer to one question: can a kernel's staged intra-workgroup combine structure be derived from program scope alone, or does it require an identity-bearing encoding? The answer decides whether the composed-reference decision has one candidate or two, so it is worth answering before that packet reaches Tom.

## Why this exists

Filed 2026-08-22 by the coordinator from the composed-reference session re-gate, which found that ADR 0112's landed witness shape is a fifth materially distinct candidate the original packet predates. That candidate carries a real prerequisite, and this spike is what tells us how large it is.

**Fact — the witness refuses a staged kernel today, and says why.** `crates/tiler-ir/src/program/contraction_witness.rs` refuses any kernel that, at anchor `declares workgroup staging`, has an intra-workgroup combine structure the witness cannot see. Its module header states the consequence at anchor `must become identity-bearing in`. Both anchors resolve exactly once in that file; verified by the coordinator at `b3c07259`.

**Fact repair — 2026-08-22 by `worker-stagedspike` at base `e7b6026f`; both anchors re-verified, and the imprecision below is load-bearing.** Both anchors still resolve exactly once. Two corrections to the *characterization*, neither of which changes what this ticket is for:

1. The anchor `declares workgroup staging` sits in the doc comment of the `TopologyUnsupported` error variant, not at the refusal. The refusal is a separate code site, at anchor `A kernel declaring workgroup staging combines inside the workgroup`.
2. The refusal predicate is **not** "has an intra-workgroup combine structure the witness cannot see". The code tests `covering.kernel().staging().len() != 0` — it refuses a kernel that declares *any* workgroup staging, including staging that carries no combine structure at all. `ReductionTopology::CooperativeContraction` stages **operand tiles**: at anchor `none of them combines` (the sentence wraps in the source, so a longer anchor fails), no invocation combines another's partial, and the lowering folds into one carried accumulator at anchor `a subtotal of their own`, so its combine tree is the plain canonical left chain the witness already derives for the direct realization. The refusal is therefore over-broad against that population, and the distinction between "stages memory" and "stages partials" is the heart of the answer below.

**Fact — the witness itself is a landed public value, not a proposal.** `ContractionF32PlanWitness` appears in `crates/tiler-ir/src/program/contraction_witness.rs`, `crates/tiler-ir/src/program/mod.rs`, `crates/tiler-compiler/tests/contraction_topology_witness.rs`, and `crates/tiler-reference/src/contraction/topology.rs` — four files, so the shape is exercised across the IR, compiler, and reference layers rather than declared in one.

## Required work

- Re-audit both Facts at your own base before starting; the citations above are anchors, not line numbers, and each was grepped against the file it names.
- Answer the question with a bounded spike under `spikes/`, run from a documented command, with explicit inputs, outputs, and stop condition. Do not add a workspace gate.
- State the answer as one of: **derivable** (structure recoverable from program scope with no encoding change — the frontier collapses to one candidate); **not derivable** (an identity-bearing encoding is required — [`encode-identity-bearing-staged-combine-structure`](encode-identity-bearing-staged-combine-structure.md) becomes a real prerequisite); or **undecided**, with exactly what would settle it.
- Whichever answer, say what it costs to be wrong in each direction.

## Non-goals

Implementing either candidate; editing `docs/decisions/`; changing the witness or the evaluator; and deciding the composed-reference surface, which is Tom's.

## Closes when

The question is answered with reproducible evidence, the cost of being wrong is stated in both directions, and the composed-reference packet's frontier is updated to match.

## Findings — 2026-08-22 by `worker-stagedspike` at base `e7b6026f`

**Answer: not derivable from program scope — but the second horn is false too, and that is the load-bearing correction.** The remedy is a *join* against the scheduled region, not a new identity-bearing encoding.

Spike: [`spikes/reference/staged-combine-derivability`](../spikes/reference/staged-combine-derivability/README.md), run with `cd spikes/reference/staged-combine-derivability && cargo run`. Ungated by construction: its own workspace, not a root member, reached by no `make` target.

**Per-Fact verdict.** Fact 1 (`the witness refuses a staged kernel today`): **verified but imprecise**, repaired above — the predicate is `staging().len() != 0`, which also refuses operand staging that carries no combine structure. Fact 2 (`the witness itself is a landed public value`): **verified** — `tiler-reference` holds a typed `&ContractionF32PlanWitness` field, `program/mod.rs` re-exports it, and the compiler integration test drives it, so "exercised across three layers" is supported rather than a count of mentions.

**Measurement.** Two verified regions over one subject — `[2, 6] -> [2]`, three participants, six contributors, differing only in tile round structure — produce the **identical** program-scope observation (`staging = [(0, "F32", "Workgroup", 3)]`, `staging().len() != 0 = true`, launch `6`/`3`, same builtins, same region ordinal) while declaring different combine trees: `(((c0+c1)+(c2+c3))+(c4+c5))` against `(((c0+c1)+c2)+((c3+c4)+c5))`. Different associations of one contributor sequence are different binary32 computations. The separating fact, the tile's round count, lives on the region's `ReductionTopology`; a `VerifiedKernel` retains only a `RegionId` and an opaque `CanonicalScheduledRegionIdentity`. A negative control rebuilding one region twice reports DETERMINED, so the probe can say *no*.

**What specifically cannot be recovered:** which role the staging plays, and hence whether it changes the combine structure at all. Both `CooperativeWorkgroup` (staged *partials*, partitioned chain) and `CooperativeContraction` (staged *operand tiles*, plain left chain) present to program scope as `staging().len() != 0`, and the `StagingParameter` rows are the same shape for both.

**Stated limit, not hidden:** the emitted bodies do differ, so program scope is not information-theoretically empty. The claim is narrower — no *declarative* record states the structure, and recovering it from the body means symbolically executing thread-id-dependent staging addresses across barrier-separated phases, a second semantics that silently yields a wrong tree wherever it disagrees with the emitter.

**Why no new encoding is needed.** The structure is already encoded, already identity-bearing, and already tag-injectivity-tested at the schedule layer (`ContributorArrival::tag`, `StagedElement::tag`, folded into `CanonicalScheduledRegionIdentity`), and `RealizationWitness` already aggregates `contributor_partition()`, `arrival()`, `rounds()`, and `accumulation()`. The join is exact and available: the spike shows `kernel.scheduled_region_identity()` accepting its own region and rejecting the crossed one.

**Population today is empty.** The compiler never schedules a contraction cooperatively — the frontier's contraction arm offers no parallel strategy at anchor `No split: a contraction's fold is the declared contributor` — and `ReductionTopology::CooperativeContraction` is constructed only in tests. So no currently compilable program is blocked by this refusal.

**Cost of being wrong.** A false *derivable* is the expensive direction: Tom accepts a composed-reference candidate whose prerequisite was never scheduled, and the witness later derives a tree from a body it guessed at — a silently wrong combine tree inside the one construct built to prevent exactly that, reached through a public `pub` constructor whose refusals are the safety argument. A false *not derivable* costs a scheduled migration nobody needed; here that cost is bounded and visible, because the migration would be a constructor-signature change plus an identity join rather than a schema or identity-domain step, and the empty population means nothing ships wrong while it is pending.

**Consequence for the two dependent tickets.** [`encode-identity-bearing-staged-combine-structure`](encode-identity-bearing-staged-combine-structure.md) does **not** close, but its scope is wrong as written: it should become "carry or join the scheduled region into the contraction witness", with no new encoding, no schema step, and no identity-domain step. [`accept-the-exact-composed-reference-session-and-event-surface`](accept-the-exact-composed-reference-session-and-event-surface.md) keeps two candidates — the frontier does not collapse — but the encoding-bearing candidate is materially cheaper than the packet assumed, which is a re-gate input rather than a decision. A separate narrow ticket is warranted for the over-broad refusal itself: `staging().len() != 0` refuses the operand-staged contraction whose tree the witness already derives correctly.
