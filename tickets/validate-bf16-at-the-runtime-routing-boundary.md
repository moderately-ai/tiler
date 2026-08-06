---
id: validate-bf16-at-the-runtime-routing-boundary
title: Validate BF16 at the runtime routing boundary before the one-way commit
status: done
priority: p1
dependencies: [carry-bf16-through-the-artifact-encoding-and-identity, lower-bf16-to-metal, admit-a-bf16-index-realization-law-and-refinement-contract]
related: [spike-bf16-through-the-second-dtype-seams, decide-per-dtype-dispatchability-as-a-target-capability, declare-host-dtype-dispatchability-at-the-consumer-boundary, move-the-runtime-semantic-validation-cells-for-f32-and-bf16]
scopes: [implementation/runtime, implementation/frontend, implementation/candle, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, runtime, routing, fail-closed]
---
## User-visible outcome

The runtime refuses a BF16 program whose bound values are not BF16, and refuses a BF16 program on a host whose target family cannot dispatch the dtype — both **before** the one-way routing commit. A caller binding an `f32` buffer to a BF16 input gets a typed refusal instead of a plausible tensor computed from misread bytes.

## Why the phase ordering is the whole ticket

**Measurement.** Finding 26 of the [Apple numerical behaviour record](../docs/research/apple-targets/numerical-behaviour.md) records that the iOS Simulator compiles and links every `bfloat` module and *then* fails pipeline creation. That failure occurs at `PreparedKernelPreflight` — one phase **after** the one-way routing commit of ADR 0051. `decide-per-dtype-dispatchability-as-a-target-capability` eliminated device preflight for exactly this reason and made dispatchability a profile-owned, family-keyed fact resolvable at the compile profile.

**Inference.** So the refusal has to come from the profile, before routing commits. [The BF16 spike](../spikes/numerics/bf16-second-dtype/README.md) showed the resolution mechanism produces `Dispatchable`, `Unsupported`, and `Unknown` at `AvailabilityPhase::CompileProfile`; this ticket is the runtime consuming it.

**Fact.** The bytes-per-element difference makes the binding check load-bearing rather than pedantic: BF16 is two bytes and `f32` is four, so a wrongly typed binding of the same buffer misreads element count and every value, and produces a full-looking result.

## Implementation keys

- Value binding validates the bound value's storage scalar against the program's declared one, refusing a mismatch by name. The existing `StorageScalarMismatch` refusal is the shape; it must reach BF16.
- Element count derives from the declared element width, so a two-byte element is not counted as four.
- A BF16 program offered a host whose profile resolves `Unsupported` or `Unknown` for `tiler::bf16@1` is refused at preflight, before the commit. `Unknown` refuses exactly as `Unsupported` does — the caller learns which, but both fail closed.
- No fallback after the commit. If a BF16 refusal is ever reachable after allocation, partial encoding, or submission, that is a defect in this ticket and not a case to handle.
- The refusal names the dtype and the family, so an explain record can say why.

## Required evidence

- An `f32` buffer bound to a BF16 input is refused, with the element-count consequence shown rather than asserted — a test that demonstrates the misread would have occurred.
- A BF16 program on a profile resolving `Unsupported` is refused at preflight; the refusal is shown to occur before the routing commit, by a check on the phase and not only on the message.
- The same on a profile resolving `Unknown`, and the two refusals are distinguishable.
- A BF16 program on the macOS profile routes and executes, so the refusals are not a blanket refusal.
- An `f32` program is unaffected on all three profiles.

## Closes when

Both refusals happen before the routing commit and are observed failing, the `Unknown` and `Unsupported` cases are distinguishable, a correctly bound BF16 program still routes on the dispatchable profile, and the `Runtime semantic validation` cell for BF16 moves.

## Graph maintenance

