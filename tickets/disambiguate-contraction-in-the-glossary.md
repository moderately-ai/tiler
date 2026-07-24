---
id: disambiguate-contraction-in-the-glossary
title: Disambiguate the two senses of contraction in the glossary
status: todo
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
