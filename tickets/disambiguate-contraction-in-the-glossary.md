---
id: disambiguate-contraction-in-the-glossary
title: Disambiguate the two senses of contraction in the glossary
status: done
priority: p2
dependencies: []
related: [scope-einsum-contraction-support, qualify-contraction-association-reassociation-permission]
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, glossary, numerics]
---
The word "contraction" names two unrelated things in this corpus and the
glossary defines only one of them.

The numerical sense is ADR 0015's permission for a separately rounded multiply
and add to fuse into one rounding. `docs/glossary.md` covers it only obliquely,
inside the "Numerical policy" row ("granular optimization permissions such as
reassociation, contraction, and approximate intrinsics"). It is implemented as
`NumericalRealization::contraction` (`tiler-ir`),
`StrictF32NumericalContract::contraction` (`tiler-compiler`), and
`MetalNumericalRequirement::NoFloatingPointContraction` (`tiler-metal`).

The tensor sense is summation over indices shared by two or more operands —
matmul, batched matmul, einsum. It appears in `docs/roadmap.md` (Milestone 6 and
its framing section), `docs/compiler/optimizer.md`,
`docs/compiler/fusion-and-scheduling.md`, and `docs/ir.md`. It has no glossary
row at all.

The two meet at exactly one place: a tensor contraction's per-contributor step is
`accumulator + a * b`, and whether that becomes one rounding is the ADR 0015
permission. A reader who conflates them concludes a tensor contraction permits
FMA by virtue of its name. Under the registered strict `f32` contract that
permission is `Forbidden`.

Add explicit glossary rows for both senses that name each other as distinct.
`docs/glossary.md` is `contracts/foundation`, which
`scope-einsum-contraction-support` does not hold.

## Outcome

**Done.** `docs/glossary.md` now carries two rows, *Contraction (numerical)* and *Contraction (tensor)*, each naming the other as distinct.

**Both halves of the hazard are stated, not just the definitions.** The ticket's concern is that a reader conflates them and concludes a tensor contraction permits fused multiply-add by virtue of its name. Defining the two terms separately does not by itself prevent that inference, so the tensor row states the meeting point explicitly: the per-contributor `accumulator + a * b` step is where the numerical permission would apply, and under the registered strict `f32` contract it is `Forbidden`.

**One thing added beyond the ticket's scope, because it is the same confusion one step further.** The tensor row also records that a contraction's *association* is separately governed — regrouping `(AB)C` to `A(BC)` consumes distributivity, not reassociation. `docs/compiler/optimizer.md` already establishes this and states why the distinction matters: reporting a forbidden reassociation "would imply that a contract permitting reassociation would admit the rewrite, which is exactly the inference the numerical contract forbids". A glossary that disambiguated the FMA sense while leaving a reader to assume reassociation governs association would have closed one wrong inference and left its neighbour open.

**The numerical row names its three implementation spellings** — `NumericalRealization::contraction`, `StrictF32NumericalContract::contraction`, `MetalNumericalRequirement::NoFloatingPointContraction` — so a reader meeting the word in code reaches the right row rather than the tensor one.

**Evidence.** `uv run --locked python scripts/docs.py render` passes at 181 records; full repository gate green.
