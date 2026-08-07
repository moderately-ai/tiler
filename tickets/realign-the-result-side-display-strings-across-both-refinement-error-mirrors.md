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

**Fact.** The four result-side `Display` arms in `crates/tiler-compiler/src/legality.rs:690-705` render `region output {position} …`, and after the partitioned-binding landing `position` is an ordered *result* position (several roots of one partitioned result report the same one) and `ResultArity`'s count is distinct output *tensors*. The compiler arms are verbatim mirrors of the IR's own strings at `crates/tiler-ir/src/index/refinement.rs:3020-3034`, so changing one side alone desynchronizes the mirror; both must move together, which spans two scopes and is why the doc-comment ticket did not take it.

**This is an observable-output change**, not documentation: rendered diagnostic text moves. Check whether any test asserts the exact strings and move those assertions in the same change.

## Closes when

Both crates' result-side strings state the counted population and the position's meaning identically, any string-asserting tests move with them, and the mirror is re-verified verbatim-equal.
