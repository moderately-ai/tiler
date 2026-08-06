---
id: record-or-validate-the-fast-honor-pragmas-selection
title: Record or validate the fast-honor-pragmas selection the measured toolchain rejects
status: review
priority: p3
dependencies: []
related: [compile-an-elementary-function-golden-through-the-metal-toolchain, emit-the-contraction-pragma-as-a-declared-metal-realization]
scopes: [implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [metal-aot, toolchain, fail-closed, doc-claim]
claimed_from: todo
assignee: agent-fp-contract
lease_expires_at: 1786038040
---

## The observation (elementary-golden work, 2026-08-06; coordinator-verified at source)

`FpContract::FastHonorPragmas` (`crates/tiler-metal-aot/src/input.rs:527`, spelled `fast-honor-pragmas` at `:538`) is rejected by the measured toolchain: `metal: error: unsupported argument 'fast-honor-pragmas' to option '-ffp-contract='` on the Xcode 27.0 / Metal 32023.921 row. The failure is closed and typed (`ToolFailure`), so nothing is silently wrong — but the enum offers a selection the measured row cannot deliver, and nothing records that anywhere a caller or a reader would find it.

## The question

Whether this is a doc fix (the variant's doc states the measured-row rejection with its boundary — the value may be valid on other toolchain rows, and clang's own `-ffp-contract` accepts `fast-honor-pragmas`, so the enum may be honestly wider than one row) or a validation gap (the input layer should refuse the selection against a target whose measured row rejects it, before the tool run). The fail-closed posture makes the doc fix the likely floor; a validation route would need a per-row capability fact the aot layer may not want to own. Note the emitted-pragma realization ticket is related: if the pragma route lands, `fast-honor-pragmas` is the one `-ffp-contract` value whose semantics interact with source-level pragmas.

## Closes when

The variant's doc states what the measured row does with it (with the boundary and the reproducing invocation), and either a validation is added with a test watched refusing, or the derivation for why tool-time failure is the right layer is recorded at the variant.

## Outcome (2026-08-06)

**Measurement — the row.** Apple M4 Max, macOS 27.0 (build 26A5388g), Xcode 27.0 (27A5228h), `Apple metal version 32023.921 (metalfe-32023.921)`, macOS SDK 27.0 (26A5388f). This is the coordination host; every measurement below is compile-time behaviour, not timing, so the M3 routing rule does not apply.

**Measurement — the rejection reproduces, and the admitted set is exactly three values.** Through the driver's own invocation shape:

```sh
xcrun -sdk macosx metal -target air64-apple-macos14.0 -std=metal3.1 -O2 \
  -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise \
  -ffp-contract=fast-honor-pragmas -c kernel.metal -o kernel.air
# metal: error: unsupported argument 'fast-honor-pragmas' to option '-ffp-contract='  (exit 1)
```

`off`, `on`, and `fast` exit 0; `fast-honor-pragmas` and clang's `on-vector` are rejected with the same diagnostic shape, identically under `-std=metal2.4`, `3.0`, `3.1`, `3.2`, and `4.0`. Not a language-revision restriction. Only one Metal toolchain is installed on this host, so no second row was reachable to compare.

**Measurement — the provenance is the driver's option table, not a front-end gap.** `metal --help` *documents* `fast-honor-pragmas` as an accepted value; the text is clang's verbatim, carrying clang's own "diectated" typo, which is the most likely source of the variant. `metal -###` shows the driver forwarding accepted values to `-cc1` unmapped. `-Xclang -ffp-contract=fast-honor-pragmas` bypasses the driver's parser, is accepted, and emits `fmul contract`/`fadd contract`; `-Xclang -ffp-contract=off` emits bare `fmul`/`fadd`, so the `-Xclang` route is demonstrably consumed and not ignored. The front end implements the value; the driver refuses to spell it.

**Measurement — the variant is also semantically redundant on this row.** Under `-ffp-contract=fast`, the two-statement multiply-add emits `fmul contract`; the same source carrying `#pragma clang fp contract(off)` (and the `#pragma METAL fp contract(off)` spelling) emits a bare `fmul` under identical flags, clean through `-Wall -Werror`. So `fast` on this driver already fuses across statements *while honouring source pragmas* — which is precisely what `fast-honor-pragmas` names. This reproduces and sharpens the pragma measurement in `docs/research/apple-targets/numerical-behaviour.md` (finding at "a source-level pragma does control contraction"), and it means the value is not merely unreachable but unneeded.

**Elimination.** Pre-tool validation was discarded: `NumericalRealization::new` is a `const` infallible constructor, so refusing there means making a public constructor fallible (a surface decision that is Tom's) *and* giving the aot layer a hardcoded toolchain-version-to-accepted-values table — a new per-row capability authority that nothing re-measures and that goes stale silently, inverting the layer's current independence from target facts. That is the brief's stop condition, and it buys nothing: the failure is already closed and typed (`DriverError::ToolFailure` at `CompileStage::Metal`), so no silently-wrong result exists to prevent. Removing the variant was also discarded here — it is a public enum surface change, which is Tom's, and the ticket authorized recording rather than redesign. What landed is the brief's named middle: deprecation-by-doc with a trigger, plus a watcher.

**What landed.**

- `crates/tiler-metal-aot/src/input.rs` — the `FpContract` enum doc now states that only three of the four values are driver-selectable; `FastHonorPragmas` carries the reproducing invocation, the driver-versus-`-cc1` boundary, the `-Xclang` evidence and why it is deliberately *not* offered as a selection (it bypasses driver option validation and `compile_flags` is the exact text reaching artifact identity), the redundancy with `Fast`, and the named trigger. `Fast` gained the measurement that it honours source contraction pragmas despite its own help text saying otherwise. `contracts_across_statements` records that its `true` for `FastHonorPragmas` answers the semantic question and is not a claim that the selection compiles.
- `crates/tiler-metal-aot/src/driver.rs` — `fast_honor_pragmas_is_rejected_by_the_metal_driver` compiles a control differing only in the contraction value (so a failure is the flag, not the fixture), then asserts the typed `ToolFailure` at the `Metal` stage. It self-skips when no toolchain resolves, matching the sibling real-toolchain tests, and asserts the typed error and stage rather than Apple's diagnostic wording.

**The check was watched failing.** Substituting `FpContract::Fast` for `FpContract::FastHonorPragmas` in the asserted slot makes the test fail at `driver.rs:755` with the trigger message ("`metal` accepted -ffp-contract=fast-honor-pragmas, so the trigger recorded on FpContract::FastHonorPragmas has fired…"), exit 100. Restored and re-run green.

**The related pragma ticket is not foreclosed, and gains a fact.** `emit-the-contraction-pragma-as-a-declared-metal-realization` can read here that `-ffp-contract=fast` already honours both pragma spellings on this row, so the pragma route needs no `fast-honor-pragmas` selection and the two values are observationally identical for it.

**Not filed, deliberately, and flagged for the coordinator to overrule.** No deferred ticket was raised for "remove the variant". The trigger is carried by a gate-time test rather than a board entry, which fires louder and more reliably than a deferral sweep; a board item would restate what the test already enforces. If the coordinator wants the public-surface question ("should `FpContract` carry a value no driver accepts?") visible to Tom on the board rather than only in the doc, that is a one-line ticket to add.

**Measurement boundary.** Everything above is one toolchain row on one host. Nothing here is evidence about any other Xcode, Metal, or SDK version, about the runtime (`MTLCompileOptions`) compiler — which has no `-ffp-contract` counterpart at all — or about whether the `-Xclang` route is stable or supported. No device was dispatched; these are compile-acceptance and emitted-IR observations, not execution results.

**Commands run.** `cargo fmt -p tiler-metal-aot -- --check`; `cargo check -p tiler-metal-aot --all-targets`; `cargo nextest run -p tiler-metal-aot` (62 passed, 0 skipped); `cargo test -p tiler-metal-aot --doc` (4 + 3 passed); `cargo clippy -p tiler-metal-aot --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-metal-aot`; `tkt lint`; `git diff --check`; `tkt guard`.
