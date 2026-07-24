---
id: record-delivered-numerical-realization
title: Record the delivered numerical realization in the artifact
status: todo
priority: p1
dependencies: [select-numerical-contract-and-compose-feasibility]
related: [draft-target-honourable-numerical-contract-adr, prototype-artifact-program-model]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, numerics, needs-tom]
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
