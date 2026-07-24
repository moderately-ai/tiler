---
id: register-governed-scalar-reference-evaluation
title: Register reference evaluation for the governed scalar profile
status: todo
priority: p1
dependencies: [wire-capability-and-refinement-into-compile-path]
related: []
scopes: [implementation/reference]
shared_scopes: []
paths: []
tags: [implementation, numerics, reference, milestone-0b]
---
`FrozenScalarRegistry::standard()` now defines `tiler.scalar::constant-f32@1`, `multiply-f32@1`, and `add-f32@1`, and the governed index-access lowerings emit regions over them. No `tiler_reference::ScalarReferenceRegistryBuilder` registration exists for those keys, so the regions the compile path now refines cannot be executed by the independent `IndexRegionEvaluator` oracle.

That leaves a precise gap. Refinement binds structure and reached authority; it explicitly does not re-derive per-point arithmetic (`crates/tiler-compiler/src/legality.rs` module documentation). The governed lowerings are therefore proved to *realize the occurrence structurally* and are not yet checked to *compute it*. Two questions come with the registration and must be answered rather than assumed:

- Whether the governed scalar `add-f32` and `multiply-f32` canonicalize an arithmetic NaN result. `tiler_ir::kernel::lower` applies `ConvertOp::CanonicalizeF32Nan` after the biasing add and after every accumulation but not after the multiply, and `tiler_reference`'s `strict_sum` canonicalizes the reduction result. The scalar definitions' normative references currently name IEEE 754-2019 binary32 operations without stating a canonicalization rule.
- Whether the governed serial-sum lowering's seed-with-the-first-contributor fold reproduces `tiler_reference::strict_sum` exactly on the sign-of-zero and NaN vectors that distinguish it from a `+0.0`-seeded fold.

**Closing evidence.** A reference scalar registry for the three governed scalar keys, and an oracle test that executes each governed family's refined region on the conformance vectors already used by `pipeline::tests::structured_fused_body_interpreter_matches_reference_evaluator` and agrees bit for bit.
