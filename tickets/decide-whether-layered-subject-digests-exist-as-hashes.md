---
id: decide-whether-layered-subject-digests-exist-as-hashes
title: Decide whether layered subject digests exist as hashes
status: done
priority: p3
dependencies: []
related: [record-the-implemented-artifact-envelope-in-the-contract, prototype-neutral-artifact-codec]
scopes: [contracts/artifacts, contracts/foundation, contracts/decisions, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, identity]
---
**Fact — `docs/artifact-abi.md`'s identity block describes hashes where the tree carries canonical bytes.** It writes `semantic_digest = H("tiler-semantic-v1" || canonical semantic bytes)` and four siblings for the index, schedule, refinement, and plan layers. Every one of those subjects is implemented as an opaque newtype over its exact canonical byte encoding, compared byte for byte: `SemanticGraphIdentity`, `CanonicalIndexRegionIdentity`, `CanonicalScheduledRegionIdentity`, `CanonicalKernelProgramIdentity`, `CanonicalArtifactProgramIdentity`. None is a hash. ADR 0074 convention 2 states that shape as the accepted convention and makes short digests presentation-only.

**Fact — the placeholder spellings match no layered identity encoder.**
Current semantic, index, schedule, kernel-program, and artifact-program
identities use versioned domain-separated canonical bytes. Their version
numbers evolve independently and are intentionally not pinned in this ticket.
The schedule domain now has the same NUL terminator discipline as the other
encoders.

**Fact — governed hashing has broader uses than these layered identities.**
Artifact envelope framing, proof payloads, and cache framing or keys use the
governed digest. That does not turn the canonical layered identity newtypes into
hashes.

**Why this is worth deciding rather than quietly deleting.** Canonical bytes are the stronger construction — identity comparison then rests on nothing, where a hash rests on collision resistance — but they are also unbounded, which is why the artifact identity budget is 64 MiB and why an envelope section carrying a kernel-program identity is budgeted at 64 MiB rather than at a digest width. A compact per-layer key is what an external cache index, a cross-reference value, or a diagnostic would actually want. The contract promises one and the tree provides none, so a reader cannot tell whether the compact keys are unbuilt or abandoned.

**What closes this.** Either specify each layer's compact key as an explicit
derivation over its canonical bytes, with a governed algorithm tag and domain
separator, and say where it is permitted to appear; or record that canonical
bytes are the only layered identity Tiler has and remove the five nonexistent
hash derivations from the contract.

**Scope note.** The ticket declares every contract and implementation area its
stated outcome may change. If research eliminates the need for an ADR or code
change, leave the unused areas untouched rather than narrowing scope after work
has begun.

## Outcome — decided: there are no layered digests, and the five derivations are removed (2026-07-27)

**Decision.** Canonical bytes are the only layered identity Tiler has. `semantic_digest`, `index_digest`, `schedule_digest`, `refinement_digest`, and `plan_digest` are removed from `docs/artifact-abi.md` rather than specified.

**This did not need Tom, because the alternative does not survive.** Specifying compact keys would create a second identity authority over a subject that already has a canonical-byte identity — the exact shape ADR 0082 names, whose agreement with the real identity "could only ever be argued, never checked". A compact key's whole value is being shorter, which means trading collision-freedom for width; ADR 0074 convention 2 already decided that trade the other way and made short digests presentation-only. So the option that looked like a design choice is one an accepted decision had already closed, and offering it would have been offering a defect as a trade-off.

**The supporting facts were measured, not assumed.** The promise appears in exactly one file — no other document or crate mentions any of the five names. Nothing implements them: none of the five domain separators (`tiler-semantic-v1` and its siblings) appears anywhere in `crates/`. And no consumer wants one: all five identity types expose `as_bytes()` and nothing else, and the expansion cache keys on a `ComposedSubject` of length-prefixed canonical byte runs rather than on layered digests.

**A defect was found in the half that stays, and fixing it removed a duplicated authority.** The block also restated the three *real* envelope derivations, spelled `"tiler-section-v1"`, `"tiler-manifest-v1"`, and `"tiler-envelope-v1"`. The crate constants are `tiler.artifact-envelope.section-digest.v1\0`, `…manifest-digest.v1\0`, and `…envelope-digest.v1\0`, and all three are already recorded verbatim under "The governed digest" earlier in the same document. Only the removed paragraph's caveat that every separator in the block was "illustrative" kept three wrong spellings from being three errors — and removing the five derivations would have removed that caveat and promoted them. The restatement is deleted and the block now points at the one authority. Caught by verifying the surviving text after the edit rather than only the removed text.

**Scope.** Only `contracts/artifacts` was touched. The ticket declared `contracts/foundation`, `contracts/decisions`, and `implementation/ir` in case its outcome reached them; the research showed it does not — no ADR asserted the layered digests and no code implements them — so those areas are left alone, as the ticket's scope note instructs.

**Trigger for reconsideration is recorded in the contract**, not only here: a consumer needing a bounded-width cross-reference to a layer, which must first answer which of the two values is the identity, what happens when they disagree, and what checks that they do not.
