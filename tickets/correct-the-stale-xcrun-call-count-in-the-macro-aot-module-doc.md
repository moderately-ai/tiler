---
id: correct-the-stale-xcrun-call-count-in-the-macro-aot-module-doc
title: Correct the stale xcrun call count in the macro AOT module documentation
status: done
priority: p3
dependencies: []
related: [drop-the-unread-sdk-path-from-the-resolved-toolchain, correct-the-warm-expansion-xcrun-requirement-in-the-testing-contract]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, macro-aot, status-drift]
---
## User-visible outcome

`crates/tiler-macros/src/aot.rs`'s module documentation stops naming a resolution the driver beside it no longer performs, so a reader costing an expansion, or auditing what a warm hit pays for, counts what the code does.

## Why this exists

**Fact — the module doc names five `xcrun` queries and three `--show-sdk-*` flags.** `crates/tiler-macros/src/aot.rs:41-44` reads "`Toolchain::prepare` runs five `xcrun` queries — two `--find` and three `--show-sdk-*` — and then executes the two located binaries to read their reported versions, on every expansion". `:52-53` repeats the count: "The five `xcrun` answers are themselves served from Apple's own `$TMPDIR/xcrun_db` cache".

**Fact — `Toolchain::resolve` makes four, with two `--show-sdk-*` flags.** `crates/tiler-metal-aot/src/driver.rs:86-97` is the whole body: `find_tool(sdk, "metal")`, `find_tool(sdk, "metallib")` — two `xcrun --sdk <sdk> --find <tool>` invocations via `capture` at `:164-173` — then `Self::tool_version` twice, which runs the *located binaries* directly (`capture_tool`, `:210-229`, deliberately not through the launcher), then `sdk_field(sdk, "--show-sdk-version")` and `sdk_field(sdk, "--show-sdk-build-version")`, which are the third and fourth launcher invocations. There is no third `--show-sdk-*` call. Reproduce with `grep -n "find_tool\|sdk_field\|tool_version" crates/tiler-metal-aot/src/driver.rs`, then read `:86-114` in full — the count is only decidable from the body, because `capture` is shared by both flags.

**Inference — the drop is why, and scope is why it survived.** [`drop-the-unread-sdk-path-from-the-resolved-toolchain`](drop-the-unread-sdk-path-from-the-resolved-toolchain.md) removed the fifth call (`--show-sdk-path`) on 2026-08-01 and swept its own scopes — `implementation/metal-aot`, `implementation/build`, `contracts/integrations`. It never held `implementation/frontend`, so it structurally could not have reached `crates/tiler-macros`; verify with `grep -n "scopes:" tickets/drop-the-unread-sdk-path-from-the-resolved-toolchain.md`.

**Inference — this is a load-bearing comment, not a stray number.** The paragraph is the frontend's own statement of what a warm expansion costs and of which observations are the load-bearing ones, and the surrounding argument — that the `--version` executions are what observe the compiler and the launcher answers come from Apple's own cache — is unaffected and correct. Only the counts are wrong.

## Work

Correct both counts in `crates/tiler-macros/src/aot.rs` from the driver body rather than from any document, and state the flags by name so the next removal falsifies a specific sentence rather than a bare number. While in the file, check the rest of that module doc's claims about `Toolchain::prepare`, the miss closure, and the measurement it cites against `crates/tiler-metal-aot/src/driver.rs` and `crates/tiler-build/src/metal_plan.rs` — a wrong count in one sentence is a reason to read its siblings.

## Boundaries

Scope is `implementation/frontend`. `docs/integration/frontends.md` already carries the corrected count at `:371` and belongs to `contracts/integrations`; do not reach into it. `docs/correctness-and-testing.md`'s warm-expansion requirement was corrected under [`correct-the-warm-expansion-xcrun-requirement-in-the-testing-contract`](correct-the-warm-expansion-xcrun-requirement-in-the-testing-contract.md) and needs nothing here.

## Closes when

No sentence in `crates/tiler-macros/src/aot.rs` states a launcher-invocation count or flag set that `Toolchain::resolve` does not perform, and each corrected sentence was derived by reading the driver body rather than by copying a document.
