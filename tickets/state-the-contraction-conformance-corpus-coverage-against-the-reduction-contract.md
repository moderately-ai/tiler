---
id: state-the-contraction-conformance-corpus-coverage-against-the-reduction-contract
title: State the contraction conformance corpus coverage against the reduction contract
status: todo
priority: p2
dependencies: [retain-contraction-conformance-evidence]
related: [reduction-semantics-contract]
scopes: [implementation/reference, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [testing, conformance, contraction, numerics]
---
## Current boundary

**Fact — the retained corpus is already a gate.** `crates/tiler-reference/tests/contraction_conformance.rs` transcribes eight retained spike cases and exact expectations into the ordinary reference test surface. It explicitly bounds the claim to “eight named exceptional cases, not a proof over the binary32 domain.” The conformance envelope separately pins the six retained workload-cell `direct` digests, compares them only on a matching environment row, and declines the retained comparison by name otherwise.

**Fact — the broad reduction checklist is larger.** The governing research record's `Required adversarial tests` section asks every supported reducer/dtype/order cell to cover axes and ranks, empty domains and seeds, both signed-zero orders, subnormals, infinities, qNaN and sNaN in every position, three-element reassociation and permutation witnesses, physical tree families, partial-state and scratch behavior, determinism, and typed verifier refusals. The eight exact-bit contraction cases cover selected exceptional-value and order discriminators; they do not discharge that whole list.

## Work

Read the complete retained reference corpus, realization-conformance surface, and governing reduction record. Add one source-adjacent or normative coverage ledger that names, without inference:

- which required adversarial subjects the eight reference cases exercise;
- which are instead exercised by another named ordinary test and exact anchor;
- which remain outside the admitted contraction/profile surface; and
- which are admitted but still uncovered.

Keep target-independent reference evidence separate from the Apple realization row. Do not add redundant copies of the eight cases or six digest cells merely to satisfy the ledger.

## Non-goals

No model-level tolerance, new reduction family, new topology, target-profile widening, identity change, or conformance claim beyond the evidence actually executed.

## Closes when

The retained contraction corpus has one current, source-backed coverage ledger against `Required adversarial tests`; every covered row names its executing test, every uncovered admitted row is explicit, and a deliberate removal of one ledger entry or named test relationship makes the coverage check fail with a diagnostic naming the missing subject.
