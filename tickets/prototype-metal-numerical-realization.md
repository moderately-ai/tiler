---
id: prototype-metal-numerical-realization
title: Realize the strict numerical profile in Metal
status: todo
priority: p0
dependencies: [prototype-metal-kir-lowering]
related: []
scopes: [implementation/metal, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, numerics]
---
Map the strict proof profile to explicit MSL operations and offline compiler flags. Preserve canonical arithmetic NaN, signed zero, materialization conversions and reduction order; prohibit unlicensed contraction/reassociation; record exact toolchain realization and fail closed for unsupported realizations.

Test these semantics independently of the offline compiler's fast/safe-math flags: the realization must be provably carried by explicit generated operations rather than inherited from a flag setting, so flipping the math-mode flags cannot silently change conforming results. This obligation previously lived in the Metal AOT integration body and is owned here now that the AOT slice is an integration gate.
