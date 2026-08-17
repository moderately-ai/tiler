---
id: correct-metal-numerical-requirements-doc-after-precise-elementary-emission
title: Correct Metal numerical requirements documentation after precise elementary emission
status: done
priority: p1
dependencies: []
related: [honor-the-precise-fp32-metal-compilation-requirement]
scopes: [implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics]
---
## User-visible outcome

`MetalTranslationUnit::numerical_requirements` documents the current governed requirement population truthfully: elementary-function emission may require precise FP32 functions, while the returned set remains a requirement subset rather than a complete compiler flag list.

## Exact-current discovery — 2026-08-17 at `b085f9dcd95c77ecdf42e93d3e083f02a584a4a8`

**Fact.** `crates/tiler-metal/src/record.rs`, anchor `contains no accuracy-mode-dependent library call`, says `-fmetal-math-fp32-functions` is unconstrained for every emitted unit. That sentence dates to `59060b58` and predates the `PreciseFp32Functions` requirement.

**Fact.** `crates/tiler-metal/src/emit.rs` inserts `MetalNumericalRequirement::PreciseFp32Functions` for admitted precise FP32 elementary operations. Existing Metal tests assert that `F32Exp` and `F32Rsqrt` units carry it. The independent build-adapter repair `honor-the-precise-fp32-metal-compilation-requirement` relies on that exact current contract.

## Required work

- Read the complete record API documentation and current requirement derivation before editing. Replace only the stale universal claim; do not weaken the rule that callers must satisfy every returned requirement or imply the slice is a complete compiler command line.
- Add a source-semantic check that retains the corrected statement and rejects the retired claim. Perturb the documented subject, not the assertion, and record the failure.
- Preserve all public Rust, emitted source, numerical requirement population, target facts, identities, schemas, domains, pins, and runtime behavior.

## Implementation and review — 2026-08-17

The implementation at `46179ccb72b0a235a06cd5a864f4e0d821d34069` over exact base `f829fecfc9ece67db418ec2ab1de4b1092437fb6` changes only the accessor documentation and one reachable source-semantic test. The documentation now distinguishes the returned requirement subset from the caller-owned complete compiler selection, names precise FP32 elementary operations as the owner of `PreciseFp32Functions`, and preserves the non-elementary absence rule. Independently perturbing the elementary subject, the non-elementary neighbour, the subset wording, and the retired universal sentence made the unchanged test fail with its corresponding subject-specific message.

An independent exact-commit review read the complete `record.rs`, the complete Metal test module, requirement derivation, neighboring positive and negative controls, and the build-side consumer. It found no finding at any severity. Focused and full Metal tests, all-target check, Clippy and rustdoc with warnings denied, doctests, formatting, `tkt lint`, `make citations`, `git diff --check`, and exact-base `tkt guard` were green. No public Rust, executable behavior, emitted source, requirement population, identity, schema, domain, pin, manifest, dependency, or runtime path changed.

## Closes when

The accessor documentation matches precise and non-elementary units, the source-semantic negative control is recorded, focused Metal docs/tests plus Clippy/rustdoc and ticket gates pass, and independent review finds no remaining contradictory claim in the surrounding public record documentation.
