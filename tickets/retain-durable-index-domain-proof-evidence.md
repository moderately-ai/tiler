---
id: retain-durable-index-domain-proof-evidence
title: Retain durable index-domain proof evidence
status: done
priority: p1
dependencies: [implement-index-domain-predicates]
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, proof]
---

## User-visible outcome

Every discharged index-domain predicate exposes durable typed evidence from the verified region, so compiler diagnostics and downstream consumers can distinguish how the predicate was established without consulting a transient proof cache.

## Implementation keys

- Add a closed evidence vocabulary that keeps sound proof, exhaustive finite evidence, empirical evidence, and `Unknown` distinct. Do not add ordering, scores, or conversion to a confidence scalar.
- Bind evidence to the exact `IndexDomainPredicate` and verified-region subject it proves. Construction is checked and foreign handles are refused.
- Include retained evidence and its predicate subject in canonical region identity when it changes what downstream consumers may rely on; exclude solver search state and memoization.
- Keep `UnknownReason::{InsufficientFacts, UnsupportedFragment, ResourceLimit}` separate from evidence. This ticket must not turn an unknown obligation into an admitted executable program.
- Add an exhaustive construction-site test covering every evidence variant and prove the test can fail by perturbing one correspondence once.
- Follow ADR 0084 and `docs/research/shapes/constraint-prover-boundary.md`.

## Closes when

Verified regions retain inspectable typed evidence for discharged predicates; the four evidence classes cannot collapse through the public type; identity covers every relied-on subject; targeted `tiler-ir` nextest, Clippy, and doc-tests pass; the new check has been observed failing under deliberate perturbation; and `make full` passes.

## Graph maintenance

- On completion, record the exact public types and identity consequence here.
- Release `carry-unknown-index-domain-obligations`.
- File any new proof lane separately rather than widening this evidence-custody ticket.

## Outcome

`tiler_ir::index` now exposes the closed, unordered `IndexDomainSoundProof` and `IndexDomainEvidence` enums plus the opaque `DischargedIndexDomainPredicate` record. A verified region returns its canonical evidence records through `VerifiedIndexRegion::discharged_index_domain_predicates` and validates exact subject/predicate lookups through `VerifiedIndexRegion::index_domain_evidence`; record construction remains private, checks every region-owned handle, and refuses `Unknown` as a discharge.

Index-region canonical identity moved from `tiler.index-region.v6` to `tiler.index-region.v7`. The encoding includes each discharged record's exact access subject, predicate expression and extent, and evidence class or sound-proof method; solver search state and memoization remain excluded.

The construction-site test exhaustively covers `SoundProof`, `ExhaustiveFinite`, `Empirical`, and `Unknown`. Deliberate perturbations proved failures for the `Unknown` disposition, every foreign-handle position, omission of an upper-bound predicate, and each identity component: subject, same-length predicate, and same-length proof method. After restoration, `cargo nextest run -p tiler-ir` passed 297 tests; `cargo clippy -p tiler-ir --all-targets -- -D warnings` and `cargo test -p tiler-ir --doc` passed. Downstream validation passed 386 `tiler-compiler` tests plus its Clippy and doc-test commands.
