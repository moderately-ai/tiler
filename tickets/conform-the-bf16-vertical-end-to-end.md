---
id: conform-the-bf16-vertical-end-to-end
title: Conform the BF16 vertical end to end against the exact reference corpus
status: blocked
priority: p2
dependencies: [validate-bf16-at-the-runtime-routing-boundary, carry-a-bf16-subnormal-realization-the-reference-can-be-told]
related: [spike-bf16-through-the-second-dtype-seams, evaluate-bf16-reference-semantics, own-the-dtype-support-maturity-matrix, lower-bf16-to-metal, dispatch-a-tiler-region-on-metal-hardware, wire-the-bf16-reference-to-the-realization-it-is-told]
scopes: [implementation/reference, contracts/numerics, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, conformance, testing]
---
## User-visible outcome

One checked conformance run carries a pure-BF16 program from semantic construction to a dispatched device result and compares it against the exact-rational oracle, so a regression anywhere in the vertical is a red test rather than a wrong tensor. Until this exists the layers are each tested against their neighbour and nothing tests the composition.

## Why a per-layer test is not this

**Fact.** Each BF16 child closes on evidence about its own layer. That is correct and it is not sufficient: the U4/F32 vertical is the standing example of a family whose physical carrier, kernel, and lowering are each tested while the composition is not, and `docs/dtype-support.md` records that non-monotone row deliberately.

**Inference.** The composition is where a dtype's *width* assumptions fail — a two-byte element counted as four survives every single-layer test that uses consistent counts on both sides, and only an end-to-end run with a hand-derived expected result catches it.

## Required evidence

- One program — BF16 constant, multiply, add — carried from semantic construction through compile, artifact, runtime routing, and device dispatch, with the exact expected result bits stated in the test rather than read back from the run.
- The corpus covers, in the end-to-end run and not only in the unit oracle: both zeros with their signs, the least positive and least negative subnormals, the greatest subnormal, the least normal, a tie resolved to even, an ordinary rounding, an overflow to infinity, both infinities, and a non-canonical NaN that canonicalizes.
- The **declared flush is applied to the reference before comparison**, and the elements it moves are named. Finding 24 measures BF16 arithmetic flushing subnormals on the macOS row, so bit equality on the subnormal cases would mean the device did not do what was measured — a passing test there is a signal to distrust, not a success.
- An execution witness on a non-subnormal operand, for the reason finding 24 gives: `flushed` and `the arithmetic was optimized away` produce the same observation without one.
- At least one perturbation of the composition itself, observed failing — for instance an element count derived from the wrong width, which every layer-local test would still pass.
- The measurement boundary stated: host, OS build, Metal version, GPU, and family, with no generalization beyond the row that ran.

## Closes when

The end-to-end run passes on the measured macOS row with hand-derived expected bits, the flush is applied and its affected elements named, the execution witness is present, the composition perturbation is observed failing, the boundary is recorded, and the BF16 `Conformance evidence` cell states what this run actually covers rather than the whole family.

## Graph maintenance

- The last of the BF16 implementation children; depends on the runtime boundary, which transitively depends on the rest.
- A host without the measured environment must still run the deterministic reference and structural half and report the unavailable measurement boundary, rather than skipping silently or claiming a pass.
- This closes the BF16 vertical for the **three operations and one target family** it names. It does not promote BF16 generally: contraction, reduction, conversion, mixed precision, and every other family and target stay where the ledger puts them.

## Blocked 2026-08-06, with the derivation

Dispatched as an implementation ticket on `implementation/reference` + `contracts/numerics` at base `afdac9c9`. Neither required half is reachable from those scopes, and one of the two is not reachable from *any* scope set a worker may assemble. Nothing was descoped: no code landed, and the two blocks are structure below rather than a shortened evidence list.

### Block 1 — the device dispatch has no host anywhere in the test suite, and its obligation has been relayed twice

**Fact.** Exactly one workspace member can reach a Metal device. `grep -rn 'metal\.workspace\|^metal = ' --include=Cargo.toml .` returns the workspace pin, `prototypes/serial-sum-run/Cargo.toml:35`, and two *out-of-workspace* spikes (`spikes/runtime/inline-dispatch`, `spikes/target-profiles/metal-grid-axis-extent`). `prototypes/serial-sum-run` maps to `implementation/runtime`. `crates/tiler-reference` depends on `tiler-ir` alone, so adding a device edge there would put a live backend under the target-independent oracle — the dependency inversion the crate exists to prevent, and an architectural change that is Tom's.

**Fact — the same member is otherwise exactly right, which is why the placement question is narrow rather than open.** `prototypes/serial-sum-run` already depends on `tiler-artifact`, `tiler-build`, `tiler-compiler`, `tiler-ir`, `tiler-metal`, `tiler-metal-aot`, `tiler-reference`, `tiler-runtime`, and (macOS-gated) `metal`; its `[[bin]]` carries `test = true`, so a `#[test]` there is reached by `cargo nextest run --workspace` and therefore by `make full`. It is the only place in the repository where "a regression anywhere in the vertical is a red test" is presently constructible.

**Fact — the obligation was relayed here rather than scheduled here.** `lower-bf16-to-metal` closed at a revised offline boundary and assigned dispatch, the flush-applied comparison, and the execution witness to its two dependents. `validate-bf16-at-the-runtime-routing-boundary` then closed holding `implementation/runtime` — the one scope that could have dispatched — and recorded "**No BF16 kernel was dispatched.** Nothing in this branch touches a device", satisfying its own "routes and executes" item with a synthetic scalar-host image whose measurement boundary states "No case here claims BF16 executes". So the obligation arrived at a ticket declaring strictly weaker scopes than the one that had already declined it. That is the graph defect, not a worker-level inconvenience: `docs/dtype-support.md`'s BF16 `Backend execution` cell names this ticket and `validate-bf16-at-the-runtime-routing-boundary` as the owners of the dispatch, and neither held a dispatchable scope at the time it closed.

