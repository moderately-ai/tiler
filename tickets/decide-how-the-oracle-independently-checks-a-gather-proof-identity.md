---
id: decide-how-the-oracle-independently-checks-a-gather-proof-identity
title: Decide how the oracle independently checks a gather proof identity
status: in-progress
priority: p2
dependencies: []
related: [admit-the-selected-data-dependent-index-representation, carry-the-gather-relation-through-the-compiler-vertical]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, identity, reference]
claimed_from: todo
assignee: worker-oracleid2
lease_expires_at: 1787449582
---
## User-visible outcome

The reference oracle can independently check that a static gather resolution's retained proof identity is the one the index layer minted, instead of being able to read the identity but never to derive anything to compare it against.

## Exact-base Fact audit and readiness gate — 2026-08-22, `f69829143a387a8e117858dbcaad416715f7e788`

Run by `worker-oracleid2`. Every verdict below comes from reading the named file in full at this base, plus two compiler probes described under "Commands run". **The audit changes what this ticket is for**, so the enumeration and recommendation that follow replace the two-option decision stated under "The decision"; that section is retained verbatim below with its own correction note rather than rewritten, so its wording stays searchable.

### Per-Fact verdict

**Fact 1 — "the boundary is deliberate and the missing constructor is the point": verified, and *understated* in the direction that matters.** `crates/tiler-ir/src/index/model.rs` declares `pub struct GatherIndexBoundsProofIdentity(pub(super) Vec<u8>)` and its doc says `No public constructor and no byte conversion`, with `as_bytes` the whole public surface. That much is exactly as stated. What the Fact does not say is that reimplementing the encoding downstream is not merely a *fork* — it does not compile. `encode_gather_bounds_identity` in `crates/tiler-ir/src/index/builder/gather.rs` consumes four things a downstream crate cannot obtain at all: the `u32` ordinals inside `VerifiedTensorAccessId`, `VerifiedTensorId`, and `VerifiedDimensionId` (whose fields are `pub(super)` and whose only accessor is `pub(super) fn as_usize`, with no other `impl` block on any of the three), and `IndexDomainFactSource::tag`, which is `pub(super) const fn tag`. The domain constant `GATHER_INDEX_BOUNDS_PROOF_DOMAIN` and the helper `bounded_index` are private too. This was verified by compiling the attempt, not inferred: see the `E0624` negative control under "Commands run". The correction matters because it raises the price of the ticket's option 1 well above what the ticket implies — a re-derivation entry point taking raw parts would have to widen three opaque handle types as well.

**Fact 2 — "a worker cannot mint the fix": verified as to authority, but its premise does not survive the enumeration.** AGENTS.md does reserve consequential public boundaries to Tom, and "Holding an identity one could not have constructed is precisely what makes it evidence the proof ran" is the correct reading of the type's doc. The premise that does not survive is the unstated one — that closing this ticket *requires* widening public surface at all. It does not; see "What the current surface already permits". The authority statement is therefore correct and inapplicable.

**Fact 3 — "the narrower slice was available and has been taken": verified as to the slice, imprecise as to what remains.** `a_retained_gather_proof_agrees_with_an_independent_classification` is present in `crates/tiler-reference/tests/index_region_oracle.rs`, covers the four cases described including `[1 << 32, 0]` where both arguments hold at once, and writes its classifier out rather than calling the deriver. All confirmed by reading. The imprecision is the closing sentence, **"What remains unchecked is the identity itself."** Two things are wrong with it. First, the slice landed in a *test*, and the clause it partially discharges was written about the **evaluator**: the accepted packet [`decide-the-data-dependent-index-representation-public-surface`](decide-the-data-dependent-index-representation-public-surface.md) states it in a paragraph prescribing `tiler-reference::IndexRegionEvaluator`, whose anchor is `a requirement resolution validates the observed value but mints no receipt or proof`. Second, at this base `bounds_resolution`, `statically_proved`, and `invocation_validation_required` occur **nowhere in `crates/tiler-reference/src/`** — only three times, all in `tests/index_region_oracle.rs`. So `IndexRegionEvaluator::gather` never consults the retained resolution at any point, even though its own doc comment claims, at the anchor `oracle that trusted a proof would stop being an independent check of it`, that a static resolution is checked defensively. What remains unchecked is not "the identity itself" but **the entire resolution arm, in the evaluator, of which the identity binding is one part**.

**Related stale Fact, outside this ticket.** [`close-the-gather-review-findings-on-the-index-layer`](close-the-gather-review-findings-on-the-index-layer.md) F6 states that `bounds_resolution`, `statically_proved`, and `invocation_validation_required` "appear nowhere in `crates/tiler-reference/`". That was true when written and is false at this base, because the narrower slice added three occurrences under `tests/`. It remains true of `crates/tiler-reference/src/`. Both that ticket and [`admit-the-selected-data-dependent-index-representation`](admit-the-selected-data-dependent-index-representation.md) are `done`; the corrections are recorded there under dated headings by this lane.

