---
id: correct-the-artifact-abis-claim-that-nothing-asserts-the-kernel-identity-crossing
title: Correct the artifact ABI's claim that nothing asserts the kernel identity crossing
status: done
priority: p2
dependencies: []
related: [pin-the-serial-sum-kernel-identitys-crossing-of-the-opaque-identity-bound]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, identity, documentation]
---
`docs/artifact-abi.md`'s "Governed budgets" section ends with a paragraph headed **"What is not pinned here, stated rather than left for a reader to assume."** Its opening sentence — "No test asserts any of these lengths, and none asserts the inequality the paragraph turns on" — was true when written and is **false as of** [`pin-the-serial-sum-kernel-identitys-crossing-of-the-opaque-identity-bound`](pin-the-serial-sum-kernel-identitys-crossing-of-the-opaque-identity-bound.md). Filed by that ticket, which could not repair it: `contracts/artifacts` is outside its four scopes and its own Out of scope section forbids touching the file.

## Facts

**Verified at `c0b2f06bfa38dced03b9d63f7ef2af96e0d5d47b` plus that ticket's landing. Re-verified in full at `acc26984`, where every claim below holds; the one correction is marked.**

`crates/tiler-conformance/src/serial_sum/tests.rs` now carries `the_serial_sum_identity_crosses_the_shared_opaque_bound_at_the_second_contributor`, which compiles a scale-then-bias-then-`StrictSerialF32Sum` program at `[4, 1]` and `[4, 2]` through `compile_under` — `tiler_compiler::session::compile` against `BoundMetalCompileDeclaration::first_macos_apple9()` — reads `VerifiedKernel::canonical_identity().as_bytes()` off the selected plan's kernels, and asserts the one-contributor identity is **under** `MAX_OPAQUE_IDENTITY_BYTES` and the two-contributor one **over** it. Observed on this host: 924 and 1,309, exactly the right column of the table above it — reproduced at `acc26984` by `cargo nextest run -p tiler-conformance --no-capture -E 'test(the_serial_sum_identity_crosses_the_shared_opaque_bound_at_the_second_contributor)'`, which prints both readings.

**Imprecise as written, corrected here:** the helper `widest_kernel_identity` does not read one kernel's length. It maps `canonical_identity().as_bytes()` over every kernel of the selected plan and returns the **widest**, because an artifact carries each entry's identity as its own `BackendEntryKey` and the bound is crossed as soon as any one of them crosses it. At both shapes the plan carries one kernel, so the two readings coincide, but a worker restating the claim as "the" kernel's identity would describe a helper the code does not have.

The same landing removed the two `vec![0x5a; 1_121]` literals in `crates/tiler-artifact/src/program/tests.rs`, so the paragraph's sentence "it builds `vec![0x5a; 1_121]` and asserts that length exceeds `MAX_OPAQUE_IDENTITY_BYTES`" is **also stale**. Both vectors are now `super::MAX_OPAQUE_IDENTITY_BYTES + 1`, and the tautological `assert!(len > MAX_OPAQUE_IDENTITY_BYTES)` that stood beside the first was deleted rather than re-derived: with a derived length it could never have failed.

## What closes this

The paragraph rewritten as a record of what *is* pinned and what still is not, in the dated form the rest of the section uses.

- **Still true and should stay:** no test asserts any of the six *lengths*, and that is deliberate — a pinned length decays when the constant offset moves, which is the defect the whole section documents.
- **Now false and must be dated rather than deleted:** the inequality *is* asserted, by name, in `tiler-conformance`. Name the test so a reader can find it, and preserve the structural argument for why it is not in `tiler-artifact` — the missing `tiler-compiler` edge, `the_consumer_links_no_compiler_emitter_or_build_provider`, ADR 0081 item 2 — because that argument is unchanged and is why the assertion lives one crate over.
- **Also false:** the description of the `tiler-artifact` case's contents. Restate it as a fabricated vector at a length *derived* from the bound.

Prefer restating what was true at the tree the sentence describes over deleting it, matching the "Recorded 2026-08-08" and "Corrected 2026-08-07" corrections already in this file.

## Out of scope

`crates/**` and `prototypes/**`: the filing ticket already landed those and its Outcome records what it did.
