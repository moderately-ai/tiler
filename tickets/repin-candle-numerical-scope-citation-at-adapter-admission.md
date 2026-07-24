---
id: repin-candle-numerical-scope-citation-at-adapter-admission
title: Re-pin the Candle numerical-scope citation when the adapter depends on Candle
status: todo
priority: p3
dependencies: [prototype-candle-metal-adapter]
related: []
scopes: [contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [candle, numerics, contracts, provenance]
---
`docs/integration/candle.md` cites `huggingface/candle` `31f35b147389700ed2a178ee66a91c3cc25cc80d` (0.11.0) for the runtime-compilation and compile-option facts under **Numerical scope across the Candle kernel boundary**.

**Fact — that is an inspected upstream revision, not a resolved pin.** Tiler declares no Candle dependency: the root `Cargo.toml`, the six `crates/*/Cargo.toml`, the two `prototypes/*/Cargo.toml`, and every `spikes/**/Cargo.toml` name none, and none of the nine checked-in `Cargo.lock` files contains a `candle` package (`find . -name Cargo.lock -not -path './target/*' -exec grep -l candle {} \;` returns nothing). The section says so in its own first paragraph, which is what keeps the citation honest under `AGENTS.md`'s rule that a source claim name the exact local dependency revision.

**The work, once an adapter crate actually depends on Candle.** Re-read `Kernels::load_library` and `get_compile_options` in `candle-metal-kernels/src/kernel.rs`, and `MetalDevice::compile` in `candle-core/src/metal_backend/device.rs`, at the revision the workspace resolves; re-pin the citation and its line numbers to that revision; and restate the section's opening paragraph, which currently records the absence of a dependency, as a fact about the pin. Coordinate with `correct-metal-provenance-candle-revision-citation` so `docs/backends/metal.md` moves to the same revision.

Treat a change in either `new_library_with_source` call site, in the `CANDLE_METAL_ENABLE_FAST_MATH` default, or in `load_library`'s cache-by-`Source` behaviour as a change to the section's premise rather than a citation refresh: each of the three carries a distinct claim in the text.

**Closes when:** the section cites the revision the workspace resolves, and its no-dependency paragraph is restated or removed.
