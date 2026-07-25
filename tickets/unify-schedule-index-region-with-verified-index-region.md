---
id: unify-schedule-index-region-with-verified-index-region
title: Unify schedule bounded index region with tiler_ir::index::VerifiedIndexRegion
status: done
priority: p2
dependencies: []
related: [prototype-scheduled-region-ir]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, refactor]
---
`tiler_ir::schedule` introduced its own bounded `IndexRegion` (iteration domain,
accesses, bounds/ownership proofs, scalar program, numerical realization) rather
than composing the existing `tiler_ir::index::VerifiedIndexRegion`. Two
"index region" concepts now coexist in one crate, with parallel bounds/ownership
proof descriptors and their own witness newtypes.

Unify them so a scheduled region references a `VerifiedIndexRegion` (or a shared
bounded projection of it) instead of duplicating the description. The intrinsic
schedule verifier must keep proving schedule-specific facts (launch/domain
coverage, tail legality, reduction topology agreement) but should not re-derive
the index-region invariants the `index` module already establishes.

Deferred deliberately by `prototype-scheduled-region-ir` because the bounded slice
did not need the full `VerifiedIndexRegion`; record whether the unified form must
preserve the schedule module's current canonical identity bytes or may re-baseline
them (identity is currently a pure function of the schedule's own descriptors).

Also settle here the descriptor-accessor style, flagged and deliberately accepted
at review time: `tiler_ir::schedule`'s leaf descriptors (`IndexRegion`, `Access`,
`KernelSchedule`, and the proof descriptors) expose `pub` fields, whereas the
sibling `tiler_ir::index` uses view accessors. This is not a soundness gap —
opacity is enforced at `VerifiedScheduledRegion`, and descriptors are only
reachable through a `&ScheduledRegion` — but it is an inconsistency between two
modules of the same crate. Decide deliberately: adopt view accessors while
unifying (preferred if the unified form needs field-level invariants), or record
why the pub-field value-data form is the intended style for schedule descriptors.

## Outcome

**The accessor question is settled and recorded. The unification is not done, and this ticket recommends against doing it: its premise does not survive reading the two modules, and carrying it out would cross an accepted architectural guardrail. Status is `awaiting-decision` because declining a ticket's stated charter is not this ticket's call.**

### The two "index regions" are not two descriptions of one thing

**Fact (inspected source, base `f286289`).** They model different objects at different layers.

`tiler_ir::index::VerifiedIndexRegion` (`index/model.rs:207`) holds `dimensions`, `tensors`, `expressions`, `accesses`, `operations`, `values`, `outputs` — an SSA symbolic region whose scalar program is an operation graph over a registry-governed, extensible vocabulary (`ScalarOpKey`, resolved through `FrozenScalarRegistry`), and whose accesses carry symbolic index expressions.

`tiler_ir::schedule::IndexRegion` (`schedule/model.rs:178`) holds an `iteration_shape`, a fixed `Vec<Access>` of one read and one owning write, witness-identified proofs, and a `ScalarProgram` that is a **closed enum of three recognized shapes** — `MultiplyThenAdd`, `StrictSerialSum`, `FusedMultiplyAddSerialSum` — each a fixed-width bit-pattern record. Its `LogicalAccess` is likewise closed: `LinearIdentity` or `ReductionContributor`.

One is open and symbolic; the other is closed and enumerated. A scheduled region referencing a `VerifiedIndexRegion` would have to either admit the whole open vocabulary into the physical layer — which is what makes a closed `ScalarProgram` checkable against a launch geometry — or carry a bounded projection that re-states the closed forms anyway, which is the current design with an indirection added.

**The proof descriptors are not parallel either.** The ticket describes "parallel bounds/ownership proof descriptors". Reading both: `index::BoundsProofView` is `VacuousEmptyDomain | Interval | Exhaustive { points }` — how in-boundedness was *established* over a symbolic access. `schedule::BoundsProofKind` is `LinearRange { element_count } | ReductionDomain { input_shape, output_shape, axes, order }` — the concrete domain *structure* a launch must cover. Neither variant set is a renaming of the other, and they answer different questions. The witness newtypes are not parallel at all: `schedule` has `BoundsWitnessId`/`OwnershipWitnessId` because its proofs are separately listed and referenced by ID, while `index` attaches its proof inline to the access (`index/model.rs:117`) and has no witness newtype to duplicate.

### The compile path does not have a `VerifiedIndexRegion` to compose

**Fact.** `crates/tiler-compiler/src/physical.rs::pointwise_region`, `reduction_region`, and `fused_region` each build a `ScheduledRegion` by struct literal directly from a `VerifiedTargetRequest`. No `VerifiedIndexRegion` is constructed, borrowed, or referenced anywhere in `physical.rs`. Reproducible as `grep -n VerifiedIndexRegion crates/tiler-compiler/src/physical.rs`, which returns nothing; the type appears only in `legality.rs` (semantic refinement and oracle checking) and `capability.rs`.

