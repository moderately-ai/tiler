---
id: remove-the-fast-honor-pragmas-variant
title: Remove the fast-honor-pragmas variant
status: done
priority: p3
dependencies: []
related: [decide-whether-fpcontract-retains-the-driver-rejected-variant, record-or-validate-the-fast-honor-pragmas-selection]
scopes: [implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal-aot, surface-removal]
---

## The decision this executes

**Tom decided on 2026-08-06 (provenance on [`decide-whether-fpcontract-retains-the-driver-rejected-variant`](decide-whether-fpcontract-retains-the-driver-rejected-variant.md)):** `FpContract::FastHonorPragmas` is removed. The grounds are on the decision node; the removal commit cites it.

## The work

- Delete the variant and every arm matching it (`crates/tiler-metal-aot/src/input.rs`: the enum, `flag_value`, `contracts_across_statements`; any other site `rg -n 'FastHonorPragmas' crates/` finds — read each).
- Retarget the watcher: `fast_honor_pragmas_is_rejected_by_the_metal_driver` becomes a test asserting the driver's admitted `-ffp-contract` set is exactly `{off, on, fast}` — probing a fourth value (`fast-honor-pragmas` as a raw string, since the variant no longer spells it) still fails typed at the metal stage, so a future toolchain accepting it still fires loudly. Watch the retargeted test fail under a deliberate perturbation before trusting it.
- Preserve the measurement: the `Fast` variant's pragma-honouring measurement paragraph stays (it documents a live variant); the dead variant's doc moves to git history with the removal commit citing the decision node — do not paste the whole measurement into the commit message, cite the node.
- The enum is `pub` without `#[non_exhaustive]`: confirm no out-of-crate matches exist (`rg -n 'FpContract::' crates/ prototypes/` outside the crate) and report the check.

## Closes when

The variant is gone, every site compiles, the retargeted watcher was watched failing, and the removal commit cites the decision node.

## Outcome

**Done.** `FpContract::FastHonorPragmas` is removed; `rg -n 'FastHonorPragmas' crates/ prototypes/` returns nothing. `rg -n 'FastHonorPragmas' crates/ prototypes/` at the base found eleven lines, eight in `input.rs` and three in `driver.rs`, each read and disposed: `input.rs:602` the variant with its whole documentation block (deleted); `:613` the `token` arm (deleted — the ticket calls this method `flag_value`, its actual name is `token`); `:639` the `contracts_across_statements` arm (narrowed to `Self::Fast => true`); `:630` the paragraph explaining that method's `true` for the dead variant (deleted); `:522` the enum header's "four values, three selectable" framing (rewritten to state the admitted set and name the watcher); `:547` the `Fast` variant's closing cross-reference (rewritten to keep the actionable advice without the dead link); `:1388` and `:1389` two assertions in `only_fast_contraction_fuses_across_statements` (deleted; nothing about a surviving variant was asserted there); `driver.rs:719`, `:748`, and `:757` the watcher's doc, its constructed realization, and its panic message (all retargeted, below). The `Fast` variant's pragma-honouring measurement paragraph is untouched; the `on-vector` fact from the removed block survives in the enum header, since it is what makes "exactly three" a measured claim rather than a coincidence.

**The retargeted watcher.** `fast_honor_pragmas_is_rejected_by_the_metal_driver` became `the_metal_driver_admits_exactly_the_three_stated_fp_contract_values`, which compiles the fixture under all three variants (tokens asserted through an exhaustive `match`, so a fourth variant is a compile error here) and then probes `fast-honor-pragmas` as a raw string. The raw value is substituted into `compile_flags`'s output and run through `Toolchain::run_stage` — the driver's own private stage runner — so the probe observes the same typed `DriverError::ToolFailure { stage: Metal }` a compilation would, without a raw-flag route on `CompileRequest`, whose flags are the exact text reaching artifact identity. The substitution count is asserted to be exactly one, so a flag list that stopped carrying `-ffp-contract` cannot leave the probe silently testing nothing.

**Watched failing.** Perturbing the probed value to `fast` (an admitted value) failed as designed: `panicked at crates/tiler-metal-aot/src/driver.rs:816:22: metal accepted -ffp-contract=fast-honor-pragmas, so this row's driver admits more than the three values FpContract spells`. That run also proves the test did not self-skip on this host — a skip returns before the probe and would have passed.

**Out-of-crate consumers: none broken.** `rg -n 'FpContract' crates/ prototypes/` outside `tiler-metal-aot` returns five sites, each read: `crates/tiler-metal/src/golden_compilation.rs:362` and `crates/tiler-build/src/metal_assembly.rs:340` compare `fp_contract == FpContract::Off` (the matches there are over `MetalNumericalRequirement`, not `FpContract`); `golden_compilation.rs:988` and `metal_assembly.rs:819,834` construct realizations with `Off`/`Fast`. No out-of-crate site names the removed variant and none matches `FpContract` exhaustively, so removal is invisible to them — confirmed by `cargo check --workspace --all-targets`.

**Identity reach: none.** `FpContract` reaches `CompilationIdentity` only through `request.compile_flags()`, which encodes the `-ffp-contract=<token>` string; `identity.rs`'s encoder is documented and written to be "free of declaration ordinals", and no discriminant or variant index is folded anywhere. Removing a variant therefore narrows the statable set and leaves every recorded identity byte-identical: the `off`, `on`, and `fast` tokens are unchanged strings in unchanged positions. No identity-domain step was needed and no pin moved.

**Checks.** `cargo fmt --check -p tiler-metal-aot`; `cargo check -p tiler-metal-aot --all-targets`; `cargo clippy -p tiler-metal-aot --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-metal-aot`; `cargo nextest run -p tiler-metal-aot` (62 passed); `cargo test -p tiler-metal-aot --doc` (4 + 3 passed); `cargo check --workspace --all-targets`; `git diff --check`. One nextest run reported a leaky verdict naming `rejects_invalid_source_when_toolchain_available`, a test this branch does not touch and which did not leak on the adjacent run — the wandering-name signature of the macOS `pipe()`/`posix_spawn` race `AGENTS.md` records, not a defect here.

**Note for the next reader.** `TILER_REQUIRE_METAL_TOOLCHAIN` is honoured only by `tiler-metal`'s `golden_compilation`; `tiler-metal-aot`'s driver tests self-skip unconditionally when `resolve` fails, so setting it while running this package changes nothing. The perturbation above is what evidences the toolchain-dependent half actually ran here.

**Superseded 2026-08-06:** the closing note above about `TILER_REQUIRE_METAL_TOOLCHAIN` being inert for this package no longer holds — [`honour-require-metal-toolchain-in-the-aot-driver-tests`](honour-require-metal-toolchain-in-the-aot-driver-tests.md) routed all five self-skip sites through a typed resolution that honours it.
