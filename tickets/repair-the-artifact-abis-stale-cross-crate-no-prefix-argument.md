---
id: repair-the-artifact-abis-stale-cross-crate-no-prefix-argument
title: Repair the artifact ABI's stale cross-crate no-prefix argument
status: in-progress
priority: p1
dependencies: [correct-the-every-ir-domain-opens-tiler-ir-premise-in-two-places]
related: []
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, identity, documentation]
claimed_from: todo
assignee: sol-artifact-abi-prefix
lease_expires_at: 1786383181
---
## Facts to re-verify before editing

The accepted contract at `docs/artifact-abi.md`, under the governed-digest union obligation, still argues that every domain admitted by the shared IR opens `tiler.ir.` and that the two sets diverge at the first byte after the shared `tiler.`. Both premises are false. `crates/tiler-ir/src/program/abi.rs` spells `EXPR_DOMAIN` inside `tiler.artifact-program.`, and the complete source-side correction in `crates/tiler-artifact/src/domains.rs` records that the first-differing-byte argument also fails.

The same contract paragraph says no crate can hold the union because `tiler-artifact` depends on `tiler-ir` and not the reverse. That direction would permit the artifact crate to check the union. The actual obstacle is that the IR-owned domain population is private and no exported enumeration exists. The source-side argument therefore uses the unique NUL terminator and the observed complete IR population instead of claiming namespace separation.

The same Fact's opening labels the cross-crate half as "discharged by construction rather than by a check"; after the repair that half is a spelling/terminator argument over the observed IR population (as in `domains.rs`), not namespace construction. The follow-on sentence after the cross-crate Fact still says the local `tiler.artifact` prefix "is still disjoint from `tiler.ir.`" — true as root prefixes, but it props up the retired IR-opens-`tiler.ir.` story and must be rewritten with the main paragraph so it does not re-assert that namespace claim.

These are current Facts to audit again at the worker base. Do not carry counts from the predecessor ticket; derive any needed population from the owning source and avoid embedding counts in the repaired contract.

## Work

Read this contract in full, the complete `crates/tiler-artifact/src/domains.rs` (especially the doc comment on `no_governed_domain_of_this_crate_prefixes_another`), the complete IR domain population owner (`PINNED_IDENTITY_DOMAINS` in the test-only `crates/tiler-ir/src/domains.rs`), and the governing digest decisions. Under "The governed digest", replace the live cross-crate Fact that currently opens with the obligation spanning crates discharged by construction rather than by a check so that:

1. it no longer claims every shared-IR domain opens `tiler.ir.`;
2. it no longer claims divergence at the first byte after `tiler.`;
3. it no longer explains the missing union check by dependency direction alone (`tiler-artifact` depends on `tiler-ir` and not the reverse is the direction that *would* allow a check);
4. it states the real obstacles: IR population is a private pin table with no exported enumeration, and `tiler-digest` deliberately knows no subject domains;
5. it states the source-true terminator / observed-population argument aligned with `crates/tiler-artifact/src/domains.rs` (unique trailing NUL on local domains; IR pins either terminate with sole trailing NUL or are terminator-free spellings no local domain extends);
6. it preserves the local crate-owned no-prefix obligation, ownership split, "domain spelled outside this crate's established prefixes breaks the argument", and "newly admitted domain must reopen the argument";
7. it rewrites the follow-on `It is still disjoint from `tiler.ir.`` scaffolding so it does not re-assert the retired namespace story (keep the true `tiler.artifact` vs `tiler.artifact-` route-requirement spelling note without leaning on the false IR quantifier).

Use a dated correction because this is an accepted contract. Quote retired wording only inside that correction and make clear that later searches land in history, not a live premise. Retired fragments to quote include at least: `every domain the shared IR admits opens `tiler.ir.``; `the two sets diverge at the first byte after the shared `tiler.``; and the reversed dependency explanation that neither crate can hold the union *because* `tiler-artifact` depends on `tiler-ir` and not the reverse. Do not embed population counts that can rot. Do not replace the false argument with another positional or numeric shortcut. Check the surrounding governed-digest claims in full; report any additional stale statement instead of silently folding an unrelated change into this carrier. At the current base, `docs/architecture.md` and `tiler-digest` notes do not restate the false IR quantifier premises and need no edit under this ticket.

## Non-goals

No domain bytes, digest algorithm, identity version, schema, public enumeration, dependency edge, encoder, or runtime behavior changes. Do not export the IR population merely to make a test possible. Do not replace the false argument with another positional or numeric shortcut.

## Closes when

The accepted contract no longer presents either namespace premise or the reversed dependency explanation as live Fact, its replacement agrees with both complete source populations, the follow-on disjoint scaffolding no longer re-asserts the retired namespace story, `make citations` and `tkt lint` pass, and exact-base `tkt guard` reports no under-declared scope. This is contract-only work, so the latest full gate may carry under the repository delta rule after fresh citations and ticket lint.

## Fact audit — 2026-08-10

**Correction — 2026-08-10.** Ticket-record honesty only; product contract edit not delivered this pass. Re-verified at the current tree:

- Live false premises still present under "The governed digest" in `docs/artifact-abi.md`: `every domain the shared IR admits opens `tiler.ir.``; `the two sets diverge at the first byte after the shared `tiler.``; `Neither crate can hold a check over the union` coupled to dependency direction; opening label `discharged by construction rather than by a check`; follow-on `It is still disjoint from `tiler.ir.``.
- Counterexample still live: `const EXPR_DOMAIN: &[u8] = b"tiler.artifact-program.abi-expr.v1\0"` in `crates/tiler-ir/src/program/abi.rs`.
- Source authority for the repair still in `crates/tiler-artifact/src/domains.rs` on `no_governed_domain_of_this_crate_prefixes_another` (retired wording quoted; terminator argument; dependency direction would permit a union check; obstacle is private `PINNED_IDENTITY_DOMAINS`).
- IR population still test-only private: `#[cfg(test)] mod domains` in `crates/tiler-ir/src/lib.rs`; `const PINNED_IDENTITY_DOMAINS` is non-`pub`.
- Dependency edge still `tiler-artifact` → `tiler-ir` (`tiler-ir.workspace = true` in artifact Cargo.toml); reverse edge absent.
- Predecessor `correct-the-every-ir-domain-opens-tiler-ir-premise-in-two-places` is `done` and maps this ticket as the accepted-contract remainder; source half landed at `f9b0b67d`. Graph (`todo`, that dependency, empty `related`, scopes `contracts/artifacts`) remains correct. Close condition unmet until `docs/artifact-abi.md` is repaired.

## Review residual — 2026-08-10

Independent review of the accepted-contract repair found two pre-existing source comments that still present the reversed dependency explanation as live. Both are outside this ticket's contract-only scope and are recorded rather than edited here:

- `crates/tiler-artifact/src/program/codec/tests.rs`, source anchor `since neither depends on the other`, says no cross-crate check can exist on that ground even though `tiler-artifact` depends on `tiler-ir`.
- `crates/tiler-ir/src/index/refinement.rs`, source anchor `so neither crate can enumerate the union`, likewise treats the dependency direction as the blocker; the actual obstacle on the depending side is the private, test-only IR pin population and absence of an exported enumeration.

Repairing the first requires `implementation/artifact`; repairing the second requires `implementation/ir`. This branch deliberately edits neither source file. The accepted-contract remainder recorded above was delivered on this branch at `8cc23ae3`; the review correction makes its terminator-free premise explicit without adding a population count.