### What the current surface already permits

This is the question the audit put first, and it dissolves the ticket. The identity is a pure, injective packing of exactly fifteen components: the domain tag, the proof kind, the fact-source tag, the region identity bytes, the access ordinal, the source and index tensor ordinals, both canonical type encodings, the source/index/result shapes, the axis, the source extent, and the framed run of domain dimension ordinals. Read `encode_gather_bounds_identity` for the list; nothing else enters the bytes.

**Every one of those components is already independently comparable from outside `tiler-ir`, by typed value, with no new public surface.** `GatherIndexBoundsProof` exposes `region()`, `access()`, `source()`, `index()`, `source_type()`, `index_type()`, `source_shape()`, `index_shape()`, `result_shape()`, `axis()`, `source_extent()`, `domain()`, `kind()`, and `facts()`, all `pub`. `CanonicalIndexRegionIdentity` derives `Eq`, and `VerifiedIndexRegion::canonical_identity()` is `pub`. The three handle types derive `Eq` even though their ordinals are private — which is the whole point: a downstream checker can compare two handles without ever learning what number either one holds. `Shape`, `Extent`, `Axis`, `ResolvedValueType`, and `IndexDomainFactSource` all derive `Eq`. The domain tag is not a comparable component but a constant discriminating a proof from a requirement, and that distinction is already carried by the `GatherIndexBoundsResolution` *type*, so no byte comparison is needed to establish it.

This was proved by compiling it from an out-of-workspace crate depending on `tiler-ir` by path, not by reading derives and inferring. The probe's `proof_binds_this_gather` compares all fourteen comparable components against values supplied by the caller from its own reading of the region, and it builds clean.

**The consequence is the finding.** An entry point that re-derives identity *bytes* and compares them is strictly weaker than what the surface already allows, and in the self-referential form it is not a check at all. Comparing `proof.identity()` against `encode(proof's own subject)` establishes only that the index layer packed its own fields consistently — an in-crate invariant, testable in-crate, with the oracle supplying no premise the index layer did not supply first. Comparing the components against the evaluator's own reading of the region is genuinely independent, names the field that disagreed, and covers the same fifteen positions. **The identity's evidentiary content is provenance, not content**: holding one is evidence because there is no constructor, and re-deriving the content to compare it would weaken that property in exact proportion to how much of the encoder it publishes, while adding nothing the component comparison misses.

### Enumeration, with eliminations before ranking

1. **Status quo, unchanged — leave the resolution unconsulted in the evaluator and record why.** *Eliminated.* Not on correctness of anything that ships, but because it is no longer the honest description: the audit found the evaluator ignores the retained resolution entirely while its doc comment claims it checks a static resolution defensively. That is a live documentation/behaviour disagreement, not a recorded deferral, and leaving it is exactly the "renamed failure" the readiness gate forbids.
2. **Verifier-side re-derivation entry point from raw parts** — publish enough for a caller to compute identity bytes. *Eliminated.* It lets a caller mint an identity for a proof that never ran, which falsifies the type's stated meaning and would make identity bytes forgeable for any future consumer that keys on them. It also requires widening `VerifiedTensorAccessId`, `VerifiedTensorId`, `VerifiedDimensionId`, and `IndexDomainFactSource::tag` — three opaque handle types and a governed tag — to expose the ordinals the encoder consumes. The gate eliminates any option letting a caller mint an identity it cannot independently justify.
3. **Verifier-side re-derivation entry point taking `&GatherIndexBoundsProof`** — `fn derive(proof) -> Identity`. *Eliminated as vacuous.* Both sides of the comparison originate in the same object, so it establishes an in-crate packing invariant and nothing about the program being evaluated. It cannot detect a proof transplanted from another region, which is the failure an oracle exists to catch.
4. **Identity-comparison predicate on the proof** — `fn identity_binds(&self, region, access, source, …) -> bool`, taking typed values so no ordinal is exposed and nothing can be minted. *Survives elimination.* Equivalent in power to option 5. Its one genuine advantage is drift: a widened `GatherIndexBoundsSubject` changes the signature and breaks the downstream caller at compile time, whereas a hand-written component list silently stops covering the new field.
5. **No new public surface — the evaluator performs the component-binding check at the current boundary.** *Survives elimination.* Cannot mint anything, because it constructs no identity and only compares typed values it already holds.
6. **Further bounded research.** *Eliminated.* The two compiler probes settled both directions of the question; there is no unknown left that reading or measurement would resolve.

### The frontier, and why one option dominates