- Depends on the artifact carrying the dtype (there is nothing to validate against otherwise) and on lowering (there is nothing to route otherwise).
- `docs/dtype-support.md` records `Runtime semantic validation` as `absent/unsupported` for `f32` as well. This ticket does **not** discharge it for `f32`; if the mechanism it adds is dtype-neutral, say so and file the `f32` row separately rather than claiming a cell this ticket did not test.
- The pre-commit ordering is the property most likely to regress silently, because a refusal that moves one phase later still refuses and still looks green. Assert the phase, not just the outcome.

### Scopes added while working, and why each is required

`ExecutionEnvironment` is the record a loading host states about itself, and the dtype row belongs there for the reason the other three fields do — a separate argument would split one concept across two values and be skippable. It has public fields, so adding one breaks every struct literal in the workspace. Three scopes beyond `implementation/runtime` hold such a literal, each edited to add exactly the new field with the derivation of its value recorded in a comment beside it, and nothing else:

- `implementation/frontend` — `crates/tiler/src/route.rs::execution_environment`, one site.
- `implementation/candle` — `prototypes/candle-metal-adapter/src/proof.rs::declared_route_environment`, one site.
- `research/target-profiles` — `spikes/target-profiles/scalar-cpu-vertical/src/vertical.rs`, five sites. A nested spike is not a workspace member and no gate reaches it, but it takes `tiler-runtime` as a path dependency, so leaving it uncompilable would break retained evidence that only re-running the spike detects.

None expands the product outcome; each is the mechanical consequence of the already-authorized change. Verified free of any live claim on 2026-08-05: the two other in-progress tickets hold `implementation/ir`, `implementation/compiler`, and `implementation/reference`.

## Outcome

**The refusal that was missing was the *named* one, not the barrier.** A dispatchability verdict already participates in the compile profile's complete descriptor, and the runtime already required exact descriptor equality, so a host that states its own profile honestly could never route a BF16 artifact onto a family that refuses BF16. What made that barrier vacuous is that both consumer paths construct their `ExecutionEnvironment` by restating the *producer's* declaration — `tiler::route::execution_environment` reads the macro-emitted route facts, and the Candle prototype's `declared_route_environment` says of itself that it is "producer-declared equality, NOT host-earned eligibility". A tautological comparison refuses nothing, and even when it does refuse it reports `DescriptorMismatch`, which tells a reader to rebuild when no rebuild will help. So the dtype row is what a loader can still refuse on, and it can name which dtype and which family.

**Fact — what landed.** `ExecutionEnvironment` gains `dtype_dispatch: BTreeMap<ArithmeticType, DTypeDispatch>`; `ExecutionEnvironment::classify_dtype` resolves it into a three-valued `DTypeDispatchResolution` where an absent declaration is `Unknown`; and `DecodedProgram::variant_eligibility` resolves every entry of every packaged variant against it, adding `VariantIneligibility::UndispatchableDType { entry, arithmetic, resolution, host_profile }`. The dtype an entry computes in is read from the delivered-realization record's own entry-to-subject binding — the only place an artifact states which arithmetic governs an entry — through `ArithmeticType`, which `tiler_artifact::program` re-exports, so the crate's dependency closure stays `[tiler-artifact]` under ADR 0081.

**Fact — the phase.** The check sits inside `select_variant`, which both `preflight` and `prepare` call first, so it is discharged before route requirements, before deferred predicates, before payload validation, and before every device stage. Every route case asserts the adapter's stage log is exactly `[Bind]` and that `AdapterRouteFailure::fallback_permitted` is true, rather than asserting only the returned class — the ticket's own warning that a refusal moved one phase later still refuses and still looks green.

**Inference — a filter, not a terminal refusal.** An undispatchable variant is a non-candidate, so its guard is never evaluated and the producer's next-ranked plan is selected. A terminal refusal would make a portfolio packaging a fallback width unroutable, which is the defect `select-executable-variants-across-registered-backend-families` corrected for backend families; a test pins the fall-through.

**Fact — silence refuses, and that is the mechanism's whole blast radius.** `Unknown` fails closed exactly as `Unsupported` does, so a host that declares nothing routes nothing. That is ADR 0043's disposal of `Unknown` applied rather than amended, and it is why the three out-of-scope literals could not be filled in with an empty map. The compile profile's fourth resolution, `Deferred`, has no spelling in the runtime vocabulary on purpose: it is the answer that arrives after the commit, so a host holding one states nothing and is refused.

