---
id: repin-candle-numerical-scope-citation-at-adapter-admission
title: Re-pin the Candle numerical-scope citation when the adapter depends on Candle
status: done
priority: p3
dependencies: [prototype-candle-metal-adapter]
related: []
scopes: [contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [candle, numerics, contracts, provenance]
---
## User-visible outcome

A reader of `docs/integration/candle.md` can trust that its numerical-scope claims describe the Candle revision this workspace *actually resolves*, not an upstream revision inspected before any dependency existed.

`docs/integration/candle.md` cites `huggingface/candle` `31f35b147389700ed2a178ee66a91c3cc25cc80d` (0.11.0) for the runtime-compilation and compile-option facts under **Numerical scope across the Candle kernel boundary**.

**Fact — that is an inspected upstream revision, not a resolved pin.** No
checked-in workspace manifest or any of the eight checked-in `Cargo.lock` files
currently resolves Candle. The cited revision is therefore upstream evidence,
not a local dependency pin. Re-run the inventory at execution time rather than
carrying forward a manifest count.

**The work, once an adapter crate actually depends on Candle.** Re-read
`Kernels::load_library`, `get_compile_options`, and `MetalDevice::compile` at
the revision the workspace resolves; re-pin the citation and line references;
and restate the section's opening paragraph as a fact about that pin. If the
resolved revision differs from the Metal provenance contract's citation, open
or update a separately scoped `contracts/artifacts` correction rather than
editing that contract from this integration-only ticket.

Treat a change in either `new_library_with_source` call site, in the `CANDLE_METAL_ENABLE_FAST_MATH` default, or in `load_library`'s cache-by-`Source` behaviour as a change to the section's premise rather than a citation refresh: each of the three carries a distinct claim in the text.

**Closes when:** the section cites the revision the workspace resolves, and its no-dependency paragraph is restated or removed.

## Outcome (2026-08-01)

**Fact.** The adapter admission resolved `candle-core 0.11.0` from crates.io, and `31f35b14` is the "Bump candle version to 0.11.0" commit, so the Metal provenance contract's citation agrees with the resolved revision — no correction owed. All five cited line references were re-verified against the resolved crate by the admitting worker, and none of the three load-bearing premises moved. The registry pin over a git rev is derived, not defaulted: the no-vendoring rule governs forks, and this is an unmodified upstream release.
