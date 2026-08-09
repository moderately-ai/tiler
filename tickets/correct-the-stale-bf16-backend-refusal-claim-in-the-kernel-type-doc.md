---
id: correct-the-stale-bf16-backend-refusal-claim-in-the-kernel-type-doc
title: Correct the stale Bf16 backend-refusal claim in the kernel type doc
status: done
priority: p3
dependencies: []
related: [re-read-the-bf16-and-elementary-support-rows-against-source, lower-bf16-to-metal]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [doc-claim, bf16, kernel]
---

## The defect (navigation re-read 2026-08-06, coordinator-verified)

`KernelType::Bf16`'s doc comment said "`crates/tiler-metal` refuses this type by name rather than spelling `bfloat`, because it carries no BF16 constant reinterpretation, canonicalization helper, or dispatch route. Verified and emittable are separate claims, and only the first holds here." All three named absences have since landed: the Metal spelling is anchored at `` `Bf16` spells `bfloat`, and it did not before``; `fn canonicalize_bf16_helper` supplies the BF16 helper; and `BinaryOp::Bf16Add` / `BinaryOp::Bf16Multiply` supply the arithmetic.

The former line-only locations in this historical navigation record are retired: their old offsets now describe unrelated source. The dated audit below uses current literal source anchors, each verified with `rg -F`, as the authoritative navigation.

A doc comment is a claim the next worker acts on; this one makes a landed capability look absent — the inverse of the usual overstatement, and just as costly to a reader sequencing BF16 work.

## The work

Rewrite the paragraph to describe current behaviour: the backend spells `bfloat` with its canonicalization helper and dispatch route, and state what boundary actually remains for this type (read `lower-bf16-to-metal`'s outcome and the BF16 support-matrix row for the current residual — the offline-vs-dispatch and profile-row boundaries — rather than asserting from this ticket). Verify each claim at source before writing, per the corpus rule.

## Fact audit and repair (2026-08-08, `c383e86d`)

- **Fact — verified.** The stale comment remained at the source anchor ``A `Bf16`-typed value is now produced``; it said that `tiler-metal` refuses the type by name.
- **Fact — verified.** The three claimed pieces are present: `` `Bf16` spells `bfloat`, and it did not before`` is the emitter's spelling anchor; `fn bf16_literal` reinterprets the exact sixteen-bit constant through `ushort`; `BinaryOp::Bf16Add` and `BinaryOp::Bf16Multiply` carry BF16 arithmetic; and `CanonicalizeBf16Nan` uses its own helper.
- **Fact — false.** The work's `offline-vs-dispatch` residual was already discharged. The BF16 support row states `one dispatched program on the measured macOS row`, and records it as a bounded run rather than general backend support.
- **Fact — imprecise.** The emitter does not own a dispatch route; its own `This is a translation fact and not a dispatch claim` documentation separates source emission from target-family execution evidence.

**Current residual.** The type's supported arithmetic is constant, multiply,
and add, and the dispatched evidence is one program on the declared macOS
Apple9 profile row. It does not establish another target family, conversion,
contraction, or an end-to-end `compile()`/artifact/routing path.

## Closes when

The doc describes what `tiler-metal` does now, with the remaining boundary stated from verified sources, and no reader can conclude the backend refuses the type.

## Outcome (2026-08-08)

`KernelType::Bf16` now says that `tiler-metal` emits `bfloat`, with the exact
`ushort` constant carrier, BF16 multiply/add arithmetic, and separate BF16 NaN
canonicalization helper. It records the repaired bounded execution evidence
without turning that one macOS Apple9 row into a generic backend, target,
operation, conversion, contraction, or routing claim.

### Evidence

- `governed_types_map_to_their_metal_spellings`,
  `a_bf16_kernel_spells_bfloat_at_every_position`,
  `bf16_immediates_are_exact_patterns_reinterpreted_through_ushort`, and
  `the_bf16_canonicalization_helper_matches_the_apple_harness_recognizer`
  passed in `cargo nextest run -p tiler-metal`.
- A deliberate temporary restoration of the retired `refuses this type by name`
  subject made the source check fail with `FAIL: retired backend-refusal subject
  is present in KernelType::Bf16 docs`; the corrected text was restored before
  commit. This demonstrates the prose subject check, not that a Cargo test
  proves documentation truth.
- `cargo fmt --check`; `cargo check -p tiler-ir -p tiler-metal --all-targets`;
  `cargo clippy -p tiler-ir -p tiler-metal --all-targets -- -D warnings`; and
  `RUSTDOCFLAGS='-D warnings' cargo doc -p tiler-ir -p tiler-metal --no-deps`
  passed. The latter package gates apply to the source commit; the ticket-only
  amendment below carries them unchanged.
