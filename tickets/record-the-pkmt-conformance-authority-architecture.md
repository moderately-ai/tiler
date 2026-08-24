---
id: record-the-pkmt-conformance-authority-architecture
title: Record the P+K+M+T conformance authority architecture
status: done
priority: p1
dependencies: [decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles]
related: [spike-a-red-yellow-first-full-conformance-suite, decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles]
scopes: []
shared_scopes: [project/tickets, contracts/decisions, contracts/navigation, contracts/numerics]
paths: []
tags: [decision, architecture, conformance-progress, conformance-authority, verification]
---
# Record the P+K+M+T conformance authority architecture

## Goal

An accepted ADR and aligned catalogs/contracts preserving Tom's 2026-08-24 `P+K+M+T` decision, its five authority classes, fail-closed bootstrap boundary, change policy, staged dependencies, terminal trust, and reversal triggers without reopening the selected product.

## Work

1. Re-audit the decision carrier and governing ADRs at the exact carrier base.
2. Copy the carrier's accepted-decision, singular-authority, change-policy, bootstrap, and counterargument sections without semantic edits into one new ADR; allocate the next ADR identity from the live catalog.
3. Record acceptance provenance: Tom, 2026-08-24, coordination conversation, with the decision carrier as the relay source.
4. Update the decisions catalog, correctness/testing contract, and any live design/status entry point that would otherwise describe protected review or signing as optional.
5. Make explicit that `P+K+M+T` governs authoritative qualification while partial deployments remain provisional, and that no runtime/kernel fast path consumes this authority.
6. Link every selected mechanism ticket and retain its dependency/stop boundary rather than embedding an unaccepted provider or schema in the ADR.

## Non-goals

- Do not choose providers, algorithms, thresholds, key holders, classifier infrastructure, witness topology, or retention service.
- Do not implement a schema, CLI, host rule, key, signature, log, or public API.
- Do not reinterpret `P+K+M+T` as a menu of optional long-term controls.

## Stop conditions

Stop if the carrier conflicts with an accepted ADR, if a proposed sentence expands a public boundary, or if recording the decision would silently resolve a mechanism ticket's provider/schema choice. Return the exact conflict instead of weakening the accepted target.

## Acceptance

- The ADR and catalogs identify all five authority classes and all four selected properties.
- Partial/bootstrap outputs are explicitly provisional and cannot establish authoritative progress or qualification.
- Change, tombstone, lineage, unavailability, terminal-trust, and reversal semantics match the carrier.
- `tkt lint`, `make citations`, and the ticket scope guard pass.

## Exact-base Fact audit

**Measurement — 2026-08-24.** Re-audited on clean claimed branch base `344e25003a2b20cee7d9ea743598bc691f708adc`, with `main` and `origin/main` at `0 0` immediately before claim creation. The accepted carrier and every governing source used below were read on that base.

| premise | verdict | exact-base evidence and consequence |
| --- | --- | --- |
| ADR 0114 is the next live decision identity. | **Verified.** | `docs/decisions/README.md` ends at ADR 0113, `rg 'ADR[- ]0114|0114-' tickets docs` found no competing allocation before this edit, and `rg --files docs/decisions` found no `0114` path. |
| Recording `P+K+M+T` would conflict with the conformance crate's accepted authority boundary. | **False.** | [ADR 0106](../docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md), anchors `Not a second semantic authority` and `No support-matrix authority`, already makes the crate an evidence consumer. ADR 0114 reinforces that boundary by keeping all five authorities with their real owners. |
| The accepted carrier chooses concrete providers, schemas, thresholds, key holders, classifier infrastructure, witness topology, or retention services. | **False.** | The carrier's anchor `Concrete provider, threshold, key custody` explicitly leaves those as bounded design and operations work. The ADR preserves that stop boundary and introduces no public API. |
| Another live design/status entry point describes protected review or signing as an optional final state. | **False.** | Searches for `protected review`, `signed profile`, `P+K+M+T`, and optional-signing spellings across `docs/status.md`, `docs/design-map.md`, `docs/open-questions.md`, `docs/correctness-and-testing.md`, and `docs/decisions/` found no competing live statement. The correctness contract was the only governing entry point missing the accepted architecture, so it is aligned here without speculative status edits. |

## Outcome

- Added accepted [ADR 0114](../docs/decisions/0114-require-protected-signed-separated-and-witnessed-conformance-authority.md) and both hand-maintained catalog entries.
- Preserved all four selected properties, all five singular authority classes, the exact-source `P+K` join, `M` and `T` bindings, tombstone and lineage policy, fail-closed unavailability, provisional bootstrap boundary, rollout order, terminal-trust requirement, and reversal evidence.
- Linked every selected design/establishment pair and kept the Tom-only movement gate explicit. No mechanism ticket's provider or schema question was resolved here.
- Aligned [Correctness and testing](../docs/correctness-and-testing.md) so partial deployments can report honest audits but cannot claim authoritative progress or qualification, and so runtime and kernel paths never consume governance authority.

## Refs

- [`decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles`](decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md)
- [`cost-protected-review-versus-signed-conformance-authority`](cost-protected-review-versus-signed-conformance-authority.md)
- [`docs/correctness-and-testing.md`](../docs/correctness-and-testing.md)
