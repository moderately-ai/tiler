---
id: express-metal-honourability-in-the-shared-form
title: Express the Metal subnormal fact as a per-dimension honourability declaration
status: done
priority: p0
dependencies: [compose-numerical-honourability-and-retire-the-strict-boolean, prototype-public-compiler-api, admit-a-caller-declared-target-profile]
related: [declare-metal-numerical-honourability, draft-target-honourable-numerical-contract-adr, construct-and-bind-the-first-authoritative-metal-compile-profile]
scopes: [implementation/metal, implementation/compiler, implementation/build, contracts/foundation, contracts/decisions, contracts/numerics, contracts/artifacts, contracts/navigation, implementation/metal-aot, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, numerics, feasibility]
---
The remaining half of `declare-metal-numerical-honourability`, split out when its two settled questions landed. ADR 0076 item 3.

## User-visible outcome

The public compiler profile can receive caller-vouched measured Metal F32 subnormal behaviour before emission, keyed independently by input/result dimension and exact scalar dtype. Compiler feasibility then produces the existing typed numerical refusal, including required behaviour, declared means, honoured behaviour, and exact profile identity. The Metal backend retains its local re-verification.

This is deliberately a bounded projection seam, not a production Metal profile. It does not source quantitative or F32-dispatchability facts, prove that supplied measurement contexts produced the independently supplied Metal fact, bind the profile to a Metal plan/artifact/runtime environment, or infer F16/BF16 behaviour.

## Ownership derivation

**Fact:** `tiler-metal` and `tiler-compiler` have no dependency edge in either direction, and the compiler core must remain independent of Metal types. `tiler-build` already depends on both authorities.

**Inference:** the checked adapter belongs in `tiler-build`. Putting Metal facts in semantic IR would reverse the public semantic/physical boundary, and a third crate has no second consumer today.

**Ratified public boundary:** Tom accepted the composable surface after API and maintainability review:

- `TargetCompileProfileMeasurementSource` fixes empirical compile-profile provenance to `CompileProfile` / measured-profile authority / exact measured environment and requires nonempty compiler-build/environment contexts.
- `TargetProfileBuilder` exposes independent measured input- and result-subnormal declaration operations. Each inserts one complete, exclusive three-row table atomically and rejects any same-subject/dimension row already present at any phase.
- `tiler-build::declare_metal_f32_subnormal_behaviour` projects only the F32 row from `MetalTargetFacts`. It stages both dimensions on a cloned builder and publishes both or neither; it does not freeze the builder, so later quantitative, dispatchability, and dtype declarations remain composable.
- `MetalSubnormalArithmetic::subnormal_mode` is the owner-side total projection into shared vocabulary. Metal emission uses it for the retained fail-closed backend comparison.

## Implementation evidence

**Fact:** the exact F32 projection refuses an unstated F32 row and does not read measured F16 or BF16 neighbours.

**Fact:** a flushing F32 declaration, paired in the test with an explicit F32 dispatchability row, reaches compiler numerical feasibility. A strict request is rejected as `TargetCompileRefusal::NumericalContract`, with an input-preserve requirement, `Unsupported` declared means, the sign-preserving flush as the honoured alternative, and the exact declaring profile identity.

**Fact:** changing the delivered behaviour, compiler build, or platform build changes the canonical profile descriptor. An existing result-subnormal row rejects the two-dimension Metal transaction without leaving an input table behind.

**Measurement — mutation checks:** changing the expected refusal dimension, omitting the result projection, and publishing the staged input table before the result declaration each made its focused test fail before restoration.

**Review result:** the accepted API keeps the compiler operation target-neutral and per dimension, keeps the Metal projection bounded and composable, chains its typed error, and states the caller-vouched provenance limitation. No serial-sum, plan, cache, artifact, or runtime call site was migrated by this ticket.

## Constraints inherited

- Honourability remains a stated fact, never something inferred from a compiled kernel. Optimized-away arithmetic can mimic preservation.
- A mismatched zero sign remains a rejection.
- Measurement detail remains on the declaring Metal type and in the supplied structured source.
- `MetalNumericalGap` and `require_declared_realization` remain as backend defence in depth.

## Closes when

The ratified boundary and implementation evidence above are recorded in the durable contracts; focused nextest, doctests, Clippy, formatting, ticket lint, and diff checks pass; the root integration gate passes; and the ticket moves to `done`. Production profile construction is not a hidden remainder of this outcome: it is the explicit p0 follow-up below.

## Graph maintenance

- `construct-and-bind-the-first-authoritative-metal-compile-profile` follows this ticket and owns real quantitative/F32-dispatchability sources, complete compiler/Metal binding, independent runtime applicability, and serial-sum migration.
- The BF16 spike follows that production-profile ticket rather than consuming this low-level F32 seam directly.
- Delivered-realization redesign may use checked synthetic evidence; production artifact wiring waits for the authoritative profile.
- If a second non-build orchestrator needs the same projection, file the evidence for extracting a shared adapter crate rather than creating one speculatively.
