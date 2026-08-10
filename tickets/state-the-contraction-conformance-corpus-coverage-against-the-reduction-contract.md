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

## Source-first Fact audit — 2026-08-10

1. **Verified — the retained corpus is already a gate.** The complete `crates/tiler-reference/tests/contraction_conformance.rs`, at the source anchor `eight named exceptional cases, not a proof over the binary32 domain`, contains exactly eight retained exact-bit tests. The complete `crates/tiler-conformance/src/retained_record.rs`, at `direct_digests` and `The device and its GPU family are refused`, reads and checks all six `direct` rows, treats device and GPU family as declining hardware differences, announces architecture, OS, offline compiler, and SDK differences while comparing, and deliberately does not compare `xcode` because `MeasurementBoundary` does not observe it. `crates/tiler-conformance/src/envelope/tests.rs`, at `the gate routes one cell carrying a retained measurement`, routes one measured cell in the ordinary gate and retains the four prefill cells under the cost-gated `the_prefill_cells_carry_their_retained_digests`. The complete `crates/tiler-compiler/src/governed/contraction_conformance.rs`, at `The two comparisons have different reaches`, is a separate host/reference comparison: two unstaged cells, one emitted-region cell, and one staged prefill correction rather than the live-device envelope.
2. **Verified — the broad reduction checklist is larger.** The complete governing record, at `Required adversarial tests`, contains the every-supported-cell preamble and all twelve bullets this ticket names. The complete `crates/tiler-ir/src/semantic/contraction.rs`, at `What is admitted today` and `strict_tensor_contraction_f32_facts`, admits a static binary F32, unseeded, nonempty contracted-domain, strict ascending-lexicographic fold with no reassociation or permutation. The complete `crates/tiler-reference/src/contraction.rs`, at `Why the contract is decoded rather than restated`, decodes and enforces all fourteen facts. The eight vectors therefore cannot discharge universal subjects such as every rank, every NaN position, or a three-element witness.

No Fact was false, and the ticket's purpose, numerical authority, evidence classes, and public boundary did not move.

## Outcome

`REDUCTION_CONTRACT_LEDGER` beside the retained corpus now derives 59 atomic subjects from the governing preamble and bullets and gives each exactly one classification:

- 5 are exercised by a directly linked and re-executed retained exact-bit case: last contracted axis, no seed, subnormals, infinities, and the serial tree. Each relationship states its exact extent; in particular, the subnormal row is one positive product and the infinity row is positive infinity times positive zero in the first position, with no other sign, boundary, or position claimed;
- 3 are exercised by checked source anchors to other ordinary target-independent tests: positive/negative coverage of the sole admitted reducer/dtype/order cell, zero reduced extent, and zero surviving extent;
- 31 are outside this registered strict-F32 contraction profile; and
- 20 are admitted but uncovered: every admitted positive rank; first, middle, multiple, and all contracted axes; singleton positive and negative zero; the two mixed-zero orders; qNaN and sNaN in every contributor position; several NaN payloads; three-element reassociation and permutation witnesses; the separately listed skewed-tree family; repeated execution under exact plan identity; and verifier refusals naming missing permission, algebraic capability, target capability, or nonempty proof. A canonical serial left fold is itself maximally skewed, but the governing bullet lists serial and skewed separately and defines no distinct skewed family, so the ledger refuses to infer that one serial case covers both rows.

The target-independent source vocabulary can cite only the contraction reference unit tests and semantic contraction tests. It cannot represent the compiler's selected host/reference workload cells, the six retained device digests, or an Apple live-device envelope as reference coverage. The plan-identity row therefore remains admitted-uncovered even though separate realization evidence exists.

The typed subject enumeration is sized by `core::mem::variant_count`, while the ledger stays a slice so a removed entry reaches the runtime census and names the omitted subject. The check also executes each exact-bit function relationship and requires every ordinary relationship to remain an actual named `#[test]` in its included source.

The first full gate rejected a local `macro_rules!` constructor and its five invocations under the workspace unsafe-site census (`unpinned macro_rules! definition` / `custom macro invocation is unsupported in the workspace`). The final form uses a private `const fn` constructor instead; `the_workspace_unsafe_sites_are_exactly_the_four_admitted_ones` then passed without widening that census or its four admitted sites.

Subject perturbations demonstrated the check can say no, then were restored:

- renaming the ordinary positive relationship failed with `reduction-contract subject SupportedCellPositiveAndNegative names missing ordinary test the_governed_signature_decodes_to_the_unseeded_binary32_fold_renamed in crates/tiler-reference/src/contraction/tests.rs`;
- deleting the rank-zero ledger row failed with `reduction-contract subject RankZero has no ledger entry`; and
- relabelling repeated plan-identity execution as the target-independent execution witness failed with `reduction-contract subject RepeatedPlanIdentityExecution names the wrong exact-bit test relationship` (`left: None`, `right: Some("the_execution_witness_is_exactly_six")`).

No vectors, workload digests, operation facts, public types, identities, topology, tolerance, or target declarations changed.

## Closes when

The retained contraction corpus has one current, source-backed coverage ledger against `Required adversarial tests`; every covered row names its executing test, every uncovered admitted row is explicit, and a deliberate removal of one ledger entry or named test relationship makes the coverage check fail with a diagnostic naming the missing subject.
