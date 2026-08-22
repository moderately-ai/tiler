---
schema: "tiler-doc/v1"
id: "tiler.spike.reference.staged-combine-derivability"
kind: "experiment"
title: "Staged intra-workgroup combine structure derivability probe"
topics: ["reference", "conformance", "numerics", "scheduling"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "exhaustive-finite"]
entrypoints: ["spikes/reference/staged-combine-derivability/harness/src/main.rs"]
last_verified: "2026-08-22"
ticket: "derive-staged-combine-structure-from-program-scope"
---

# Staged intra-workgroup combine structure derivability probe

This harness answers one question with evidence rather than with a reading of the witness: **can a kernel's staged intra-workgroup combine structure be derived from program scope alone?** [`derive-staged-combine-structure-from-program-scope`](../../../tickets/derive-staged-combine-structure-from-program-scope.md) carries the answer, and [`accept-the-exact-composed-reference-session-and-event-surface`](../../../tickets/accept-the-exact-composed-reference-session-and-event-surface.md) is blocked on it.

```sh
cd spikes/reference/staged-combine-derivability
cargo run
```

Nothing runs it automatically; no `make` target reaches `spikes/`, and this workspace is not a member of the root one. That is deliberate — an exploratory dependency must not become a repository gate.

## The question, and why the witness raises it

[`ContractionF32PlanWitness::from_program`](../../../crates/tiler-ir/src/program/contraction_witness.rs) refuses any kernel that declares workgroup staging. The refusal is at the anchor `A kernel declaring workgroup staging combines inside the workgroup`, and the module header states the consequence at anchor `must become identity-bearing in`. If the structure were derivable from what the program already carries, that refusal could be narrowed and no encoding work would be needed.

## Design, fixed before the run

**Inputs.** Two verified scheduled regions over one subject — `[2, 6] -> [2]`, three participants, six contributors per row — differing *only* in the round structure of their cooperative tile:

| region | partitions | contributors per partition | rounds | contributors covered |
| --- | --- | --- | --- | --- |
| single round | 3 | 2 | 1 | 6 |
| two rounds | 3 | 1 | 2 | 6 |

Both are built through `tiler-ir`'s public `ScheduledRegionBuilder` and `workgroup_tree_tile`, and both are lowered by the public `lower_scheduled_region`. The shape mirrors the fixture `tiler-conformance`'s loop-carried suite builds, so the subject is one the repository already verifies rather than one invented here.

**Outputs.** Per region, two vectors: the *program-scope observation* — the staging parameter list, the `staging().len() != 0` predicate the witness actually tests, the buffer signature, the admitted builtins, the launch geometry a stage publishes, and the scheduled-region ordinal — and the *schedule-scope grouping*, from which the exact binary combine tree is rendered.

**Metric.** Whether the two program-scope observations are equal in every component while the two combine trees differ.

**Stop condition.** UNDECIDED, naming the separating field, if either region fails to verify or lower, or if the observations differ anywhere. DETERMINED if the groupings coincide. Only equal observations over differing trees support NOT DERIVABLE.

