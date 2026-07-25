---
id: decide-strict-realization-fallback-availability
title: Decide whether a strict numerical contract may fall back to Candle
status: done
priority: p2
dependencies: []
related: []
scopes: [contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [candle, numerics, decision, contracts]
---
`scope-tiler-numerical-claims-across-the-candle-kernel-boundary` established two facts that meet in one already-accepted rule. The meeting point is a product choice, not a derivation, so it is stated here for Tom rather than settled.

**Fact — Candle's own kernels are fast-math by default.** At `huggingface/candle` `31f35b147389700ed2a178ee66a91c3cc25cc80d` (0.11.0), `get_compile_options` in `candle-metal-kernels/src/kernel.rs:182` reads `CANDLE_METAL_ENABLE_FAST_MATH` with default `true` and, on macOS 15 or iOS 18 and later, sets `MTLMathMode::Fast` together with `MTLMathFloatingPointFunctions::Fast`. Tiler's strict baseline on the qualified toolchain row is the opposite corner of the same axis: `-fmetal-math-mode=safe`, `-fmetal-math-fp32-functions=precise`, `-ffp-contract=off`.

**Fact — the accepted adapter rule already resolves the collision one way.** `docs/integration/candle.md` states that the unfused Candle fallback "is valid only when its numerical and autograd contract matches the requested semantics". Applied to the fact above, a request for a strict `f32` realization has no valid Candle fallback, because the ordinary Candle expression delivers a different contract. The wrapper must fail closed.

## The question

**Is losing fallback entirely the right behaviour for a strict numerical contract, or should a strict request be permitted to fall back under a recorded numerical difference?**

**Option A — keep the accepted rule. A strict contract has no Candle fallback and fails closed, naming the unmet realization.** Enables: one program, one numerical contract, and a reference comparison whose expectation does not depend on which path ran. Prevents: any strict-contract program from running at all on a shape, layout, or dtype no Tiler variant covers. The safety net that makes the adapter adoptable is then unavailable to exactly the consumers who care most about numerics.

**Option B — permit fallback under a declared numerical difference, recorded in explain output and in the delivered-realization record.** Enables: availability, so a strict program still runs wherever Candle runs. Prevents: the guarantee that a result was produced under the requested contract. The observable value becomes a function of which path was selected.

**Recommendation: Option A.** ADR 0076 item 5 forbids delivering anything other than the declared contract; a fallback that delivers a different one is that same defect relocated from the kernel to the router, and it reintroduces precisely the misattribution the parent ticket exists to prevent. The availability cost is real, which is why this is a decision about who the adapter is for rather than about what is correct. Under Option A the mitigation is variant coverage, not a numerical escape hatch.

**Fact bearing on whether Option B is even implementable.** `Kernels::load_library` caches libraries by `Source` and calls `get_compile_options` only on a cache miss, so the math mode compiled into an already-cached Candle library is whatever the environment variable and the OS-version test yielded at its first load in that process. An adapter reading `CANDLE_METAL_ENABLE_FAST_MATH` at fallback time therefore learns nothing reliable about what the fallback path will deliver. Option B could only record "a different, unidentified realization", not a specific one, unless Candle grows a way to report a loaded library's compile options.

**Trigger:** the ticket that implements variant selection and fallback in a real adapter crate, or any earlier proposal to register a numerical contract stricter than Candle's default. Until then `docs/integration/candle.md` documents Option A, because Option A is what the accepted text already says.

## Outcome

**Resolved by the coordinator, 2026-07-25 — not escalated.**

**Option A, and it was not escalated, because Option B does not survive the correctness rules.**

The ticket's own closing fact settles it. `Kernels::load_library` caches libraries by `Source` and calls `get_compile_options` only on a cache miss, so the math mode compiled into an already-cached Candle library is whatever the environment variable and the OS-version test yielded at its first load in that process. An adapter reading `CANDLE_METAL_ENABLE_FAST_MATH` at fallback time therefore learns nothing reliable about what the fallback will deliver.

So Option B cannot record *which* realization it delivered — only that it delivered a different, unidentified one. That is not a declared numerical difference; it is an undeclared one wearing a record. `AGENTS.md` forbids returning an incorrect result to preserve a fast path, and ADR 0076 item 5 forbids delivering anything other than the declared contract; a router that delivers an unidentified contract is that defect relocated rather than mitigated.

A strict contract therefore has no Candle fallback and fails closed, naming the unmet realization. The availability cost is real and the mitigation is variant coverage, not a numerical escape hatch.

**This would return to Tom as a genuine choice if Candle grew a way to report a loaded library's compile options**, because Option B would then be able to name what it delivered. That is the trigger.
