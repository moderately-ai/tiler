---
id: correct-the-reachable-only-under-test-claim-the-delivered-realization-domain-falsifies
title: Correct the reachable-only-under-test claim the delivered realization domain falsifies
status: done
priority: p3
dependencies: []
related: [correct-the-dangling-digest-parts-reference-in-the-artifact-program-module, pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, identity]
---

A domain census in `crates/tiler-artifact/src/program/mod.rs` says its fourteen named domains are "reachable only under test". **Thirteen are. One is public.**

## Facts, coordinator-verified at the merge that found it

**Fact.** The sentence is anchored by `The envelope's seven governed domains and this module's seven` and ends "reachable only under test".

**Fact.** `DELIVERED_REALIZATION_DOMAIN` is declared `pub const` in `crates/tiler-artifact/src/program/realization.rs` and publicly re-exported from `program/mod.rs` alongside `AssessmentDisposition` and `DeliveredRealizationBuilder`. The other thirteen are `#[cfg(test)] pub(crate)`.

**Fact — the two counts the sentence rests on are correct** and should not be touched: `DomainContainer::ENVELOPE = 7` and `DomainContainer::PROGRAM_IDENTITY = 7`. Only the reachability clause is wrong.

**Corrected 2026-08-08 — the counts are checked against `variant_count`, not sized by it.** Both are hand-written `pub(crate) const` values in `crates/tiler-artifact/src/domains.rs`. Two separate mechanisms hold them: the `const` block anchored by `the per-container governed-domain counts must account for every variant` asserts that `ENVELOPE + PROOF_SIDECAR + PROGRAM_IDENTITY == variant_count::<GovernedDomain>()`, which pins only the *total*; the runtime test `each_container_admits_the_number_of_domains_the_contract_records` pins the *split*, which a total cannot. `GovernedDomain::ALL` is the one item literally sized by `variant_count`. The correction does not change the verdict — both counts are 7 and both are correct — only the mechanism a reader would go looking for.

## Why it is filed rather than waved through

The same argument the sibling ticket made for `digest_parts` applies: a reader could conclude **no** domain constant is publicly reachable, and then reason about the crate's public surface from a false premise. A domain that is `pub` is contract — its value is observable, and under ADR 0075 its surface is Tom's.

Note the sibling's finding about how this class arises: the false `digest_parts` sentence was **authored by the commit that deleted the symbol**, which rewrote an accurate sentence into an inaccurate one. Check whether this clause was accurate before some later change made `DELIVERED_REALIZATION_DOMAIN` public — `git log -S DELIVERED_REALIZATION_DOMAIN` will say — because that changes the correction from "someone was careless" to "a change moved a symbol and left its description behind", and the note should say which.

## What closes this

The clause restated so a reader can tell which of the fourteen is publicly reachable and which are test-only. Do not restate the counts; they are correct and derived.

**If the correction implies the public domain is an accepted surface, it is not** — say so rather than implying acceptance. Under ADR 0075 a `pub` item is a labelled draft until Tom accepts its exact included and excluded surface; report the surface, do not decide it.

**Check the neighbouring blocks while you are in it.** The sibling worker checked eighteen claims across three comment blocks here and found two false and this one imprecise — a base rate of roughly one in six. **Name the count you checked**, so a clean result is distinguishable from an unexamined one.

Cite by searchable anchor, not line number, and run the anchor's grep before committing to it.

## Worker findings, 2026-08-08

**Fact — this is an authoring slip, not stale text left behind by a symbol move.** `git log -S "The envelope's seven governed domains" -- crates/` returns one commit, `96dfe333` (2026-08-08), which rewrote an accurate predecessor into an inaccurate successor. Its predecessor read `The envelope's four governed digest domains, reachable only under test.` — true, because those four were the only domains the sentence covered and all four were `#[cfg(test)] pub(crate)`. `96dfe333` widened the subject to fourteen and carried the reachability clause across unchanged. `DELIVERED_REALIZATION_DOMAIN` had already been `pub const` for three days by then: `git log -S "pub const DELIVERED_REALIZATION_DOMAIN" -- crates/` returns `8bfcd432` (2026-08-05), which promoted it from the module-private `const` that `b9def62d` (2026-07-25) first declared. The commit message of `96dfe333` states "No new `pub` item", which is true of what it added and is exactly the reasoning that missed the one domain reaching the enumeration through a pre-existing `pub use realization::{…}` rather than through a `#[cfg(test)] pub(crate) use`.

