---
id: pin-the-serial-sum-kernel-identitys-crossing-of-the-opaque-identity-bound
title: Pin the serial sum kernel identity's crossing of the opaque identity bound
status: todo
priority: p2
dependencies: []
related: [date-or-regenerate-the-six-kernel-identity-lengths-in-the-artifact-abi, bound-the-backend-entry-key-by-the-identity-it-carries]
scopes: [implementation/conformance, implementation/artifact, implementation/metal-aot, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, identity, testing]
---

The argument that changed `BackendEntryKey`'s bound is asserted by nothing, and the one test that looks as though it asserts it compares a fabricated byte vector against a constant. Filed by [`date-or-regenerate-the-six-kernel-identity-lengths-in-the-artifact-abi`](date-or-regenerate-the-six-kernel-identity-lengths-in-the-artifact-abi.md), which regenerated the figures in `docs/artifact-abi.md` and could not add the assertion because `crates/**` is outside its scope.

## Facts

**Verified at `68ba010ab117fb6840b5473154e2fbf83db5a46f`, each read at that base.**

`an_opaque_identity_takes_the_bound_of_the_authority_that_mints_it` in `crates/tiler-artifact/src/program/tests.rs` builds `let measured_kernel_identity = vec![0x5a; 1_121];` and asserts `measured_kernel_identity.len() > super::MAX_OPAQUE_IDENTITY_BYTES`. That is a statement about a literal, not about any kernel: it cannot observe the real identity moving, and it did not. **Its doc comment is false at this base** — it says the 1,121-byte case "is the measured one, not a chosen one: it is the canonical kernel identity of a serial `f32` sum reducing two or more contributors", and that identity now measures **1,309** bytes. The same fabricated length appears again as `let long_key = vec![0x5a; 1_121];` in `an_artifact_encodes_an_entry_key_longer_than_the_digest_bound`.

**The test cannot be repaired in place, and the reason is structural rather than an oversight.** `crates/tiler-artifact/Cargo.toml` deliberately carries no `tiler-compiler` edge, with the reason in a comment above `[dependencies]`: `tiler-runtime`'s `the_consumer_links_no_compiler_emitter_or_build_provider` walks `Cargo.lock`, which merges normal and development edges per package, so a dev edge here would put `tiler-compiler` in the consumer's closure and fail that test against ADR 0081 item 2. The crate that owns `MAX_OPAQUE_IDENTITY_BYTES` therefore can never compile a real reduction to compare against it.

**The measurement the assertion should hold, at `68ba010a`, Apple M4 Max, macOS 27.0 (26A5388g), toolchain `nightly-2026-07-19`.** A scale-then-bias-then-`StrictSerialF32Sum` program over `[4, 1]` reducing axis 1 yields a canonical kernel identity of **924** bytes; over `[4, 2]`, `[4, 3]`, `[4, 4]`, or `[4, 8]` it yields **1,309**. `MAX_OPAQUE_IDENTITY_BYTES` is `1_024` in `crates/tiler-artifact/src/program/keys.rs`. Both readings are recorded with their construction in the filing ticket's Outcome and in `docs/artifact-abi.md`'s "Governed budgets" section.

## What closes this

**The two-sided inequality asserted, not a length.** That a one-contributor serial `f32` sum's canonical kernel identity is *under* `MAX_OPAQUE_IDENTITY_BYTES` and a two-contributor one is *over* it. That is the entire argument the bound was changed on, it fails loudly from either direction, and it does not decay when the constant offset moves again — which a length pinned to 1,309 would, exactly as 1,121 did between 2026-07-25 and 2026-08-08.

**Where it belongs: a crate that already reaches both the compiler and the artifact layer.** `crates/tiler-conformance` holds `serial_sum_program` and `compile_under` in `src/serial_sum.rs` and already depends on both; `prototypes/serial-sum-compile`'s test module is the other candidate and is where the original sweep ran. Pick one and say why. Note that `tiler-conformance`'s `serial_sum_program` builds `Shape::from_dims([rows, columns])` reducing `Axis::new(1)` — rank 2 only — which is enough for this inequality and *not* enough for the rank-3-through-8 growth figures, so do not widen it for those.

**A route caveat the check must survive.** At this base `tiler_compiler::session::compile_governed` refuses `[4, 3, 3]` reducing `[2]`, `[64, 3]`, and `[4096, 3]` as `NoFeasiblePlan`, though it admitted all three in July. It still admits `[4, 1]` and `[4, 2]`, so either route reaches the two shapes this inequality needs; where both routes admit a shape they agree on the length exactly. State which route the assertion uses.

**Perturb the subject, not the assertion.** Show the check going red by moving the program — reduce one contributor where it expects two — and quote the failure text. A check that only reddens when its own literal is edited has demonstrated nothing.

**Three stale comments to correct while here**, each stating a superseded value in the present tense:

- `crates/tiler-artifact/src/program/tests.rs`, the doc comment on `an_opaque_identity_takes_the_bound_of_the_authority_that_mints_it` quoted above. If the fabricated vector stays — it may have to, given the dependency boundary — its comment must say it is a fabricated length chosen to exceed the bound, and name where the real inequality is asserted.
- `prototypes/serial-sum-compile/src/main.rs`, `COLUMNS`'s doc comment: "the canonical kernel identity this producer hands it measures 1,121 bytes for any reduction with two or more contributors".
- `prototypes/serial-sum-run/src/proof.rs`, two sites: "a two-or-more-contributor serial sum's kernel identity measures 1,121", and "the canonical kernel identity of a serial sum with two or more contributors measures 1,121 bytes".

Each is a historical narrative and the history is right; only the tense and the number are wrong. Prefer restating them as what was true at the closure they describe over refreshing the digits, since refreshing rebuilds the defect one identity step later.

## Out of scope

`docs/artifact-abi.md` is already repaired and dated by the filing ticket. Do not restate any regenerated length there as a fresh prose figure.
