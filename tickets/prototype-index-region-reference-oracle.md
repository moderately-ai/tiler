---
id: prototype-index-region-reference-oracle
title: Implement the generic IndexRegion reference oracle
status: todo
priority: p0
dependencies: [prototype-canonical-index-region-slice, correct-reference-value-and-authority-contracts]
related: []
scopes: [implementation/reference]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, reference, indexing, oracle]
---
Implement a slow generic checked IndexRegion oracle in tiler-reference. Resolve
registered scalar evaluators without downcasting, execute ordered multi-result
SSA and N-state lexical reductions, preserve exact dtype bits and empty-domain
semantics, and fail closed for missing authority. Fusion legality must compare
against this independent path rather than a graph-specific host expression.

Index-expression arithmetic is a decided constraint, not an open choice.
`tiler_ir::index::IndexInteger` deliberately exposes no public arithmetic (only
`from_i128`/`from_u64`/`from_sign_magnitude`/`to_sign_magnitude`), while
coefficients admit large magnitudes and intermediates can cancel past `i128`.
Implement exact evaluation inside `tiler-reference` over the sign-magnitude
representation, rejecting oversized intermediates with a typed fail-closed
error rather than saturating or wrapping. Do not widen `tiler-ir`'s public
surface to obtain arithmetic: this ticket does not hold `implementation/ir`,
that scope is contended by the p0 spine, and exposing checked `IndexInteger`
arithmetic is a separate reviewed boundary decision. Adding a bignum dependency
is permitted — `implementation/cargo-lock` is declared for exactly that — but
prefer the bounded in-crate path if it satisfies the admitted domain.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.