**Fact — the draft public surface, reported not decided.** `DELIVERED_REALIZATION_DOMAIN: &[u8] = b"tiler.artifact-program.delivered-realization.v2\0"` (`crates/tiler-artifact/src/program/realization.rs`). It is `pub` in that module and re-exported by `pub use realization::{…}` in `program/mod.rs`, so both the name and its exact bytes are observable to any consumer. Under ADR 0075 and the crate header's `# Public boundary status` section — "[`program`] is a **reviewed draft boundary** … it is not an accepted public facade until Tom accepts the exact interface" — this is a labelled draft. Whether the domain bytes belong in the accepted surface is Tom's.

**Neighbouring census: 21 claims across 7 comment blocks in the re-export region of `program/mod.rs`, all read against source at this base. 19 verified, 2 defective — neither is this ticket's subject and neither was edited here.**

1. Block `Re-exported because this module's own public accessors return it` (2 claims, both verified): `DecodedBinding::access` returns `BufferAccess` (`crates/tiler-artifact/src/program/codec/view.rs`, `pub fn access(self) -> BufferAccess`); `tiler-runtime`'s `[dependencies]` is `tiler-artifact.workspace = true` alone, with `tiler-ir` dev-only.
2. Block `Re-exported for the reason [\`BufferAccess\`] is: a delivered-realization` (3 claims, all verified): `NumericalObligationKey::occurrence` returns `SemanticOccurrence` and `::new` takes one (`crates/tiler-ir/src/numerics.rs`); the ADR 0081 closure claim as above.
3. Block `The governed digest algorithm, which \`docs/artifact-abi.md\` requires every` (7 claims, all verified — this is the block the sibling ticket repaired, re-read rather than assumed): Tom's 2026-07-25 cache decision is recorded in `tickets/decide-the-expansion-cache-owner-and-digest-authority.md` under `## Decision — Tom, 2026-07-25`; ADR 0050 states validation "on every hit"; ADR 0104 records Tom's 2026-08-06 answer "(b), as a new crate below both"; `codec/mod.rs` carries `pub use tiler_digest::{DIGEST_BYTES, Digest, DigestAlgorithm};` so the three names still resolve; no `digest_parts` exists anywhere under `crates/`; `envelope_digest` is `pub(crate)`.
4. Block `[\`envelope_digest\`] *is* the proof sidecar's association with an envelope` (2 claims, both verified): every use outside the re-export is under `crates/tiler-artifact/src/proof/`; `mod codec;` is a private declaration.
5. The subject block (3 claims): both counts verified; the `crate::domains` union sentence verified against `no_governed_domain_of_this_crate_prefixes_another`, which is pairwise over `GovernedDomain::ALL` rather than per container; the reachability clause false and now corrected.
6. Block `The one shared scalar-arithmetic policy vocabulary, named by re-export rather` (3 claims, 2 verified, **1 defective**): the `tiler-compiler` claim holds (`crates/tiler-compiler/src/policy.rs` and `session.rs` name `tiler_ir::numerics` types directly) and the ADR 0081 closure claim holds, but **the block is attached to the wrong item**. Its subject is the scalar-arithmetic vocabulary re-exported by `pub use tiler_ir::numerics::{…}`; it sits above `pub use requirement::{…}`, which re-exports route-requirement types. `git show 8bfcd432 -- crates/tiler-artifact/src/program/mod.rs` shows it authored in that position, so it has never been attached to the item it describes.
7. Block `[\`DimensionBehaviour\`]'s own payloads, and the arithmetic type a subject` (3 claims, 2 verified, **1 defective**): every artifact does carry a delivered-realization record (`ArtifactVerificationError::MissingDeliveredRealization`), and the ADR 0081 closure claim holds, but **"[`DecodedNumerical`]'s accessors already returned four of them" is wrong by one**. That impl block returns exactly three of the re-exported types — `SubnormalMode`, `NumericalPermission`, `ExceptionalValueAssumption` — plus `&str` and `u32`, which are not among them. It returned the same three at `002b1d63`, the commit that authored the sentence, so this was never true.

Both defects are one-line repairs in the same file and the same `implementation/artifact` scope, deliberately left for the coordinator to ticket rather than folded into a p3 whose brief says not to expand it.

## Outcome and neighbour disposition — 2026-08-09

Commit `08ecf9c5` corrected the subject clause in `program/mod.rs`: the envelope
and program-identity populations remain unchanged, thirteen domain constants
are test-only, and `DELIVERED_REALIZATION_DOMAIN` is named as the one publicly
reachable, still-reviewed-draft constant. Commit `25604880` closed this bounded
repair and created `reattach-the-scalar-arithmetic-block-and-correct-its-accessor-count`
for the two neighbouring defects rather than hiding them in this p3.

That follow-up landed at `6b63c278`. The scalar-arithmetic rationale now sits on
the numerics re-export it describes, and the `DecodedNumerical` note names its
three returned vocabulary types instead of repeating the false count of four.
No domain bytes, visibility, accepted public surface, identity, or codec
behavior changed in either repair.