**Unsupported cases, named rather than silently excluded.** Contraction occurrences staged by the compiler: the population is empty, because the frontier's `RegionSpellingKind::Contraction` arm offers no cooperative strategy (anchor `No split: a contraction's fold is the declared contributor`). `ReductionTopology::CooperativeContraction`: constructed nowhere outside tests. Logarithmic trees: not statable in this tile vocabulary at all, which `cooperative.rs` states at anchor `A logarithmic tree is still not statable`.

## Result

The two regions produce the **identical** program-scope observation — `staging = [(0, "F32", "Workgroup", 3)]`, `staging().len() != 0 = true`, launch `6` threads at `3` per workgroup, the same two builtins, the same region ordinal — while declaring different combine trees over the same six contributors:

```text
single round (3 x 2 x 1) -> (((c0+c1)+(c2+c3))+(c4+c5))
two rounds   (3 x 1 x 2) -> (((c0+c1)+c2)+((c3+c4)+c5))
```

These are different associations of one contributor sequence, so they are different binary32 computations. The separating fact — the tile's round structure — lives on the region's `ReductionTopology`, which program scope does not carry: a `VerifiedKernel` retains only a `RegionId` and an opaque `CanonicalScheduledRegionIdentity`.

**Answer: not derivable from program scope.**

### The negative control, and what it is for

Pair 2 rebuilds the *same* region twice and must report DETERMINED. The subject is perturbed, never the assertion: a probe that reported non-determination for a pair that has none would be broken, and this is what shows the check can say *no*.

### What the probe deliberately does not claim

The two kernel identities are **not** equal — 1944 against 2227 bytes — because the emitted bodies differ. Program scope is therefore not information-theoretically empty; the structure is implied by the body's staged addresses, barrier phases, and loop bounds. The load-bearing claim is narrower: **no declarative record in program scope states it.** Recovering it from the body would mean symbolically executing thread-id-dependent staging addresses across barrier-separated phases, predicated commits, and serial loops whose bounds may be live values — a second semantics of the body that must agree exactly with the emitter, and which silently yields a *wrong* tree wherever it does not. That is the failure the witness exists to prevent, stated at its own anchor `happens to equal a value tree B can produce`.

### The evidence boundary: the executable pair is a reduction

The two regions the probe builds are `StrictSerialSum` reductions under `ReductionTopology::CooperativeWorkgroup`, not contractions. That is forced rather than chosen: the compiler builds no cooperative contraction at all, so no contraction subject with a staged combine exists to measure. The transfer to the contraction witness is therefore an **inference**, and this is its basis, verified by reading rather than by the probe: the program layer carries no reduction topology for *any* operation family. `ReductionTopology` and `CooperativeTile` both occur zero times across `crates/tiler-ir/src/program/model.rs`, `verify.rs`, `builder.rs`, and `contraction_witness.rs`; the only two mentions in `program/mod.rs` sit inside a `//!` doc example that builds a scheduled region, not a program field. Program scope is blind to the topology uniformly, so a contraction fares no better than the reduction measured here.

What the probe consequently does **not** establish is that a contraction occurrence can be paired with a staged kernel at all. Through the ordinary pipeline it cannot. `from_program` is a public constructor that accepts any `VerifiedKernelProgram`, so a hand-built program could still present one; that population is unmeasured here.

### The remedy is a join, not a new encoding

Pair 3 probes it. The combine structure is already encoded, already identity-bearing, and already tag-injectivity-tested at the schedule layer: `ContributorArrival::tag`, `StagedElement::tag`, and the tile all fold into `CanonicalScheduledRegionIdentity`, and [`RealizationWitness`](../../../crates/tiler-ir/src/schedule/witness.rs) already aggregates exactly the needed fields — `contributor_partition()`, `arrival()`, `rounds()`, `accumulation()`. What is missing is that the witness constructor takes no region.

The join is available and exact, and the probe shows it accepting the true pairing and rejecting the crossed one:

```text
single round kernel joined against its own region:   ACCEPT
single round kernel joined against the other region: REJECT
```

So the prerequisite is "carry or join the scheduled region", not "design an identity-bearing encoding".

## Host-specific versus portable

Nothing here is host-specific. The probe performs no dispatch, no timing, and no device work; it compares declared records and lowered kernel signatures, so its verdict is a property of the repository's types at this commit and reproduces on any host that builds the workspace.

## Standing hazard

`spikes/` sits outside every gate, so this harness breaks **silently** when the workspace API moves. It will stop compiling if `ScheduledRegionBuilder`, `KernelSchedule`, `ReductionTopology::CooperativeWorkgroup`, `workgroup_tree_tile`, `lower_scheduled_region`, or the `VerifiedKernel` accessors change shape — a loud failure. The dangerous direction is quieter: if `ReductionTopology` gains a field that also reaches the kernel signature, the two observations could begin to differ and the probe would report UNDECIDED rather than a wrong answer, which is the fail-closed direction but is easy to miss when nobody runs it. A future reader finds out by running the command above; there is no other signal, and `last_verified` in the frontmatter is the date that was last true.