**Scope added, and why it belongs here.** `implementation/runtime` is added as declaration and scheduling metadata under the already-authorized outcome: the ticket's own `User-visible outcome` says "to a dispatched device result", and that sentence cannot be executed from any other scope. Verified free of any live claim on 2026-08-06 — the four in-progress tickets hold `implementation/compiler`, `implementation/ir`, `contracts/navigation`, and `research/numerics`. Three were read branch-side (`git show tkt/<id>:tickets/<id>.md`); `retire-the-corrected-softmax-fact-quotations` has no `tkt/` branch, so there is no branch-local scope addition to read and the integration copy is the only source that exists for it. None of the four holds `implementation/runtime`, `implementation/reference`, or `contracts/numerics`.

**The placement fork that remains, and is the coordinator's or Tom's rather than a worker's.** Adding a BF16 pointwise conformance run to `prototypes/serial-sum-run` repurposes the member whose manifest describes it as the "runner for Tiler's serial-Sum value proof", and whose `run()` is one linear F32 narrative. The alternative is a sibling prototype member, which is crate admission — `implementation/workspace` and `implementation/cargo-lock` for the manifest and lockfile, a new `[scopes]` entry in `ticketsplease.toml` (a path under `prototypes/` matching no existing glob is a guard scope-escape, exit 6), and Tom's decision under the scaffolding boundary. This ticket declares the scopes the first arm needs and deliberately does not pre-declare the second arm's, because declaring them would pre-commit the fork.

### Block 2 — the declared flush cannot be applied to the reference at all

**Fact.** `ReferenceNumericalConformance` reaches every capability through `ReferenceEvaluationRequest::conformance`, but applies its two subnormal dimensions with `apply_to_operand(f32) -> f32` and `apply_to_result(f32) -> f32` (`crates/tiler-reference/src/conformance.rs:234,240`). The three BF16 capabilities perform no binary32 arithmetic and read the conformance nowhere: `grep -n conformance crates/tiler-reference/src/bf16.rs` returns two hits, both in the module header explaining the gap. So this ticket's third required-evidence item — the declared flush applied to the reference before comparison — is presently unstatable, and a comparison run without it would assert bit equality on exactly the subnormal cases finding 24 measures the device moving.

**Fact — that gap is owned, its trigger has now fired, and its first deliverable is Tom's.** `carry-a-bf16-subnormal-realization-the-reference-can-be-told` owns it. Both of its triggers fired and were logged on 2026-08-06 (the authoritative profile now declares measured BF16 flush rows; a registered contract now resolves subnormal dimensions per format), and it moved from `deferred` to `todo`. Its first deliverable is a declaration naming which format a subnormal resolution speaks about, which that ticket states is a public boundary and not self-accepted — and the derived-versus-declared fork recorded there is a genuine one. Added to `dependencies:`, because this ticket cannot state its central comparison until that lands.

### Landed on this branch

Only the reachable in-scope correction, in `contracts/numerics`: [Correctness and testing](../docs/correctness-and-testing.md#semantic-authority) asserted without qualification that "The comparison follows the declared numerical contract and conformance level". One registered family cannot follow it, and nothing said so — an overstated contract sentence that makes this ticket's evidence list look reachable. The exception is now stated with its reproducing check, its directional consequence (the oracle is the side that would be called wrong), and its owning ticket, which is instructed to retire the paragraph when it closes.

### Not done, and not to be read as partially done

No conformance run, no corpus, no perturbation, no execution witness, no measurement boundary claimed, and no cell of any maturity ledger moved. `docs/dtype-support.md` is `contracts/navigation`, which this ticket does not hold and which was live-claimed on 2026-08-06 by `re-read-the-bf16-and-elementary-support-rows-against-source`; the BF16 `Conformance evidence` cell must stay exactly as it is, because this branch produced no run for it to describe.

### To unblock

Resolve the placement fork in Block 1, land `carry-a-bf16-subnormal-realization-the-reference-can-be-told`, then move this to `todo` and redispatch with `implementation/reference`, `contracts/numerics`, and `implementation/runtime`.

## Dependency note, 2026-08-07 — both dependencies are now `done`, and this stays blocked

Both edges are satisfied: `validate-bf16-at-the-runtime-routing-boundary` and [`carry-a-bf16-subnormal-realization-the-reference-can-be-told`](carry-a-bf16-subnormal-realization-the-reference-can-be-told.md) both read `done`, the latter closed on 2026-08-07 when Tom decided its parked fork (arm A — the format derived at the point of use).

**That does not unblock this ticket, and the status is deliberately left at `blocked`.** The two blocks recorded on 2026-08-06 are structural and neither is a dependency edge:

- **Block 1 is unchanged and is Tom's.** Only `prototypes/serial-sum-run` can reach a Metal device, and putting a device edge under `crates/tiler-reference` would invert the dependency the target-independent oracle exists to prevent. Where a device-reaching conformance test may live is an architecture decision, not a scope a worker may assemble.
- **The flush half is now closer but not landed.** This ticket's evidence list requires "the declared flush is applied to the reference before comparison". The reference can be *told* a realization, and after Tom's decision the route that tells it is settled — but the wiring itself is [`wire-the-bf16-reference-to-the-realization-it-is-told`](wire-the-bf16-reference-to-the-realization-it-is-told.md) and has not landed. Until it does, applying the flush here would mean stating the realization by hand, which is what the module header describes and not what this ticket asks for.

Add a dependency on the wiring ticket when it is claimed; it is left as a relation for now so this node is not re-pointed at work that may land before Block 1 is answered either way.
