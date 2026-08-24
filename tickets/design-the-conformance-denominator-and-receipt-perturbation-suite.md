---
id: design-the-conformance-denominator-and-receipt-perturbation-suite
title: Design the conformance denominator and receipt perturbation suite
status: todo
priority: p1
dependencies: [cost-protected-review-versus-signed-conformance-authority, design-the-conformance-audit-regress-and-qualify-command-contracts, assemble-the-first-versioned-conformance-goal-profile, design-the-machine-readable-and-explorable-conformance-report]
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, design, conformance-progress, testing]
---
# Design the conformance denominator and receipt perturbation suite

## Goal

A complete negative-control design showing that the conformance architecture can fail for each independent denominator, authority, receipt, context, and status-manipulation defect it claims to detect.

## Work

1. Derive the threat population from the accepted first profile, receipt model, command contracts, and machine report.
2. Specify independent subject perturbations for at least: registered feature addition; feature/profile deletion; missing tombstone; corpus shrinkage; duplicate stable identity; required-to-optional; required-to-`N/A`; wrong applicability authority; wrong oracle/reference authority; wrong selected plan; wrong schedule/KIR/program/artifact; receipt tampering; stale source/profile/environment; missing terminal completion; unavailable route; evidence-kind substitution; baseline replacement; and report omission.
3. Perturb each property separately. A mutation that reddens every check does not establish which verifier is load-bearing.
4. Name the command that must fail, stable failure class/code, exact subject printed, and population before/after.
5. Demonstrate why yellow-to-red discovery is retained without making audit fail for dishonesty, while green-to-red under an identical subject/context fails regression.
6. Separate repository-local controls from threats requiring protected or external authority.
7. Produce an implementation ordering that makes each new check fail before it is trusted.

## Non-goals

- Do not implement mutations by editing assertions.
- Do not weaken the subject, profile, oracle, or baseline to obtain convenient failures.
- Do not claim repository-local checks defend against an actor controlling every authority.

## Stop conditions

Stop and repair the owning model when a promised failure is unreachable, ambiguous, or only detectable by a hand-read log.

## Acceptance

- Every independent promise has a reachable subject perturbation and exact expected failure.
- Population sizes and searched spellings are explicit.
- Audit, regress, and qualify failures are distinguished.
- External-authority gaps are visible rather than papered over.

## Refs

- [`design-the-conformance-audit-regress-and-qualify-command-contracts`](design-the-conformance-audit-regress-and-qualify-command-contracts.md)
- [`cost-protected-review-versus-signed-conformance-authority`](cost-protected-review-versus-signed-conformance-authority.md)
- [`assemble-the-first-versioned-conformance-goal-profile`](assemble-the-first-versioned-conformance-goal-profile.md)
