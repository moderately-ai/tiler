---
id: correct-the-stale-xcrun-count-in-the-macro-aot-module-docs
title: Correct the stale xcrun count in the macro AOT module docs
status: todo
priority: p3
dependencies: []
related: [avoid-toolchain-resolution-on-a-warm-expansion-cache-hit, drop-the-unread-sdk-path-from-the-resolved-toolchain]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, macro-aot, status-drift]
---
## User-visible outcome

`crates/tiler-macros/src/aot.rs`'s module documentation reports the number of `xcrun` invocations a resolution actually makes.

## Why this exists

**Fact — the doc comment says five, and names a call that no longer exists.** `crates/tiler-macros/src/aot.rs:41-43` reads "`Toolchain::prepare` runs five `xcrun` queries — two `--find` and three `--show-sdk-*` — and then executes the two located binaries to read their reported versions, on every expansion", and line 52 repeats "The five `xcrun` answers".

**Fact — `resolve` makes four.** `crates/tiler-metal-aot/src/driver.rs:86-97` calls `find_tool` twice (`metal`, `metallib`) and `sdk_field` twice (`--show-sdk-version`, `--show-sdk-build-version`). There is no third `--show-sdk-*`.

**Fact — the contract already records the correction and the code comment did not follow.** `docs/integration/frontends.md`'s measurement table preamble states "The table measures the code as it stood on that date, when a resolution made five `xcrun` calls. **A resolution now makes four.**", attributing the removal of `--show-sdk-path` to `drop-the-unread-sdk-path-from-the-resolved-toolchain` on 2026-08-01. `crates/tiler-metal-aot/src/record.rs:32` carries the same history.

**Why it matters.** AGENTS.md: a doc comment is a claim, and the next worker reads it as fact. This one is also load-bearing for the measurement discipline in the same paragraph — it is the sentence a reader would use to check a shim transcript's line count against, and it would make a correct transcript look wrong by one.

## Boundary

Documentation only, inside `crates/tiler-macros`. Do not change what `resolve` calls; the removal was deliberate and measured. Check the surrounding sentences in the same comment block for the same drift rather than editing the two numbers — the block also asserts which calls are served from `$TMPDIR/xcrun_db` and which are direct executions.

## Closes when

The module documentation's call count and call breakdown agree with `Toolchain::resolve`, verified by reading both, and no other sentence in that block asserts a count that source contradicts.
