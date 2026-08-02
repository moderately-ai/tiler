---
id: correct-the-declined-strategy-record-for-an-unsplittable-reduction
title: Correct the declined-strategy explain record for an unsplittable reduction
status: in-progress
priority: p1
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, defect, explain]
claimed_from: todo
assignee: agent-decline
lease_expires_at: 1785704070
---
## User-visible outcome

A reduction with too few contributors to split compiles under a reassociation-permitting contract, instead of failing the whole compilation with `InvalidCompilerOutput(Explain(InvalidStageEvent))`.

## Why this exists, and that it predates the recognizer work

**Measurement — reproduced on `d0b8445`, before any recognizer change.** The program `sum(a * 2.0 + 1.0, axis 1)` over `Shape::from_dims([2, 2])`, compiled through `crate::pipeline::compile` with `CompilationRequest::governed_under` for each entry of `StrictF32NumericalContract::governed_profile()`:

| contract | outcome |
| --- | --- |
| `tiler.strict-f32.v1` | compiles |
| `tiler.flush-f32.v1` | compiles |
| `tiler.relaxed-f32.v1` | `InvalidCompilerOutput(Explain(InvalidStageEvent))` |
| `tiler.reassociate-f32.v1` | `InvalidCompilerOutput(Explain(InvalidStageEvent))` |

The same program at `Shape::from_dims([1, 4])` compiles under all four. The difference is the contributor count: axis 1 has extent 2 at the failing shape and 4 at the passing one, and `governed_partition` returns `None` below four contributors.

**Inference — the decline record is the suspect, not the decline.** Declining a split for an extent that admits no balanced exact partition is correct and is the documented behaviour of `SplitUnavailable::NoAdmissiblePartition`. What is wrong is that recording that decline produces an explain stage event the writer rejects, and a rejected record fails the compilation as invalid compiler output. The two permitting contracts are exactly the ones that reach `propose_split`/`propose_workgroup_tree` at all, which is why the strict pair is unaffected.

**Inference — it is a correctness defect, not a cosmetic one.** A caller whose program is legal, whose contract is registered, and whose target is capable gets a compiler-defect class rather than a plan.

## Boundaries

- Fix the record, not the decline. Suppressing the decline would remove the explanation that tells a reader why the serial alternative stands alone.
- The perturbation that must be watched: a decline record that *is* well formed must still reach the trace, so a fix that stopped emitting declines would pass this ticket's positive case and lose the evidence.

## Closes when

The two-contributor program above compiles under all four registered contracts; its trace contains a declined-strategy record naming `no-admissible-partition`; and a deliberately malformed stage event is still rejected by the explain writer, observed failing.
