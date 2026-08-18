---
id: repair-the-private-intra-doc-links-the-public-rustdoc-gate-cannot-see
title: Repair the private intra-doc links the public rustdoc gate cannot see
status: done
priority: p3
dependencies: []
related: [state-the-rule-that-a-deterministic-budget-is-a-derivation]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

Every intra-doc link in `tiler-compiler`'s private items resolves, and the population the public rustdoc gate cannot reach is either covered by a check that can fail or truthfully recorded as unchecked.

## Why this exists — filed 2026-08-18 from the deterministic-budget delivery

**Fact (worker-reported, coordinator-relayed; re-verify at your base).** The workspace gate runs `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`, which never renders `pub(crate)` items, so a broken intra-doc link in private docs cannot fail it — the AGENTS.md case about confirming a check reaches its subject. `cargo doc --no-deps --document-private-items -p tiler-compiler` exits 101 at base `2e1cef3c` on **sixteen pre-existing broken intra-doc links** across `cover.rs`, `estimate.rs`, `frontier.rs`, `governed.rs`, `lowering.rs`, `pipeline.rs`, `pipeline/trace.rs`, `program.rs`, `region.rs`, `request.rs`, and `target/feasibility.rs`. Reproduce with that command and enumerate the exact population at your own base before repairing.

## Required content

