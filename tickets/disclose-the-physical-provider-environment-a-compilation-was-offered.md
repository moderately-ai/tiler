---
id: disclose-the-physical-provider-environment-a-compilation-was-offered
title: Disclose the physical-provider environment a compilation was offered
status: todo
priority: p2
dependencies: []
related: [drive-an-external-physical-implementation-provider-through-compilation, expose-explicit-backend-provider-and-selection-policy-composition, audit-backend-authoring-against-all-thirteen-responsibilities]
scopes: [implementation/compiler, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, identity, explainability, compiler]
---
## User-visible outcome

`Compilation::offered_providers` either reports the complete frozen provider environment its name and documentation promise, or says in its own documentation which environment it reports — so that the compilation environment folded into durable artifact provenance stops silently excluding the physical authority.

## Why this exists

**Fact — the accessor's documentation states something the source refutes.** `crates/tiler-compiler/src/session.rs:757-763` documents `offered_providers` as "the complete frozen provider set offered to this compilation" and explains that returning "the compiler-minted set prevents an assembler from reconstructing that environment from the selected subset". The field is populated at `session.rs:2092-2093` as `Arc::from(capabilities.0.lowering().providers())` — the lowering registry alone. The governed physical provider's own identity, `tiler/prototype-serial-sum-physical` (`frontier.rs:2939-2941`, `:2977`), can therefore never appear in it. Under `AGENTS.md`'s rule that a doc comment is a claim and the source wins, the comment is the defect.

**Inference — the gap now reaches durable provenance, which it did not have to when ADR 0090 named it.** `crates/tiler-build/src/plan_artifact.rs:167` constructs the artifact's `CompilationEnvironment` from `compilation.offered_providers()`, so the set that omits the physical authority is the set recorded in the artifact. This is **not observably wrong today**: exactly one physical provider exists, nothing can vary it, and every selection the environment is checked against is a lowering selection, so the check is internally consistent. It becomes wrong on the first installed second provider — the moment two compilations differing only in physical-provider environment would record one environment and ADR 0072's "the complete frozen registry environment is inside compilation-request provenance" would be false of the artifact.

**Fact — ADR 0090 named the disclosure gap and did not name this consequence.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) item 5 records that "today neither is answerable at all" and proposes `offered_physical_providers` and `selected_implementations` as separate accessors. It cites `session.rs:1513` as the population site; at `51e9374a` that site is `session.rs:2092`. The accessors are owned by [`drive-an-external-physical-implementation-provider-through-compilation`](drive-an-external-physical-implementation-provider-through-compilation.md), which this ticket does not duplicate.

## Implementation keys

- The cheap half is correctness-derived and needs no new public surface: make the doc comment describe the lowering environment it actually returns, so a reader stops building on a false completeness claim. Do this whether or not the accessors land.
- Do not widen `offered_providers` itself to include physical providers as a shortcut. ADR 0090 item 5's whole ground is that "this provider was available and lost on cost" and "this provider was never installed" are the two findings a composition failure most needs to tell apart, and merging the sets into one accessor destroys exactly that distinction.
- Decide explicitly whether the artifact's `CompilationEnvironment` is a lowering-only subject or a whole-provider-environment subject, and record the answer where the type is defined rather than in this ticket. That choice, not the accessor, is what the artifact identity consequence turns on.
- Any new accessor is a public boundary and goes to Tom under ADR 0075 rather than being self-accepted.

## Closes when

The accessor's documentation and its behaviour agree; the `CompilationEnvironment` subject question has a recorded answer at the type; a negative control demonstrates the chosen behaviour failing when perturbed; and targeted nextest plus per-package Clippy pass.
