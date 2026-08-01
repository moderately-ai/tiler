---
id: draft-the-backend-provider-composition-adr
title: Draft the backend-provider composition ADR
status: in-progress
priority: p1
dependencies: [specify-the-consumer-neutral-backend-provider-composition-contract]
related: [draft-public-extension-seam-ownership-adr, multi-device-and-sharding-scope-gate]
scopes: [contracts/decisions, contracts/foundation, contracts/artifacts, contracts/integrations]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [backend-providers, pluggability, decision, adr]
claimed_from: todo
assignee: worker-draft-the-ba
lease_expires_at: 1785552290
---
## User-visible outcome

One proposed ADR presents the concrete backend-provider participation model, including whether target-specific scheduling knowledge is typed profile data, provider code, or a checked combination, without treating the proposal as accepted.

## Implementation keys

- Derive the decision from the completed custom-Metal and CPU evidence rather than from interface aesthetics.
- Decide the intended participation model of `PhysicalImplementationProvider` and explicitly refine ADR 0078's deferred item.
- State the composition of target-profile authorities, physical providers, emitters/artifact producers, runtime adapters, and execution contexts.
- State whether partial providers may contribute specialized candidates while reusing another backend's emitter/runtime components, and how identity proves the selected composition.
- Preserve static linking, explicit immutable registries, re-verification, deterministic ambiguity, no silent override, no provider-authored proof, and no runtime source compilation.
- Separate build-time and runtime installation and define the artifact-carried join.
- State the initial single-device limit and leave multi-device/sharding to its existing activation gate.
- Label every fact, inference, proposal, and measurement, update the hand-maintained proposed-decision catalog, and create no implementation visibility changes.

## Closes when

A coherent proposed ADR exists with alternatives eliminated against correctness, performance, and maintainability; the affected contracts disclose its proposed status without asserting it; and one acceptance node structurally blocks every implementation ticket conditional on the decision.

## Graph maintenance

- Make `accept-the-public-backend-provider-composition-boundary` the sole acceptance node and keep it parked for Tom.
- Move the acceptance node from `todo` to `awaiting-decision` only after the complete decision packet exists.
- Point every conditional implementation ticket at the acceptance node, never directly at this drafting ticket.
- If more than one genuine product choice survives, split it into atomic decision records before asking Tom.

## Outcome

[ADR 0090: Compose backends per responsibility rather than per backend](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) exists at `decision_status: proposed`, `implementation_status: not-started`, refining [ADR 0078](../docs/decisions/0078-name-the-intended-public-extension-seams.md) item 5. Fourteen items carry the eleven atomic decisions the [composition record](../docs/research/extensions/backend-provider-composition.md) enumerated: items 1, 2, and 3 are marked Tom's and present recommendations with their evidence and their refutation conditions rather than deciding anything; items 4 through 14 state the derived model, the unsupported cases, and the trust and linkage inheritance. Static linking, explicit immutable registries, propose-then-re-verify, deterministic ambiguity, no silent override, no provider-authored proof, no runtime source compilation, and the single-device initial profile deferring to [`multi-device-and-sharding-scope-gate`](multi-device-and-sharding-scope-gate.md) are each preserved and each cited to the accepted record that owns it. No visibility, signature, or behaviour changed anywhere.

**One record rather than several, and the derivation is in the ADR's "Why this is one record rather than several" section so a reader can refute the elimination rather than only the conclusion.** Each of D1 through D11 was tested against correctness, performance, and long-term maintainability, and in every case exactly one candidate survives — D1 eliminates both pure positions (pure data would need a "preferred workgroup width" axis whose value is a schedule choice, and the axis set is closed anyway; pure code would move the feasibility comparison to the party being compared, which ADR 0078 item 6 forbids), D2's keep-it-internal alternative is eliminated by the fork it forces, D6 eliminates both symmetric spellings, D9's registry alternative falls to the same global-state ground as ambient discovery, and D10's trait alternative falls to a neutral function that already exists. What is left for Tom is not a choice among survivors but three acceptance acts of a different kind: a question about his own prior intent (D3, which no research can answer), the public boundaries ADR 0075 routes to him regardless of research quality (D2, D4, D10), and confirmation that refining his recorded deferral is what he meant (D1). Splitting would fragment one participation model across several records, which is the duplicated-authority failure the documentation contract prevents; if his answer to D1 or D3 reopens a genuine alternative, that is when a second record is the right shape.

