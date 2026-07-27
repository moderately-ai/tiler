---
id: prototype-public-compiler-api
title: Expose caller-composed compilation requests and provider installation
status: done
priority: p0
dependencies: [prototype-optimizer-conformance-gate]
related: [report-per-target-compilation-outcomes]
scopes: [implementation/compiler, implementation/ir, contracts/optimizer, contracts/foundation, implementation/metal-aot]
shared_scopes: [project/tickets, contracts/decisions]
paths: []
tags: [implementation, compiler-api, dx]
---
The reviewed `tiler_compiler::session` facade and its opaque explain surface
have landed. `compile_governed` is the bounded convenience path, but an
external frontend still cannot construct the consumer-independent
`CompilationRequest` required by ADR 0069 or install a lowering-capability
authority through the public boundary.

## User-visible outcome

Let an external frontend state every semantically meaningful compilation input
that has more than one admitted value through one checked boundary:

- an ordered numerical-contract preference;
- target profiles;
- the shape environment and caller-known specialization inputs;
- resource or proof budgets and supported options; and
- installed lowering capabilities with their governed identities and
  revisions.

Preserve `compile_governed` as the simple bounded profile. Unsupported
combinations fail with typed diagnostics and a complete explain trace whenever
the trace boundary was reached.

## Boundaries

- Do not expose private strategy choices, temporary cardinality assumptions, or
  compiler-internal arenas merely because the current implementation uses them.
- Provider installation must preserve validation, deterministic resolution,
  versioned identity, and fail-closed ambiguity.
- Public request identity must cover every caller choice that can change
  semantics, feasibility, selected implementation, or produced bytes.
- Per-target outcomes are owned separately by
  `report-per-target-compilation-outcomes`.

## Public review

Tom already accepted the existing `session` facade. The exact request builder,
provider-installation call site, and the reshaped `CompileFailure` signature
remain consequential public boundaries and require review before this ticket
closes.

## Closes when

An out-of-crate frontend can construct and compile the admitted request profile,
install an external provider without an in-crate test hook, receive typed
failure plus complete explain evidence, and use the governed convenience path
without assembling the full request. The public contract and implementation
agree, and `make full` passes.

## Outcome

Done. The half that remained is closed: an out-of-crate caller can now install its own lowering-capability registry into a compilation.

**Half of this ticket had already landed, which reshaped it.** ADR 0078 item 4 records that `session::compile_governed` was approved on 2026-07-25 under this ticket, and states the consequence — "the facade landed … and installation is still not reachable, because that promotion exposed the entry point and deliberately not the request." That same item fixes the closing condition: *a public path lets an out-of-crate caller supply its own `FrozenLoweringCapabilityRegistry` to a compilation.* This change is that path.

**What is public now.** `session::InstalledCapabilities` — `governed()` for the shipped set, `installed(lowering, scalars)` for a caller's own. `session::CompileRequest`, built and consumed. `session::compile`, taking it by value. `capability::install_governed_index_access`, which registers the shipped families onto a caller's builder except those it names — the affordance `GovernedIndexAccess`'s own documentation described and kept crate-private, so substituting one family no longer means re-implementing the other three.

**The registry and its scalar authority are taken together**, because they are only meaningful as a pair: every resolved provider emits against, and is revalidated under, that scalar snapshot. The request boundary already refused a mismatched pair and still does.

**`compile_governed` is now a caller of the general path** rather than a second path beside it. That is also the cheapest proof the general surface is usable: a convenience wrapper that could not be expressed through it would mean the surface was wrong.

**The surface is deliberately smaller than the internal request.** Promoting `CompilationRequest` wholesale would have dragged the `honourability` vocabulary into the public API through `RequestError`'s transitive references, so an opaque wrapper carries what a caller can meaningfully choose and the request model stays private. Budgets and the shape environment admit exactly one governed value today; target-profile *declaration* is a validation job rather than a visibility change.

**Evidence, out-of-crate by construction.** `prototypes/serial-sum-compile` takes `tiler-compiler` as an ordinary dependency and sees only its public surface. `an_out_of_crate_caller_installs_its_own_capability_registry` composes a registry there and compiles through it; `an_installed_registry_missing_a_family_fails_closed` omits the multiply family and observes the refusal. The second exists because the first alone would pass against a `with_capabilities` that ignored its argument — confirmed by making it ignore its argument and watching only the second fail. The compiler's own conformance case for this composition still lives in `pipeline.rs`; what changed is that it no longer *has* to.

**ADR 0078 updated in the same change.** A decision recorded is not a decision applied: item 4's "left as it is" proposal was superseded rather than deleted, its trigger recorded as fired, the seam-inventory rung for `IndexAccessLoweringProvider` raised to include installation, and the record's own `implementation_status` sentence corrected — it named items 4 and 5 as holding the status short of implemented, and only item 5 does now.

## What this did not deliver

**Target-profile declaration.** A caller selects the governed profile and cannot author one; `verify_request` rejects any other profile twice over. `express-metal-honourability-in-the-shared-form` needs *declaration* to get a Metal honourability row into compiler feasibility, so the dependency edge I added from it to this ticket earlier today is directionally right and not sufficient on its own. Recorded on that ticket.

**ADR 0069's fifth failure class.** `CompileFailureClass` has four variants and ADR 0069 requires five; `InvalidRequest` and `UnsupportedCapability` are merged into `Unsupported { rule }`. That merge is deliberate and argued at `class_of`, and the argument is sound about information and wrong about what a class is for — split out as `distinguish-the-five-compile-failure-classes` rather than half-done here.

**`ScalarLoweringProvider` installation** is reachable by the same path and still has no compile-path caller, so nothing new is claimed for it.

Gate: `make full` green (973 nextest + 11 doc-tests, rustdoc, release numerical tests, `tkt lint`, shellcheck). Artifact identity for the serial-Sum program is unchanged — the producer's two-process determinism test passes.