Options 4 and 5 are the frontier. They compare as follows.

| | 4 — `identity_binds` predicate | 5 — component check, current surface |
|---|---|---|
| Correctness / independence | identical | identical |
| Can mint an identity | no | no |
| Public surface added | one method whose signature must track the subject | **none** |
| Diagnostic on failure | one `bool`, names no field | names the disagreeing field |
| Subject-widening drift | caught at the caller, by compile error | not caught at the caller |
| Maintenance | a `tiler-ir` method to keep in lockstep with the subject | nothing |

Option 4 wins exactly one row. That row is real and is the strongest argument against the recommendation, so it should be answered rather than waved off — and it is answerable **more cheaply, in-crate, with no public surface at all**. `encode_gather_bounds_identity` today reads its subject through field access (`subject.region`, `subject.access`, …), so adding a field to `GatherIndexBoundsSubject` compiles and silently produces a narrower identity. Destructuring the subject in the encoder instead makes a widened subject a **build error at the encoder**, which is AGENTS.md's "size enumerations from the type" applied at the place the type lives. That discharges the drift concern at its source, for every consumer at once rather than for one downstream caller, and it costs one line.

With drift handled in-crate, **option 5 dominates option 4 on every remaining dimension**, and the readiness gate says to take the dominant option rather than manufacture a choice.

### Recommendation, and whether it is Tom's

**It is not Tom's.** The recommendation adds no public item. Item by item, everything it touches is already `pub` and already stable: the fourteen `GatherIndexBoundsProof` accessors, `VerifiedIndexRegion::canonical_identity`, and the `Eq` derives on `CanonicalIndexRegionIdentity`, the three verified handle types, `Shape`, `Extent`, `Axis`, `ResolvedValueType`, and `IndexDomainFactSource`. The encoder-destructuring change is to a private `fn` in a private module. There is no included-or-excluded surface for Tom to accept, so no question is asked here.

What this ticket should close as: **the oracle checks a gather proof identity by comparing every component that identity binds against its own reading of the region, at the public surface as it stands; the identity bytes are deliberately never re-derived downstream, because provenance rather than content is what makes holding one evidence.** The reconsideration trigger, should the conclusion need revisiting: a consumer appears that must key on identity *bytes* across a crate boundary — a cache, a manifest row, or a serialized artifact — because such a consumer needs equality over bytes it did not receive from the proof, which the current surface genuinely cannot provide.

### Strongest counterargument to the recommendation

That a component list is a hand-maintained census over a population it cannot type, and AGENTS.md is explicit that such a census is only as complete as its own vocabulary. The evidence that would reverse the recommendation: a demonstration that the encoder-destructuring alarm does not in fact fire on a widened subject, or a consumer requirement for byte-level identity equality across the boundary. The negative control that tests it: add a field to `GatherIndexBoundsSubject` and confirm the destructured encoder fails to build, and separately confirm that the component check in the evaluator still passes — the second half is what shows the alarm is load-bearing rather than incidental, and it must be perturbed separately from the first.

### Follow-up tickets

- [`check-the-retained-gather-resolution-in-the-reference-evaluator`](check-the-retained-gather-resolution-in-the-reference-evaluator.md) — the evaluator work, in `implementation/reference`.
- [`destructure-the-gather-bounds-subject-in-its-identity-encoder`](destructure-the-gather-bounds-subject-in-its-identity-encoder.md) — the one-line drift alarm, in `implementation/ir`.

### Commands run

All at `f69829143a387a8e117858dbcaad416715f7e788`.

```sh
# The resolution accessors reach only the test, never the evaluator.
grep -rn "bounds_resolution\|statically_proved\|invocation_validation_required" crates/tiler-reference/
# 3 lines, all crates/tiler-reference/tests/index_region_oracle.rs

# No second impl block re-exposes a verified handle's ordinal.
grep -rn "impl VerifiedTensorId\|impl VerifiedTensorAccessId\|impl VerifiedDimensionId" --include="*.rs" crates/tiler-ir/src
# no output
```

Two compiler probes were run from a scratch crate outside the workspace, depending on `tiler-ir` by path and pinned to `nightly-2026-07-19`. The positive probe compares all fourteen comparable identity components against caller-supplied values and **builds clean**. The negative control attempts what a downstream re-implementation of the encoding needs, and fails with three `E0624`s:

```text
error[E0624]: method `tag` is private
   --> src/lib.rs:48:28
    |
 48 |     out.push(proof.facts().tag());
    |                            ^^^ private method
error[E0624]: method `as_usize` is private
   --> src/lib.rs:49:44
error[E0624]: method `as_usize` is private
   --> src/lib.rs:50:44
```

## Why this exists

Filed 2026-08-22 by the coordinator. Three successive gather lanes have now reached this and each stopped rather than work around it, which is the correct call every time.

