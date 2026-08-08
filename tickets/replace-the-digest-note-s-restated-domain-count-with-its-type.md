---
id: replace-the-digest-note-s-restated-domain-count-with-its-type
title: Replace the digest note s restated domain count with its type
status: done
priority: p3
dependencies: []
related: [repoint-tiler-digest-s-domain-separation-note-at-the-moved-union-check, correct-the-coverage-graph-digest-domain-s-eight-count-and-hyphenated-artifact-prefix]
scopes: [implementation/digest]
shared_scopes: [project/tickets]
paths: []
tags: [identity, digest, documentation]
---

The `tiler-digest` header was repaired on 2026-08-08 to point at the moved union check — and the repair **restated the domain count in prose**, which is the exact rot schedule that repair existed to break.

## Facts

**Verified at `670e7a31` by reading `crates/tiler-digest/src/lib.rs` in full.** The domain-separation note named "eighteen domains" in prose alongside naming `tiler_artifact::domains::GovernedDomain` as what sizes the population. Anchor for the retired wording: `whole admitted set — eighteen domains` (one hit before this ticket's edit, zero after).

**Corrected — the prose figure was *accurate* when this ticket was filed, which is the point rather than a mitigation.** `GovernedDomain` declares eighteen variants at `670e7a31` (`pub(crate) const ALL: [Self; variant_count::<Self>()]`, anchor `Self::ProgramRouteRequirement,`), so "eighteen" and the type agreed. The defect is the maintenance mechanism, not the value: a hand-written number beside a type-sized population is a disagreement scheduled for the next domain, not a present error. Correcting the number would therefore have been a no-op that reset the same clock.

**Corrected — the note did not previously say a bare "eight".** The retired wording (`5d4a30eb`, `git show 5d4a30eb -- crates/tiler-digest/src/lib.rs`) read "is the authority for the envelope's and sidecar's eight" and named the since-retired check `tiler_artifact::proof::tests::no_governed_domain_of_either_container_prefixes_another`. So the stale figure counted **two containers**, not the whole admitted set; the 2026-08-08 repair widened the authority to the union check and the population to all three containers. The rot schedule the ticket describes is right; the quantity that rotted was the envelope-plus-sidecar count, which stands at eleven today (`ENVELOPE` 7 + `PROOF_SIDECAR` 4).

## What closes this

The prose count removed, leaving `GovernedDomain` as the sole statement of the population — so a disagreement between prose and type becomes impossible rather than merely detectable. Keep the container split only if it is derived; if it is written by hand it has the same defect one level down.

**Do not replace "eighteen" with a corrected number.** That is the move this ticket exists to prevent. If a magnitude genuinely helps the reader, bind it to a commit as a historical fact — the sibling did exactly that, pinning its one surviving figure to `d48a33af` so it cannot rot.

**Beware the count you inherit.** "8 of 18" has been repeated across several tickets and **appears in no source file**; the authoritative statement is `crates/tiler-artifact/src/domains.rs`'s module header saying the retired check covered **8 of 11**. Read the source rather than a sibling ticket.

**Non-goals:** do not edit `crates/tiler-artifact/**` — read it to describe it correctly. Do not restate the union check's design; the note points at it, which is right.

Cite by searchable anchor, run the anchor's grep before committing to it, and **name the count of neighbouring claims you checked** so a clean result is distinguishable from an unexamined one.

## Worker audit, 2026-08-08, at base `670e7a31`

**Fact — the note carried no container split, so the conditional in "What closes this" was moot.** It named the three containers without numbering them (retired wording: `across the envelope, the proof sidecar, and the artifact program's identity`). Nothing to remove there.

**Fact — the sibling's finding about the split itself is confirmed by reading `crates/tiler-artifact/src/domains.rs` in full.** `DomainContainer::ENVELOPE`, `PROOF_SIDECAR`, and `PROGRAM_IDENTITY` are hand-written `pub(crate) const usize` values of 7, 4, and 7 (anchor `pub(crate) const ENVELOPE: usize = 7;`). Only their **sum** is compile-time asserted against `variant_count::<GovernedDomain>()` (anchor `the per-container governed-domain counts must account for every variant`); the split is held by the runtime test `each_container_admits_the_number_of_domains_the_contract_records`. That crate is `implementation/artifact` and was read, not edited.

**Fact — "8 of 18" appears in no source file at this base.** `grep -rn "8 of 18"` returns two hits, both in `tickets/` and both warnings against the figure. `crates/tiler-artifact/src/domains.rs` states the authoritative history (anchor `so the check covered 8 of 11`).

**The neighbouring-claims census: 21 claims checked, 20 held, 1 false.** A "claim" here is an assertion in the module header falsifiable against a source outside its own sentence; at that granularity the header carries 21. Verified against their cited sources: the two `docs/artifact-abi.md` requirements (explicit algorithm-and-domain naming, and no inference from digest width, both at anchor `never infers one from a digest width`); the envelope's fixed-header algorithm tag (`docs/artifact-abi.md` byte 16, anchor `governed digest algorithm tag`); this crate as the sole tag-to-implementation mapping (`crates/tiler-digest/src/lib.rs` is the only file in `crates/*/src/` matching `sha2::`); the governed key and tag `0x01`; every consumer-hashed separator being a governed constant; the `sha2` 0.11.0 adoption (`tickets/select-the-governed-artifact-digest-implementation.md`, anchor `0.11.0 adopted (2026-07-27)`, and `Cargo.lock`); the three pinning claims, one per test in this file; the algorithm's former home in `tiler-artifact` (`crates/tiler-artifact/Cargo.toml`, anchor `was here until ADR 0104 moved the governed digest`); ADR 0050's every-hit section-digest validation (anchor `manifest, section lengths/digests, and required meanings on every hit`); Tom's 2026-07-25 decision in ADR 0082 (anchor `Tom decided this on 2026-07-25`); `tiler-ir` minting `IndexRefinementExecutableCoverageIdentity`; Tom's 2026-08-06 decision and its three stated grounds (ADR 0104, anchor `the governed digest moves to a new bottom crate below`); the three re-exported names (`crates/tiler-artifact/src/program/mod.rs`, anchor `pub use codec::{DIGEST_BYTES, Digest, DigestAlgorithm};`); the absence of a parts-digest entry point and the two admitted signatures; the deletion rather than promotion of the general parts-digest (ADR 0082, anchor `no longer exists in any crate`, and no `digest_parts` anywhere under `crates/`); and `docs/artifact-abi.md` recording the obligation and per-container split normatively (anchor `the no-prefix obligation is over the crate's`).

Every anchor above is backtick-free and was grepped against the file it names before being written here. Four earlier drafts of this paragraph cited anchors containing backticks, which nest badly inside a markdown code span and hand a reader a string that does not match the source — the same failure mode as an anchor spanning a line break, in the direction that reads as absence.

**The one false claim, fixed in the same commit — off this ticket's stated subject.** It asserted "`tiler-ir` is the crate every other member depends on" (retired anchor `is the crate every other member`). Two of the twelve other workspace members do not depend on `tiler-ir` — `tiler-metal-aot` declares no `[dependencies]` section at all, and `tiler-digest` itself depends only on `sha2`, which the *same paragraph* explains by siting this crate below `tiler-ir`. The claim was load-bearing for "it cannot be relocated above `tiler-artifact`", so it was replaced with the exact fact that argument needs (`tiler-artifact` is built on `tiler-ir`; `crates/tiler-artifact/Cargo.toml` declares `tiler-ir.workspace = true`) rather than with a repaired census — a per-crate enumeration in prose would reintroduce this ticket's own defect one level over. The same false universal in `crates/tiler-digest/Cargo.toml`'s dependency-list comment was repaired identically.
