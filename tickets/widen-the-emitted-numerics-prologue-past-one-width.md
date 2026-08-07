---
id: widen-the-emitted-numerics-prologue-past-one-width
title: Widen the emitted numerics prologue past one width
status: in-progress
priority: p2
dependencies: [lower-bf16-to-metal]
related: [raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells, declare-the-bf16-rows-on-the-authoritative-metal-profile]
scopes: [implementation/metal, implementation/build, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, identity, bf16, documentation]
claimed_from: todo
assignee: agent-numerics-prologue
lease_expires_at: 1786071930
---
## User-visible outcome

The emitted Metal provenance header states the three carried numerical properties at every width the backend emits, so a reader who keeps only the generated source of a `bf16` module learns what its immediates are, instead of reading a sentence about `f32` immediates the module does not contain.

## Why this is not a comment edit

**Fact, at `lower-bf16-to-metal`.** `crates/tiler-metal/src/emit.rs`'s `assemble` writes three fixed lines beginning `// Carried by these operations under every math mode: every f32 immediate`. All three properties — exact-bit-pattern immediates, one arithmetic operation per statement, an integer-only NaN predicate — hold at `bf16` exactly as at `f32`, and the emitter now emits `bfloat` constants through the `ushort` carrier. The sentence is therefore narrower than the guarantee it describes, and silent about a width the backend emits.

**Measurement, 2026-08-05, on the `lower-bf16-to-metal` branch at base `55652b2b`.** Rewording those three lines to `every floating-point immediate …` and rebaselining the six `f32` goldens turned `cargo nextest run --workspace` red at exactly one test: `tiler-build metal_plan::tests::the_standard_metal_path_publishes_its_recorded_identities`, `crates/tiler-build/src/metal_plan.rs:1251`, standard Metal artifact identity `d22c0d11f8486a15b3df7651feee543eb5d0f8d398a7eb9047ae45b15f9ce832` → `5c366e94094ae958d1a741c8288701b1ec46c5e26948635a1e5473f76e199753`. 2,688 of 2,689 tests passed.

**Inference.** The emitted source is content of the standard Metal artifact, so the header's wording is inside an identity domain. A wording change is an identity-domain step: the pin at `metal_plan.rs` moves, the cache-subject pin beside it must be recomputed on the tree the step lands into, and the ledger paragraph in `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md` that records the current pins has to move in the same commit. `lower-bf16-to-metal` holds neither `implementation/build` nor `research/target-profiles`, and its own required evidence includes leaving the F32 goldens unchanged, so it reverted the wording and filed this instead of taking half a step.

**Fact — a second branch is already moving these pins.** `raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells` records the same two pins moving to `3f98afa59d9ef46999acc211f2153a7d194444f5be3d0dd946f4128b57674a69` and `8bca5e7825cdd1dc37da5135b0ea7d6dbd3e9ce1557097f2ee9e60e79fe23d07`. Two branches each rebaselining one pinned identity can both be green and still not compose, so this ticket must sequence after that one lands and recompute on the merged tree rather than copying either side's value.

## Implementation keys

- Reword the three `assemble` lines so the claim covers every emitted width. Do not add a per-width line: the properties are width-independent, and the per-dtype subnormal block above already exists for the facts that are not.
- Rebaseline every golden in `crates/tiler-metal/goldens/` in the same change. Their bodies do not move; only the three header lines do.
- Execute the identity step completely or not at all: move the pins at their owning layer, recompute each on the tree the step lands into rather than transcribing a value from a branch, and enumerate every moved pin in the report.
- Update the ledger paragraph that records the current standard Metal artifact identity and cache subject.
- Remove the comment in `assemble` that names this ticket as the owner of the step.

## Required evidence

- The reworded header appears in every golden and the bodies are byte-identical to their current ones apart from those three lines.
- Every moved pin is enumerated with its before and after value, each recomputed on the merged tree.
- `cargo nextest run --workspace` is green, and the run is shown to have exercised `the_standard_metal_path_publishes_its_recorded_identities` rather than skipped it.
- The ledger's recorded pins and the pins in the source agree after the change.

## Closes when

The prologue states the guarantee at every emitted width, every golden is rebaselined, the identity step is complete with each moved pin enumerated and recomputed on the merged tree, and the ledger agrees with the source.

## Graph maintenance

- Depends on `lower-bf16-to-metal`, which is what makes the wording wrong rather than merely narrow — before it there was one emitted width.
- Sequence after `raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells` rather than beside it; both move the same two pins.
- This changes no emitted semantics and no compiled behaviour. A reviewer should expect the AIR and the linked libraries to be unaffected, and the identities to move purely because the source bytes are content.
