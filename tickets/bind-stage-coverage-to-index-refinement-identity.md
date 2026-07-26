---
id: bind-stage-coverage-to-index-refinement-identity
title: Bind kernel-program stage coverage to the index refinements it rests on
status: todo
priority: p1
dependencies: [correct-adr-0071-retained-lower-layer-identity-cardinality]
related: [bind-the-scheduled-region-to-the-verified-index-region-identity]
scopes: [implementation/ir, implementation/compiler, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, identity]
---
**Fact — a verified index region's identity reaches no verified product.** `crates/tiler-compiler/src/legality.rs::emit_region` builds and verifies a `VerifiedIndexRegion` per semantic occurrence and `IndexRefinement` carries its `CanonicalIndexRegionIdentity`. The exact check is `grep -rn 'CanonicalIndexRegionIdentity' crates/tiler-ir/src/schedule crates/tiler-ir/src/kernel crates/tiler-ir/src/program crates/tiler-artifact/src crates/tiler-metal/src`, which returns nothing.

**Sharpened 2026-07-25 while landing `correct-adr-0071-retained-lower-layer-identity-cardinality`, because the original wording understates what already exists and a worker who inherits it will build the wrong thing.** This ticket first said the identity's "only consumer" is `pipeline.rs::refinement_label`. That is not accurate inside `legality.rs`, where the identity is already folded into two complete compiler-owned identities: `encode_content_identity` folds it into `RefinementContentIdentity`, `encode_occurrence_identity` folds that into `IndexRefinementIdentity`, and `tiler_ir::index::ScalarAuthorityEvidence` binds its region-bound receipt to the same bytes. The accurate statement is that the chain *terminates* there. `pipeline.rs` retains the `IndexRefinement` in `CompletePlans` and consumes it in exactly two places, both explain — `record_refinement`, which renders eight trailing bytes through `refinement_label` into a presentation handle, and `record_numerical_equivalence`, which reads only the resolved provider. The artifact plan records `ResolvedLowering::providers()`, deduplicated lowering provenance, and no region or refinement identity. So the work here is to carry an identity that already exists and is already complete into a verified product, not to derive one.

**Fact — the program layer is where the cardinality already fits.** `crates/tiler-ir/src/program/model.rs` gives each stage a `coverage: Vec<SemanticOccurrence>`, folded into program identity at its encoder. Coverage is already one-stage-to-many-occurrences, which is the same shape as one scheduled region to many refined index regions. A stage that named the *refinement identity* alongside the occurrence would state which verified index region proves it implements that occurrence, rather than only which occurrence it claims.

**Why not on the schedule.** `bind-the-scheduled-region-to-the-verified-index-region-identity` records the evidence: a scheduled region stands over several refined regions, so a single retained identity on it could name only one of them. That ticket's outcome has the full argument.

## Scope

Carry the refinement identity from `IndexRefinement` into the verified kernel program's stage coverage and into program identity, so a program names the exact verified index regions its stages rest on. Two decisions this ticket owns: whether coverage becomes a pair type or gains a parallel vector — the former keeps an occurrence and its evidence inseparable — and whether a stage with a recorded proof gap rather than a refinement is representable, since `pipeline.rs` already distinguishes `OccurrenceEvidence::Refined` from a gap and collapsing them would let an unproved stage look proved.

Changing the stage coverage type is a public-boundary change in `tiler_ir::program`, so the exact signature is Tom's to accept.

## Closes when

A verified kernel program names the refinement identity behind each covered occurrence, program identity separates two programs that differ only in which verified index region proves a stage, a recorded proof gap stays distinguishable from a refinement, and `make full` passes.

## Scope finding 2026-07-25: this is not landable inside `implementation/ir` + `implementation/compiler`

Claimed by `agent-api` on base `6fae4f3`, scoped, **not implemented**, and released with what the scoping established. Nothing below is a measurement of an attempted change — no code was written for this ticket — so treat the blast radius as enumerated rather than compiled.

**Fact — a second encoder outside both declared scopes folds stage coverage into a different identity.** `crates/tiler-artifact/src/program/model.rs::stage_key` builds an artifact-program stage key from exactly the same two ingredients as `tiler_ir::program::model.rs::stage_key` — the bound kernel's canonical identity, then the covered occurrences — under its own domain tag. The exact check is `grep -n "STAGE_KEY_DOMAIN" crates/tiler-artifact/src/program/model.rs crates/tiler-ir/src/program/model.rs`, which shows `b"tiler.artifact-program.stage.v1\0"` beside `b"tiler.kernel-program.stage.v1\0"`.

