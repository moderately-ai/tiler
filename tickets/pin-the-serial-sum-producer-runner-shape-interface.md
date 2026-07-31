---
id: pin-the-serial-sum-producer-runner-shape-interface
title: Pin the serial-sum producer/runner shape interface the way its filenames are pinned
status: todo
priority: p2
dependencies: []
related: [construct-and-bind-the-first-authoritative-metal-compile-profile, prototype-metal-runtime-proof]
scopes: [implementation/runtime, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [runtime, metal, prototype, testing]
---
## User-visible outcome

A drift between what `prototypes/serial-sum-compile` publishes and what `prototypes/serial-sum-run` expects fails in the repository gate, rather than only when someone runs the hardware proof by hand.

## Why this is worth a ticket

**Fact — it already happened, and stayed hidden for a month.** `prove_member` compiled `serial_sum_program(ROWS, columns)` from the *runner's* own `ROWS = 4` and routed it against artifacts the producer publishes with `ROWS = 1`. Every packaged program was therefore foreign, and the whole matrix pass — six members, thirty operand cases — could not prove a single one. It was introduced when 0b7e59d (2026-07-30, `Defer Metal workgroup limits to prepared pipelines`) moved the producer to one row, three days after 1f4b7fc added the matrix pass. `construct-and-bind-the-first-authoritative-metal-compile-profile` found it by running the proof and fixed it: `prove_member` now reads the shape from the artifact, as the deep proof already did.

**Fact — the same class of drift is already defended against, for filenames only.** The two crates share no code and no Cargo edge, so the member names and the `.proof` suffix are each pinned by a test *in both crates* that names the other side, precisely because a rename once broke the slice end to end for a whole commit under a green gate. The shape has no such pin.

**Inference — reading the shape from the artifact is the right fix and is not the whole fix.** It removes this instance and makes the runner correct for any shape a producer publishes. It does not make a *disagreement* detectable: a producer that published a shape the runner's operand patterns cannot fill would still only fail on hardware.

## Work

Choose and implement one, stating the elimination:

- A pinned pair like the filename pair: both crates assert the published shape matrix in a test naming the other side. Cheap, and it re-couples two crates that deliberately share nothing.
- A gate-reachable fixture: the runner's test module already assembles an envelope from the live builder for the fail-closed probes, and `prove_member`'s shape handling could run against one. This is the option that would have caught the defect.
- An explicit statement in the sidecar the runner validates its own expectations against, making the shape part of the interface both halves already read.

## Closes when

A producer/runner shape disagreement is a red gate, the chosen option's cost is recorded, and the prototypes' own documentation says which mechanism holds the interface.
