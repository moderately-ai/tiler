---
id: design-the-adapter-owned-route-requirement-answer-channel
title: Design the adapter-owned route-requirement answer channel
status: in-progress
priority: p1
dependencies: []
related: [dispatch-a-tiler-region-on-metal-hardware, route-an-embedded-artifact-through-a-consumer-storage-seam, realize-parallel-reduction-strategies-on-metal]
scopes: [research/runtime, research/extensions]
shared_scopes: [project/tickets]
paths: []
tags: [research, design, runtime, backend-providers, public-boundary]
claimed_from: todo
assignee: worker-answer-channel
lease_expires_at: 1785594086
---
## User-visible outcome

A dispatching consumer can answer a backend-typed live-device route requirement — `tiler.metal.route-requirement.minimum-gpu-family` is the concrete case, one `MTLDevice::supportsFamily` call away — through a designed channel, instead of every such row being permanently `Unrecognized` on the facade path.

## The decision Tom made, and what remains

**Fact — reviewed 2026-08-01.** Tom reviewed three candidates and chose the design-ticket route with a stated lean toward the adapter-owned channel, with fail-closed as the explicit interim:

- **(a) Re-export the `tiler-metal` vocabulary through the facade — eliminated.** No facade-reachable signature names `MetalGpuFamily` (the consumer must *produce* one, not read one, so the `tiler::runtime` re-export precedent does not apply); it would put a backend crate in every consumer's closure including fallback-only ones; and a second backend would add a second crate.
- **(c) Fail-closed forever — rejected as a terminus, accepted as the interim.** `spikes/runtime/inline-dispatch` answers `Unrecognized` today and that stays correct while this design runs. As an end state it leaves the compiler minting requirements nothing on the primary consumer path can answer.
- **(b2) Adapter-owned answers — the lean this ticket derives or refutes.** A *dispatching* consumer is already backend-specific (the spike links `metal` itself); the facade rule "a consumer names `tiler` alone" is a property of the fallback path. The shape to derive: the applicability vocabulary becomes a deliberately public, versioned boundary of `tiler-metal` that the consumer's **adapter** — the device authority that observed the device — may name; the runtime validates the constructed answer against the carried requirement without the neutral layer naming the backend. The dependency arrow stays consumer→backend, never core→backend.
- **(b1) Neutralize the vocabulary into the runtime/artifact layer** — carried as the alternative b2 must eliminate on the record rather than by assertion. Its stated hazard: the value set is irreducibly a backend fact, so a neutral carrier is either a disguised backend registry or opaque bytes — an unvalidated second authority.

## Questions the design must answer, each with its elimination stated

- Exactly which items of `tiler_metal::applicability` go public, under what versioned identity, and what the compatibility contract of that boundary is — a backend vocabulary a consumer names is a surface that can no longer be reshaped freely.
- How the neutral runtime validates a constructed answer against the carried requirement without depending on the backend: what travels typed, what travels canonical-bytes-with-backend-validation, and where the validation authority lives.
- How ADR 0086's eligibility gate composes with an answered row: answering a GPU-family requirement is a device *capability* fact, not translation attestation — state precisely which routing conclusions an answer does and does not license, so a satisfied family row is never read as host-earned translation eligibility.
- What a second backend (the CPU family is the live candidate under the current target-device priorities) does with the same channel — the design generalizes or says why it deliberately does not.
- What the fallback-only consumer's contract remains: naming `tiler` alone must stay sufficient for every non-dispatching use.

## Closes when

The channel is designed with the b1/b2 elimination written where a reader can refute it, the exact public boundary items are enumerated and taken to Tom under ADR 0075 rather than self-accepted, the interim fail-closed behaviour is restated in the spike and route documentation as deliberate, and the outcome is an accepted design, a recorded deferral with trigger, or a bounded experiment.

## Outcome

**The design is complete, b2 survives, and every one of the five questions is answered with a stated elimination.** The derivation, the b1/b2 elimination, both worked examples, the public-boundary list, the measurement boundary, and an ADR body written to be landed verbatim live in [Backend-scoped route-requirement answers](../docs/research/runtime/backend-scoped-route-requirement-answers.md). Research and design only: no crate gained an item, the spike stays fail-closed, and nothing was compiled or measured beyond reading source and one SDK header.

### The finding that reframes the ticket

**The answer channel already exists, is already neutral, and already works end to end.** `LiveDeviceObservation::Feature(bool)` is what crosses the seam for a backend-scoped row; the loader owns the owner check, the shape check, and the satisfaction decision; and `crates/tiler-runtime/tests/adapter_route` exercises all three refusal classes for a backend feature row in the ordinary gate against a fictional backend. **What a consumer cannot do is decide which `bool` to report**, because that means decoding the payload under a vocabulary it may not name. The ticket asks for a channel; the actual gap is a *decoder*. That reframing is load-bearing: a design that added an answer variant or a payload accessor would widen a boundary that is already correct.

### Two corrections to the ticket's own premises, stated rather than absorbed