**Fact — the boundary is deliberate and the missing constructor is the point.** `GatherIndexBoundsProofIdentity` is declared `pub(super) Vec<u8>` and its doc states there is no public constructor and no byte conversion, so `as_bytes` is the entire surface a downstream crate has. `tiler-reference` can therefore *read* a retained identity but cannot derive the bytes to compare it against without reimplementing the encoding — which would fork the identity domain the module exists to solely own, the exact defect the missing constructor prevents.

**Fact — a worker cannot mint the fix.** Closing this widens accepted public surface, and AGENTS.md reserves consequential public boundaries to Tom. Holding an identity one could not have constructed is precisely what makes it evidence the proof ran; handing out a constructor is not a mechanical convenience.

**Fact — the narrower slice was available and has been taken, so this ticket is not blocking correctness today.** The third gather pass landed a check of the retained `kind()` and `index_shape()` against an independent derivation from the operand shapes, written out rather than called, covering four cases including both arguments holding at once. It is the only check anywhere that catches a precedence inversion from outside the crate that decides it, and its perturbation proves it does: inverting the deriver's U32-before-empty precedence reddens it across the crate boundary. **What remains unchecked is the identity itself.**

> **Correction — 2026-08-22 by `worker-oracleid2`, at `f69829143a387a8e117858dbcaad416715f7e788`.** The three Facts above are retained verbatim so their wording stays searchable. Their verdicts are in the audit at the top of this ticket: the first is verified but understates the boundary, the second is correct on authority and rests on a premise the enumeration retires, and the third is verified as to the slice but its closing sentence — that what remains unchecked is the identity itself — is imprecise. The clause the slice partially discharges was written about `IndexRegionEvaluator`, and at this base that evaluator does not consult the retained resolution at all.

## The decision

Two shapes, and they are not equivalent:

1. **A verifier-side re-derivation entry point** — the oracle recomputes the identity from the verified program and compares. Strongest, and the most surface: it publishes enough to mint an identity, which is what the current boundary refuses.
2. **An identity-comparison entry point** — the oracle hands back what it holds and the owning module answers whether it matches. Publishes a predicate rather than a constructor, so a caller still cannot mint one.

Enumerate at your base rather than treating this pair as closed, and include the status quo: the narrower slice already lands, so **"leave the identity unchecked and record why" is a real candidate**, not a placeholder.

> **Correction — 2026-08-22 by `worker-oracleid2`.** The pair above is retained verbatim and is superseded by the six-option enumeration in the audit. Shape 1 splits into two materially different options — from raw parts, which is eliminated for letting a caller mint an identity, and from the proof itself, which is eliminated as vacuous. Shape 2 survives elimination as option 4 and is then dominated. The option the ticket did not contain is the one that wins: no new public surface at all.

## Required work

- Apply AGENTS.md's decision-packet readiness gate in full. Re-audit all three Facts at your base with a per-Fact verdict first.
- For each survivor, state exactly what public surface it adds, whether a caller could use it to mint an identity, and what that would let a wrong program claim.
- State the identity and schema consequence of each. **Expected: none** — these are read-side entry points over already-minted bytes — but derive it rather than copying that expectation.
- If one option dominates, recommend it rather than manufacturing a choice. If a real trade-off survives, ask Tom exactly one concrete question.
- For every survivor: strongest counterargument, the evidence that would reverse it, and the negative controls that would test it.

The identity and schema consequence, derived rather than copied: **none, for every survivor.** No option in the enumeration changes what is encoded, changes a tag, or changes when an identity is minted. Options 4 and 5 are read-side comparisons over already-minted values, and the recommended encoder-destructuring change alters no byte the encoder writes — it only makes a future field addition a build error instead of a silent narrowing. The expectation the ticket recorded is confirmed, and the derivation is that `encode_gather_bounds_identity` is untouched by all of them.

## Non-goals

Implementing whichever option is chosen; widening any other part of the gather surface; and the compiler vertical, which is [`carry-the-gather-relation-through-the-compiler-vertical`](carry-the-gather-relation-through-the-compiler-vertical.md).

## Closes when

Tom has accepted one route for the oracle to check a gather proof identity, or has accepted that it stays unchecked with the reason and a reconsideration trigger recorded — and in either case the surface each option would add is stated rather than implied.

> **Correction — 2026-08-22 by `worker-oracleid2`.** The closing condition above is retained verbatim and is met in its second clause's spirit rather than its letter: the audit finds no route requires Tom, because the dominant option adds no public surface and every item it uses is already `pub`. The surface each enumerated option would add is stated in the enumeration. What remains before this can close is the coordinator's review of the packet and the two follow-up tickets it files; the route itself needs no acceptance.
