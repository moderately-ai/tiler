---
id: align-runtime-dependency-records-with-negative-reachability
title: Align runtime dependency records with the negative-reachability check
status: done
priority: p2
dependencies: [date-adr-0081-s-neither-closure-is-checked-correction-against-the-runtime-test]
related: []
scopes: [contracts/foundation, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, runtime, architecture]
---

The runtime dependency boundary is now described three different ways, two of
which predate its mechanical negative-reachability check. Align the live
architecture contract and the manifest comment with what the test actually
proves; do not change any dependency or public surface.

## Fact audit at creation (2026-08-08)

- **Verified — architecture overstates uniqueness.** Reading
  `docs/architecture.md` in full, the paragraph anchored by `These edges are a
  description maintained by reading rather than a checked contract, with one
  exception` says the frontend frontier is the only checked edge class and the
  first check since `scripts/check_workspace.py` was deleted. That is no longer
  current: the runtime test below is a second checked slice of the same live
  packaging block.
- **Verified — the runtime manifest conflates a direct set with a closure.**
  Reading `crates/tiler-runtime/Cargo.toml` in full, the comment anchored by
  `crate's *dependency closure* at` calls `[tiler-artifact]` the complete
  dependency closure and says development edges do not enter it.
  `cargo tree -p tiler-runtime -e normal --depth 1` shows that
  `tiler-artifact` is the sole normal **direct dependency**. It is not the full
  resolved closure, and `cargo tree -p tiler-runtime -e all --depth 1` also
  shows the direct development edges to `tiler-ir` and `tiler-reference`.
- **Verified — the existing check is deliberately negative and
  dev-inclusive.** Reading all of
  `crates/tiler-runtime/tests/identity_join/main.rs`, the test anchored by
  `fn the_consumer_links_no_compiler_emitter_or_build_provider` parses
  `Cargo.lock`, walks transitive reachability from `tiler-runtime`, requires the
  three reachable positive controls `tiler-artifact`, `tiler-ir`, and
  `tiler-reference`, and refuses five packages: `tiler-build`,
  `tiler-compiler`, `tiler-cache`, `tiler-metal`, and `tiler-metal-aot`.
  `cargo test -p tiler-runtime --test identity_join
  the_consumer_links_no_compiler_emitter_or_build_provider -- --exact` passes.
- **Verified — this is record alignment, not graph work.** The current normal
  direct set and the broader dev-inclusive resolved negative reachability are
  compatible facts. Neither establishes an exact full closure, and neither
  calls for a manifest edge change.

## Outcome

- Correct the architecture paragraph so it names both mechanically checked
  slices without implying that either pins the entire packaging table: the
  frontend direct-edge frontier and the runtime dev-inclusive transitive
  refusal.
- Correct the runtime manifest comment to distinguish the one normal direct
  dependency from the dev-inclusive resolved closure checked by
  `identity_join`.
- Preserve ADR 0081's decision and the current dependency graph. Do not add,
  remove, or promote dependencies, and do not edit public runtime APIs.
- Re-read both full files and the complete test at the worker's exact base;
  repair this ticket first if any Fact has drifted.

## What closes this

The two live records agree on the exact property and test boundary; the target
runtime test, `tkt lint`, `make citations`, `git diff --check`, and `tkt guard`
against the true base pass. Report the literal source-safe anchors used and the
output of both `cargo tree` commands. Because `crates/tiler-runtime/Cargo.toml`
is in the crate gate population, run the touched-package check/test/Clippy and
rustdoc gates rather than treating the edit as documentation-only.