**The compiler mints no route requirement today.** `grep -rn 'RouteRequirement' --include='*.rs' crates/tiler-build/src/` returns nothing. The `minimum-gpu-family` row exists only in two prototypes and in artifact-layer tests, so no consumer is refused for this reason at `6f7caf3` and the ticket's "compiler minting requirements nothing can answer" is prospective. It does not weaken the case — it changes the landing order, because the producing and consuming halves are one codec.

**(a)'s closure-cost elimination is weaker than the ticket states.** `crates/tiler-metal/Cargo.toml` depends on `tiler-artifact` and `tiler-ir` and nothing else, and both are already in the facade's closure — so (a) adds no new *transitive* crate, only `tiler-metal` itself (4,076 non-test lines, mostly the MSL emitter). (a)'s elimination survives on its other two grounds, which do not depend on closure size. A future reader should not rest on the cost argument.

### The five answers

1. **Four items go public, and neither `MetalGpuFamily` nor the governed constants are among them.** `tiler-metal` publishes an observation function taking the caller's probe as a callback over an opaque raw Apple constant, a decision function taking the whole `RouteRequirement`, the three-valued answer type the consumer maps, `MetalGpuFamilySupport` as a pass-through, and the minting constructor. **Re-export `applicability`** eliminated — its other half is the ADR 0086 policy, whose public items are a reviewed draft containing an uninhabited receipt, and publishing it would put the policy's exact-equality family comparison beside the route row's minimum comparison. **Publish `MetalGpuFamily` and let consumers observe the device** eliminated on ADR 0074 conventions 5b/5c — see the defect below. **Publish `as_str` and let consumers write the codec** eliminated as the duplication being removed. **Mint a fresh consumer-facing enum** eliminated as a second spelling authority.
2. **The neutral layer validates *owner and shape*, never content, and that suffices because the loader's decision is total over the answer.** Typed: owner, key, version, and the three-valued `LiveDeviceObservation`. Canonical bytes with backend validation: the payload alone. Authority runs in four ordered places — owner in the loader without a device, meaning in the backend, shape in the loader's wildcard-free 2×3 match, satisfaction in the loader. **There is no answer a backend can construct that the loader cannot rule on**, which is what makes backend-free validation achievable. **A registered payload validator** eliminated — a Rust object cannot survive the process boundary an artifact crosses. **A richer answer the loader compares** eliminated — it is b1 by another route. **The loader re-checking the decision** eliminated as vacuous. The apparent conflict with ADR 0090 item 4 is resolved explicitly: item 4 forbids reversing a *capacity* comparison, and `Feature`'s own documentation says the owning adapter *decides* the qualitative row.
3. **A satisfied row licenses continuation of that route and answers one of `MetalHostPredicate`'s seven predicates; it licenses nothing about profile eligibility.** The sharp finding is that **the two comparisons over the one vocabulary run in opposite directions**: `evaluate_metal_host_applicability` requires the family *exactly* (`applicability.rs:843`, with the stated reason that a higher family would extend a bounded measurement), and the route row requires *at least*. The composition is safe by construction because `variant_eligibility` runs during `select_variant`, **before** route requirements are built, and because `evaluate_metal_host_applicability` takes a `MetalHostObservation` and nothing else with two compile-fail doc-tests pinning it. **Letting a satisfied row feed the policy** eliminated — six-of-seven progress in explain output for a host whose missing predicate is the one ADR 0086 item 3 says the measured row cannot stand in for. **One shared function for both** eliminated — it would have to pick a direction and be wrong for one caller; the *observation* is correctly shared and the *comparisons* correctly are not.
4. **It generalizes structurally; the pattern costs each backend a consumer-nameable crate; and the CPU case pays two extra costs.** The neutral half is already proven against the fictional `tiler.test.scalar-host`. CPU costs: the availability phase is borrowed, because an ISA fact is process-bound and no phase names a bound host process (ADR 0090 item 14); and one row forces a CPU plan off `preflight` onto `prepare`, where both device stages then run unconditionally — verified that the scalar CPU vertical implements no `RuntimeAdapter` at all and carries zero rows. Structural precondition: there is no `tiler-cpu`. The CPU case also does *not* need the raw-constant callback, because an ISA probe takes a string literal the backend can hold itself — so the shape generalizes and its exact spelling is per-backend. **Metal-only** eliminated as contradicted by the in-gate test. **A neutral floor instead** eliminated for ISA level on the same ordering ground as b1b, and *not* eliminated for genuinely quantitative CPU facts.
5. **Naming `tiler` alone stays sufficient for every non-dispatching use, unchanged**; a *dispatching* consumer names `tiler` plus the backend crate whose device it drives, which is already true in every observable form except the vocabulary. **Feature-gating the vocabulary onto the facade** eliminated on Cargo semantics — features unify across a build graph, so one crate enabling `metal` restores the exact property (a)'s elimination rests on, by a mechanism that hides it. **A thin `tiler-metal-consumer` crate** deferred rather than eliminated, with its trigger recorded.

