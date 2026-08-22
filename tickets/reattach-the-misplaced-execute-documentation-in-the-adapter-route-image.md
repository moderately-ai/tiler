---
id: reattach-the-misplaced-execute-documentation-in-the-adapter-route-image
title: Reattach the misplaced execute documentation in the adapter-route image
status: in-progress
priority: p3
dependencies: []
related: [keep-the-path-shared-route-gate-spike-compiling-or-make-its-breakage-loud]
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, runtime, test-fixture]
claimed_from: todo
assignee: worker-imagedoc
lease_expires_at: 1787415365
---
## User-visible outcome

`execute` carries its own documentation, and `contributor_columns` carries a description of what it actually does — so a reader of the shared adapter-route fixture is not told that a function returning a `u64` column count runs a launch grid and panics.

## Why this exists

Found 2026-08-19 by `worker-routegate` while reading `crates/tiler-runtime/tests/adapter_route/image.rs` in full for a different repair, and recorded rather than folded in because it has a different cause. Verified independently by the coordinator at `ef6ab079`.

**Fact — one doc block describes one function and is attached to another.** In `crates/tiler-runtime/tests/adapter_route/image.rs`, the block opening at the anchor `Runs one scalar entry over its launch grid on the calling thread` — including its `# Errors` and `# Panics` sections — is attached to `pub fn contributor_columns(extents: &[RoutedExtentParameter], entry: &ScalarEntry) -> u64`. The function it describes is `pub fn execute`, declared immediately below it, which has **no documentation of its own**. `contributor_columns` returns a count and neither runs a grid nor panics for the stated reasons.

**Fact — no lint can see this, which is why it survived.** `missing_docs` cannot reach either function: the module is private inside a test binary. So the misattachment is invisible to every gate this repository runs, and only reading finds it. This is the same class as the bare-path citations `check-citations.sh` deliberately does not check — recorded here so the pattern is visible rather than treated as a one-off.

**Fact — this file is shared, so the wrong documentation reaches four roots.** *(Corrected 2026-08-22 by `worker-imagedoc` at base `3cca5438`; the sentence below previously read that `image.rs` "is taken through `#[path]` by both `crates/tiler-runtime/tests/adapter_route/main.rs` and `crates/tiler-runtime/tests/identity_join/main.rs`" and that a reader arrives "from any of the three". Both halves were imprecise — the owning root does not use `#[path]`, and a fourth root was omitted.)* `image.rs` is compiled by four roots. Its owning root `crates/tiler-runtime/tests/adapter_route/main.rs` takes it as a plain directory module — `mod image;`, no `#[path]`. Three further roots take it through `#[path]`: `crates/tiler-runtime/tests/identity_join/main.rs`, `crates/tiler-runtime/tests/adapter_route_portability.rs`, and `spikes/target-profiles/metal-subgroup-width-route-gate/src/main.rs`, the last of which this module's own path-shared note names and which does not currently build (see the related ticket). A reader arriving from any of the four sees the same misattachment. Enumerate the three `#[path]` roots with `grep -rn 'path = ".*adapter_route/image.rs"' crates spikes`; the fourth is the owning directory itself. Do not enumerate with `grep -rn 'mod image;'` — `crates/tiler-build/tests/custom_backend/image.rs` and `spikes/target-profiles/scalar-cpu-vertical/src/image.rs` are unrelated files with their own `image` module, so that pattern reports six roots for a file that has four.

## Required work

- Re-audit the Facts above at your actual base before editing and report a per-Fact verdict; re-read the two functions rather than trusting this ticket's description of them.
- Move the block to `execute` and give `contributor_columns` documentation that states what it computes and returns. Do not invent an `# Errors` or `# Panics` section for a function that has neither — check each claim against the body.
- While there, check the file's **sibling** doc blocks for the same off-by-one attachment; per AGENTS.md, finding one instance of a pattern obliges checking its siblings. Report what you found and what you found clean — the negative result is evidence.

## Sibling scan — 2026-08-22 by `worker-imagedoc` at base `3cca5438`

Every doc comment in `crates/tiler-runtime/tests/adapter_route/image.rs` was read against the item beneath it. **One** off-by-one attachment, the one this ticket names. The rest are clean, recorded here so the negative result is evidence that this was a single site rather than a class.

Clean: the module `//!` block; `IMAGE_DOMAIN` (sixteen bytes, and `decode` does compare it exactly); `IMAGE_SCHEMA`; `ELEMENT_BYTES`; `IDENTITY_SCALE_BITS` (`0x3f80_0000` is `1.0f32`); `IDENTITY_BIAS_BITS` (`0x8000_0000` is `-0.0f32`, as its prose turns on); `ScalarEntry` and all seven of its field docs in order; `ScalarImage` and its field doc; `encode`; `Cursor` (its `take` does return `Truncated` rather than panic); `decode` and its `# Errors`; `ScalarImage::entry_for` and its `# Errors`; `ScalarPayloadRefusal` and all nine variants with every field doc; `Placement` and its three field docs; `ExecutionFault` and all three variants with every field doc, including the claim that every variant names its entry, which all three do; and `addresses_program_input`.

The defect's actual shape was narrower than "one block on the wrong function": the block held **two** functions' documentation. `execute`'s summary, `# Errors`, and `# Panics` ran from `Runs one scalar entry over its launch grid` to `rather than a route it should have refused`, and `contributor_columns`' own one-sentence summary was appended after it, inside the same comment. So the repair was a split, not a move — `contributor_columns` already had a true summary and it stayed put. Consistent with a later insertion of `contributor_columns` between `execute`'s doc block and `execute`.

Two claims checked against bodies rather than carried over. `execute`'s `# Panics` is true: it indexes `allocations[read.allocation]` and `allocations[write.allocation]`, and `ScalarHostAdapter` fills `self.allocations` from its pre-commit plan before the dispatch loop. `contributor_columns` got no `# Errors` and no `# Panics` because it has neither — it returns `u64` and its body is one `map_or_else` with no indexing, unwrap, or conversion.

Two wording precisions, both verified in the body and neither behavioural. `contributor_columns` reads `extents.first()`, so its summary now says *first* bound live extent; the previous wording said "when the route published one", which misdescribes a route publishing two. `execute`'s summary now names its `u64` return as the invocations that ran, which is what `ScalarHostAdapter` compares against `launch.grid_threads()` to raise `ExecutionFault::Incomplete`; "invocation" is this module's own word for a grid row.

### Out of scope, reported not repaired

The module's path-shared note says it is compiled into `tests/identity_join/main.rs` and the route-gate spike "through `#[path]`", which omits `crates/tiler-runtime/tests/adapter_route_portability.rs`. That is a stale enumeration, a different defect class from this ticket's attachment off-by-one, and the sentence does defer the full arrangement to `fixture.rs`. Left for the coordinator to route.

## Non-goals

Any behavioural change; renaming either function; changing the `#[path]` sharing arrangement or the portability guard that now holds it; and repairs outside `crates/tiler-runtime/`.

## Closes when

`execute` and `contributor_columns` each carry documentation true of themselves, the sibling scan is reported with both its findings and its clean results, and the touched-package `cargo nextest`, Clippy-with-warnings-denied, and rustdoc gates are green.
