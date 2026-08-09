---
id: repair-the-artifact-abis-stale-cross-crate-no-prefix-argument
title: Repair the artifact ABI's stale cross-crate no-prefix argument
status: todo
priority: p1
dependencies: [correct-the-every-ir-domain-opens-tiler-ir-premise-in-two-places]
related: []
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, identity, documentation]
---
## Facts to re-verify before editing

The accepted contract at `docs/artifact-abi.md`, under the governed-digest union obligation, still argues that every domain admitted by the shared IR opens `tiler.ir.` and that the two sets diverge at the first byte after the shared `tiler.`. Both premises are false. `crates/tiler-ir/src/program/abi.rs` spells `EXPR_DOMAIN` inside `tiler.artifact-program.`, and the complete source-side correction in `crates/tiler-artifact/src/domains.rs` records that the first-differing-byte argument also fails.

The same contract paragraph says no crate can hold the union because `tiler-artifact` depends on `tiler-ir` and not the reverse. That direction would permit the artifact crate to check the union. The actual obstacle is that the IR-owned domain population is private and no exported enumeration exists. The source-side argument therefore uses the unique NUL terminator and the observed complete IR population instead of claiming namespace separation.

These are current Facts to audit again at the worker base. Do not carry counts from the predecessor ticket; derive any needed population from the owning source and avoid embedding counts in the repaired contract.

## Work

Read this contract in full, the complete `crates/tiler-artifact/src/domains.rs`, the complete IR domain population owner, and the governing digest decisions. Replace only the live false cross-crate reasoning with the source-true terminator argument and the real explanation for why the union is not directly checked. Preserve the local crate-owned no-prefix obligation, the ownership split, and the rule that a newly admitted domain must reopen the argument.

Use a dated correction because this is an accepted contract. Quote retired wording only inside that correction and make clear that later searches land in history, not a live premise. Check the surrounding governed-digest claims in full; report any additional stale statement instead of silently folding an unrelated change into this carrier.

## Non-goals

No domain bytes, digest algorithm, identity version, schema, public enumeration, dependency edge, encoder, or runtime behavior changes. Do not export the IR population merely to make a test possible. Do not replace the false argument with another positional or numeric shortcut.

## Closes when

The accepted contract no longer presents either namespace premise or the reversed dependency explanation as live Fact, its replacement agrees with both complete source populations, `make citations` and `tkt lint` pass, and exact-base `tkt guard` reports no under-declared scope. This is contract-only work, so the latest full gate may carry under the repository delta rule after fresh citations and ticket lint.
