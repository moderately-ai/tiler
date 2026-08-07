---
id: stop-the-conformance-crate-s-lint-table-drifting-from-the-workspace
title: Stop the conformance crate's lint table drifting from the workspace
status: in-progress
priority: p2
dependencies: []
related: [carry-the-device-executed-value-proof-into-the-conformance-crate, decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access]
scopes: [implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: [lints, maintainability]
claimed_from: todo
assignee: w-lint-table
lease_expires_at: 1786141741
---
## The risk this creates

`crates/tiler-conformance/Cargo.toml` **restates** the workspace lint table rather than inheriting it, because it needs `unsafe_code = "deny"` where the workspace sets `forbid` and a member cannot inherit a table and then relax one entry. Its own comment says so: "Mirrors the workspace, which this crate cannot inherit; see the note there."

That is the correct mechanism for the decision Tom made on 2026-08-07 — `deny` with named per-site allows, never a crate-level allow — and it has a cost the decision did not name: **this crate's lints can now drift from the workspace's silently.** A lint added, tightened, or removed workspace-wide reaches every other member and not this one, and nothing fails when they diverge.

`AGENTS.md` already names the general hazard — "crates should inherit workspace Rust and Clippy lints; inspect `[lints]` changes because inheritance is not enforced" — which is guidance to a reviewer rather than a check.

**Corrected 2026-08-07 — this crate is the *second* member that cannot inherit, not the first.** The sentence here read "the first member that *cannot* inherit, so it is the first place that guidance has no fallback", and it is false at both ends. `prototypes/serial-sum-run` dropped `[lints] workspace = true` first, at `43f685f` on 2026-07-25, and carries a byte-identical divergence: the same restated table with the same one entry at `deny`. `for f in crates/*/Cargo.toml prototypes/*/Cargo.toml; do grep -q '^\[lints\]' "$f" || echo "$f"; done` returns both members and nothing else. [ADR 0079](../docs/decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md) records the same thing twice — "**Superseded — 2026-08-07, on 'the diverging crate' being one crate**" and "A second member did drop `[lints] workspace = true`: `crates/tiler-conformance`" — and this crate's own manifest calls `prototypes/serial-sum-run` "the precedent it matches in shape". What is true is the narrower claim the rest of this ticket rests on, which the correction does not disturb: the guidance has no fallback in either diverging member, and neither had a check.

## What this owes

A mechanism, or a recorded decision that none is worth its cost. Candidates, none settled:

- **A test that compares the two tables** and fails when they diverge on anything except the one entry deliberately different. It would have to read both manifests, which is a text-parsing test of the kind `dependency_direction.rs` and `workspace_population.rs` already establish precedent for — both hand-parse rather than take a dependency, and both exist because "review catches it only for as long as someone reviews."
- **Invert the exception**: set the workspace to `deny` and have every *other* member forbid it. Larger blast radius, and it weakens the default for members that should never contain unsafe — probably wrong, but it is the alternative that removes the divergence rather than watching it.
- **Record the divergence and accept it**, with the one intended difference named at both sites so a reader of either finds the other. Cheapest, and legitimate — but it should be a decision rather than the default that happens by not choosing.

Whichever lands, the **one intended difference must be stated at both ends** — the workspace table and the crate's — so neither reads as an oversight.

## Explicit non-goals

Do not change the crate's `unsafe_code = "deny"`; that is Tom's decision and this ticket implements nothing about unsafe. Do not relax any other lint to make the tables match — matching by weakening is the failure this exists to prevent.

## Closes when

Either a divergence between the two tables fails a check, or the divergence is recorded as accepted with the reason and the single intended difference named at both ends.

## Outcome

**The first candidate landed: a check that compares the two tables.** `crates/tiler-conformance/src/lints.rs` reads `[workspace.lints.*]` from the root manifest and `[lints.*]` from this crate's, and fails unless what they differ by is exactly `rust.unsafe_code` at `"forbid"` against `"deny"`. It derives rather than restates — the expectation is one line naming that lint and its two levels, and every other lint name and level is read out of the two files, so a check that had to be edited each time the workspace gained a lint is not what landed. It counts the population and refuses a floor of four, because two tables that parsed as empty compare equal and would report no drift. It accumulates physical lines into one logical entry until quotes and brackets balance, so a wrapped construct is read rather than missed, and it panics naming file and line on anything it cannot read rather than skipping it.

**Two tests, both `#[cfg(test)]` in the crate**, split because they fail for unrelated reasons: `both_lint_tables_are_read_from_their_manifests_rather_than_assumed` goes red when the scan stops finding lints, and `this_crates_lint_table_differs_from_the_workspace_by_exactly_one_level` goes red when it finds them and they disagree.

**Eleven perturbations, each observed failing (or passing) before the check was trusted.** Widening `deny` to `allow`; setting it back to `forbid`; adding a lint this crate has and the workspace does not; removing one; weakening a shared lint to make the tables match; emptying the `rust` table; a malformed entry; adding a lint to the *root* manifest that inheritance did not carry here; relaxing the *root* to `deny` so no difference remains; wrapping an inline table across two lines (must stay green, and does); and the same wrap with the level changed underneath it (must fail, and does). Two of these are informative rather than clean hits: setting the level back to `forbid` is caught by rustc before the test runs, since the two live `#[allow]` sites become `E0453`, and a malformed entry is caught by Cargo's own TOML parse — so the scan's fail-closed asserts guard against *its own* misreading of valid TOML, not against invalid TOML.

**The "both ends" requirement is half-met, and the unmet half is out of scope.** The crate end now names the difference and points at the check. The workspace end is the root `Cargo.toml`, which `implementation/conformance` does not cover, so it is carried by `pin-lint-inheritance-across-the-workspace-member-set` along with the two workspace-scoped properties this check does not hold: a *third* member dropping `[lints] workspace = true`, and `prototypes/serial-sum-run`'s identical divergence, which still has no check of any kind.

Test count moved from 54 to 56, so `portability::DEVICE_FREE_TEST_FLOOR` rose from 50 to 52 to keep its stated property — the smallest device-free collapse, two tests, still fails.
