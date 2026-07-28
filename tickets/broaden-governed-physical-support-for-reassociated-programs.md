---
id: broaden-governed-physical-support-for-reassociated-programs
title: Broaden governed physical support for reassociated programs
status: done
priority: p1
dependencies: [implement-first-algebraic-rewrite-portfolio]
related: []
scopes: [implementation/compiler, implementation/ir, implementation/artifact, implementation/metal, implementation/build, contracts/foundation, contracts/optimizer, contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, rewrites, lowering]
---
## User-visible outcome

A reassociation accepted by the first algebraic portfolio can complete the ordinary governed compile path and participate in verified global selection, rather than stopping at the physical profile's existing serial-sum recognizer boundary.

## Exact blocker

`request::normalize_serial_sum` admits exactly one input and one output over four or five operations shaped as `strict-serial-sum(add(multiply(input, scale), bias))`; `RECOGNIZED_OPERATIONS_MIN/MAX` and the producer walk reject a three-leaf add-only or multiply-only chain before region planning. The later governed physical and program builders read `request.serial_sum()` and construct only the scale/bias pointwise stage plus strict reduction, so relaxing the operation-count check alone would misdescribe members, buffers, schedules, and program coverage. This is why the accepted algebraic rules are live and independently readmitted but cannot yet furnish a physically compilable positive fixture.

## Implementation keys

- Generalize governed request recognition around verified semantic occurrences and interfaces needed by the accepted reassociated add and multiply programs; do not forge them into `NormalizedSerialSum` or add caller-declared ABI facts.
- Extend lowering, region-role/frontier recognition, schedule construction, and verified program assembly together so every retained physical alternative covers the exact reassociated semantic program and derives its ABI, buffers, stages, dependencies, and output coverage from that program.
- Preserve the existing serial-sum path and artifact identities unless the generalized derivation proves an identity change is required. Recompute every pinned identity on the merged tree rather than copying a branch value.
- Add an end-to-end relaxed-contract fixture that proves an accepted reassociation reaches physical planning, produces at least one verified program alternative under its own semantic/request owner, and appears in global semantic selection. Include the strict-contract negative control and a forged-owner or incomplete-coverage rejection.
- Prove each new recognizer and verifier check can fail by perturbing the fixture before accepting its green result.

## Scope boundary

This ticket owns governed physical recognition, lowering, scheduling, program assembly, the checked `tiler-ir` pointwise-expression schedule representation those stages require, and their tests under `implementation/compiler` and `implementation/ir`. It does not add algebraic rules, widen the eleven-dimension numerical vocabulary, change operation-owned ordered-associativity declarations, alter the exact ordered-result-set oracle, or redesign composite explain; those outcomes belong to `implement-first-algebraic-rewrite-portfolio`.

The original compiler-only premise did not survive source inspection: every existing `ScalarProgram` variant denotes a different arithmetic program, and projecting an add-only or multiply-only chain into fused scale/bias serial sum would insert operations for which the existing rewrite and fusion evidence supplies no proof. The correctness-derived boundary is a bounded verified pointwise-`f32` expression with input, constant, add, and multiply nodes and an explicit root, carried by a new exhaustive `ScalarProgram` variant. Tom granted standing approval on 2026-07-28 for a public interface change when correctness leaves one surviving design; this is that case. The initial compiler recognizer remains narrower than the representation and admits only one tensor input plus two scalar constants in one three-leaf same-operation chain.

## Closes when

- At least one accepted add or multiply reassociation compiles end to end through the ordinary entry point under the relaxed governed contract.
- The baseline remains available, the reassociated candidate retains exact semantic/request ownership, and global selection verifies every flattened physical alternative before choosing.
- Unsupported graph shapes still fail closed with typed capability or verification reasons rather than being projected into the serial-sum structure.
- Existing strict and flush-to-zero contracts continue to decline reassociation numerically before physical planning.
- Targeted `tiler-compiler` nextest and Clippy pass, the new negative checks have been observed failing under perturbation, and `make full` passes.

## Graph maintenance

- Depends on `implement-first-algebraic-rewrite-portfolio`; do not claim until that ticket is done.
- On delivery, update the implemented-physical-reachability statements in `docs/compiler/optimizer.md`, `docs/numerical-semantics.md`, and `docs/ir.md`.
- Record whether the serial-sum artifact identity and explain census moved, with the exact recomputation or no-movement check.
- Close this ticket only when the ordinary compile fixture reaches verified program assembly; a unit-only recognizer or lowering result is not the user-visible outcome.

## Outcome

