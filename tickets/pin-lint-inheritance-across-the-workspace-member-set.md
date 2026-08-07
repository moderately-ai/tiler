---
id: pin-lint-inheritance-across-the-workspace-member-set
title: Pin lint inheritance across the workspace member set
status: todo
priority: p2
dependencies: []
related: [stop-the-conformance-crate-s-lint-table-drifting-from-the-workspace, decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access]
scopes: [implementation/workspace, implementation/frontend, implementation/runtime]
shared_scopes: []
paths: []
tags: [lints, maintainability]
---
## What is still unheld

`stop-the-conformance-crate-s-lint-table-drifting-from-the-workspace` landed `crates/tiler-conformance/src/lints.rs`, which fails when **that crate's** restated lint table diverges from `[workspace.lints]` by anything other than the one intended level. It is scoped to `implementation/conformance` and cannot reach past it, so three properties remain exactly where [ADR 0079](../docs/decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md) left them after `e197176` deleted `scripts/check_workspace.py`:

1. **A third member dropping `[lints] workspace = true`** goes unlinted with nothing failing. `UNINHERITED_LINT_MEMBERS` named the one member permitted to diverge and that half is gone. The current population is two, reproducible with:

   ```sh
   for f in crates/*/Cargo.toml prototypes/*/Cargo.toml; do grep -q '^\[lints\]' "$f" || echo "$f"; done
   # crates/tiler-conformance/Cargo.toml
   # prototypes/serial-sum-run/Cargo.toml
   ```

2. **`prototypes/serial-sum-run`'s table has no check of any kind.** Its divergence is the same shape as the conformance crate's — the workspace table restated with the unsafe-code lint at `deny` — and its `deny` may still be widened to `allow`, or a lint added or dropped, with nothing failing. It also has no per-site check: `bf16_vertical::tests::the_unsafe_site_population_is_the_two_named_ones` is rooted at `CARGO_MANIFEST_DIR` and cannot see it, so a third unsafe site *there* still compiles and passes the complete gate. ADR 0079's Consequences already record this and say the scope is two crates.

3. **The workspace end of the intended difference is unstated.** `stop-the-conformance-crate-s-lint-table-drifting-from-the-workspace` required the one intended difference to be named at *both* ends so neither reads as an oversight. The crate end is done — `crates/tiler-conformance/Cargo.toml` names it and points at the check. The root `Cargo.toml`'s `[workspace.lints]` still says nothing about either diverging member, so a reader of the workspace table has no way to reach the exceptions to it.

## Why one ticket

All three are the same missing instrument seen from three sides: nothing reads the member set and holds it to a declared inheritance policy. Splitting them would produce three checks over the same file population.

## Where it would live

Not in `tiler-conformance`. That crate owns cross-layer *executed* evidence, and its header says so explicitly; a workspace manifest policy check is neither cross-layer nor executed, and putting it there would make the crate the place a test goes when nobody decided where it belongs. The existing precedent for a workspace-population check is `crates/tiler/tests/workspace_population.rs`, which derives the member set from `cargo metadata` and holds it to a declared list, and `crates/tiler/tests/dependency_direction.rs` beside it, which hand-parses `Cargo.lock`. Note that `cargo metadata` does **not** emit the lints table, so the member manifests have to be read as text — `crates/tiler-conformance/src/lints.rs` is a working, fail-closed reader for exactly that shape and should be reused rather than rewritten.

## Also worth checking while there

The root `Cargo.toml`'s `sha2` comment says a workspace crate "cannot reach the ARMv8 crypto instructions `sha2` selects at runtime, because workspace policy forbids unsafe code". That is still true of `tiler-digest`, which inherits `forbid`, but "workspace policy forbids unsafe code" now has two exceptions and reads as a claim about the whole workspace.

## Closes when

A check fails on a member that drops lint inheritance without being the declared exception, and on either declared exception's table diverging from the workspace's by more than its named lint level; and the workspace table names its exceptions.
