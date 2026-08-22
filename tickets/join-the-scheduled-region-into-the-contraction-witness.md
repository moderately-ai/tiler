---
id: join-the-scheduled-region-into-the-contraction-witness
title: Join the scheduled region into the contraction witness
status: todo
priority: p1
dependencies: [derive-staged-combine-structure-from-program-scope]
related: [accept-the-exact-composed-reference-session-and-event-surface]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [ir, scheduling, conformance, witness]
---
## User-visible outcome

The contraction plan witness derives a staged kernel's combine tree from the scheduled region that already states it, instead of refusing every kernel that declares any workgroup staging.

## Rescoped 2026-08-22 — this was filed as a schema/identity migration and it is not one

I filed this as *"Encode identity-bearing staged combine structure"*, on the reading that the witness's own module header names the remedy as making a coordinate-dependent tree mapping identity-bearing. The executable spike [`derive-staged-combine-structure-from-program-scope`](derive-staged-combine-structure-from-program-scope.md) showed the first half of that is right and the second half is wrong.

**The structure is already encoded, already identity-bearing, and already tag-injectivity-tested — at the schedule layer.** `crates/tiler-ir/src/schedule/witness.rs` publishes `contributor_partition()`, `arrival()`, `rounds()`, and `accumulation()` on `RealizationWitness`; verified by the coordinator at `470004be`, all four present. So **no new encoding is required and no identity domain steps.** What is missing is a *join*: the witness retains only a `RegionId` and an opaque `CanonicalScheduledRegionIdentity`, and never reaches the region that states the tree. The spike shows the join is exact and available — `kernel.scheduled_region_identity()` accepts its own region and rejects a crossed one.

**Fact — the structure genuinely cannot come from program scope.** Two verified regions over one subject (`[2,6]→[2]`, 3 participants, 6 contributors), differing only in tile round structure, produce **identical** program-scope observations — same staging row, same `staging().len() != 0`, same launch, same builtins, same region ordinal — while declaring different associations of the same contributors: `(((c0+c1)+(c2+c3))+(c4+c5))` versus `(((c0+c1)+c2)+((c3+c4)+c5))`. Different associations are different binary32 computations. A negative control rebuilding one region twice reports DETERMINED, so the probe can say *no*. Reproduce: `cd spikes/reference/staged-combine-derivability && cargo run`.

**What specifically is unrecoverable is the staging's *role*.** `CooperativeWorkgroup` stages partials of a partitioned chain; `CooperativeContraction` stages operand tiles and folds into one carried accumulator, so its tree is the plain left chain the witness already derives. Both present to program scope as `staging().len() != 0` with identically-shaped `StagingParameter` rows.

## Required work

- Re-audit every Fact above at your base and report a per-Fact verdict; the spike is another worker's, and the coordinator has verified only the four accessors and the refusal predicate.
- Join the scheduled region into the witness so the combine tree comes from the record that states it. **Do not recover it from the kernel body** — that means symbolically executing thread-id-dependent staging addresses across barrier-separated phases, which is a second semantics that yields a silently wrong tree wherever it disagrees with the emitter. That is the exact failure the witness exists to prevent.
- **Expected: no identity or schema value moves, and no domain steps.** Rederive rather than copy that expectation; if one moves, **stop and report** — the whole point of this rescope is that it should not.
- Perturb the subject separately for each new behaviour and quote the failure text, including a crossed-region control that must be rejected.

## Non-goals

Adding any new encoding, tag, column, or identity domain — the rescope above is precisely that this is unnecessary. Widening the over-broad refusal itself, which is [`narrow-the-contraction-witness-refusal-to-staging-it-cannot-read`](narrow-the-contraction-witness-refusal-to-staging-it-cannot-read.md). Choosing the composed-reference surface, which is Tom's.

## Closes when

A staged kernel whose scheduled region states its combine tree yields a witness rather than a refusal, the tree comes from the schedule record and never from the body, no identity or schema value has moved, and the crossed-region control is watched being rejected.
