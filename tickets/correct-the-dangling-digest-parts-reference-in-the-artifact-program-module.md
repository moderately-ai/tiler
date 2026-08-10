---
id: correct-the-dangling-digest-parts-reference-in-the-artifact-program-module
title: Correct the dangling digest parts reference in the artifact program module
status: done
priority: p3
dependencies: []
related: [repoint-tiler-digest-s-domain-separation-note-at-the-moved-union-check, correct-the-reachable-only-under-test-claim-the-delivered-realization-domain-falsifies]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, digest]
---

`crates/tiler-artifact/src/program/mod.rs` refers to a symbol that does not exist, in the crate that owns the re-export it is describing.

## Facts

**Reported by the `tiler-digest` note repair, not coordinator-verified — check it first.** The module is said to state `with \`digest_parts\` private to \`tiler-digest\``. No `digest_parts` exists in `tiler-digest`: the general parts-digest form was **removed**, not made private. The `tiler-digest` header states this correctly — "the general form is gone rather than promoted" — so the two crates disagree, and the one that is wrong is the one describing its own dependency.

**Correction — 2026-08-10.** The present-tense defect claims above (dangling ``digest_parts` private to `tiler-digest``, two crates disagree) are **historical filing-time facts**, not live tree state. At the 2026-08-10 audit base and the current tree the artifact program re-export comment already matches `tiler-digest`'s removal-not-promotion terms; see Outcome.

## Why p3, and why it is filed rather than ignored

It is a doc comment with no gate behind it and no caller misled at compile time — a wrong `private to` reading costs a reader one failed search. But a dangling symbol reference in the crate that owns the re-export is the kind of thing someone later cites as evidence that a private general form exists, and then designs around it. The cost of leaving it is small and cumulative; the cost of fixing it is one sentence.

## What closes this

The sentence restated to match what `tiler-digest` actually exposes, cited by **searchable anchor** rather than line number. Prefer describing the removal in the terms the owning crate uses over inventing a second phrasing — two crates describing the same absence differently is how this drifted in the first place.

**Check the surrounding paragraph's other claims about `tiler-digest`.** A sentence that survived because it reads plausibly usually has neighbours, and this one describes a boundary the reader cannot see from here. **Name the count you checked**, so a clean result is distinguishable from an unexamined one.

Do not edit `crates/tiler-digest/**` (`implementation/digest`, not this scope). If the correct fix turns out to be a change there instead, report it rather than widening.

## Outcome — 2026-08-10

**Code subject already delivered at audit base.** The dangling ``digest_parts` private to `tiler-digest`` wording in `crates/tiler-artifact/src/program/mod.rs` was replaced by the sentence that the general parts-digest this crate carried is gone rather than promoted across the boundary, aligned with `tiler-digest`'s header (`the general form is gone rather than promoted across a crate boundary`). `envelope_digest` remains `pub(crate)` in `tiler-artifact` (`pub(crate) use codec::envelope_digest` / `pub(crate) fn envelope_digest`). No `crates/tiler-digest/**` edit was required or made. No landing commit hash is recorded here for the comment rewrite; code state is the authority.

**Neighbouring digest-block census (close-condition process).** Eight claims checked in the block anchored by `The governed digest algorithm, which \`docs/artifact-abi.md\` requires every` through the `pub use codec::{DIGEST_BYTES, Digest, DigestAlgorithm};` line; all eight verified at the 2026-08-10 audit base and re-verified on the current tree:

1. `docs/artifact-abi.md` requires every digest use to name the governed algorithm explicitly.
2. The surface was promoted for `tiler-cache` on Tom's 2026-07-25 decision (`decide-the-expansion-cache-owner-and-digest-authority`).
3. The expansion cache validates section digests on every hit (ADR 0050).
4. This crate owned the algorithm until ADR 0104 needed it in `tiler-ir`.
5. Tom 2026-08-06: the governed digest is the bottom crate `tiler-digest`.
6. The re-export keeps `tiler_artifact::program::{DIGEST_BYTES, Digest, DigestAlgorithm}` resolving.
7. The general parts-digest is gone rather than promoted; the surface is the algorithm and the opaque digest.
8. `envelope_digest` is crate-private here so an outside caller cannot construct the envelope association.

Reproduce: `rg 'digest_parts' crates/` → empty; `rg 'parts-digest this crate carried gone rather than promoted' crates/tiler-artifact/src/program/mod.rs` → 1; `rg 'the general form is gone rather than promoted' crates/tiler-digest/src/lib.rs` → 1 (line-broken module docs); `rg 'private to .tiler-digest' crates/tiler-artifact` → empty. Sibling `correct-the-reachable-only-under-test-claim-the-delivered-realization-domain-falsifies` re-read the same block as 7 claims after repair; this Outcome records the independent 8-claim census of the digest re-export block alone.
