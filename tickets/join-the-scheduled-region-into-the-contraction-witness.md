---
id: join-the-scheduled-region-into-the-contraction-witness
title: Join the scheduled region into the contraction witness
status: in-progress
priority: p1
dependencies: [derive-staged-combine-structure-from-program-scope]
related: [accept-the-exact-composed-reference-session-and-event-surface]
scopes: [implementation/ir, contracts/foundation, research/reference]
shared_scopes: [project/tickets]
paths: []
tags: [ir, scheduling, conformance, witness]
claimed_from: todo
assignee: worker-witnessjoin
lease_expires_at: 1787438923
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

## Findings — 2026-08-22 by `worker-witnessjoin` at base `3ba89314`

### Per-Fact verdict

1. **"The structure is already encoded, identity-bearing, and tag-injectivity-tested at the schedule layer; `RealizationWitness` publishes `contributor_partition()`, `arrival()`, `rounds()`, `accumulation()`."** **Verified.** All four accessors read in full in `crates/tiler-ir/src/schedule/witness.rs`; `arrival()` returns `Some` only for `CooperativeWorkgroup`, and `None` for `CooperativeContraction`, which carries no `arrival` field at all.
2. **"The witness retains only a `RegionId` and an opaque `CanonicalScheduledRegionIdentity`."** **Verified.** `VerifiedKernel` in `crates/tiler-ir/src/kernel/model.rs` carries exactly those two, and the identity's own doc states it is *a pure function of the normalized schedule content* — which is what makes the join exact in both directions rather than merely usually right.
3. **"The structure genuinely cannot come from program scope"** (two regions, identical program-scope observation, different trees). **Verified by re-running the spike at this base**, not by reading its report: `cd spikes/reference/staged-combine-derivability && cargo run` reproduces the recorded output exactly — identical staging row, launch, builtins and region ordinal against `(((c0+c1)+(c2+c3))+(c4+c5))` versus `(((c0+c1)+c2)+((c3+c4)+c5))`; the negative control still reports DETERMINED and pair 3 still reports ACCEPT/REJECT.
4. **"`CooperativeContraction` stages operand tiles and folds into one carried accumulator, so its tree is the plain left chain."** **Verified, from three independent statements.** The variant's own `permits_reassociation` doc at anchor `ascending contracted order straight through the round`; `verify_cooperative_contraction` at anchor `ascending contracted order across the whole round`; and — as corroboration rather than as the source — `emit_cooperative_contraction`'s `fold_tile` at anchor `into a subtotal of their own`, which seeds from the first product and threads one accumulator through every round. All three anchors were grepped against the files they name.
5. **"No identity domain should step."** **Held, derived rather than asserted.** The whole diff is two files, both under `crates/tiler-ir/src/program/`. No encoder, no `tag()` function, no identity-domain version string, and no `ScalarProgram`/`ReductionTopology`/`ContributorArrival` variant is touched, so nothing that feeds a canonical encoding was reachable from this change. The added error variant carries no tag and is folded into no identity. The derived tree is byte-identical to what the module already produced: the join converts a refusal into an admission and never reshapes a tree.

### New finding — the join's admitted population is narrower than the rescope implies, and provably so

The rescope reads as though the join must eventually express the cooperative-workgroup partitioned chain for a contraction. **It cannot arise, by two independent structural walls, both read rather than inferred.**

- A `CooperativeWorkgroup` region cannot *carry* a contraction. `builder/intrinsic.rs` routes `ScalarProgram::StrictTensorContraction` to `verify_contraction`, never to the fold gate; and `verify_cooperative_semantics` requires `split_family(...)` to be `Some`, which `builder/family.rs` returns `None` for on contraction.
- A cooperative-workgroup *kernel* cannot cover a contraction occurrence in a verified program. The fold gate hands such a region exactly `[read, write]`, so its kernel has one read buffer, while a contraction occurrence has two operands. Watched by `a_cooperative_workgroup_kernel_cannot_cover_a_contraction_occurrence`, which asserts the `IncompleteComponentSet` refusal and fails loudly if that wall ever moves.

So `ReductionTopology::CooperativeContraction` is the only staged topology the contraction witness can meet, and its tree is the canonical left chain. The join admits exactly that and refuses every other topology through an exhaustive match, so a widened `ReductionTopology` is a build error rather than a staging shape that silently keeps the chain.

### Two refusal arms are reserved, and this is stated rather than left looking exercised

- `StagedRole::admits`' contributor-count mismatch is unreachable for a direct realization, because the program layer already ties a stage's operand extent to its kernel's buffer extent — probed, and refused as `StageElementCount { position: 0, expected: 256, actual: 512 }` before a witness is ever asked for. Only a declared `PartialReduction` could separate the two counts, and no contraction split is expressible: `ContractionAxisSource` offers only `Output` and `Contracted`, so a contracted axis cannot be factored into a partition axis and a within-partition axis.
- The joined-but-unreadable-topology arm is unreachable for the reason in the section above.

Both are kept because they are the correct relations the day a split becomes expressible, and both are documented as reserved in the same idiom the error enum already uses.

### Perturbations, subject-side, with the failure text

- Join the admission test against a region the kernel does not refine — `the joined region states this kernel's combine tree: ScheduledRegionUnjoined`.
- Make the crossed region identical to the true one — ``assertion `left != right` failed: the two regions differ, so the join has something to reject``, so the control cannot be satisfied by a region that is not genuinely crossed.
- Supply the region where the test asserts none was — `an empty region set joins nothing: ContractionF32PlanWitness { .. }`, i.e. the refusal really does flip to an admission on the join alone.
- Give the workgroup-wall test its second operand value — ``assertion `left == right` failed  left: [UnusedValue]  right: [IncompleteComponentSet]``, so both spellings of the wall are real and the check reaches its subject.

### Deliberately not done

The over-broad `staging().len() != 0` arm on `from_program` is untouched, so [`narrow-the-contraction-witness-refusal-to-staging-it-cannot-read`](narrow-the-contraction-witness-refusal-to-staging-it-cannot-read.md) keeps its whole subject. That ticket's own "decide by reading whether staging carrying no combine structure is distinguishable at this layer" now has a read answer: **it is not distinguishable from program scope**, so its documented fallback — repair the prose to state the real predicate — is the correct close, and the join is the route for callers who hold the regions.

### Scope added — `research/reference`

Added as scheduling metadata, not as an expansion of outcome. `docs/research/reference/plan-freedom-sites.md` pins two citations into the exact refusal site this ticket moves, so `make citations` fails without the repair — verified by watching it fail with *"anchor occurs nowhere in crates/tiler-ir/src/program/contraction_witness.rs"* for both. The repair re-points both anchors and adds a dated note recording the one substantive consequence: site 4.11's relaxed reference route is no longer closed to a `CooperativeContraction` region unconditionally. The site's bucket, the twenty-seven headline, and every other bucket are unchanged, and the retired wording that record preserves verbatim was deliberately left untouched — editing it would have desynchronized the correction that quotes it.