Delivered a checked `PointwiseF32Expression` physical projection and carried exact three-leaf same-family `f32` add and multiply programs through request normalization, four occurrence-owned index refinements, whole-region fusion legality, physical frontier selection, schedule and KIR verification, single-stage program assembly, artifact planning, and global semantic selection. The relaxed end-to-end fixture retains the baseline and a reassociated alternative under distinct exact owners; the strict control declines reassociation numerically. Missing, extra, or reordered semantic coverage, a forged physical expression, mixed operation families, unsupported input cardinalities, shared or repeated leaves, extra reachable operations, wrong outputs, and invalid semantic arity, shape, or dtype all fail closed.

The physical vocabulary remains deliberately narrower than the semantic and index scalar vocabularies. It has no dtype field and admits only exact binary32 input, constant, add, and multiply nodes. Integers, predicates, conversions, mixed precision, multi-result operations, and compound encoded or quantized values remain separate unsupported verticals until their numerical, schedule, KIR, backend, reference, identity, and ABI contracts are complete.

The public-boundary review required three hardenings before acceptance: whole-expression build failures return the intact builder with typed diagnostics; `PointwiseF32Node` is intentionally exhaustive with an out-of-crate total-match tripwire; and canonical expression encoding writes framed nodes directly without one allocation per node. Exact signed-zero and NaN-payload identity checks prevent literal normalization.

### Identity and explain evidence

The serial request-subject encoding and target-profile descriptor did not move. The existing explain census also did not move: `every_wired_authority_emits_its_typed_explain_records` retained its exact rule-count map.

The selected fused serial-sum products did not move. Regenerating the six-member producer matrix at base `6a7278f` and on this tree, then comparing every envelope and sidecar with `cmp -s`, reported `SAME` for all three selected envelopes and all three selected sidecars. Their SHA-256 values remained:

- empty-domain selected envelope `cdb9eb3dc206eb42cea23b23478bcfffedcd9627c953564d4efe07b1e887cced`, sidecar `6127cb9cfe1436472aff0ef594780e3e4271a0162a647ba5f21e5a50d57ba4e3`;
- singleton selected envelope `9546c4cdb5ac40d54de8bf980b72139db99fcf38fc8684502764d750f7c599b3`, sidecar `355c7e90eacf193dd7172ef3725c696ea3c118baaa4dd0ce3be531043b75d99a`;
- nontrivial selected envelope `0f38068d949e8079430912b9e58769b703a2f56c97e781d2f649be5556d023c7`, sidecar `588c728ce5e7e6ee454d8dbcfc83048c8eb9650217474e4f8ac14eeee86b732c`.

The three materialized envelopes and sidecars intentionally moved because their first scheduled stage changed from the redundant fixed `MultiplyThenAdd` record to the exact framed `PointwiseF32` graph. Base-to-current envelope SHA-256 pairs are empty-domain `a22c8f06eb79f05f7076e6b8bb680c4cc8fd1706eff3c63bcf68da23cd39052b` → `23c55a3a546f07369b267c76316a02ff9ee04fc2c2f8878742760f66fa9272da`, singleton `94df9c8e78492be8d639ff241739f6721e1d5872b3a264fb27c304575cf68c30` → `414fda18bf27a164577258d682c8d358d90190dbc12513d8ba845d3bea76c1bd`, and nontrivial `0192697ac66fc2b27b4195a4e3ac622942c1f9f1fa5307e390056a92abf754cf` → `67ff68feb6b0e919e19078b717581332ac4c6ceba66766314049549a12138b99`. The artifact identity domain remains `v8` and manifest schema remains `6.0`; the content-derived identity changed, not its authority.

### Failure-path evidence

Deliberate perturbations made the new checks fail before restoration: canonical node order, dead-node rejection, destination roles, constant-bit identity, per-operation NaN canonicalization, builder recovery, framed encoding length, exhaustive node matching, reassociation association labels, input cardinality, lowering member coverage, refinement evidence class, exact physical expression equality, same-family contraction evidence, future custom arithmetic refusal, and semantic arity, shape, and dtype admission. The relaxed mixed multiply/add serial region remains `Unknown` with `unrealized-contraction`, proving the pointwise exception did not broaden the existing numerical capability.

Targeted final results were `tiler-ir` 291/291, `tiler-compiler` 386/386 with one skipped, `tiler-artifact` 197/197 with two skipped, `tiler-metal` 58/58, and `tiler-build` 13/13. Their per-package Clippy and applicable doc-tests passed, as did formatting, `git diff --check`, and `tkt lint`. The final `make full` passed 1,200 workspace tests with four skipped, every doc-test, warning-denied rustdoc, 445 release-profile numerical tests with one skipped, `tkt lint`, and shellcheck.