### The b1/b2 elimination, in one refutable line

b1 splits three ways. **b1a** (a neutral Apple-family enum) is refused by a contract sentence that names this exact case: `requirement.rs:45-47` says reading the payload in the neutral layer "would put a backend's vocabulary — an Apple GPU family, say — inside the neutral core". **b1c** (a neutral carrier of opaque bytes the backend validates) is not an alternative — it is what `BackendFeatureRequirement` already is and what b2 keeps. **b1b** (a neutral ordered token) is the one that looks like it works, and it is refuted by a counterexample a reader can check by hand: a neutral layer's only ordering is lexicographic, and `"Apple10"` compares **less** than `"Apple9"` byte-for-byte because `'1'` (0x31) precedes `'9'` (0x39). The general statement is stronger than the instance — a vocabulary's ordering is a fact about the backend that mints it — and carrying a rank integer instead just moves the same authority problem into data minted twice.

The (a)-versus-b2 difference reduces to `FACADE_FORBIDDEN_DEPENDENCIES` (`crates/tiler/tests/dependency_direction.rs:38`) with opposite signs: (a) requires that list not to grow, b2 requires it to gain `tiler-metal`.

### A defect found while deriving, filed separately because it is live today

**`MetalGpuFamily` is already an ADR 0074 convention 5b type and its stated reason is false at `6f7caf3`.** Its doc comment says "no consumer outside this crate classifies it by exhaustive match"; `prototypes/candle-metal-adapter/src/adapter.rs:584-590` does, as a five-element pair table. Written as a *table* rather than a match, adding `Apple10` compiles cleanly, the device is never probed for it, and every route requiring Apple10 is refused on a device that satisfies it — convention 5c's named failure, reached without the attribute being involved. This is why the design's public surface **removes** the out-of-crate total map rather than publishing the vocabulary: the probe loop belongs to `tiler-metal`, and the Apple constant crosses as a raw value. [`close-the-metal-gpu-family-out-of-crate-total-map`](close-the-metal-gpu-family-out-of-crate-total-map.md) carries it, independent of whether this design is ever accepted.

**Fact — the raw constant is primary-source.** `MTLDevice.h` in the installed macOS 26.5 SDK declares `MTLGPUFamilyApple1 = 1001` through `MTLGPUFamilyApple9 = 1009` (`$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLDevice.h:233-241`); `metal` 0.33.0 transcribes the same values. Passing the constant rather than a typed family is binding-agnostic, which matters because `metal` models `MTLGPUFamily` as a Rust enum and `objc2-metal` as a newtype over `NSInteger`.

### A second defect, in the obvious enforcement change

Adding `tiler-metal` to `FACADE_FORBIDDEN_DEPENDENCIES` **makes the test fail**, and reading the guard rather than the assertion is what finds it. `dependency_direction.rs:121-128` asserts, per forbidden name, that `tiler-macros` really holds that edge before concluding the facade does not — a genuine anti-vacuity guard — and `tiler-macros` does not depend on `tiler-metal`. The enforcement needs a second list for edges no frontend package holds, or a per-entry witness. Recorded as a deferral with its trigger.

### Public-boundary items, listed for Tom and not self-accepted

Under ADR 0075's mechanical categories, only one fires: a new `pub mod` in `tiler-metal`'s crate root, which this record recommends over adding to `applicability` precisely because mixing the two family comparisons in one module is the confusion question 3 exists to prevent. Six more are Tom's under AGENTS.md's broader clause and are named as such: reclassifying `tiler-metal` as a crate a consumer may name (which amends `docs/architecture.md:389`); the exact shapes of the observation function, the decision function, the raw-constant newtype, and the three-valued answer; whether the observation crosses as a raw Apple constant at all; the minting constructor's shape; `MetalGpuFamilySupport` becoming a compatibility surface; and whether the governed key and version stay private. Two items need no approval: the test change and this documentation.

### Dispatch note — the ADR and both catalogs are out of scope

`ticketsplease.toml` maps `docs/decisions/[0-9]*.md` to `contracts/decisions` and maps **both** `docs/decisions/README.md` and `docs/research/README.md` to `contracts/navigation`. This ticket holds `research/runtime` and `research/extensions` exclusively and `project/tickets` shared, so the ADR file and both catalog rows are a guard escape from this branch. [`land-the-backend-scoped-route-requirement-answer-adr`](land-the-backend-scoped-route-requirement-answer-adr.md) carries all three, following the `land-the-bf16-conversion-and-accumulator-adr` precedent; the ADR body is written to be landed verbatim. The `docs/architecture.md` sentence decision item 6 amends is `contracts/foundation`, held by a live sibling, and is part of the acceptance sweep rather than of landing a proposal.

### Verification

`tkt lint`, `git diff --check`, `tkt guard --base 6f7caf3`, and `make full` on the completed branch; `make full` is expected to be a no-op for a docs-and-tickets change and was run to prove the crates were untouched.
