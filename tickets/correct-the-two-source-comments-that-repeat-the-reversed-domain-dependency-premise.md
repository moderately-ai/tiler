---
id: correct-the-two-source-comments-that-repeat-the-reversed-domain-dependency-premise
title: Correct the two source comments that repeat the reversed domain dependency premise
status: done
priority: p2
dependencies: []
related: [repair-the-artifact-abis-stale-cross-crate-no-prefix-argument]
scopes: [implementation/artifact, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## Fact — two live source comments retain the retired dependency and namespace argument

`crates/tiler-artifact/src/program/codec/tests.rs`, anchor `since neither depends on the other`, says no crate can hold a cross-crate check and also says the IR population opens only `tiler.ir.`. `crates/tiler-ir/src/index/refinement.rs`, anchor `so neither crate can enumerate the union`, repeats the reversed dependency explanation and first-differing-byte namespace argument. The artifact manifest has a live `tiler-ir.workspace = true` dependency, while the reverse edge is absent; the complete IR pin population is private and test-only rather than unreachable because of dependency direction. `EXPR_DOMAIN` also lives under `tiler.artifact-program.`, so the namespace premise is false.

## Outcome

Replace both live comments with the count-free spelling-and-terminator argument already established by the source authority and accepted artifact ABI contract. Preserve the local no-prefix checks and their ownership limits. Do not change any production domain bytes, public boundary, encoder, schema, or identity.

## Checks

- Re-read both complete source files, the complete domain census modules, both crate manifests, and the accepted cross-crate paragraph before editing.
- Run `cargo test -p tiler-artifact domains::` and `cargo test -p tiler-ir domains::`.
- Run `cargo fmt --all --check`, package Clippy/rustdoc as applicable, `make citations`, `tkt lint`, `git diff --check`, and exact-base `tkt guard`.
- Negative grep must show the two retired live anchors are absent outside dated correction or ticket history.

## Outcome

**Fact audit at `b3b1652faa6c0060e4958782c2d5d37b563b9f8b`: verified; no ticket Fact required repair.** The two anchors existed in their named live source comments; `tiler-artifact` depended on `tiler-ir` while the reverse manifest edge was absent; `PINNED_IDENTITY_DOMAINS` was the private test-only IR population; and `EXPR_DOMAIN` was `tiler.artifact-program.abi-expr.v1\0`. The accepted artifact ABI paragraph at anchor `the cross-crate no-prefix obligation is discharged` established the replacement spelling-and-terminator argument.

Replaced exactly those two comments. Both now state that an artifact-side union check would be reachable if the IR population were exported, that the actual obstacle is its private test-only pin population, and that `tiler-digest` deliberately owns no subject domains. Review found that the IR comment initially attributed `no_governed_domain_of_this_crate_prefixes_another` to its own crate; corrected it to identify `tiler_artifact::domains::no_governed_domain_of_this_crate_prefixes_another` as the artifact-side local check and the IR pins as the inspected private population.

Checks passed: `cargo test -p tiler-artifact domains::`; `cargo test -p tiler-ir domains::`; `cargo fmt --all --check`; `cargo clippy -p tiler-artifact -p tiler-ir --all-targets -- -D warnings`; `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps -p tiler-artifact -p tiler-ir`; `make citations`; `tkt lint`; `git diff --check`; source negative greps for both retired anchors; and `make full`. Final `tkt guard tkt/correct-the-two-source-comments-that-repeat-the-reversed-domain-dependency-premise --format json` is against base `main` whose merge base is the exact base above; it reports no under-declaration (overlapping live tickets remain non-gating warnings). Final source tip: `4dd6463d79774f4f5d676198ad2c35bb53a983ea` before this ticket record update.
