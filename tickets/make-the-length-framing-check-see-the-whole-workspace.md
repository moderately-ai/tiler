---
id: make-the-length-framing-check-see-the-whole-workspace
title: Make the length-framing check see the whole workspace
status: in-progress
priority: p1
dependencies: []
related: [finish-consolidating-tiler-ir-length-framing, pin-the-admitted-unsafe-sites-in-the-workspace-gate, implement-boundary-property-model]
scopes: [implementation/compiler, implementation/reference, implementation/metal-aot, implementation/cache, implementation/workspace, implementation/ir]
shared_scopes: []
paths: []
tags: [implementation, identity, gate]
claimed_from: todo
assignee: agent-framing
lease_expires_at: 1785041764
---
The canonical length-framing consolidation was never workspace-wide, and the test that enforces it is structurally incapable of noticing.

**Fact — the enforcing test is crate-local.** `crates/tiler-ir/src/identity.rs::length_framing_has_exactly_one_definition_in_this_crate` walks `env!("CARGO_MANIFEST_DIR")/src`. Its own documentation says it exists because "stating a convention did not hold it" after five copies grew inside `tiler-ir`. It cannot see any other crate, and its name says so — but the module doc above it states the rule as though it governed identity derivation generally.

**Fact — `implement-boundary-property-model` found a sixth and seventh copy in `tiler-compiler`** (`frontier.rs`, `selection.rs`) and consolidated them. That work is done; it is cited here as how the gap surfaced, not as remaining work.

**Fact — copies remaining at `8674378`, from `grep -rn -F <pattern> --include='*.rs' crates prototypes`:**

| Crate | Sites | Reaches `tiler-ir`? |
| --- | --- | --- |
| `tiler-compiler` | `encode_len` in `cover.rs`, `fusion_legality.rs`, `legality.rs`, `capability.rs`; `encode_bytes` in those four plus `region.rs`, `explain.rs` | yes |
| `tiler-reference` | `encode_len`, `encode_bytes` in `lib.rs` | yes |
| `tiler-metal-aot` | `push_len` in `identity.rs` and `family.rs`; `push_slice` in `family.rs` | **no** |
| `tiler-cache` | assorted `.len() as u64` in `bundle.rs`, `store.rs`, `collect.rs`, `tests.rs` | **no** |

**Fact — the dependency closures decide which of those are defects.** `crates/tiler-compiler/Cargo.toml` and `crates/tiler-reference/Cargo.toml` both declare `tiler-ir`, so their twelve sites are ordinary duplication and consolidating them is mechanical. `tiler-metal-aot` declares no workspace dependency and `scripts/check_workspace.py` pins `"tiler-metal-aot": []` (ADR 0077 item 2), so **its copies are forced by an accepted decision, not a defect against `crate::identity`** — but it holding *two* copies of one framing is a defect within that crate. `tiler-cache` declares only `tiler-artifact`, and a transitive dependency is not usable in Rust without declaring it.

**Do not sweep `tiler-cache` on the grep.** Read each site: `if bytes.len() as u64 > limit`, `self.entries.len() as u64`, and `selected: selected.len() as u64` are *counting and comparison*, not canonical framing, and rewriting them onto a framing helper would be wrong. `AGENTS.md`: grep counts are not scope assessments, and duplication must be understood before it is consolidated.

**Do not touch the two deliberate independent assertions.** `crates/tiler-ir/src/shape/env.rs:1218` and `crates/tiler-compiler/src/feasibility.rs:1458` spell the eight-byte prefix out by hand *in tests*, and `identity.rs`'s own documentation states that this independence is exactly what would catch the framing width changing — a test that checked the encoder with the encoder's own helper could not. Both are inside `#[cfg(test)]`, which is why the existing crate-local check already skips them; whatever replaces it must skip them for the same stated reason rather than by accident.

## Closes when

The twelve `tiler-compiler` and `tiler-reference` sites use `tiler_ir::identity`; `tiler-metal-aot` holds exactly one definition of its own framing with a comment naming the closure decision that forces it to have one at all; each `tiler-cache` site is classified as framing or counting with the framing ones consolidated onto whatever single definition that crate is permitted; the enforcement sees every workspace member rather than one crate; and the full gate passes.

**The enforcement's shape is the design question.** The established instrument is `scripts/check_workspace.py`, which already owns workspace-wide mechanical predicates and already carries admitted-exception tables with reasons — `ADMITTED_UNSAFE_SITES` pins `(path, item signature, reason)` triples so that adding, moving, renaming or rewording one fails the gate until the pin is updated in the same change. A closure-forbidden crate needs exactly that treatment: its copy is admitted, with the decision that forces it named beside it, so a *second* admitted copy is a diff someone must look at. Decide whether the crate-local test is then retired or kept as a faster inner loop; keeping both is defensible but only if the module documentation stops implying the crate-local one is the authority.

**A caution the work should not repeat.** The existing check reports success by finding nothing, and finding nothing is what a broken search also does — this ticket was itself opened after a first sweep returned six clean results because `zsh` expanded an unquoted `--include=*.rs`. Whatever check lands must name and count its population, so that a scan matching zero files fails loudly instead of passing.
