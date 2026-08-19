---
id: reattach-the-misplaced-execute-documentation-in-the-adapter-route-image
title: Reattach the misplaced execute documentation in the adapter-route image
status: todo
priority: p3
dependencies: []
related: [keep-the-path-shared-route-gate-spike-compiling-or-make-its-breakage-loud]
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, runtime, test-fixture]
---
## User-visible outcome

`execute` carries its own documentation, and `contributor_columns` carries a description of what it actually does — so a reader of the shared adapter-route fixture is not told that a function returning a `u64` column count runs a launch grid and panics.

## Why this exists

Found 2026-08-19 by `worker-routegate` while reading `crates/tiler-runtime/tests/adapter_route/image.rs` in full for a different repair, and recorded rather than folded in because it has a different cause. Verified independently by the coordinator at `ef6ab079`.

**Fact — one doc block describes one function and is attached to another.** In `crates/tiler-runtime/tests/adapter_route/image.rs`, the block opening at the anchor `Runs one scalar entry over its launch grid on the calling thread` — including its `# Errors` and `# Panics` sections — is attached to `pub fn contributor_columns(extents: &[RoutedExtentParameter], entry: &ScalarEntry) -> u64`. The function it describes is `pub fn execute`, declared immediately below it, which has **no documentation of its own**. `contributor_columns` returns a count and neither runs a grid nor panics for the stated reasons.

**Fact — no lint can see this, which is why it survived.** `missing_docs` cannot reach either function: the module is private inside a test binary. So the misattachment is invisible to every gate this repository runs, and only reading finds it. This is the same class as the bare-path citations `check-citations.sh` deliberately does not check — recorded here so the pattern is visible rather than treated as a one-off.

**Fact — this file is shared, so the wrong documentation reaches two consumers.** `image.rs` is taken through `#[path]` by both `crates/tiler-runtime/tests/adapter_route/main.rs` and `crates/tiler-runtime/tests/identity_join/main.rs`, and is now additionally compiled by `crates/tiler-runtime/tests/adapter_route_portability.rs`. A reader arriving from any of the three sees the same misattachment.

## Required work

- Re-audit the Facts above at your actual base before editing and report a per-Fact verdict; re-read the two functions rather than trusting this ticket's description of them.
- Move the block to `execute` and give `contributor_columns` documentation that states what it computes and returns. Do not invent an `# Errors` or `# Panics` section for a function that has neither — check each claim against the body.
- While there, check the file's **sibling** doc blocks for the same off-by-one attachment; per AGENTS.md, finding one instance of a pattern obliges checking its siblings. Report what you found and what you found clean — the negative result is evidence.

## Non-goals

Any behavioural change; renaming either function; changing the `#[path]` sharing arrangement or the portability guard that now holds it; and repairs outside `crates/tiler-runtime/`.

## Closes when

`execute` and `contributor_columns` each carry documentation true of themselves, the sibling scan is reported with both its findings and its clean results, and the touched-package `cargo nextest`, Clippy-with-warnings-denied, and rustdoc gates are green.
