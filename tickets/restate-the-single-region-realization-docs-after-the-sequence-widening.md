---
id: restate-the-single-region-realization-docs-after-the-sequence-widening
title: Restate the single-region realization docs after the sequence widening
status: todo
priority: p2
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: []
paths: []
tags: []
---
## User-visible outcome

The index-law and refinement docs describe the staged-realization model as landed, so a reader stops treating single-region accessors and claims as total.

## Why this exists (drift audit 2026-08-06 — one coherent story from the staged-law landing, wanting one reader who holds the whole sequence model)

The cluster, each verified by the audit at source: refinement.rs:1186/1324 claims "the one-stage realization every registered law produces" (the staged law registers in this file's own tests); law.rs:1933 and refinement.rs:4655 claim the single-region `verify` path is "the one the compiler drives today" (the compiler drives `verify_sequence`; the only `.verify(` sites in tiler-compiler are cfg(test)); legality.rs:38-41's module header states unconditional oracle-evaluability that `single_region()`'s own doc refutes for chains; oracle.rs:426 counts two uninstalled scalar capabilities where the registry has five; law.rs's module header describes region-identity comparison where the authority compares sequence identities; law.rs:270 promises a pre-interface refusal only `verify` delivers; refinement.rs:72-79's MAX_OPERAND_BINDINGS rationale predates per-stage bindings and no longer explains the constant's safety; refinement.rs:1478's # Errors omits the first refusal its body returns.

## Closes when

Every listed claim states sequence-model truth, the # Errors lists match their bodies, and the bindings-constant rationale derives the per-stage population.