So unification is not a refactor of `tiler-ir`. It would require the physical planner to first produce a refined index region and then project it — a change to the compile path in `implementation/compiler`, which this ticket does not hold, and a reordering of two stages that currently run on different axes for different purposes.

### It would cross a guardrail

AGENTS.md: "Keep semantic/logical IR, symbolic access relations, fusion alternatives, physical schedules, structured kernel IR, artifact programs, and runtime state distinct. Do not build a universal IR or densify physical choices into the logical graph." Symbolic access relations and physical schedules are named as separate items in that list. Two types called "index region" in one crate is a naming collision worth fixing; it is not evidence that the layers should merge.

**Recommendation.** Decline the unification and close the naming collision instead — rename `tiler_ir::schedule::IndexRegion` to something that names what it is, such as `ScheduledDomain` or `PhysicalIndexDomain`, so the crate stops having two `IndexRegion`s. That is a contained rename inside `implementation/ir` plus its `tiler-compiler` construction sites, and it removes the confusion that motivated this ticket without merging two layers the guardrail separates. If Tom prefers the unification regardless, it needs a ticket that declares `implementation/compiler` and states which stage produces the refined region.

### Identity question, answered

**Fact.** `schedule/model.rs::encode_identity` is a pure function of the schedule's own descriptors: the `tiler.schedule.v1` domain separator, the iteration shape, the accesses, the bounds proofs, the ownership proof, the scalar program, the numerical realization, and the kernel schedule — with the transient `RegionId` deliberately excluded. Nothing in it reads an index-module identity. So a unified form **would** rebaseline `CanonicalScheduledRegionIdentity`, because the encoded subject would change from the closed descriptors to whatever projection replaced them. Under the recommendation above the question is moot; a rename alone does not touch the encoding, since no type name appears in the bytes.

### Accessor style, settled: `pub` fields are correct here, and it is now recorded

The apparent inconsistency dissolves on reading: **the two modules' descriptors sit on opposite sides of a verification boundary.**

`tiler_ir::index`'s public type *is* the verified product, so it is opaque and hands out views. This module's verified product is `VerifiedScheduledRegion` (`schedule/model.rs:327`), which is **equally opaque** — private fields, a `pub(super) fn new`, and three read-only accessors. `ScheduledRegion` is the *unverified proposal* passed to `ScheduledRegionBuilder::from_region`, and the read-only borrow `VerifiedScheduledRegion::region` returns. Comparing `ScheduledRegion`'s fields against `VerifiedIndexRegion`'s accessors compares an input to an output; the honest comparison is verified-to-verified, and both are opaque.

The conditional the ticket attached to accessors — "preferred if the unified form needs field-level invariants" — does not fire, because no unification is recommended and because no schedule descriptor maintains a field-level invariant: each is a closed enum, a `Shape`, or a fixed-width bit pattern, and every invariant relating them is a whole-region property the intrinsic verifier proves at `build`. Accessors would add ceremony without moving a check earlier.

Struct-literal construction also earns something accessors would cost: adding a descriptor field is a compile error at every construction site, so a new physical fact cannot be silently defaulted by a producer that has not been taught about it. A constructor function would have to be exhaustive to match that, which is the same thing with more typing.

This reasoning is now recorded in the `tiler_ir::schedule` module documentation under "Why the leaf descriptors expose fields", so the next reader finds the decision rather than the apparent inconsistency.

### Split out

`finish-consolidating-tiler-ir-length-framing` — found while reading `encode_identity` for the identity question. `crates/tiler-ir/src/identity.rs` was meant to be the one definition of canonical length framing, and three private copies remain in the crate. `schedule/model.rs` has four raw `as u64` narrowing casts, which is the exact form `identity.rs` documents as the hazard it removed. Latent rather than live on the gate's 64-bit profiles, and deliberately not fixed here so that a unification ticket does not silently rebaseline identity encoders.

## Resolved by the coordinator — 2026-07-25

**Do not unify. Rename the schedule type instead.** Auto-resolved rather than escalated, because only one option survives the architectural guardrails and a question with one admissible answer is not a decision.

The agent that read both modules in full found the ticket's premise does not survive: `index` is an open SSA symbolic region over a registry-governed vocabulary, `schedule` is a closed enumerated physical descriptor. Their proof descriptors are not parallel because they answer different questions, and `physical.rs` never holds a `VerifiedIndexRegion` to compose with.

`AGENTS.md` requires symbolic access relations and physical schedules to stay distinct representations. Merging them crosses that guardrail to remove a name collision, which the glossary sweep has already shown is fixable by naming. The accessor half is separately settled and recorded: the two sit on opposite sides of a verification boundary, so comparing `ScheduledRegion`'s fields to `VerifiedIndexRegion`'s accessors compares an input to an output.

Remaining work is the rename, which carries no decision.
