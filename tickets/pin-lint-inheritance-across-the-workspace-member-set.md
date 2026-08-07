---
id: pin-lint-inheritance-across-the-workspace-member-set
title: Pin lint inheritance across the workspace member set
status: in-progress
priority: p2
dependencies: []
related: [stop-the-conformance-crate-s-lint-table-drifting-from-the-workspace, decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access]
scopes: [implementation/workspace, implementation/frontend, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [lints, maintainability]
claimed_from: todo
assignee: w-pin-lint
lease_expires_at: 1786144902
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

## Per-Fact audit, 2026-08-07 at `d8913a9d`

Every source this ticket names was re-read in full at that base. **No Fact is false and none needed repair** — unusual for this corpus, and stated as an outcome rather than as a default. Verdicts:

- **Intro** — verified. `stop-the-conformance-crate-s-lint-table-drifting-from-the-workspace` carries `scopes: [implementation/conformance]` in its frontmatter, so it genuinely could not reach past that crate. `git log --oneline -1 e197176` is `e197176f Replace the Python gate with a Makefile of cargo commands`, and its stat shows `scripts/check_workspace.py` deleted at 952 lines.
- **Fact 1, the population is two** — verified, and the command reproduces exactly. `for f in crates/*/Cargo.toml prototypes/*/Cargo.toml; do grep -q '^\[lints\]' "$f" || echo "$f"; done` returns `crates/tiler-conformance/Cargo.toml` and `prototypes/serial-sum-run/Cargo.toml` and nothing else, over sixteen member manifests. The check now prints the same partition as a census: 14 of 16 inherit.
- **Fact 1, `UNINHERITED_LINT_MEMBERS`** — verified against ADR 0079's "Fact — the exception was pinned, and is now permitted without a pin".
- **Fact 2, same divergence shape** — verified, and it is *stronger* than "same shape". The two restated tables are identical, which is what made the reuse below work: `crates/tiler-conformance/src/lints.rs` compiled unmodified from the prototype's root passes both its tests, because every assertion in it is stated relative to `CARGO_MANIFEST_DIR`.
- **Fact 2, the unsafe-site census cannot see the prototype** — verified. `the_unsafe_site_population_is_the_two_named_ones` roots at `Path::new(env!("CARGO_MANIFEST_DIR")).join("src")`. **This half is not closed by this ticket** — see the remainder below.
- **Fact 3, the workspace end is unstated** — verified at base; closed here.
- **"Where it would live", `cargo metadata` does not emit lint tables** — verified by measurement rather than accepted. `cargo metadata --no-deps --format-version 1` emits 59,759 bytes at this base containing zero occurrences of `lints`, and zero of `missing_docs`, `unsafe_code`, `pedantic`, or `too_many_lines`. A text reader is therefore the only option, which is what makes reusing the existing one rather than writing a second load-bearing.
- **"Also worth checking while there"** — verified. `crates/tiler-digest/Cargo.toml` declares `[lints]` with `workspace = true`, so it does inherit `forbid`, and the `sha2` comment's "workspace policy forbids unsafe code" was the over-broad half. Narrowed to a claim about `tiler-digest`.

**One citation elsewhere has drifted, and it is outside this ticket's scopes.** ADR 0079's Consequences place `the_unsafe_site_population_is_the_two_named_ones` at "lines 497–548" of `crates/tiler-conformance/src/bf16_vertical/tests.rs`; at this base it occupies 696–747. Same 52-line extent, moved down 199 lines. `contracts/decisions` is not held here, so this is reported rather than repaired.

## Outcome

Three checks, none of which restates the workspace lint table.

- `crates/tiler/tests/workspace_lint_inheritance.rs` — new. Reads `[workspace] members` from the root manifest, partitions the sixteen members on whether each declares `[lints]` with `workspace = true`, and holds the diverging set equal to a declared `UNINHERITED_LINT_MEMBERS` — the name from the deleted Python gate, carried back deliberately. Both directions fail: an undeclared member that stops inheriting, and a declared exception that starts. A third test is the seam between the member half and the table half: each declared exception must name a table check, that check must lie inside the member it governs, and it must reach the one shared reader.
- `prototypes/serial-sum-run/tests/lint_table.rs` — new, and it is one `#[path]` declaration. It runs `crates/tiler-conformance/src/lints.rs` unmodified from this member's root. **Zero duplicated parsing.** The conformance crate was not edited; `implementation/conformance` is not held here.
- The root `Cargo.toml` names both exceptions under `[workspace.lints.rust]`, with the reason and all three checks, so the difference is stated at both ends rather than one.

**Why `crates/tiler/tests/` and not `tiler-conformance`.** The previous worker's argument was evaluated rather than inherited, and it holds — with a second reason it did not give. Its reason: that crate owns cross-layer *executed* evidence and a manifest policy executes nothing. The stronger one: `tiler-conformance` is **one of the two members the check polices**, and a census living inside a member of the population it enumerates has to special-case its own exception. `crates/tiler` inherits the workspace table like every other non-exception member, and `workspace_population.rs`, `dependency_direction.rs`, and `labelled_diagnostic.rs` are already there doing workspace-wide reads across the same frontier.

**Each property was perturbed separately and every failure message quoted in the worker report.** A third member dropping inheritance (`crates/tiler-digest`); the prototype's `deny` widened to `allow`; a lint added to the workspace side only; a lint added to the prototype side only; the member population collapsed below its floor; a declared exception losing its table check; and that check replaced by a second reader. Each reddened exactly one assertion and left the others green.

## Remainder, not closed here

`prototypes/serial-sum-run` still has **no per-site unsafe check**. This ticket closed its *table*; the site census in `tiler-conformance` remains rooted at its own `CARGO_MANIFEST_DIR`, so a third `#[allow(unsafe_code)]` site added under `prototypes/serial-sum-run/src/` still compiles and still passes the complete gate, exactly as ADR 0079's Consequences record. Fact 2 bundles the two gaps in one paragraph and only the table half is discharged. This wants its own ticket.
