---
id: correct-metal-provenance-candle-revision-citation
title: Correct the Metal provenance section's Candle revision citation
status: done
priority: p2
dependencies: []
related: [record-metal-runtime-compiler-provenance-gap]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, candle, metal, provenance]
---
`docs/backends/metal.md` states, under **Fact — the exclusion scopes Tiler's kernels, not the host process**, that the runtime-compilation call sites were read "in the local `huggingface/candle` working checkout at revision `4bb954d`". Two things are wrong with that citation, and the claim it supports is not one of them.

**Measurement — `4bb954d` is not a `huggingface/candle` revision.** In `/Users/tsanterre/workspace/github.com/huggingface/candle`, `git branch --show-current` reports `tomsanbear-dev`, `git remote -v` shows `origin` at `https://github.com/huggingface/candle` and `tomsanbear` at `git@github.com:tomsanbear/candle.git`, and `git branch -r --contains 4bb954d6ee1f5a539b14c5d0028a3ebb049218be` lists exactly `tomsanbear/tomsanbear-dev`. The commit is a fork-branch commit that upstream does not contain, so a reader cannot resolve it against the repository the sentence names. The abbreviated form compounds this: no other revision citation in the corpus is abbreviated to seven characters.

**Fact — the claim itself is true at the revision the rest of the corpus cites.** `git grep -n new_library_with_source 31f35b147389700ed2a178ee66a91c3cc25cc80d -- candle-metal-kernels candle-core` reports `candle-metal-kernels/src/kernel.rs:122` and `candle-core/src/metal_backend/device.rs:111`, inside `Kernels::load_library` (line 109) and `MetalDevice::compile` (line 101) respectively. That revision is Candle 0.11.0, is the checkout's `origin/main`, and is already cited by `docs/research/runtime/candle-metal-post-wait-error-checking.md`, `docs/research/runtime/runtime-execution-contract.md`, `docs/research/shapes/public-static-shape-spelling.md`, `docs/research/transfers/transfer-synchronization-and-resource-lifetime.md`, `docs/research/extensions/operation-extension-api.md`, and `docs/research/target-profiles/physical-feasibility-model.md`.

**The work.** Re-pin that one sentence to the full `31f35b147389700ed2a178ee66a91c3cc25cc80d`, with the line numbers above, so `docs/backends/metal.md` and `docs/integration/candle.md` cite the same inspected revision. `docs/integration/candle.md` already does, under **Numerical scope across the Candle kernel boundary**, and additionally records that Tiler declares no Candle dependency — a fact the metal.md sentence's "working checkout" phrasing obscures rather than states. Do not restate the boundary itself; metal.md already delegates it.

**Closes when:** `docs/backends/metal.md` cites a revision reachable in the repository it names, and the two documents' Candle citations agree.

## Outcome

`docs/backends/metal.md` now cites `huggingface/candle` `31f35b147389700ed2a178ee66a91c3cc25cc80d` (0.11.0) in full, with the exact line numbers — `Kernels::load_library` at `candle-metal-kernels/src/kernel.rs:109` calling `new_library_with_source` at 122, and `MetalDevice::compile` at `candle-core/src/metal_backend/device.rs:101` calling it at 111. That is the revision `docs/integration/candle.md` and six other corpus documents already use, so the two Candle citations now agree and a reader can resolve the commit against the repository the sentence names.

The sentence also stopped calling it a "working checkout" and now records the fact that phrasing obscured: Tiler declares no Candle dependency, so this is an inspected upstream revision rather than a resolved pin. Re-pinning it when an adapter crate actually depends on Candle is `repin-candle-numerical-scope-citation-at-adapter-admission`.

The claim the citation supports was never in doubt and is unchanged; only its provenance was wrong. The boundary itself is still delegated to `docs/integration/candle.md` rather than restated here.

`uv run --locked python scripts/docs.py render` and `uv run --locked python scripts/check_repository.py` pass.
