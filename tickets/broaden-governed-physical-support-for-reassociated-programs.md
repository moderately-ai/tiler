---
id: broaden-governed-physical-support-for-reassociated-programs
title: Broaden governed physical support for reassociated programs
status: todo
priority: p1
dependencies: [implement-first-algebraic-rewrite-portfolio]
related: []
scopes: [implementation/compiler]
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

This ticket owns governed physical recognition, lowering, scheduling, program assembly, and their compiler tests under `implementation/compiler`. It does not add algebraic rules, widen the eleven-dimension numerical vocabulary, change operation-owned ordered-associativity declarations, alter the exact ordered-result-set oracle, or redesign composite explain; those outcomes belong to `implement-first-algebraic-rewrite-portfolio`. No public crate, module, trait, or type boundary is implied.

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
