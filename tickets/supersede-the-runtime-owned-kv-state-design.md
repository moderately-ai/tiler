---
id: supersede-the-runtime-owned-kv-state-design
title: Supersede the runtime-owned KV-state design with generic invocation contracts
status: done
priority: p0
dependencies: [reconcile-the-roadmap-and-public-facades-with-the-consumer-neutral-mission]
related: [design-autoregressive-state-and-kv-cache, define-the-runtime-kv-state-boundary, establish-a-dynamic-kv-physical-layout-authority, complete-the-kv-ownership-supersession-sweep]
scopes: [contracts/foundation, contracts/integrations, contracts/navigation, research/runtime, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [architecture, runtime, consumer-neutral, supersession]
---
## User-visible outcome

The runtime executes an artifact invocation from explicit bindings and returns
explicit outputs. It does not own a KV cache, model cursor, layer ordinal, decode
generation, poisoned model state, or any other transformer-specific session object.
Consumers express repeated invocations and retained tensors by composing ordinary
program inputs and outputs.

## Evidence and disposition

- **Fact:** the accepted semantic graph is pure, finite, acyclic MIMO dataflow.
- **Fact:** no KV-specific public type is present in the merged crate facades at
  the ticket's creation base.
- **Fact:** `docs/research/runtime/autoregressive-state-and-kv-cache.md`,
  `docs/research/runtime/dynamic-kv-physical-layout.md`, and the L5 ticket chain
  assign workload-specific state ownership to the generic runtime.
- **Inference:** generic facts found by that work remain useful: explicit bindings,
  extent validation before routing commit, placement/context compatibility,
  asynchronous resource retention, atomic publication, and fail-closed execution.
  Their KV-named ownership conclusion does not.

Audit the complete research records, glossary entries, roadmap dependencies, and
ticket chain. Preserve measurements and generic contracts while clearly superseding
the runtime-owned KV abstraction. Recast valid workload cases as integration or
conformance tests over ordinary tensors. Close, supersede, split, or rewrite each
affected ticket so no scheduler path can recreate the rejected abstraction. Preserve
the unmerged `tkt/define-the-runtime-kv-state-boundary` draft as review evidence; do
not merge it and do not silently delete its rationale.

This ticket is documentation and graph maintenance only. It does not authorize code
changes or a new public runtime boundary.

## Closes when

Every KV/model-state ownership claim has an explicit disposition; retained generic
requirements have consumer-neutral owners and correct dependencies; workload tests
bind explicit tensors through the ordinary invocation boundary; and no ready ticket
can introduce a KV-, transformer-, model-, or decode-specific core/runtime type.

**Correction — 2026-08-10.** The residual ownership-sentence population that the first pass left outside correction markers was finished the same day under [`complete-the-kv-ownership-supersession-sweep`](complete-the-kv-ownership-supersession-sweep.md); this ticket's first pass alone did not empty that population.

## Outcome

**Documentation and graph supersession, 2026-08-04.** This ticket is documentation and graph maintenance only; it authorized no code change and no new public runtime boundary. Substantive close conditions are met when counting the same-day remainder sweep as part of the campaign (see below).

### Research records

- [`docs/research/runtime/autoregressive-state-and-kv-cache.md`](../docs/research/runtime/autoregressive-state-and-kv-cache.md) and [`docs/research/runtime/dynamic-kv-physical-layout.md`](../docs/research/runtime/dynamic-kv-physical-layout.md) received dated supersession sections under this ticket: runtime ownership of capacity, cursor, generation, pool banks, and poison is withdrawn; measurements and generic obligations are retained with consumer-neutral owners (semantic program, ADR 0051 adapter, ADR 0047 placement, shape-relation preflight, and related seams). Disposition remains `partially-adopted`.
- Architecture and vision consumer-ownership prose already states that the runtime executes one pure finite MIMO invocation from explicit bindings and does not own KV caches, decode cursors, or session state.

### Boundary tickets closed superseded

- [`define-the-runtime-kv-state-boundary`](define-the-runtime-kv-state-boundary.md) — `status: closed`, `closed_reason: superseded`; draft branch preserved unmerged as review evidence (`preserved/define-the-runtime-kv-state-boundary-5812ff6c` and `origin/tkt/define-the-runtime-kv-state-boundary`). Do not merge; do not silently delete rationale.
- [`define-the-model-execution-state-boundary`](define-the-model-execution-state-boundary.md) — `status: closed`, `closed_reason: superseded` (L6 twin of the runtime KV boundary).

### L5 delivery chain rewritten

Per the L5 record's delivery table correction under this ticket:

- Ticket 3 (`define-the-runtime-kv-state-boundary`) closed as superseded.
- Ticket 4 rewritten and renamed to [`bind-repeated-invocations-over-caller-retained-tensors`](bind-repeated-invocations-over-caller-retained-tensors.md); the further capability ticket [`scope-an-in-place-append-into-a-caller-retained-allocation`](scope-an-in-place-append-into-a-caller-retained-allocation.md) was likewise rewritten and renamed.
- Tickets 5–9 recast as consumer conformance over ordinary tensors: [`execute-the-stateful-prefill-path`](execute-the-stateful-prefill-path.md), [`execute-the-decode-step-path`](execute-the-decode-step-path.md), [`integrate-the-autoregressive-decode-loop`](integrate-the-autoregressive-decode-loop.md), [`test-the-autoregressive-state-failure-cases`](test-the-autoregressive-state-failure-cases.md), [`prove-the-c1-stateful-attention-vertical`](prove-the-c1-stateful-attention-vertical.md). Capacity/poison cases withdrawn with the runtime object. Implementation harnesses remain open by design of this documentation-only ticket.

### Residual ownership sweep (same day)

The first pass corrected ownership tables, record headers, and the ticket chain. Residual sentences outside correction markers — and the contraction structures 2/3 exclusion that still named a withdrawn L5 state model — were finished the same day under [`complete-the-kv-ownership-supersession-sweep`](complete-the-kv-ownership-supersession-sweep.md) (done, Outcome 2026-08-04): enumerated sites corrected with dated withdrawals, structures 2/3 exclusion lifted (dependency misattributed; operands are ordinary caller-bound tensors), ladder signposts completed. That remainder is the authority for residual corpus ownership disposition after this ticket's first pass.