**Measurement — every refusal observed failing.** Five deliberate perturbations, each reverted:

| Perturbation | Tests that failed |
| --- | --- |
| `if !resolution.is_dispatchable()` → `if false` | 6 |
| `Unknown` made permissive in `is_dispatchable` | 5 (3 unit, 2 route) |
| flat portfolio base dropped (`.take(variant.routing_rank())` → `.take(0)`) | 1 |
| only the variant's first entry resolved | 1 |
| canonical stage-key order → execution order for the within-variant ordinal | **0** |

The last row is the honest gap and is recorded in `CanonicalEntryOrdinals`' own documentation rather than left for a reader to discover: no fixture in this workspace packages a variant whose execution order differs from its canonical stage-key order — the two-entry materialized member's coincide, checked directly — so reading the wrong permutation is caught by nothing. The code is written against the ordinal space `DecodedArtifact::delivered_realization` documents, which is a claim about the code and not a measured one.

**Measurement boundary — what the fixtures vary, and what they do not.** `FixtureEntry::arithmetic` varies the *recorded* arithmetic association, which is the whole of what this loader reads and which `validate_against_artifact` deliberately never cross-checks against a kernel — `tiler-artifact`'s own fixture documents that as the reason the subject is a producer parameter there too. The carried payload stays this suite's `f32` scalar image. **No case here claims BF16 executes**, and `docs/dtype-support.md` records BF16 backend execution as absent; a fixture implying otherwise would contradict it. The three families are named after the measured Apple rows the BF16 spike declares (macOS dispatches `bfloat`, the iOS Simulator refuses it at pipeline creation) and restate those verdicts for the fixture's own scalar-host profile — nothing here measures an Apple device.

### What this ticket did not discharge

- **The binding refusal (implementation key 1) was already discharged elsewhere and is dtype-neutral.** `crates/tiler/src/expansion.rs::bind_region` compares `reported.storage_scalar() != declared.storage_scalar` totally over the `StorageScalar` vocabulary and raises `BindError::StorageScalarMismatch`, so it reaches BF16 by construction; `crates/tiler/src/value.rs::dense_bytes` derives element count through `StorageScalar::byte_width`, so a two-byte element is never counted as four. Below it, `crates/tiler-artifact/src/program/codec/validate.rs:361` refuses at decode any binding whose storage scalar disagrees with the component it addresses. Restating either in `tiler-runtime` would make this crate a second authority over the artifact, which its own module documentation forbids — and it could not carry `StorageScalar` in a public refusal anyway, because `tiler_artifact::program` does not re-export it and this crate has no `tiler-ir` edge.
- **`docs/dtype-support.md` is `contracts/navigation` and was not edited.** The `Runtime semantic validation` cell for BF16 is supported by this work and should move to a tested guarantee bounded to *refusal at the routing boundary*, not to execution. The mechanism is dtype-neutral, so the `f32` cell is equally supported — filed separately as the ticket's own graph-maintenance note requires, rather than claimed here.
- **A host-earned dispatchability declaration does not exist.** All three out-of-scope literals now restate a producer declaration, which leaves the check tautological on those paths for the same reason the profile classification already was. `declare-host-dtype-dispatchability-at-the-consumer-boundary` carries the work of emitting the declared row into `RouteFacts` and deriving it from a bound device.
- **No BF16 kernel was dispatched.** Nothing in this branch touches a device.

## Routing surface — accepted

Accepted by Tom on 2026-08-06 at the morning decision review, witnessed first-hand by the coordinator: `ExecutionEnvironment.dtype_dispatch`, `DTypeDispatch`, `DTypeDispatchResolution` with silence refusing, `VariantIneligibility::UndispatchableDType`, and the deliberate non-spelling of `Deferred` at routing. Acceptance is not stabilization.
