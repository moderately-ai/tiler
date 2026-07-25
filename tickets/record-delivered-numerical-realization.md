---
id: record-delivered-numerical-realization
title: Record the delivered numerical realization in the artifact
status: blocked
priority: p1
dependencies: [select-numerical-contract-and-compose-feasibility, declare-metal-numerical-honourability]
related: [draft-target-honourable-numerical-contract-adr, prototype-artifact-program-model]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, numerics, needs-tom]
claimed_from: todo
assignee: agent-runtime
lease_expires_at: 1784997340
---
ADR 0076 item 4. A produced artifact carries a first-class, **readable** record of the numerical realization actually delivered: the resolved contract complete over every dimension, each dimension's means of honouring it, the target facts relied on, and the identity of the profile that declared them.

A consumer comparing generated output against a CPU reference reads this record. It does not reconstruct it from the request, from the selected compiler flags, or from the target's name.

## Why flags cannot substitute — this is measured, not assumed

Under `-fmetal-math-mode=relaxed` the emitted module records `!"air.compile.fast_math_disable"` while every floating-point operation in it carries `reassoc nsz arcp contract afn`. The module-level flag is therefore not a faithful summary of the licences actually applied, and an artifact-side reader that inferred the delivered realization from it would read **the opposite of the truth**. That single measurement is the whole argument for a first-class record.

## Why identity is not enough either

`docs/artifact-abi.md` already puts the numerical contract and the exact flags into artifact *identity*, which is what makes two artifacts distinguishable. That is a different job. A digest is comparable and not readable: it lets a consumer detect that two artifacts differ, and tells it nothing about what either one means. This ticket adds the readable statement alongside the existing digest; it does not replace or duplicate the identity encoding, and it must not become a second authority over what identity commits to.

## What the record's content is fixed by

Because ADR 0076 item 5 forbids delivering anything other than the declared contract, the delivered realization always equals the declared one for any artifact that exists. So the record is **not** a channel for reporting a downgrade — there are none by construction. It is the evidence that no downgrade occurred, plus the means by which each dimension was honoured. The means is the part a caller cannot derive, and it changes what a reference comparison should expect from a dimension honoured by emulation rather than natively.

Do not design an "actual versus requested" shape. There is no divergence to report, and a schema that admits one would invite a future implementation to fill it in.

## Boundary — this needs Tom

`tiler-artifact` gained its first public content in `prototype-artifact-program-model`. This ticket adds a public numerical surface to it, which under ADR 0075 is Tom's to approve before it is accepted or merged. Build a tested implementation as a concrete draft, present the boundary as an atomic decision with alternatives, and pause — a tested implementation is not implicit approval of its public interface.

Apply the ADR 0074 conventions: typed non-erasing errors, opaque identities with `as_bytes()` and presentation-only `label()` accessors, domain-tagged and length-prefixed encodings with no ordinal dependence, a transactional builder with a consuming `build()`, no `pub` fields on the verified product. On `#[non_exhaustive]`, apply the amended convention 5 rather than the blanket rule: an enum an out-of-crate consumer maps *totally* must stay exhaustive, because a wildcard there makes a missed variant silently wrong instead of a build error. `tiler-artifact` already encodes `KernelType`, `AddressSpace`, and `BufferAccess` into identity cross-crate, which is the worked precedent for that judgement.

## Blocked 2026-07-25 — the declared dependency was satisfied; the real one is not

Attempted from `implementation/artifact`. The frontmatter listed only `select-numerical-contract-and-compose-feasibility`, which is `done`, so this ticket presented as ready. It is not, and ADR 0076 says so in its own words.

**Fact — ADR 0076's implementation boundary, item 4, evidence refresh 2026-07-24** (`docs/decisions/0076-declare-target-honourable-numerical-realizations.md:160`): "`tiler-artifact` now carries an artifact-program model and a bounded neutral envelope codec, and that codec already encodes the resolved contract complete over its dimensions — `NumericalFacts` and `ResourceRequirements` both write all four dimensions through exhaustive tag maps. What item 4 still owns is everything the contract's *values* do not supply: each dimension's means of being honoured, the target facts relied on, and the declaring profile's identity. **None of that exists, because none of it can before ticket 3 declares it.**"

**Fact — the contract-values half is indeed already carried.** `crates/tiler-artifact/src/program/codec/model.rs:176-183`: `NumericalFacts` carries `profile_key`, `canonical_arithmetic_nan_bits`, `input_subnormals`, `result_subnormals`, `contraction`, and `reassociation`. `EntryRef::numerical()` reads the same realization off a verified artifact. So of the four content items this ticket owes, the resolved contract is done and the declaring profile's identity is expressible today (`TargetProfileRef`, already public in `program::keys`).

**Fact — the means vocabulary does not exist.** Exact check: `grep -rn "SupportedExactly\|SupportedWithExactEmulation\|SupportedOnlyUnderDeclaredRelaxation\|Honourab\|Honorab" crates/` returns no match. The four outcomes are named in `docs/numerical-semantics.md` and in ADR 0076 item 3; no crate implements them.

**Inference — building the draft now would invent the vocabulary ticket 3 owns.** ADR 0076 item 3 fixes the means declaration as "a stated, versioned profile fact with the same provenance discipline `CapabilityFact` already carries — an availability phase, a validity scope, an authority, and the declaring profile's identity", and assigns it to `tiler-metal` (`declare-metal-numerical-honourability`, p0, todo, scopes `implementation/metal` + `contracts/artifacts`) composed by `tiler-compiler` (`compose-numerical-honourability-and-retire-the-strict-boolean`, p1, todo). Defining a parallel means enum in `tiler-artifact` first would create a second authority over the same terms — which ADR 0076 itself forbids at line 58: "This record must therefore not invent a vocabulary. Doing so would create a second authority over the same terms, which the documentation contract forbids."

The same applies with more force to "the target facts relied on": no target-neutral representation of a relied-upon target fact exists, and inventing one so the record has a field to fill would be exactly the producer-less placeholder this repository has repeatedly had to retract.

**What changed here.** The missing dependency edge on `declare-metal-numerical-honourability` was added, so the work graph stops advertising this ticket as ready. Nothing was implemented; no public surface was added to `tiler-artifact`.

**Trigger for reconsideration.** When `declare-metal-numerical-honourability` lands the means vocabulary and the target-fact shape, this ticket becomes a projection of them into the artifact record plus its identity encoding, and its `needs-tom` public-surface question becomes answerable with a concrete draft rather than an invented one.
