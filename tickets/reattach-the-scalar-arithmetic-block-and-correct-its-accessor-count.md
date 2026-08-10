---
id: reattach-the-scalar-arithmetic-block-and-correct-its-accessor-count
title: Reattach the scalar arithmetic block and correct its accessor count
status: done
priority: p3
dependencies: []
related: [correct-the-five-key-domains-noun-on-the-model-re-export-block]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: []
---

Two comment defects in `crates/tiler-artifact/src/program/mod.rs`, both found by a sibling worker reading past its own scope and both confirmed **never true at any commit**, so both take ADR 0106's substitution treatment rather than a dated-beside note.

## Facts

**Fact — the ticket was dispatched with an empty body.** It carried frontmatter only (367 bytes at `c81f9257`), so the dispatching brief was the sole statement of the work. Every claim below was re-derived from source at that base rather than inherited.

**Fact — the scalar-arithmetic vocabulary block was attached to the route-requirement re-export, and never to the item it describes.** The block anchored `The one shared scalar-arithmetic policy vocabulary, named by re-export rather` describes a dimension set, behaviour spaces, means, locus, and structured provenance — the vocabulary `pub use tiler_ir::numerics::{…}` re-exports. It sat immediately above `pub use requirement::{…}`, which re-exports `RouteRequirement`, `RouteRequirementSubject`, `RouteResourceDimension`, and their siblings. `git show 8bfcd432:crates/tiler-artifact/src/program/mod.rs` puts the block at line 505 and `pub use requirement::{` at 514 with `pub use tiler_ir::numerics::{` at 519; the block is absent from `8bfcd432^`. It was therefore misattached by the commit that authored it and has never annotated the numerics re-export.

**Fact — `DecodedNumerical`'s accessors return three of the re-exported types, not four, and returned three when the sentence was written.** `crates/tiler-artifact/src/program/codec/view.rs` declares ten accessors on `DecodedNumerical`: `profile_key` returns `&'a str` and `canonical_arithmetic_nan_bits` returns `u32`, neither of which the `pub use tiler_ir::schedule::{…}` list names; the remaining eight return `SubnormalMode` (twice), `NumericalPermission` (four times), and `ExceptionalValueAssumption` (twice) — three distinct re-exported types. `git show 002b1d63:crates/tiler-artifact/src/program/codec/view.rs` carries the same ten accessors with the same return types, and `002b1d63` is the commit that wrote `accessors already returned four of them`. No reading of the sentence yields four: counting accessors rather than types gives eight, and counting all returned types gives five.

**Fact — dated corrections inside Rust comments are established practice in this crate tree, so the treatment needed no new convention.** `rg -n "2026-0" crates/ --glob '*.rs'` matches **186** lines at this base (re-measured 2026-08-10). `crates/tiler-conformance/src/bf16_vertical.rs` carries three Corrected-2026-08-07 blocks, of which two open `The reason given here was retired and is struck. Corrected 2026-08-07.` and one opens `A boundary claimed here was retired and is struck. Corrected 2026-08-07.`, each with the retired wording quoted; `crates/tiler-cache/src/expansion/collect.rs` carries `Corrected 2026-08-06 — …`. The repairs follow that idiom, in plain prose rather than markdown blockquote because these are `//` comments and are never rendered.

**Correction — 2026-08-10.** The live sentence above previously said `grep -rn "2026-0" crates/ --include='*.rs'` gives 159 hits and that bf16_vertical carries three blocks opening the exact phrase `The reason given here was retired and is struck. Corrected 2026-08-07.` Measured at close the count was 159; at audit base `c99ac549` and on re-measure 2026-08-10 it is 186. Only two of the three Corrected-2026-08-07 blocks open with that exact phrase; the third uses the distinct opener quoted in the live Fact.

## Outcome

Both repairs landed in `crates/tiler-artifact/src/program/mod.rs`.

The vocabulary block now sits above `pub use tiler_ir::numerics::{…}` and carries a `Reattached 2026-08-08` note recording that it was misattached from `8bfcd432` and that the requirement re-export lost nothing, because a reader who remembers a rationale above `pub use requirement::{…}` would otherwise read the move as a deletion.

The accessor sentence now reads `already returned three of them` and names `SubnormalMode`, `NumericalPermission`, and `ExceptionalValueAssumption` rather than only counting them, so the claim is checkable against the accessor list. A `Corrected 2026-08-08` note quotes the retired wording, records that it was false at `002b1d63` as well, and states inline that quoting it keeps it greppable — a later hit on `returned four of them` reaches the note and proves the string is present, not that the claim stands.

## Neighbouring census

The five re-export comment blocks surrounding the two defects were read and their claims checked against source: **26 claims, 25 verified, 1 imprecise, 0 false.** Verified: `DecodedBinding::access` returns `BufferAccess`; `tiler-runtime` reaches `tiler-ir` only as a dev-dependency, so its linked closure is `[tiler-artifact]` as ADR 0081 item 2 fixes it; `NumericalObligationKey::new` takes and `::occurrence` returns a `SemanticOccurrence`; the digest block's ADR 0050, 0075, 0081, 0082, and 0104 references, its `decide-the-expansion-cache-owner-and-digest-authority` ticket, and its claim that only the plain and qualified digest forms survive in `tiler-digest`; `envelope_digest` is `pub(crate)`; and the domain block's counts — seven envelope domains, seven program-identity domains, thirteen crate-visible under `cfg(test)` as 7 + 5 + 1 — each of which `crates/tiler-artifact/src/domains.rs` independently pins against `variant_count`.

The one imprecision was outside this ticket's two repairs and is filed as [`correct-the-five-key-domains-noun-on-the-model-re-export-block`](correct-the-five-key-domains-noun-on-the-model-re-export-block.md): the domain block calls the `model` re-exports `the five key domains`, and while five is the right count, `ARTIFACT_DOMAIN` is the artifact-identity separator rather than a key domain — `domains.rs` distinguishes them, documenting `ProgramArtifact` as `Separator opening the canonical artifact-program identity` against the four `Separator of one … canonical key` variants. The count holds; the noun covers four of the five. The imprecision remains live in source until that remainder lands.

Two sibling sweeps preceded this one over the same file, checking 12 and 21 claims; the 21-claim sweep is recorded under `## Worker findings, 2026-08-08` in [`correct-the-reachable-only-under-test-claim-the-delivered-realization-domain-falsifies`](correct-the-reachable-only-under-test-claim-the-delivered-realization-domain-falsifies.md) and covers the same seven blocks. This census was run against source rather than read off that record, reaches the same verdict on every claim it shares, and splits some of them finer, which is where the higher count comes from. The `five key domains` imprecision is the delta — no prior sweep names it. Both defects repaired here survived two bounded sweeps, so treat this census as a floor rather than a clearance.
