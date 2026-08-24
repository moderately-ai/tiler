---
id: derive-the-optimizer-and-planner-capability-obligation-manifest
title: Derive the optimizer and planner capability-obligation manifest
status: todo
priority: p1
dependencies: [inventory-the-closed-world-conformance-claim-universe-by-owner, define-the-conformance-obligation-and-evidence-requirement-algebra]
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, spike, conformance-progress, optimizer]
---
# Derive the optimizer and planner capability-obligation manifest

## Goal

An exact owner-derived manifest and obligation taxonomy for Tiler's internal optimizer and planner capabilities, so progress includes rewrites, normalization, region/cover search, physical alternatives, feasibility, selection, explainability, schedule/KIR/program refinement, and cost claims rather than only end-user-visible operations.

## Work

1. Read the complete optimizer contract and all construction, registry, production, retention, selection, refusal, explain, verification, and public-session paths at the exact base.
2. Enumerate every declared rewrite, normalization rule, lowering family/provider, physical provider/strategy, region and cover behavior, feasibility rule, selection policy, budget disposition, schedule/KIR/program refinement, and measured-cost claim. Derive from typed registries/vocabularies where possible; mark missing enumerators explicitly.
3. Define family-specific obligations. For a rewrite/strategy consider: construction; production reachability; semantic/numerical authorization; adoption or typed decline; retention beside valid neighbours; selectability; selection under a subject perturbation; selected-plan identity; schedule/KIR/program receipts; independent semantic preservation; typed refusal; explain attribution; and measured cost only when cost is claimed.
4. Distinguish smoke, construction, reachability, retention, selection, preservation, execution, and cost evidence. `compile().is_ok()` and construction-only tests cannot satisfy later obligations.
5. Map existing tests and receipts to exact cells without hand-marking support. Missing or inaccessible cells remain explicit yellow candidates.
6. Identify the minimal observation-boundary work needed for owner-private rule inventories or typed explain records; do not solve it by parsing rendered explain text.
7. Give every claimed complete census a subject perturbation and every obligation family at least one negative control.
8. Produce an exact machine-readable candidate manifest and a human matrix under `spikes/verification/`, with no scalar completion authority.

## Non-goals

- Do not make optimizer internals public or add a universal optimizer plugin API.
- Do not execute, fix, or broaden unsupported optimizer capabilities.
- Do not replace the retiring host KIR simulators with another interpreter.
- Do not equate a test, strategy name, or frontier member with sufficient preservation evidence.

## Stop conditions

Stop and split a decision when a capability lacks stable identity/owner, when its preservation oracle is not independent, or when observing it requires a consequential public boundary.

## Acceptance

- Every declared internal optimizer/planner family is present or explicitly unenumerable with a follow-up.
- Obligations distinguish production reachability, retention, selection, preservation, refusal, explain, execution, and cost.
- Every mapped evidence cell names its exact source, authority, subject, and limits.
- Missing obligations are explicit and cannot disappear by denominator shrinkage.
- The result states which follow-ups are research-, decision-, or implementation-ready.

## Refs

- [`docs/compiler/optimizer.md`](../docs/compiler/optimizer.md)
- [`prototype-optimizer-conformance-gate`](prototype-optimizer-conformance-gate.md)
- [`replace-host-kir-simulator-claims-with-authoritative-evidence`](replace-host-kir-simulator-claims-with-authoritative-evidence.md)
- [`delete-the-two-host-kir-simulators`](delete-the-two-host-kir-simulators.md)
