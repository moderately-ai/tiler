---
id: compose-the-complete-expansion-cache-subject
title: Compose the complete expansion cache subject
status: done
priority: p1
dependencies: []
related: [implement-the-expansion-cache-protocol, derive-the-pre-compilation-artifact-program-subject]
scopes: [implementation/cache, implementation/metal-aot, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [cache, identity, correctness]
---
`tiler-cache` keys an entry by the governed digest of a canonical subject the producer supplies, and a bundle carries a **whole artifact envelope**. A conforming subject must therefore determine every byte of that envelope, which is what `docs/backends/metal.md` already requires when it says "full artifact identity is the key".

**No component emits that subject as one canonical byte run.** `crates/tiler-metal-aot/src/identity.rs` emits the half that determines the `metallib` — source, target, exact ordered compile and link flags, SDK and tool versions, evidence class — and says nothing about the plan variants, ABI bindings, routing, or interface wrapped around it. Two artifacts that agree on the compilation and differ in their plan portfolio would hash to one key, and the cache would serve either for the other. That is a silently wrong result, not a lost hit.

`tiler-cache` deliberately did not invent the composition: it cannot compose a subject without becoming an authority over encodings it does not own, and `crates/tiler-cache/src/expansion/key.rs` states the obligation and this gap in terms rather than assuming a caller will meet it.

## What this ticket owes

- Decide where the composed subject is derived. The artifact layer already derives `CanonicalArtifactProgramIdentity` from a verified artifact, and the driver already derives its compilation subject; the open question is which component composes them and whether the composition happens before compilation, which it must, since the key is needed on a miss.
- Establish the composition by a mechanism rather than by vigilance, as `identity.rs` does — a new identity-bearing input must fail to compile until it reaches the subject.
- Keep it domain-separated and length-prefixed, so no two component subjects can be re-split into a third.
- State whether the composed subject subsumes `tiler-metal-aot`'s or wraps it, and preserve its `SameHost` reuse bound either way.

Until this lands, a caller passing the driver's subject alone is under-keying and `tiler-cache` cannot detect it.

## Outcome

**Retraction first.** This ticket's own framing said `identity.rs` had landed "the complete compilation key subject". It had not: it is complete with respect to the *compilation* and silent about the *artifact*. The dispatch that opened this ticket inherited the same wrong version and it is retracted here rather than quietly corrected.

### What landed

`tiler_cache::expansion::ComposedSubject` composes the key subject; `crates/tiler-cache/src/expansion/subject.rs` is the whole of it.

- **Where.** In `tiler-cache`. `tiler-metal-aot` was eliminated because ADR 0082 item 1 keeps its dependency closure empty, so it can never name the artifact program. `tiler-artifact` was eliminated because ADR 0082's first rejected alternative already refused to merge cache concerns into it, and it knows nothing about the compilation. A new orchestrator crate was eliminated because ADR 0075 makes a new publicly reachable namespace Tom's decision. `tiler-cache` survives because composing needs no knowledge of the component encodings: the frame — which roles exist, in which order, how their runs are delimited — is exactly the part no producer can own, because no producer can see the others.
- **Wraps, does not subsume.** `tiler-metal-aot`'s bytes appear unaltered as one run of the `BackendCompilations` facet. Subsuming would mean restating its encoding in the cache, which is the second-authority failure ADR 0082 rejected for the digest. The `SameHost` bound survives with no work: the evidence tag is encoded *inside* those bytes, they travel through the frame byte for byte, so two evidence classes still give two composed subjects and two keys. **No dependency was added** — the facet is opaque bytes.
- **The compilations facet is a counted, ordered sequence, not one run.** One artifact may carry up to sixteen payloads, and a selection naming three Apple families is three compilations producing three independently identified payloads. A facet naming one compilation would under-key every multi-family artifact.
- **Mechanism, not vigilance.** `SubjectFacets` is destructured irrefutably in `compose`, so a new field fails to compile until it reaches the bytes; the facet table is typed `[(SubjectFacet, &[&[u8]]); SubjectFacet::ORDER.len()]`, so a new facet fails to compile from the other direction. This is a claim about *roles*. Completeness within a facet stays the supplying authority's obligation, discharged by that authority's own mechanism.
- **Framing.** Versioned domain `tiler.cache.composed-subject.v1\0`, a leading facet count, then per facet a stated `u32` tag, a `u64` run count, and each run `u64`-length-prefixed. No ordinals, no wildcard match, big-endian throughout.
- **The public surface changed so under-keying is unrepresentable.** `lookup` and `get_or_publish` take `&ComposedSubject`; `CacheKey::derive` narrowed to `&ComposedSubject` and a crate-private `derive_bytes` serves the bundle decoder, which must hash carried bytes *without* parsing them. Recorded for `accept-the-tiler-cache-public-boundary`, including the two shape questions it leaves open.

### How the headline defect was proved closed

`two_artifacts_differing_only_in_plan_portfolio_key_differently` composes two subjects sharing one backend compilation run byte for byte and differing only in the artifact-program facet, and asserts the composed bytes differ, the derived `CacheKey`s differ, and — the half that stops it passing for the wrong reason — that identical facets still derive one key. Before this change both artifacts keyed on the compilation alone and the cache served either for the other.

Nine tests in total: domain separation, determinism, per-facet movement under an exhaustive `match` with no wildcard, re-splitting defeated within a facet and across the facet boundary, order and cardinality of the compilation sequence, the refusal for an unfillable facet at every position, the refusal naming its facet, and — found while adding a third constant — that no governed digest domain in this crate prefixes another, which `tiler-artifact` checks over its own domains and this crate did not.

### What the composed subject determines, and what it does not

**Determines:** that both facets are named and non-empty; the exact bytes of each; the count and order of the compilation runs; and that no two facet contents can be re-split into a third subject.

**Does not determine:** that a facet's bytes are the supplying authority's real subject. Distinguishing a genuine artifact-program subject from one a caller invented means parsing an encoding this crate does not own. Requiring every facet to be *named and non-empty* is the strongest check available here, and it is stated as that rather than as completeness.

### The split, and the state it leaves the crate in

**No component can fill the `ArtifactProgram` facet.** `CanonicalArtifactProgramIdentity` folds exactly the right subjects and is derived from a *verified* artifact, which requires a `PayloadDigest` and therefore the compiled bytes — while the key is needed on a **miss**, before compilation. `derive-the-pre-compilation-artifact-program-subject` (p1, `implementation/artifact`) owns deriving one.

`compose` refuses an empty facet, so `tiler-cache` is **composable and not yet usable**. That is the deliberate choice: the alternative — admitting an empty facet — is the silent under-key this ticket exists to remove, and the crate has no production caller today, so refusing costs nothing and closes the hole absolutely. ADR 0050 and ADR 0082 now say this in those words rather than describing a gap that no longer exists.

### Not claimed

Nothing here is evidence about crash or race behaviour; the composition is pure. `bind-the-cache-subject-to-the-carried-payload-provenance` is **not** made unnecessary — composition decides what a key covers, that ticket decides whether the covered subject describes the artifact beside it. It is made *reachable*: the cache now owns the outer frame and can count and bound the compilation facet's runs without parsing any producer's encoding. That ticket records the narrowed choice.

### Also corrected

`crates/tiler-metal-aot/src/identity.rs` claimed the expansion cache had no accepted owner and pointed at a ticket ADR 0082 has since closed. Its module documentation and its `dead_code` reason now name `tiler-cache`, and a new section states plainly that these bytes are complete of the compilation and not of the artifact, so the next reader does not repeat the mistake this ticket retracts.
