---
id: realign-the-result-side-display-strings-across-both-refinement-error-mirrors
title: Realign the result-side Display strings across both refinement error mirrors
status: todo
priority: p3
dependencies: []
related: [realign-the-compiler-refinement-error-mirror-with-the-grouped-result-arity]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, diagnostics]
---
## User-visible outcome

The rendered result-side refinement errors say what their fields now carry, in both crates at once, so a diagnostic reader is not told a result position is a region output.

## The finding, from the doc-comment realignment

**Fact — verified by source anchors, not stale line numbers.** The four result-side `Display` arms in `crates/tiler-compiler/src/legality.rs` under `impl fmt::Display for RefinementError` render `region output {position} …`, and after the partitioned-binding landing `position` is an ordered *result* position (several roots of one partitioned result report the same one) and `ResultArity`'s count is distinct output *tensors*. The compiler arms mirror the IR strings under `impl fmt::Display for IndexRefinementVerificationError`; changing one side alone desynchronizes the mirror, so both must move together.

**This is an observable-output change**, not documentation: rendered diagnostic text moves. Check whether any test asserts the exact strings and move those assertions in the same change.

## Closes when

Both crates' result-side strings state the counted population and the position's meaning identically, any string-asserting tests move with them, and the mirror is re-verified verbatim-equal.