The tags differ, so these are two deliberately separated subjects and **not** a duplicated-authority defect. That is precisely why they matter here: they are independent encoders that happen to agree today about what a stage *is*. Adding refinement evidence to coverage in `tiler-ir` alone changes one of them and not the other, and the consequence is not cosmetic — the artifact's stage key would stop distinguishing two stages the shared IR now distinguishes, at the layer where dedup and caching actually happen. Whether refinement evidence enters artifact-program stage identity is therefore a decision this ticket forces and cannot defer, and it belongs to `implementation/artifact`.

**Fact — the builder signature change breaks three sites outside both declared scopes.** Changing `KernelProgramBuilder::push_stage`'s coverage parameter breaks `crates/tiler-artifact/src/program/mod.rs:208` (a doctest, so a compile failure in the Rust gate's doctest phase rather than in `cargo check`) and `crates/tiler-artifact/src/program/tests.rs:186` and `:347`. The exact check is `grep -rn "push_stage" crates/ prototypes/ spikes/`.

**Note for whoever reads that grep.** It also matches `ArtifactProgramBuilder::push_stage`, a different builder on a different type with the same method name. The three sites above are the ones that construct a `tiler_ir` kernel program as a fixture; do not conclude from the method name alone which builder a hit belongs to.

**Consequence for scheduling.** Add `implementation/artifact` to this ticket's scopes before dispatching it, and give it to a worker holding all three. At the time of this finding `implementation/artifact` was held by `prototype-artifact-family-delivery` (in-progress) and `carry-the-metal-payload-in-an-artifact-envelope` (review), so the scope was contended; check `tkt guard` again rather than trusting that snapshot.

**Unverified.** Whether the change also moves any `tiler-metal` golden was not established. `crates/tiler-metal/goldens/*.metal` embed the *kernel* identity digest and the *scheduled region* identity digest, neither of which folds program stage coverage, so the expectation is that it does not — but that is an inference from what the goldens contain, not a compiled result.

## Base correction 2026-07-25: `push_stage` moved, so this must not be based on `568682b`

`complete-program-identity-with-abi-guards-and-routing` landed on `tkt/complete-program-identity-with-abi-guards-and-routing` and changed the same function this ticket changes. `KernelProgramBuilder::push_stage` now takes four parameters — `(&VerifiedKernel, &[SemanticOccurrence], &[StageAccess], StageLaunch)` — and `StageAccess` gained an `accessible_bytes: AbiExprId` field. Basing this ticket on `568682b` and changing the coverage parameter would produce two edits to one signature that no merge can reconcile mechanically. **Base it on the merged result of that ticket, not on `568682b`.**

Three consequences for what is written above.

**The declared-scope finding is applied.** `implementation/artifact` is now on this ticket's `scopes`. The two contended holders named above — `prototype-artifact-family-delivery` and `carry-the-metal-payload-in-an-artifact-envelope` — both show expired claims as of this note; re-check `tkt claims` rather than trusting that.

**The `push_stage` call-site count is now larger and its distribution has changed.** The three sites named above are still the artifact ones, but `crates/tiler-ir/src/program/tests.rs` grew a `wire_two_stage_storage` helper and a `FixtureAbi` fixture, and the `read`/`write` helpers now take an ABI handle. The reproducible check is unchanged — `grep -rn "push_stage" crates/ prototypes/ spikes/` — but run it against the new base, because its answer moved.

**The two `stage_key` encoders are still two, and still deliberately so.** That ticket folded the entry ABI, the applicability guard and the routing-commit contract into *program* identity and bumped `tiler.kernel-program.v1` to `v2`, but left `tiler.kernel-program.stage.v1` and `tiler.artifact-program.stage.v1` both at `v1` and both folding exactly the bound kernel's identity and the covered occurrences — the launch geometry went beside the stage key in the program encoding rather than into it, so the cross-reference key would keep meaning what it meant. The decision this ticket forces, whether refinement evidence enters artifact-program stage identity, is therefore untouched and still owned here.