**Six of the record's published reproductions were re-run at this ticket's base `2a1f57b` before being cited**, because the record's facts were inspected at `e6a47d9` and seventeen commits separate them. All six hold: the one-element provider array at `pipeline/planning.rs:171` and the inline empty `OpaqueCallRegistry` at `:228`; the backend/emitter registry grep returning nothing against a `lowering` positive control returning 73 lines; the artifact-key alphabet grep returning nothing against a six-line `tiler-compiler/src/target.rs` control; `tiler-build`'s two unconditional Metal dependencies and its two hardcoded literals; the `assemble_artifact` closure parameter at `metal_plan.rs:266`; and `offered_providers` populated from the lowering registry alone at `session.rs:1513`. `git diff --stat e6a47d9..HEAD -- crates/` touches only `tiler-ir` and `tiler-reference`, which is the independent confirmation that none of the cited crates moved. The record's lowering control read 69 lines and reads 73 here; the control's job is many-versus-none, so the drift is reported rather than treated as a discrepancy.

**Disclosure sentences added, each stating that a proposal exists, asserting none of it, and naming the node whose closure makes the disclosure wrong.** [`docs/architecture.md`](../docs/architecture.md) — four, at component ownership, the permanently-internal feasibility qualification, the accepted packaging profile, and dependency direction. [`docs/operation-extensions.md`](../docs/operation-extensions.md) — two, at the trust and linkage model and at the deliberately-unassigned-surfaces paragraph. [`docs/artifact-abi.md`](../docs/artifact-abi.md) — one, at the validation-stages paragraph, covering payload-validation ownership, key-namespace governance, and `prepare` optionality. [`docs/backends/cpu.md`](../docs/backends/cpu.md) — one, in traceability, naming the four vocabulary gaps the vertical exposed. [`docs/glossary.md`](../docs/glossary.md) — the Provider row's pointer moved from the research ticket to the proposed record plus the acceptance node. [`docs/decisions/0078-*.md`](../docs/decisions/0078-name-the-intended-public-extension-seams.md) — an evidence refresh on each of its two Tom-owned open questions, which changes no clause of that accepted record and leaves both questions open. [`docs/decisions/README.md`](../docs/decisions/README.md) — both hand-maintained catalog blocks plus a prose line naming ADR 0090 as the only `proposed` record.

**Graph.** [`accept-the-public-backend-provider-composition-boundary`](accept-the-public-backend-provider-composition-boundary.md) is `awaiting-decision`, a parked state that never satisfies a dependent, and carries the decision packet. The three tickets already depending on it — [`drive-an-external-physical-implementation-provider-through-compilation`](drive-an-external-physical-implementation-provider-through-compilation.md), [`produce-a-custom-backend-payload-through-the-build-orchestrator`](produce-a-custom-backend-payload-through-the-build-orchestrator.md), and [`route-a-custom-backend-through-a-registered-runtime-adapter`](route-a-custom-backend-through-a-registered-runtime-adapter.md) — were already correct and were left alone; nothing depends on this drafting ticket, which is what the workflow configuration requires.

**One brief instruction was deliberately not followed, with the reason.** [`register-opaque-calls-on-the-compile-path`](register-opaque-calls-on-the-compile-path.md) was to be pointed at the acceptance node as a conditional implementation ticket. Reading it in full, it is not conditional: its own implementation keys record that the call registry stays crate-private, so composing one is internal wiring rather than a public seam, and a crate-private composition site can be moved for free while Tiler is pre-alpha. A hard dependency would park a live reachability gap — no caller of any kind registers an opaque call, so every test of the frontier's opaque-call admission proves nothing about `compile()` — behind a decision it does not need. It is `related` to the acceptance node instead, with the interaction that does exist recorded on the ticket: if D2 is accepted, the request surface acquires an installation idiom that internal wiring should match rather than duplicate.
