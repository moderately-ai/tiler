---
schema: "tiler-doc/v1"
id: "tiler.research.numerics.conversion-family-decomposition-across-pairs"
kind: "research"
title: "Conversion family decomposition across pairs"
topics: ["numerics", "conversion", "dtypes", "operations", "bf16", "f16"]
catalog_group: "numerical-operations"
research_status: "complete"
disposition: "adopted"
adopted_by: ["ADR-0102"]
implementation_status: "not-started"
evidence_classes: ["exhaustive-finite", "primary-source-synthesis"]
informs: ["tiler.contract.numerical-semantics"]
ticket: "test-the-directional-conversion-pair-generalization"
---

# Conversion family decomposition across pairs

- **Status:** the answer to `RQ-OP-04`. It registers nothing, selects no key, moves no support-matrix rung, and reopens no accepted decision.
- **Ticket:** [`test-the-directional-conversion-pair-generalization`](../../../tickets/test-the-directional-conversion-pair-generalization.md), track **O-22** of the [operation-family delivery graph](../semantic-graph/operation-family-delivery-graph.md).
- **Research date:** 2026-08-05.

## Traceability

- **Current disposition:** adopted, by [ADR 0102](../../decisions/0102-key-conversion-families-by-the-ordered-pair-and-derive-their-fields.md), which landed `proposed` on 2026-08-06 and which Tom accepted the same day at the live session's decision round, relayed by the coordinator; the `adopted_by` edge is set in the acceptance sweep's own change, as this line previously promised it would be. The negative finding below is derived from stated layouts and accepted decisions; the positive rule it suggests is labelled **Proposal** throughout because that is the authority it carried when it was written, and the ADR — not this record — is what a reader now cites for the rule.
- **Normative destination:** [Numerical semantics](../../numerical-semantics.md#a-conversion-family-is-keyed-by-the-ordered-pair-and-a-mode-and-its-owed-fields-are-derived) now states the keying and derived-field rule, beside the `### Floating-point widening and narrowing, derived at the BF16/binary32 pair` section it reframes as the comparable-pair case. That section needed no correction from *this* record while the ADR was proposed, and the acceptance sweep made the edit.
- **Question answered:** `RQ-OP-04` of the [mature operation and signature taxonomy](../semantic-graph/mature-operation-and-signature-taxonomy.md), which blocks families F-18 and F-19.
- **Accepted authorities this record preserves rather than amends:** [ADR 0010](../../decisions/0010-typed-conversion-contracts.md) (a conversion family defines only the fields relevant to its semantics, and a universal optional-field bag "makes invalid combinations representable"), [ADR 0041](../../decisions/0041-separate-float-to-integer-conversion-families.md) (four float-to-integer families, "a discriminated semantic contract, not a universal bag of independently optional cast fields"), [ADR 0018](../../decisions/0018-portable-bitwise-nans.md) ("Numeric conversions follow their resolved conversion contract"), and [ADR 0091](../../decisions/0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md) (BF16/binary32 conversion is two directional families with disjoint field sets).
- **Retained enumerator:** [`spikes/numerics/conversion_field_obligations.py`](../../../spikes/numerics/conversion_field_obligations.py), standard library only, run by hand from the repository root. Every table below is its output and every row is also one line of arithmetic over the stated layouts, so the script is a convenience rather than the authority.

## The question, and what this record is not deciding

`RQ-OP-04` asks whether ADR 0091's directional-pair decision "generalize[s] to every conversion pair, or is one keyed family parameterized by source, destination, and mode correct at scale?", and fixes the test: "Closes when a second pair is examined field by field. The test is falsifiable: if a second directional pair's field set is *not* disjoint in the way BF16/binary32's is, the generalization is refuted and the parameterized form wins. The `n²` growth is a cost to be stated, not the deciding argument."

Four things are deliberately out of this record's reach. It registers no conversion key and proposes no Rust spelling — both are Tom's under [ADR 0075](../../decisions/0075-scope-public-boundary-approval-by-change-category.md), and ADR 0091 already reserved exactly these. It does not reopen ADR 0091 or ADR 0041: ADR 0091's decision is about the BF16/binary32 pair and this record confirms that scope is narrow rather than contesting the decision inside it. It does not touch in-type precision reduction, which changes no result type and is owned by [`scope-the-in-type-precision-reduction-family`](../../../tickets/scope-the-in-type-precision-reduction-family.md). And it does not touch bit reinterpretation, which changes no numeric value.

## What ADR 0091 actually decided, restated so the generalization is checkable

**Fact.** ADR 0091 item 2 decides that "BF16/F32 conversion is two separate typed families, one per direction. The widening family carries no rounding, overflow, or NaN-mapping field, because BF16-to-binary32 widening is exact and total. The narrowing family carries round-to-nearest-ties-to-even, overflow to a signed infinity at the inclusive midpoint above the largest finite BF16 value, canonicalization of every NaN to `0x7fc0`, gradual underflow, and preserved signed zero. A contract carrying a field its direction does not have is refused at construction."

**Fact.** The generalization under test is not that decision but the taxonomy's inference from it, which states that narrowing "owes a rounding rule, an overflow rule, and a subnormal rule that widening does not, and widening owes an exactness claim that narrowing cannot make. That asymmetry is a property of the direction rather than of BF16, so the same shape should hold for every pair". That sentence — *direction* is the discriminant — is what a second pair can falsify.

**Fact — one accepted authority already says the derivation does not travel.** [Numerical semantics](../../numerical-semantics.md) states that BF16-to-binary32 exactness "is a property of BF16's parameters and its derivation does not transfer", noting that binary16's widening to binary32 is exact "by a *different* argument". The [dtype-family research tracks](dtype-family-research-tracks.md) restate it from the dtype side for track D-3: conversion is "Owed per ordered pair; [ADR 0091]'s disjoint-field derivation is specific to BF16/binary32 and does not transfer". This record supplies the field-by-field examination those two sentences assert without walking.

## Method: four predicates over an ordered pair, each derived rather than invented

**Proposal.** A conversion contract must define an observable result for every input, so a field is *owed* exactly when some source input's result is not already determined by the two formats. Over binary floating formats that gives four predicates on the ordered pair `(src, dst)`, and each of the four is a field ADR 0010, ADR 0041, or ADR 0091 already names.

| Predicate | Owed when | Named by |
| --- | --- | --- |
| **rounding** | some in-range source value is not exactly representable in the destination — the destination's significand is narrower (`t_src > t_dst`), or its finest quantum is coarser (`emin_src − t_src < emin_dst − t_dst`) | ADR 0091's narrowing; ADR 0041's "Rounded and saturating families carry an explicit rounding rule; exact conversion does not" |
| **overflow** | some finite source magnitude exceeds the destination's largest finite magnitude | ADR 0091's "overflow to a signed infinity at the inclusive midpoint"; ADR 0041's ordered saturation |
| **underflow** | the destination's finest quantum is coarser than the source's, so a source value either falls below the destination's smallest nonzero or lands inexactly in its subnormal range | ADR 0091's "gradual underflow"; ADR 0010's "subnormal handling" |
| **NaN mapping** | the source carries more NaN payload bits than the destination, so no payload-preserving map is total | ADR 0091's canonicalization to `0x7fc0`; ADR 0018's "Numeric conversions follow their resolved conversion contract" |

**Inference — the rounding predicate needs its second clause, and dropping it is the mistake this walk nearly made.** Comparing significand widths alone is not enough, because a source *normal* whose magnitude falls inside the destination's *subnormal* range is held there at the destination's fixed smallest quantum rather than at its full precision. Any pair where the source's exponent range extends below the destination's therefore rounds even when the source's significand is the narrower of the two — which is exactly the case the decisive pair below turns on.

**Fact — the population.** The five layouts are the binary float rows the [mature dtype taxonomy](mature-dtype-taxonomy.md) states, restricted to formats that use the IEEE all-ones-exponent-reserved convention: `f16` 1/5/10, `bf16` 1/8/7, `f32` 1/8/23, `f64` 1/11/52, `f128` 1/15/112. Twenty ordered pairs. The catalog's `FN` and `FNUZ` rows are held out and the [boundary section](#boundary-acquisition-requests-and-unsupported-cases) says why.

## The nearest second pair: binary16 and binary32, walked field by field

This is the closest possible second case — another IEEE binary pair against binary32, and the pair ADR 0091's own open question names as its trigger. If the generalization fails here it fails everywhere; if it holds, a harder pair is needed.

**Fact — `f16 → f32` owes nothing, so it matches ADR 0091's widening shape.** `f16`'s precision 11 is inside binary32's 24, and its finest quantum `2^-24` is far above binary32's `2^-149`, so every `f16` value is exactly representable. Its largest finite magnitude 65 504 is far inside binary32's, so no overflow. Its nine payload bits fit binary32's twenty-two, so a payload-preserving map is total. Field set: `{}`.

**Fact — `f32 → f16` owes all four, so it matches ADR 0091's narrowing shape.** Field set: `{rounding, overflow, underflow, NaN mapping}`.

**Inference — the shape holds at this pair, but the *derivation* behind the widening does not, and the difference is load-bearing.** BF16-to-binary32 widening is a sixteen-bit shift: the two formats share an exponent field, so a subnormal stays subnormal and a NaN payload zero-extends with its quiet bit in place, and the payload map is a consequence of the encoding relation rather than a choice. Binary16 has a strictly narrower exponent range, so every `f16` subnormal becomes a binary32 **normal**; the conversion is not a shift and must be defined value-wise, and a value-wise definition says nothing about where the payload goes. `f16 → f32` therefore *chooses* a NaN payload map where `bf16 → f32` inherits one. Both choices are total, so no field is owed either way — but "the widening family" is not one family appearing at two pairs, it is two families that agree on their field set and differ on a behaviour a reader would have to look up.

**Verdict at this pair: the generalization survives.** So the taxonomy's `RQ-OP-04` is not closed by the nearest second pair, and a third is needed. The rest of this record supplies it, which is the branch the ticket's closes-when anticipated when it asked for "the third pair that would separate them".

## The decisive pair: BF16 and binary16

`bf16` and `f16` are the first pair in the catalog whose value sets are **incomparable** — `bf16` has the wider exponent range and `f16` the wider significand, so neither format contains the other. Every pair examined before this one was nested.

**Fact — `bf16 → f16` owes `{rounding, overflow, underflow}` and not a NaN mapping.**

- *rounding, owed.* `bf16`'s significand is the narrower of the two (7 trailing bits against 10), so the first clause does not fire — but its finest quantum is `2^-133` against `f16`'s `2^-24`, so every `bf16` value whose magnitude lies in `[2^-24, 2^-14)` lands in `f16`'s subnormal range and is held at a coarser step than `bf16` states it at. A rounding rule is owed, by the second clause only.
- *overflow, owed.* `bf16`'s largest finite magnitude is about `3.39 × 10^38` against `f16`'s 65 504.
- *underflow, owed.* Every `bf16` magnitude below `2^-24` is below `f16`'s smallest nonzero.
- *NaN mapping, not owed.* `bf16` carries six payload bits and `f16` nine, so a payload-preserving map is injective and total.

**Fact — `f16 → bf16` owes `{rounding, NaN mapping}` and neither overflow nor underflow.**

- *rounding, owed.* `f16`'s precision 11 exceeds `bf16`'s 8.
- *overflow, not owed.* 65 504 is far inside `bf16`'s finite range.
- *underflow, not owed.* `f16`'s smallest nonzero `2^-24` is far above `bf16`'s smallest normal `2^-126`, so every finite nonzero `f16` value is a `bf16` normal and nothing lands in `bf16`'s subnormal range.
- *NaN mapping, owed, and forced for the same reason ADR 0091 gives at its own pair.* `f16` carries nine payload bits and `bf16` six, so a payload-preserving narrowing is not total: the signalling `f16` NaN `0x7c01` carries its payload only in bits a seven-bit significand does not reach, and a map that preserves a payload prefix sends it to `0x7f80` — the `bf16` **infinity** encoding. That is the identical failure numerical semantics records for `0x7f800001` narrowing to `bf16`, arriving at a pair where the *other* direction is the one ADR 0091 would call narrowing.

**Fact — the two field sets intersect.** `{rounding, overflow, underflow} ∩ {rounding, NaN mapping} = {rounding}`. Non-empty.

**This is `RQ-OP-04`'s own falsification condition, met on a float-to-float pair.** The question's test says that if a second directional pair's field set "is *not* disjoint in the way BF16/binary32's is, the generalization is refuted".

## The exhaustive enumeration

**Fact — all twenty ordered pairs over the five stated layouts, from [`conversion_field_obligations.py`](../../../spikes/numerics/conversion_field_obligations.py).**

| Ordered pair | Owed fields | Ordered pair | Owed fields |
| --- | --- | --- | --- |
| `f16 → f32` | `{}` | `f32 → f16` | `{rounding, overflow, underflow, NaN}` |
| `f16 → f64` | `{}` | `f32 → bf16` | `{rounding, overflow, underflow, NaN}` |
| `f16 → f128` | `{}` | `f64 → f16` | `{rounding, overflow, underflow, NaN}` |
| `bf16 → f32` | `{}` | `f64 → bf16` | `{rounding, overflow, underflow, NaN}` |
| `bf16 → f64` | `{}` | `f64 → f32` | `{rounding, overflow, underflow, NaN}` |
| `bf16 → f128` | `{}` | `f128 → f16` | `{rounding, overflow, underflow, NaN}` |
| `f32 → f64` | `{}` | `f128 → bf16` | `{rounding, overflow, underflow, NaN}` |
| `f32 → f128` | `{}` | `f128 → f32` | `{rounding, overflow, underflow, NaN}` |
| `f64 → f128` | `{}` | `f128 → f64` | `{rounding, overflow, underflow, NaN}` |
| **`bf16 → f16`** | **`{rounding, overflow, underflow}`** | **`f16 → bf16`** | **`{rounding, NaN}`** |

**Fact — four distinct owed-field-set classes over twenty ordered pairs**: nine pairs owe nothing, nine owe all four, and the remaining two — exactly the two directions of `bf16`/`f16` — owe proper subsets that ADR 0091's shape has no name for. **Fact — one unordered pair of ten has non-disjoint direction field sets**, and it is that same pair.

**Inference — the two exceptions are exactly the incomparable pair, and that is why the shape looked general.** Among `{f16, f32, f64, f128}` the value sets are nested by construction, and `bf16` is contained in `f32`, `f64`, and `f128`. Nine of ten unordered pairs are therefore comparable, and on a comparable pair one direction is a containment (owing nothing) and the other is its inverse (owing everything) — which is precisely ADR 0091's two field sets. The shape is not a property of *direction*; it is a property of *comparability*, and it degenerates into "widening and narrowing" only when the pair happens to be nested. `bf16` and `f16` are the first two formats in the recognized catalog that are not, and the corpus has never had occasion to convert between them.

**Measurement — the enumerator was watched failing.** With `bf16`'s exponent width falsified from 8 to 5 in a copy of the script — making the pair nested — `bf16 → f16` becomes `{}`, `f16 → bf16` becomes all four, and the non-disjoint count falls from 1 of 10 to 0 of 10. The decisive result therefore rests on `bf16`'s actual stated layout rather than on the script's structure. Reproduce by editing the one `"bf16": (8, 7),` line in a copy and rerunning.

## The float-to-integer directions, and a correction to how ADR 0041 is being read

The ticket that commissioned this record describes ADR 0041's four families as "already a directional decomposition of one logical conversion and therefore evidence on one side of the question". That reading does not survive reading the ADR.

**Fact — ADR 0041's four families are all in one direction.** Strict rounded, exact, ordered saturating, and total saturating NaN-to-zero are four float-**to**-integer families. Integer-to-float appears nowhere in ADR 0041; [Numerical semantics](../../numerical-semantics.md) lists "floating-point to integer and integer to floating-point" among the initial families, and only the first half has an accepted decision. ADR 0041 is therefore a **mode** decomposition *within* one ordered pair, not a directional decomposition of a pair.

**Fact — three of the four share a field set and differ only in a field's value.** The taxonomy's F-19 row records that "the four differ in exactly those three fields" — rounding, out-of-range behaviour, and the NaN result. Strict rounded, ordered saturating, and total saturating NaN-to-zero all carry all three; they differ in what the out-of-range and NaN fields *say*. Only *exact* differs in field presence, because "an exact conversion carries no rounding rule". ADR 0041's own consequences give the ground for keeping the last two apart: "The word 'saturating' never silently implies an arbitrary NaN mapping", and NaN-to-zero "remains directly representable but is visibly different from ordered saturation".

**Inference — field-set disjointness is not the corpus's criterion for family separation, and an accepted ADR refutes it without any second pair being examined.** ADR 0041's ordered-saturating and NaN-to-zero families have *identical* field sets and were accepted as two families anyway. Whatever separates conversion families, it is not that their field sets are disjoint.

**Fact — the integer directions independently break disjointness, and the verdict depends on which of ADR 0041's four modes is chosen.** Integer-to-float owes a rounding rule whenever the integer's magnitude range exceeds the destination's precision — `i32 → f32` is inexact at `2^24 + 1`, while `i8 → f32` is exact and total and owes nothing at all. Float-to-integer under the strict rounded, ordered saturating, or NaN-to-zero mode also carries a rounding rule, so `f32 → i32` and `i32 → f32` intersect at `{rounding}`; under the *exact* mode they do not. A test asking whether "the pair's two directions" are disjoint has no single answer once modes exist, which is a second and independent reason it cannot decide `RQ-OP-04`.

**Inference — so ADR 0041 is evidence on the axis the question did not ask about.** Read together, the two accepted decisions say a conversion family is discriminated by *both* the ordered pair (which fixes which fields are owed) and a mode (which fixes what an owed field says when more than one total answer exists). Neither of `RQ-OP-04`'s two candidates carries both axes: the directional candidate has only the pair axis under a name that misdescribes it, and the parameterized candidate has both but as free attributes.

## Why the stated closure test cannot decide, and what replaces it

**Inference — the test fires, and the conclusion it prescribes is a candidate three accepted ADRs have already eliminated.** `RQ-OP-04` says that a non-disjoint second pair means "the parameterized form wins". The parameterized form is "one keyed family parameterized by source, destination, and mode". ADR 0091's alternatives-considered rejects it by name: "One float-to-float conversion family with a direction field. Rejected under ADR 0010 and 0041: it makes 'widening with ties-away' constructible, and the exactness result is precisely that no such thing exists." ADR 0010's alternatives-considered rejects it: "A universal structure containing every possible conversion field makes invalid combinations representable and weakens diagnostics." ADR 0041 rejects it: "This is a discriminated semantic contract, not a universal bag of independently optional cast fields." A test whose pass condition hands victory to an eliminated candidate decides nothing, and answering `RQ-OP-04` requires replacing it rather than reporting its verdict.

**Proposal — the replacement test, derived from the two grounds the corpus actually uses.** Two families merge into one keyed family with a discriminating field only when *both* hold:

1. **Constructibility.** Every combination of the merged field values denotes a coherent conversion. This is ADR 0010's rule and the reason widening and narrowing may not merge at a nested pair — the merged form makes "an exact conversion with a rounding rule" constructible, and no such conversion exists.
2. **Legibility.** No field value silently changes what the shared name means. This is the ground ADR 0041 gives for keeping NaN-to-zero out of ordered saturation, and it bites where constructibility does not.

Disjointness is a symptom of the first at nested pairs and is not a criterion of its own. Stated this way the test is decidable at every pair, including incomparable ones and including pairs carrying modes.

## The answer to `RQ-OP-04`

**Inference — stated in three parts, because collapsing them is what produced the question.**

1. **The per-ordered-pair decomposition generalizes, and is confirmed at every pair walked.** No pair's contract is recoverable from an unordered pair plus a direction flag, because the flag does not determine the field set: at `bf16`/`f16` one flag value gives `{rounding, overflow, underflow}` and the other `{rounding, NaN mapping}`, and neither is derivable from "this is the narrowing one". The taxonomy's F-18 classification — one family per *ordered directional pair* rather than one parameterized `Cast` — stands.
2. **ADR 0091's field assignment does not generalize, and "widening" and "narrowing" are not the discriminant.** The empty-and-full split holds on the eighteen ordered pairs whose formats are comparable and fails on the two that are not. It is a property of value-set containment, not of direction.
3. **Neither of the question's two candidates is the survivor.** The directional candidate is right about the key and wrong about what determines the fields. The parameterized candidate, with mode as a free attribute, is eliminated by three accepted ADRs. What survives is a family keyed by the ordered pair and a mode, whose owed field set is **derived** from containment predicates over the pair rather than chosen — so an invalid combination is not representable, which is the property all three ADRs were protecting.

Part 3 was labelled a **Proposal** and is now a decision. It is derived from stated layouts and accepted decisions and remains unmeasured and unregistered, but it is no longer unaccepted: [the drafted ADR body](#drafted-adr-body--landed-as-adr-0102-and-accepted-on-2026-08-06) below was landed `proposed` by a carrier and accepted by Tom on 2026-08-06, so [ADR 0102](../../decisions/0102-key-conversion-families-by-the-ordered-pair-and-derive-their-fields.md) is the authority for the rule and this part is the derivation behind it.

**What this does not close.** ADR 0091's own deferral — "whether the conversion family generalizes beyond BF16/F32" — closes on "the second float pair a workload selects", with the F16 vertical as its trigger. That is a *registration*-time question about a specific pair's contract and this record does not answer it; the trigger has not fired, and D-3's own trigger in the [dtype-family research tracks](dtype-family-research-tracks.md) has not fired either.

## The `n²` growth, stated as a cost rather than as the argument

`RQ-OP-04` requires the cost be stated and not treated as decisive, and the taxonomy's F-18 row calls it "the known cost".

**Fact — the growth is real and is in identities.** Over the [mature dtype taxonomy](mature-dtype-taxonomy.md)'s sixteen recognized float rows alone the ordered float-to-float pairs number `16 × 15 = 240`; adding the recognized integer widths and both cross directions, and then ADR 0041's four modes on the float-to-integer half, puts the mature space in the high hundreds before any key is spelled.

**Inference — but the schema and implementation counts do not grow with it, and that is what makes the cost payable.** Over the five stated layouts, twenty ordered pairs collapse to **four** distinct owed-field-set classes, and the class of a pair is computed from four predicates rather than looked up. A registered key is a durable identity; a contract *schema* is one of a small number of shapes; and the repository's architectural contract in `AGENTS.md` already licenses the separation, since "code organization may share implementations without collapsing semantic distinctions". The expensive reading of `n²` — `n²` schemas and `n²` evaluators — is not the one that obtains.

**Inference — the residual cost is migration, not enumeration, which is why the shape is worth fixing before the first key.** Identities are cheap to mint and expensive to move: the [delivery graph](../semantic-graph/operation-family-delivery-graph.md) records that "whichever shape the first registered conversion takes becomes the precedent every later pair is read against", and ADR 0087 already paid that cost once when it chose one keyed family for the contraction.

**Inference — "route through a common wider format" does not retire the growth, and treating it as though it does is a numerical error.** The composition `bf16 → f32 → f16` happens to agree bit for bit with a direct `bf16 → f16` because the intermediate contains the source exactly, so exactly one rounding occurs. That is a derived property of the specific triple, not a general licence. **Fact — a worked counterexample.** For the binary64 value `33603583 / 2^25`, exact in binary64, a direct round-to-nearest-ties-to-even conversion to `f16` gives `1025/1024`, while `f64 → f32 → f16` gives `513/512` — one `f16` ulp apart, because the intermediate lands exactly on the `f16` midpoint and ties-to-even then goes the other way. Any scheme that replaces a direct pair with a route owes a proof that each hop's source is exactly contained, which is the same containment predicate the field derivation uses.

## Drafted ADR body — landed as [ADR 0102](../../decisions/0102-key-conversion-families-by-the-ordered-pair-and-derive-their-fields.md) and accepted on 2026-08-06

**The record of the decision is [ADR 0102](../../decisions/0102-key-conversion-families-by-the-ordered-pair-and-derive-their-fields.md); the span below is retained as the drafted text it landed from, and is not a second authority over the same subject.** It landed `proposed` and Tom accepted it the same day, so the ADR is now the authority under [the decisions index](../../decisions/README.md)'s own preamble, and this record's **Proposal** labels are read as the authority the derivation carried when it was written rather than as a claim that the rule is still undecided. None of them was rewritten after the fact. The frontmatter reciprocal now exists as well: `adopted_by: ["ADR-0102"]` is set above, which [the metadata contract](../../document-metadata.md) admits as one of the two optional typed fields a `research` record may carry, and which was deliberately withheld while the record it would point at was proposed.

**This section was transferred byte-identically into `docs/decisions/` by its carrier ticket, [`land-the-conversion-pair-decomposition-adr`](../../../tickets/land-the-conversion-pair-decomposition-adr.md)**, because this record's scopes do not reach `docs/decisions/`. The transfer covered the context, the five numbered decisions, the five consequences, and the three alternatives-considered entries, with the section headings promoted one level — from `###` nested under this heading to `##` under the ADR's own title — and nothing else changed. The carrier supplied the frontmatter, the title heading, a status block, a traceability section, and an open-questions section of its own. The acceptance was recorded in three of those — the frontmatter's `decision_status`, the status block, and traceability's normative-owner paragraph — and the transferred span below was not touched by it.

**The two ADR links in the Context paragraph below are written `docs/decisions/`-relative and therefore do not resolve from this record.** `0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md` and `0041-separate-float-to-integer-conversion-families.md` are siblings at the destination and are spelled as the destination will need them. This is stated here rather than repointed, because repointing would break the byte-identity the transfer depends on and a transfer that edits is a fork — the same trade [ADR 0092's source record](../runtime/backend-scoped-route-requirement-answers.md) had to make explicit, and the one the BF16 draft avoided only by carrying no such link. A reader wanting those two ADRs from *this* page should follow them from the [Traceability](#traceability) section above, where they are spelled correctly for this location.

### Context

[ADR 0091](0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md) decided that BF16/binary32 conversion is two directional families with disjoint field sets, the widening carrying no field and the narrowing carrying rounding, overflow, NaN canonicalization, and gradual underflow. [ADR 0041](0041-separate-float-to-integer-conversion-families.md) independently decided four float-to-integer families that differ in three fields. Nothing decides what a *third* pair owes, and no conversion key is registered in any direction, so the first registration would fix by accident a precedent every later pair is read against.

### Decision

1. **A conversion family is keyed by the ordered `(source, destination)` pair together with a mode.** The ordered pair fixes which contract fields are owed; the mode fixes what an owed field says where more than one total answer exists, as ADR 0041's four float-to-integer modes already do within one pair. An unordered pair plus a direction discriminator is not sufficient, because the discriminator does not determine the field set.
2. **The owed field set is derived from the pair, never declared on it.** A rounding field is owed when some in-range source value is not exactly representable in the destination — either because the destination's significand is narrower or because its finest quantum is coarser. An overflow field is owed when some finite source magnitude exceeds the destination's largest finite magnitude. An underflow field is owed when the destination's finest quantum is coarser than the source's. An exceptional-mapping field is owed when the destination cannot represent every source exceptional value injectively. A contract carrying a field its pair does not owe is refused at construction, and a contract missing one its pair does owe is refused at construction.
3. **"Widening" and "narrowing" are not the discriminant, and neither term is a family name.** ADR 0091's empty-and-full split is the behaviour of a pair whose value sets are *comparable*. Where they are incomparable — BF16 and binary16 are the first such pair in the recognized catalog, BF16 having the wider exponent range and binary16 the wider significand — both directions owe proper non-empty subsets and their field sets intersect. A family named for a direction would misname those two.
4. **One keyed family parameterized by source, destination, and mode as free attributes is refused,** on the ground ADR 0010 and ADR 0041 already state: it makes an invalid combination representable, such as an exact conversion carrying a rounding rule. Deriving the field set under clause 2 achieves the parameterized form's economy without its defect.
5. **Two candidate families merge only when both constructibility and legibility hold** — when every combination of the merged field values denotes a coherent conversion, and when no field value silently changes what the shared name means. Field-set disjointness is a symptom of the first at comparable pairs and is not itself a criterion.

### Consequences

- The number of registered conversion identities grows with the ordered pairs, while the number of contract schemas does not: over the five IEEE-convention binary float layouts the corpus recognizes, twenty ordered pairs carry four distinct owed-field sets.
- ADR 0091 remains correct and remains scoped to its pair. Its two field sets are the comparable-pair case of clause 2, and nothing that rests on it moves.
- ADR 0041's four families are a mode decomposition within one ordered pair, and clause 1 makes that reading explicit rather than leaving it to be inferred from the ADR's title.
- Replacing a direct conversion with a route through a wider intermediate is legal only where each hop's source is exactly contained in its destination; otherwise the route double-rounds and is a different operation.
- Every conversion key, name, version, and Rust spelling remains reserved to Tom under ADR 0075. This record registers nothing.

### Alternatives considered

**One keyed family with a direction field.** Rejected as ADR 0091 already rejected it, and additionally because at an incomparable pair the direction field does not determine the field set, so the family would be underspecified rather than merely permissive.

**Declaring the field set on each registered pair instead of deriving it.** Rejected because it makes two registrations of the same pair disagree about what the pair owes, and the disagreement would be invisible: both would validate.

**Waiting for a workload to select a second float pair before deciding the shape.** Rejected because the first registered conversion fixes the precedent whether or not the shape was decided, and migrating an identity is more expensive than deciding it. The *contract* of any specific second pair does still wait for a workload; this record decides the shape and no pair's contents.

## Boundary, acquisition requests, and unsupported cases

**Boundary.** This is a derivation over stated layouts and accepted decisions. It takes no measurement, runs no program on any target, and claims nothing about any rounding realization, any instruction, or any target's honourability. The enumerated population is exactly the five IEEE-convention binary float layouts named above.

**Unsupported here, and named so a reader can tell a gap from an omission.** The catalog's `FN` and `FNUZ` float rows are held out of the enumeration deliberately, and they matter more than their absence suggests: the taxonomy records `f6E2M3FN`, `f6E3M2FN`, and `f4E2M1FN` as "finite-only; no Inf/NaN", and `f8E4M3FNUZ`, `f8E5M2FNUZ`, and `f8E4M3B11FNUZ` as carrying "unsigned zero". **Inference.** A conversion *into* a format with no infinity owes an infinity-mapping field, which is a decision the corpus already separates from overflow in the float-to-integer direction — [Numerical semantics](../../numerical-semantics.md) states that "Saturation determines endpoint behavior for ordered values and infinities but does not by itself determine a NaN mapping", and ADR 0041's own context gives the ground, that endpoint saturation "cannot determine a NaN result because NaN is unordered" — and which no float-to-float pair among the IEEE-convention formats owes at all. A conversion into an unsigned-zero format owes a signed-zero-mapping field, which ADR 0091's narrowing lists as "preserved signed zero" precisely because at its pair the answer is forced. Both are field classes ADR 0091's two-family shape has no slot for, and both strengthen the finding rather than qualifying it. They are excluded from the *counted* population because deriving their exact value sets requires documents this repository does not hold.

**Acquisition request — IEEE Std 754-2019.** The [preservation record](sources/README.md) classifies `ieee-754-2019` metadata-only with no byte stream ever retrieved, behind a purchase or subscription wall at `https://standards.ieee.org/ieee/754/6210/`. Attempted here: none — the record states the wall and the official route, and this record did not repeat a retrieval the manifest already documents as impossible. **What it would decide:** whether the standard's `convertFormat` operation *recommends* NaN payload preservation across a widening. If it does, the `f16 → f32` payload map is a normative default rather than a free choice and the record's "two families that agree on their field set and differ on a behaviour" observation weakens to a matter of documentation; if it does not, the observation stands as written. Nothing else in this record depends on it: the five layouts are stated by the [mature dtype taxonomy](mature-dtype-taxonomy.md) and every predicate is derived from them.

**Acquisition request — OCP OFP8 v1.0 and OCP MX v1.0.** Both are classified metadata-only in the [preservation record](sources/README.md), acquired by hand on 2026-07-31 with their exact digests recorded, and not vendored because neither carries a self-contained redistribution grant. Attempted here: none, for the same reason. **What they would decide:** the exact value sets, exceptional-value contracts, and zero encodings of the `FN` and `FNUZ` rows, which is what an enumeration extended to the full recognized float catalog needs. Without them the infinity-mapping and signed-zero-mapping field classes above are supported by the taxonomy's own layout column — enough to establish that the classes exist, not enough to count pairs into them.

## Where this record differs from the corpus it composes with

- The [minimum correct physical realization profile](../program-planning/minimum-correct-physical-realization-profile.md) said that "`RQ-OP-04` leaves conversion's family decomposition open, and neither changes the route, which is a scalar kernel operation under either answer". The second half was always true and stayed true — the physical route is unchanged and the family's classification in that profile does not move — and only the word "open" went stale. The producing ticket did not hold that document's scope and recorded the staleness here instead; ADR 0102's acceptance sweep held it and made the one-clause correction.
- [Numerical semantics](../../numerical-semantics.md) needed nothing from *this* record. Its BF16/binary32 section states in its own words that the derivation "does not transfer", so it was already scoped correctly before this record existed; what the acceptance of ADR 0102 added beside it is the general keying and derived-field rule, which became normative at acceptance and not before.

## Deferred questions, with their triggers

- **Whether a direct `bf16 → f16` family is ever wanted, or whether the routed spelling suffices.** This record shows the route is bit-identical for that specific triple and that the general workaround is unsound. **Closes with:** a named producer and consumer for the direct pair. **Trigger:** a workload holding `bf16` storage and requiring an `f16` operand, or the reverse.
- **What an incomparable pair's *contract* says, as opposed to which fields it owes.** This record derives the field set and deliberately fixes no value: the rounding rule, the overflow destination, the underflow behaviour, and the NaN canonicalization pattern of `bf16 → f16` are all undecided. **Closes with:** the ordinary admission path for that pair. **Trigger:** the same as above.
- **The counted class partition over the full recognized float catalog.** **Closes with:** the two acquisition requests above being satisfied, after which the enumerator extends to the `FN` and `FNUZ` rows and the four-class count is recomputed rather than assumed to hold.