- Repair each broken link (or convert to plain text where the target genuinely has no path from the doc's scope), reading each doc's intent rather than mechanically satisfying the resolver.
- Decide and record whether `--document-private-items` should join a gate: if yes, wire it and make it fail deliberately once (quote the failure); if no, record where the unchecked population is and why the cost is declined. Do not silently leave the check unreachable while implying coverage.
- Census the final state: the command exits 0, or the ticket names exactly what remains unresolved and why.

## Closes when

The private-items rustdoc run is clean for tiler-compiler (or its residual is enumerated with reasons), the gate decision is recorded, and any new check has been observed failing.

## Fact audit at base `01fc9682` (2026-08-18)

The ticket's one Fact is **verified with one correction**: it cites base `2e1cef3c`, and this work ran from `01fc9682`. Re-enumerated at that base with the ticket's own command, `cargo doc --no-deps --document-private-items -p tiler-compiler` exits 101 on **sixteen** broken intra-doc links, and the eleven named files are exactly the eleven that carry them. The per-file split the Fact does not give is: `cover.rs` 3, `pipeline.rs` 2, `request.rs` 2, `target/feasibility.rs` 2, and one each in `estimate.rs`, `frontier.rs`, `governed.rs`, `lowering.rs`, `pipeline/trace.rs`, `program.rs`, `region.rs`.

The Fact's premise is verified too, and on a live subject rather than by inspection — see the first perturbation below: with one link broken, the private-items run exits 101 and the public gate exits 0 on the same tree.

## Repair census — sixteen links, sixteen dispositions

Nine had a resolvable target and are now linked to it. Seven name an item rustdoc never compiles, and those are the AGENTS.md `#[cfg(test)]` case verbatim: no path exists from any scope, so the link became a plain code span with the doc's claim intact.

**Linked to the right item.**

| site | was | now | why |
| --- | --- | --- | --- |
| `cover.rs`, `admits the member` | `duplication_refusal` | `DuplicationLegality::refusal` | The decision is `refusal` on the private `DuplicationLegality`. `git show 3e1e1a6a:crates/tiler-compiler/src/cover.rs` has the same method and no free function of that name, so the link never resolved rather than rotting. |
| `cover.rs`, `admits each of them` | `duplication_refusal` | `DuplicationLegality::refusal` | Same referent. |
| `cover.rs`, `refuses that member with` | `duplication_refusal` | `DuplicationLegality::refusal` | Same referent. |
| `estimate.rs`, `exact or proven-upper-bound` | `ResourceRequirements` | `tiler_ir::schedule::ResourceRequirements` | The doc's own sentence already says "already exists, in `tiler_ir::schedule`". `crates/tiler-metal/src/synchronization_requirement.rs` spells the same dependency link the same way. |
| `frontier.rs`, `takes that carrier's width from` | `ByteAlignment::natural_for` | `crate::boundary::ByteAlignment::natural_for` | `frontier.rs` imports `AlignmentRequirement` and `AlignmentGuarantee` from `crate::boundary` but not `ByteAlignment`; the module re-exports it as `pub(crate) use tiler_ir::program::{…}`, and this file's own tests already call it by that path. |
| `lowering.rs`, module doc | `LoweringFamily::IndexAccess` | `crate::capability::LoweringFamily::IndexAccess` | Not imported here. `legality.rs` writes the identical full path for the identical referent. |
| `pipeline/trace.rs`, `for every cover region` | `PlanRejection::RegionUnimplemented` | `crate::selection::PlanRejection::RegionUnimplemented` | `trace.rs` sees its parent through `use super::*`, and `pipeline.rs` imports `CoverFrontiers, PlanStructuralCost, RegionFrontier, SelectedPlan, SelectedPortfolio, SelectionError, …` from `crate::selection` — not `PlanRejection`. |
| `program.rs`, `travels as the planner's own` | `SemanticMemberId` | `crate::region::SemanticMemberId` | The field three lines below is declared `occurrence: crate::region::SemanticMemberId`; the type is not imported into this module. |
| `target/feasibility.rs`, `an axis …` | `Relation` | `CapabilityAxis::relation` | The sentence lists the things whose change bumps the revision, and every other entry is an item: `satisfies`, `CapabilityAxis::admits`, `authority_matches_phase`. What varies per axis is the `relation` method. Reworded to "an axis's" so the possessive is explicit. |
| `target/feasibility.rs`, `a comparison …` | `Relation` | `CapabilityRelation` | Here the sentence enumerates what each axis *has* — a `u64` bound, a `Quantity` unit, a comparison relation — so the referent is the type. |

**Converted to a plain code span, with the referent named.** Each is `#[cfg(test)]`, so rustdoc does not compile it and no path to it exists in either mode.

| site | referent | evidence it is unreachable |
| --- | --- | --- |
| `governed.rs`, `The count is stated by` | `GOVERNED_INDEX_ACCESS_CAPABILITIES` | Declared `#[cfg(test)] pub(crate) const …: usize = 21;` nine lines above. Now reads "the test-only `GOVERNED_INDEX_ACCESS_CAPABILITIES`". |
| `pipeline.rs`, `Separate from` | `compile` | `compile` carries `#[cfg(test)]` and its own doc says "It is `cfg(test)` for that reason". Now "the test-only `compile`". |
| `pipeline.rs`, `Both are what this build ships when` | `compile` | Same item. |
| `region.rs`, `stays the law-blind entry` | `form_region_candidates` | Declared `#[cfg(test)]` twenty lines above `form_region_candidates_with_realizations`. Now "the test-only `form_region_candidates`". |
| `request.rs`, `two derivations of one measurement are what drift` | `crate::pipeline::conformance` | `pipeline.rs` declares `#[cfg(test)] mod conformance;`. The neighbouring test names in the same paragraph were already plain code spans, so this is now consistent with them. |
| `request.rs`, `declaration now accounts for` | `crate::pipeline::conformance` | Same module. |

Nothing was deleted to satisfy the resolver: every sentence still names its referent, and the four sites where the referent is test-only now say so, which the rendered page could not otherwise show — rustdoc omits the item entirely, so a bare name would send a reader looking for a page that does not exist.

## Gate decision: yes, `--document-private-items` should join the gate — and it needs one prerequisite

**Recommendation: wire it, as a second command on the `Makefile`'s `doc` target, after the remaining crates are repaired.** The `Makefile` is on the gate-carry list and belongs to the coordinator, so nothing here edits it. The proposed change:

```make
doc:
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items --locked
```

The argument is the AGENTS.md rule about a check reaching its subject. `tiler-compiler` declares five `pub` modules — `capability`, `legality`, `physical_provider`, `session`, `target` — out of the forty-odd in `lib.rs`, so the current rustdoc step cannot report a diagnostic about most of the crate. It is the same shape as the `#[cfg(test)]` example AGENTS.md already gives: not a check that is failing to notice, but one whose subject is out of frame.

Cost is not the obstacle. On this host with a warm target directory, `cargo doc --no-deps -p tiler-compiler` takes 1.28s and the same command with `--document-private-items` takes 1.52s, both measured after `touch crates/tiler-compiler/src/lib.rs`.

**Both commands, not one.** `rustdoc::private_intra_doc_links` fires in both modes — the second perturbation below shows it — so the private run does not lose the public run's checks. Keeping the public pass is what states that the *shipped* page set is clean, which is a different claim from the internal one and would be silently retired if the private run were the only one.

**The prerequisite is real and out of this ticket's scope.** With `tiler-compiler` clean, the workspace private-items run still reports 36 diagnostics across seven crates — `tiler-ir` 18, `tiler-reference` 6, `tiler-macros` 4, `tiler-artifact` 3, `tiler` 2, `tiler-build` 2, `tiler-cache` 1 — of which 24 are `rustdoc::redundant_explicit_links` and 12 are broken links. Every other member is clean, including the three prototypes and `tiler-conformance`. Wiring the command today would land a red gate. That work and the wiring are filed as [`repair-the-remaining-private-intra-doc-links-and-wire-the-private-items-rustdoc-gate`](repair-the-remaining-private-intra-doc-links-and-wire-the-private-items-rustdoc-gate.md).

**So the unchecked population, stated rather than implied:** until that ticket lands, every private and `pub(crate)` item's documentation in every crate remains outside the gate. `tiler-compiler`'s share of it is now clean and is held there only by running the command by hand.

### Perturbation 1 — the private-items command can say no, and the public gate cannot

`cover.rs`'s repaired link was reverted to `[`duplication_refusal`]` and both commands run on that tree:

```text
$ RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items -p tiler-compiler --locked
error: unresolved link to `duplication_refusal`
   --> crates/tiler-compiler/src/cover.rs:189:7
    |
189 | /// [`duplication_refusal`] admits the member.
    |       ^^^^^^^^^^^^^^^^^^^ no item named `duplication_refusal` in scope
error: could not document `tiler-compiler`
priv EXIT=101

$ RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-compiler --locked
pub EXIT=0
```

Reverted; `shasum -a 256 crates/tiler-compiler/src/cover.rs` returns `9a9389ac98a3382fc0baee01ebc7f1b959c6fa9069d2722ab810fd4b6cc5770e` before and after.

### Perturbation 2 — the two runs are not the same check

A temporary line linking the public `LoweringFamily` to the private `crate::cover::CoverPolicy` was added to `capability.rs`. Both runs refuse it, with different notes:

```text
$ RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-compiler --locked
error: public documentation for `LoweringFamily` links to private item `crate::cover::CoverPolicy`
    = note: this link will resolve properly if you pass `--document-private-items`
    = note: `-D rustdoc::private-intra-doc-links` implied by `-D warnings`
pub EXIT=101

$ RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items -p tiler-compiler --locked
error: public documentation for `LoweringFamily` links to private item `crate::cover::CoverPolicy`
    = note: this link resolves only because you passed `--document-private-items`, but will break without
priv EXIT=101
```

That is what settles "add" versus "replace": the private run keeps the public-to-private check, so adding it removes nothing, and the reason to keep the public run is the shipped page set rather than a lost lint. Reverted; `capability.rs` returns to `e25af41f3e708c391d3e07827ebd120c168c10937dd1b4dda172cb233f5d0afd` and leaves `git status` clean.

## Final census and commands (base `01fc9682`, repairs at `c36f80fe`)

`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items -p tiler-compiler` **exits 0**. Nothing remains unresolved in this crate. Every command below was run on `c36f80fe`'s tree.

| command | result |
| --- | --- |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items -p tiler-compiler` | exit 0 |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-compiler` | exit 0 (public gate unchanged) |
| `cargo check -p tiler-compiler` | exit 0 |
| `cargo nextest run -p tiler-compiler` | 959 passed, 1 skipped |
| `cargo fmt --check` | exit 0 |
| `make citations` | exit 0 |
| `tkt lint` | exit 0 |
| `git diff --check` | exit 0 |

No test reads doc text: the suite is byte-for-byte the same population before and after, and the change touches only `///` and `//!` lines — no signature, item, visibility, or behaviour changed.
