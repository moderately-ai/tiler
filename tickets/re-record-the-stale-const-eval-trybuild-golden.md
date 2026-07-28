---
id: re-record-the-stale-const-eval-trybuild-golden
title: Re-record the stale const-eval trybuild golden in tiler-ir
status: todo
priority: p1
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: []
paths: []
tags: [implementation, toolchain]
---
`crates/tiler-ir/tests/shape-evidence/fail/shape_array_rank_limit.stderr` no longer matches what the pinned compiler emits, and the mismatch fails the Rust sub-gate's `cargo test --workspace` phase for every branch.

**Measurement — environment.** `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, commit `eff8269f797067c30555e77f160ec84c0ed15cd9`, host `aarch64-apple-darwin`, LLVM 22.1.8, selected through `rustup run nightly-2026-07-19` — the toolchain `rust-toolchain.toml` pins. `trybuild` 1.0.118.

**Measurement — the failure is on the base, not on a branch.** Reproduced in a detached worktree at `06af0c62594b93e3070c17e2487fafe38e129208` with `git status --porcelain` empty:

```sh
cargo nextest run -p tiler-ir -E 'test(shape_evidence_contract)'
```

`1 of 7 tests failed`; the six other cases pass. Only `tests/shape-evidence/fail/shape_array_rank_limit.rs` mismatches.

**Fact — the whole difference is one rendering block.** The recorded golden renders the `E0080` const-eval panic's location as

```text
 --> $RUST/core/src/panic.rs
  |
  = note: evaluation of `tiler_ir::shape::Shape::from_dims::<4097>::{constant#1}` failed here
```

and the compiler now renders it as the source line plus a caret span carrying the same sentence:

```text
 --> $RUST/core/src/panic.rs
  |
  |         $crate::panicking::panic_fmt($crate::const_format_args!($($t)+));
  |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ evaluation of `tiler_ir::shape::Shape::from_dims::<4097>::{constant#1}` failed here
```

Every other line of the diagnostic — the error code, the message, the `::: src/shape.rs` context, the `erroneous constant encountered` note, and the instantiation note pointing at the test's own line 4 — is identical. The claim the fixture exists to make is unchanged: `Shape::from_dims::<4097>` fails at compile time with `shape array exceeds MAX_SHAPE_RANK`.

**Inference — the golden was recorded on a different rustc build than the one now installed for this pin.** A dated nightly channel is immutable, the source and the golden are both unmodified since the commit that recorded them, and a compiler is deterministic, so the same three inputs cannot produce two outputs. Something re-resolved the pin's local artefact, or the golden was recorded against a neighbouring build. AGENTS.md's toolchain-provenance rule is what this ticket exists to satisfy: the exact resulting component must be recorded, and any measurement blocked by the mismatch rerun.

**Deliberately not blessed by the ticket that found it.** `implement-first-profile-numerical-policies` reproduced this at the base and left it alone. A `.stderr` is a positive claim about what a compiler emits; overwriting one is a decision about which compiler build the repository's evidence is recorded against, not a mechanical repair, and doing it inside an unrelated numerics change would have hidden the provenance question this ticket records.

## Closes when

The exact rustc build the repository gate resolves for `nightly-2026-07-19` is confirmed and recorded, the golden is re-recorded against it — `TRYBUILD=overwrite cargo nextest run -p tiler-ir -E 'test(shape_evidence_contract)'`, then the diff read to confirm only the rendering block moved and no claim did — and `make full` passes on an otherwise clean checkout of `main`. (Citation corrected at landing: the Python gate this ticket originally named was retired by `e197176`.) If instead the installed toolchain proves to differ from the pinned one, the pin is restored and the golden is left as recorded.
