---
id: correct-the-discharged-bf16-target-profile-claim-in-compiler-docs
title: Correct the discharged BF16 target-profile claim in the compiler's capability documentation
status: closed
priority: p3
dependencies: []
related: [refresh-the-reduced-precision-float-matrix-row-after-the-bf16-gate-landings, declare-the-bf16-rows-on-the-authoritative-metal-profile]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [documentation, bf16, compiler, numerics]
closed_reason: obsolete
closed_note: Current policy and test docs already carry the corrected BF16 reasoning; no source edit remains.
---
## User-visible outcome

`operation_capabilities`' item documentation stops telling a reader that no target profile can speak about BF16, because one now does. A reader deciding whether to add a BF16 capability row gets the reason that is still true rather than one the 2026-08-02 profile landing discharged.

## The exact drift

**Fact.** `crates/tiler-compiler/src/policy.rs`'s doc comment on `operation_capabilities` ends: "adding a row would widen `is_consumable`'s union for an operation no target profile can even state a numerical contract for." Reproduce with `grep -n "no target profile can even state" crates/tiler-compiler/src/policy.rs`.

**Fact.** `crates/tiler-build/src/metal_declaration.rs` declares BF16 `Dispatchable` on `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` and projects the measured sign-preserving flush into complete exclusive BF16 input and result subnormal tables, through `ScalarArithmetic::new(ArithmeticType::Bf16, Bf16::resolved_type())`. A target profile therefore does state BF16 numerical facts.

**Fact — the same file already carries the corrected reading, in its test module.** The doc comment on `UNPLANNED_OPERATIONS` says "A BF16 numerical row *is* statable on a target profile now that `ScalarArithmetic` derives the arithmetic/value-type association from the registered descriptor, and that does not change this: a subject a profile can speak about is not an operation this build can plan." That is the argument the item doc should be making.

**Inference.** The item doc and the test doc were updated at different times and now disagree about a fact. The source wins over both, and the reason the BF16 rows carry no capability entry is the one the test doc states — no arithmetic in this build realizes a BF16 operation — not the discharged claim about target profiles.

## Implementation keys

- Correct the one clause in the item doc so it names the reason that survives: a registered operation this build cannot plan consumes no numerical freedom, and a subject a profile can speak about is not an operation the build can realize.
- Do not change `UNPLANNED_OPERATIONS`, the capability table, or any check. This is a comment defect, and the behaviour it describes is correct.
- Check the neighbouring BF16 comments in the same crate against the same fact while there — `crates/tiler-compiler/src/explain.rs` carries a rebaseline note reading "no target declaration names bf16", which the same landing discharges.

## Closes when

Neither doc comment asserts that no target profile can state a BF16 numerical fact, both give the reason that is still true, and the capability table and its checks are unchanged.

## Repository audit and closure — 2026-08-09

**Original Facts: false as current work; historical only.** The complete current `operation_capabilities` documentation, anchored at `The ground moved and the conclusion did not`, already says a BF16 program is recognized and planned, explicitly marks the old “no target profile can even state” sentence false, and gives the surviving width-specific evidence reason for keeping BF16 out of the capability table. The `UNPLANNED_OPERATIONS` test documentation says the same thing and `every_unplanned_operation_is_registered_and_consumes_no_dimension` still checks the rowless population. A full-file search of `crates/tiler-compiler/src/explain.rs` finds no `no target declaration names bf16` note to repair.

The source correction landed before this ticket was revisited, so changing `policy.rs`, `explain.rs`, the capability table, or its tests would revert or duplicate correct work. This ticket closes as obsolete with a ticket-only record; `implementation/compiler` is removed because no compiler edit remains.

## Fact audit — 2026-08-10

**Correction — 2026-08-10.** The 2026-08-09 closure overstated completeness. Primary defects this ticket named are already gone: `operation_capabilities` quotes the old “no target profile can even state” sentence only to mark it false (anchor `The ground moved and the conclusion did not`), and `crates/tiler-compiler/src/explain.rs` carries no `no target declaration names bf16` rebaseline note. One residual discharged claim of the same family remains live in `UNPLANNED_OPERATIONS` documentation: the gather-contrast paragraph still says “The BF16 rows are unplanned because no target has been asked about their format,” which contradicts the corrected intro in the same const doc and Fact 2 (Metal `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` does state BF16 numerical facts). Reproduce: `rg -n 'no target has been asked about their format' crates/tiler-compiler/src/policy.rs`. Parent stays `closed` / `obsolete` for the original primary wording; residual is one comment clause in `policy.rs` (capability table, keys, and tests unchanged). Optional narrow remainder: remove that residual discharged BF16 target-profile clause from the `UNPLANNED_OPERATIONS` doc only.
