---
id: encode-identity-bearing-staged-combine-structure
title: Encode identity-bearing staged combine structure
status: todo
priority: p1
dependencies: [derive-staged-combine-structure-from-program-scope]
related: [accept-the-exact-composed-reference-session-and-event-surface]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [identity, schema, ir, scheduling]
---
## User-visible outcome

A kernel's staged intra-workgroup combine structure is representable in program scope as an identity-bearing value, so a plan witness can be derived for a staged kernel rather than refused — which is what ADR 0112's witness shape needs before it can answer the composed-reference comparison.

## Why this exists

Filed 2026-08-22 by the coordinator as the named prerequisite of candidate 5 in the composed-reference session packet. **This ticket exists so that candidate is not chosen with its prerequisite treated as implementation detail** — AGENTS.md's readiness gate requires missing prerequisites to be split into the graph rather than absorbed.

**Fact — the refusal and its stated remedy are both in source.** `crates/tiler-ir/src/program/contraction_witness.rs` refuses a kernel that at anchor `declares workgroup staging` carries a combine structure the witness cannot see, and its module header names the remedy at anchor `must become identity-bearing in`. Verified by the coordinator at `b3c07259`; both anchors resolve exactly once.

**Inference — this is a schema and identity change, not a local one.** Making a coordinate-dependent tree mapping identity-bearing moves what schedule, kernel, and artifact encoding carry. Treat every identity domain, pin, and golden as in scope for the audit even if none ends up moving, and rederive rather than assume.

## Required work

- **Do not start until [`derive-staged-combine-structure-from-program-scope`](derive-staged-combine-structure-from-program-scope.md) reports.** If it finds the structure derivable, this ticket is closed rather than done, and that is the good outcome.
- Re-audit both Facts at your own base first and report a per-Fact verdict.
- Enumerate which identity domains step and which do not, with the derivation for each. State whether previously encodable bytes move.
- Perturb each new refusal separately, subject not assertion, with the quoted failure text.

## Non-goals

Choosing the composed-reference surface; changing the reference evaluator's contract; and any change that lets a producer supply the combine structure rather than deriving it from the verified program — that inverts the property the witness exists for.

## Closes when

A staged kernel yields a witness rather than a refusal, every identity consequence is derived and stated, and each new refusal has been watched failing on its own subject.
