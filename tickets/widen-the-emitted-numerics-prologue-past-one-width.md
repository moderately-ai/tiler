---
id: widen-the-emitted-numerics-prologue-past-one-width
title: Widen the emitted numerics prologue past one width
status: review
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

## Outcome — 2026-08-06: the prologue states the guarantee at every emitted width and the identity step landed whole

Branch `tkt/widen-the-emitted-numerics-prologue-past-one-width` over base `fd98cbe5`. The wording, the ten rebaselined goldens, all three recomputed pins, both ledger paragraphs, and the removal of the owner-naming comment are commit `617c22e592c98e7497adb45c07c9c1dc3f5e6d76`; this Outcome is the commit on top of it, and `make full` was run on the tree the two commits carry.

### Fact — the wording

`crates/tiler-metal/src/emit.rs`'s `assemble` emitted

```text
// Carried by these operations under every math mode: every f32 immediate
// is its exact bit pattern, every arithmetic operation is one statement,
// and every NaN test is an integer test over reinterpreted bits.
```

and emits

```text
// Carried by these operations under every math mode: every floating-point
// immediate is its exact bit pattern, every arithmetic operation is one
// statement, and every NaN test is an integer test over reinterpreted bits.
```

Still three lines, and no per-width line: the three properties are width-independent consequences of how the emitter writes operations, which is what the replacement comment in `assemble` now says in place of the deferral note. `every floating-point immediate` is the module's own already-general phrasing — `emit.rs`'s header reads "Floating-point immediates are emitted as exact bit patterns through `as_type`" and names the `uint` and `ushort` carriers — so the header text now agrees with the doc that describes it rather than narrowing it. The prose grew by exactly eleven bytes (`f32` → `floating-point`); the three `push_str` calls are unchanged in count, and `rustfmt` wraps the third onto a continuation line because the widened literal reaches 102 columns.

### Fact — the goldens moved by the header alone

All ten fixtures in `crates/tiler-metal/goldens/`, `f32` and `bf16` alike:

- `git diff --numstat -- crates/tiler-metal/goldens/` is `3 3` for every one of the ten files — 30 insertions, 30 deletions, no other line touched.
- Filtering both sides to their non-header lines is empty for every file: for each golden, `diff <(sed '13,15d' <base version>) <(sed '13,15d' <new version>)` prints nothing. The header block is lines 13–15 in every fixture before and after, so the filter removes exactly the changed lines and nothing else.
- The ten `*_matches_its_golden_source` tests pass, which is what proves the rebaselined text is the emitter's output rather than a hand edit.

### Fact — the pin triple, recomputed on this tree in assertion order

Three runs of `cargo nextest run -p tiler-build -E 'test(the_standard_metal_path_publishes_its_recorded_identities)'`, each taking the failing assertion's `left`, as the ledger at the pin directs; a fourth run is green.

| pin | before | after |
|---|---|---|
| standard Metal artifact identity | `2b0162eb461edeaa8069a022e54057572bf7992970205a5a33f1efee2df896ca` | `17a16aa4d15b35a0eae7e382b9e96ea3fca7c01a5a1c80495600aace20f2e63d` |
| standard Metal cache subject | `8e48d6fbfca8c490c883a557be2c7c5dfcb8264a751c84e585c574d4cd12f186` | `a3d44827bf86b5979f3d79eaf7e9392f997255ae88376edfb6f8f304e51cdfe8` |
| published envelope fixed content | 64,699 bytes | 64,710 bytes |

The third is a cross-check on the other two rather than a third unknown: the emitted comment grew by eleven bytes and the envelope's fixed content grew by eleven bytes, which is what a content change that steps no domain and reframes nothing must do. That is also the first superseded *triple* in the pin's ledger, and its annotation — which previously read that every superseded entry is a pair because the byte count did not exist yet — now distinguishes the one triple from the pairs below it.

### Fact — the ledger paragraph, and a stale value found in it

`docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md` recorded, in the present tense, artifact identity `3f98afa59d9ef46999acc211f2153a7d194444f5be3d0dd946f4128b57674a69` and cache subject `8bca5e7825cdd1dc37da5135b0ea7d6dbd3e9ce1557097f2ee9e60e79fe23d07`. Those are the grid-axis branch's **branch-local** values: the pin's own doc comment lists them as `grid-row-only`, superseded at integration by the composed `886ed671…` pair and by four steps since. So the ledger and the source had already disagreed before this ticket, and the required "ledger agrees with the source" could not be met by editing one number.

Both halves are now correct. The grid-axis paragraph states its two values as what that branch computed, and says they were recomputed when it composed with its sibling. A new paragraph beside it carries what the pins are today — `17a16aa4…`, `a3d44827…`, 64,710 bytes — names `the_standard_metal_path_publishes_its_recorded_identities` as their authority and itself as a mirror, and lists what moved them since the grid-axis step without touching a row in this ledger: the `tiler.kernel-program` v9, v10 and v11 folds, the executable-coverage fold, and this header widening.

### Required evidence

- `cargo nextest run --workspace`: 2,892 tests run, 2,892 passed, 7 skipped, 0 failed.
- The pin test was exercised, not skipped. Re-run as `cargo nextest run --workspace --status-level pass`, whose log carries `PASS [ 0.074s] ( 338/2892) tiler-build metal_plan::tests::the_standard_metal_path_publishes_its_recorded_identities`; the same log carries ten `*_matches_its_golden_source` PASS lines, and no `SKIP` line names either. The default nextest profile prints no PASS lines, which is why the second run exists.
- `cargo test --workspace --doc`: green, no failures across all seventeen reported results.
- `cargo fmt --all --check`, `cargo check`, `cargo clippy --all-targets -- -D warnings`, and `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` for `tiler-metal` and `tiler-build`: all clean. `make full` green on the branch.

### Fact — one stale byte count left alone, and why

`docs/research/artifacts/manifest-fixed-content-growth.md:187` records 64,699 bytes. It is a dated implementation note — "Implemented 2026-08-06 at `562b02e543e177509575d2f50a9a002e1bd78859`" — describing what that commit pinned, so it stays true as written; rewriting its number would falsify its own anchor. It is also `research/artifacts`, which this ticket does not hold. Flagged rather than edited.

### Deliberately not done

No per-width prologue line, no change to the per-dtype subnormal block above it, no emitted semantics, no change to any golden body, and no other pinned identity touched — the workspace run above is the check that no third pin moved quietly.
