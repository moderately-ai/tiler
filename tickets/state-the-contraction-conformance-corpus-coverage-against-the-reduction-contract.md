---
id: state-the-contraction-conformance-corpus-coverage-against-the-reduction-contract
title: State the contraction conformance corpus coverage against the reduction contract
status: in-progress
priority: p2
dependencies: [retain-contraction-conformance-evidence]
related: [reduction-semantics-contract]
scopes: [implementation/reference, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [testing, conformance, contraction, numerics]
claimed_from: todo
assignee: sol-contraction-coverage
lease_expires_at: 1786409092
---
## Current boundary

**Fact — the retained corpus is already a gate.** `crates/tiler-reference/tests/contraction_conformance.rs` transcribes eight retained spike cases and exact expectations into the ordinary reference test surface. It explicitly bounds the claim to “eight named exceptional cases, not a proof over the binary32 domain.” The conformance envelope separately pins the six retained workload-cell `direct` digests against the retained record on every host. Retained digest comparison declines only when hardware fields (device, gpu-family) differ, naming those fields; toolchain fields (architecture, os, offline-compiler, sdk) are announced and comparison proceeds; `xcode` is deliberately not compared. The ordinary gate routes one retained correctness cell; the four prefill cells remain under a cost-gated `#[ignore]` run.

**Fact — the broad reduction checklist is larger.** The governing research record's `Required adversarial tests` section is the ledger's subject set: its bullets, not this ticket's paraphrase. The inventory below is a non-exhaustive compression of that section. Every supported reducer/dtype/order cell is asked to cover axes and ranks (including duplicate/out-of-range/dynamic), empty domains and seeds (including seed-conversion), both signed-zero orders, subnormals, infinities, qNaN and sNaN in every position and several NaN payloads, three-element reassociation and permutation witnesses, physical tree families (serial/balanced/skewed/SIMD/threadgroup/contiguous multi-pass/noncontiguous lane/atomic-arrival), empty partials with masks/`has_value` and invalid replications, integer wrapping/saturating/checked/widening boundary vectors, f16/bf16 accumulate-and-finalize paths, scratch round-trips, determinism under claimed artifact/variant/target identity, and typed verifier refusals. The eight exact-bit contraction cases cover selected exceptional-value and order discriminators; they do not discharge that whole list.

## Work

Read the complete retained reference corpus, realization-conformance surface, and governing reduction record. Source ledger subjects from the research section's `Required adversarial tests` bullets (including the every-cell preamble and every bullet listed there), not only from the compression above. Add one source-adjacent or normative coverage ledger that names, without inference:

- which required adversarial subjects the eight reference cases exercise;
- which are instead exercised by another named ordinary test and exact anchor;
- which remain outside the admitted contraction/profile surface; and
- which are admitted but still uncovered.

Keep target-independent reference evidence separate from the Apple realization row. Do not add redundant copies of the eight cases or six digest cells merely to satisfy the ledger.

Placement stays open among the homes already in scope (`implementation/reference` source-adjacent to `contraction_conformance.rs`, or a normative section under `contracts/numerics`). If the ledger or its removal-sensitive check lands under `crates/tiler-conformance/**`, add `implementation/conformance` before editing; if it lands in `docs/research/numerics/reduction-semantics-and-legality.md`, add `research/numerics` before editing.

## Non-goals

No model-level tolerance, new reduction family, new topology, target-profile widening, identity change, or conformance claim beyond the evidence actually executed.

## Closes when

The retained contraction corpus has one current, source-backed coverage ledger against `Required adversarial tests`; every covered row names its executing test, every uncovered admitted row is explicit, and a deliberate removal of one ledger entry or named test relationship makes the coverage check fail with a diagnostic naming the missing subject.
