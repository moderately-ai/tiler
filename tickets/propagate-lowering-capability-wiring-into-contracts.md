---
id: propagate-lowering-capability-wiring-into-contracts
title: Propagate the wired capability and refinement stage into governed contracts
status: todo
priority: p1
dependencies: [wire-capability-and-refinement-into-compile-path]
related: []
scopes: [contracts/optimizer, contracts/numerics, contracts/foundation]
shared_scopes: []
paths: []
tags: [documentation, optimizer, capability, milestone-0b]
---
`wire-capability-and-refinement-into-compile-path` was scoped to `implementation/compiler` and `implementation/ir`, so the governed contracts that describe the compile path were deliberately left untouched. They now under-describe it.

At least the following changed and must be represented:

- The compile path runs a lowering-capability resolution stage before cover enumeration. Resolution is unconditional and fails closed on a missing or contended capability; the artifact plan's lowering provenance is the set of providers that resolution returned, not a compile-time constant.
- Index-region refinement is attached per recognized occurrence as exhaustive finite evidence, and degrades to a typed `BudgetStop` at `ExplainStage::KernelRefinement` plus an `Unknown` assessment — never to a rejection — when `MAX_EXHAUSTIVE_PROOF_CELLS` cannot afford the emitted region's access proof.
- Refinement's scalar-authority conformance rule changed from equality to containment: a region must reach nothing beyond what its capability declared it may emit. `crates/tiler-compiler/src/legality.rs` records the reasoning; `docs/compiler/optimizer.md` and `docs/ir.md` may still state the equality form.
- The fused alternative is no longer gated by an installed fused-provider constant. Its availability is decided by fusion legality and target feasibility alone.
- `docs/correctness-and-testing.md` records that the optimizer conformance owner must drive an external operation through the ordinary capability and refinement path before the public compiler facade is accepted. `pipeline::conformance::an_externally_registered_lowering_provider_drives_the_compile_path` is that evidence; the record should cite it.

**Closing evidence.** `uv run --locked python scripts/docs.py render` and `uv run --locked python scripts/check_repository.py` pass with the contracts updated, and no governed record still states the superseded equality rule or the constant-provider gate.
