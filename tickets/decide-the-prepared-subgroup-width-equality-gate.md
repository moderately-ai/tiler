---
id: decide-the-prepared-subgroup-width-equality-gate
title: Decide the prepared subgroup-width equality gate
status: done
priority: p1
dependencies: [accept-adr-0094-subgroup-execution-tier]
related: [admit-an-atomic-subgroup-realization-subject-to-target-profiles, make-prepared-entry-observations-typed-and-key-dispatched, generalize-deferred-target-provenance-beyond-capability-axes, bind-prepared-pipeline-caches-to-loader-derived-route-identity, carry-subgroup-width-through-exact-prepared-entry-equality, declare-metal-subgroup-realization-facts-in-the-target-profile, measure-metal-thread-execution-width-across-prepared-pipelines]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/runtime, implementation/metal, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [subgroup, metal, preflight, routing, feasibility, public-boundary, decision, needs-tom]
---
## User-visible outcome

A subgroup route commits only after the prepared pipeline's actual execution width exactly equals the width whose tree was verified. A missing observation or mismatch refuses that attempt before routing commit. The gate never substitutes another width or backend; ADR 0051 still permits the caller to begin a separate explicit attempt after this precommit refusal.

## Fact

ADR 0094 public-boundary item 6 requires a `PreparedKernelPreflight` comparison of Metal `threadExecutionWidth`. The value belongs to `MTLComputePipelineState`, not the compile profile or live device. Existing prepared-entry requirements can express exact observed-versus-required comparison, while `RouteResourceDimension::SubgroupThreads` already means width equality and Metal correctly cannot answer it at device preflight.

## Decision questions

- Whether the verified program emits a dedicated typed prepared-entry subgroup requirement or reuses the generic route-resource dimension without losing phase/subject information.
- Which component reads `threadExecutionWidth`, constructs the observation, and proves it corresponds to the exact prepared payload/pipeline being routed.
- How the one-way routing commit orders preparation, comparison, publication/cache insertion, and any alternative selection.
- Which canonical subject binds required width, prepared payload identity, observation authority, and refusal without folding live observations into reusable artifact identity.

## Strict boundary

No compile-profile row alone can discharge this gate. No device-wide constant, cached observation from another pipeline, default width, “at least” comparison, pipeline rebuild, or post-commit fallback is admissible.

## Closes when

Tom accepts one exact preflight carrier and ordering, with complete identity/accounting and typed missing/mismatch failures demonstrated against the real prepared-entry path.

## Source-first corrections — 2026-08-11

The existing artifact carrier is sufficient, but the surrounding producer and observer paths are not. `PreparedEntryTargetRequirement` already encodes the exact entry's query, required quantity, and `ObservedEqualsRequired`. The compiler's deferred provenance is nevertheless hard-wired to `CapabilityAxis`, while subgroup width is derived from the accepted atomic `SubgroupRealizationSubject` and must not become an independently satisfiable capability row.

The runtime observer is also not strict enough for a second property. `RuntimeAdapter::observe_prepared_entry` returns a bare `u64`, and the current Candle Metal adapter ignores the requested key/provider and answers every query with `maxTotalThreadsPerThreadgroup`. A subgroup-width row could therefore be answered by the wrong property and even accidentally satisfy equality when the numbers coincide. Zero and `u64::MAX` sentinels do not repair this because they cannot distinguish an unknown property from an observed quantity.

`RouteResourceDimension::SubgroupThreads` remains the correct live-device vocabulary for targets such as CUDA or Vulkan that publish a device-scoped width. Metal correctly reports it `Unrecognized`; carrying that row on a Metal route would refuse before the prepared pipeline exists and make the correct query unreachable.

## Accepted decision — 2026-08-11

Tom accepted the existing generic exact-entry prepared requirement with a dedicated governed subgroup-width property key, plus the prerequisite strictness repairs:

- A profile declaring any `SubgroupRealizationSubject` as `Realized` must also carry exactly one profile-level `PreparedKernelPreflight` subgroup-width query. Silence is `Unknown`; `Unrealizable` requires no positive query. No query is inferred from backend family.
- Compiler deferred provenance becomes a typed subject capable of naming either an ordinary quantitative capability axis or the subgroup-width confirmation derived from the complete atomic subgroup subject. It lowers to the existing `PreparedEntryTargetRequirement`; no independent `CapabilityAxis::SubgroupThreads` is admitted.
- Every exact entry using a subgroup tree receives one `ObservedEqualsRequired` requirement whose required value is the schedule/profile width. Multiple shuffles in one entry do not duplicate it; different entries are never deduplicated because their prepared pipelines may differ.
- Prepared observations become an exhaustive `Quantity(u64)` / `Unrecognized` vocabulary. The adapter exact-matches provider identity and property key before reading that same retained pipeline's `threadExecutionWidth`. Unknown and mismatched observations are distinct typed precommit refusals; no sentinel or default exists.
- The order remains payload validation, live-device checks, preparation of every exact entry, observation and loader-owned equality comparison, dispatch planning, consuming infallible commit, allocation, and encoding. Nothing rebuilds or substitutes a pipeline after observation.
- The route/cache boundary must derive the prepared cache subject from loader-authenticated artifact/entry identity rather than a caller-vouched byte string. A cached prepared object may be retained before equality only when it is the exact device/artifact/entry/pipeline object; a satisfaction verdict is never cached.
- Existing requirement grammar already carries entry, key, phase, provider, required value, and relation, so no new artifact row or broad schema/domain step is justified. The new row changes affected artifact/cache identities naturally. Because legacy adapters parse but misanswer unknown keys, subgroup-bearing artifacts require an explicit derived required-feature fence unless lockstep source compatibility is proved to make old readers unreachable.

Delivery is split into the four linked prerequisite/implementation tickets above. This decision is complete; the subgroup route remains unavailable until those tickets and the retained Metal measurement finish.
