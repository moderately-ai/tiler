---
id: store-the-verified-request-subject-instead-of-rebuilding-it
title: Store the verified request subject instead of rebuilding it
status: todo
priority: p1
dependencies: [measure-compiler-and-artifact-hot-paths]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [performance, compiler, maintainability]
---
**Measured: 55 request-subject rebuilds per compile of a 5-operation program.**

## Fact

`VerifiedTargetRequest::subject()` (`request.rs:757`) calls `request_subject` (`request.rs:1062`), which deep-clones the `SemanticIdentity` (~4.1 KB across its four parts), both `Shape`s, `reduction_axes`, `RecognizedSerialSumMembers` (two `Vec`s), the `NumericalContractPreference`, and the registry identity. `is_authoritative()` (`request.rs:770`) builds a **second** copy and deep-compares.

The hottest site builds it twice back to back, `physical.rs:505-506`:

```rust
let subject = request.subject();
if !request.is_authoritative()      // builds it again, then compares
```

inside `verify_schedule_with_feasibility`, which runs per proposal per region per cover. Other rebuild sites: `program.rs:803`, `explain.rs:210`, `explain.rs:267`, `fusion.rs:167`, `request.rs:1030`.

`explain.rs:267` is the clearest: `from_fusion_numerical` rebuilds bytes the `ExplainWriter` is already holding, and `push` then byte-compares the two to confirm they match.

## Why this improves maintainability as much as speed

`VerifiedTargetRequest` **already stores** this value as `authority`. Computing it once in `for_target` and returning `&VerifiedRequestSubject` makes "the subject is the authority" an invariant of the type. Today it is a comparison that a new call site can simply forget to make — the failure mode is silent and the fix is structural.

The request is immutable, so verifying once and reusing preserves the check exactly.

## Scope

Compute the subject once in `for_target`; return a borrow from `subject()`; reduce `is_authoritative()` to the check it actually needs. Consider `Arc` for the explain subject (`explain.rs:958` clones the full subject bytes per record and `:1021`/`:1213` deep-compare on every push).

## Closes when

One compile rebuilds the subject at most once per target, pinned by a work-count guard; the authority check is preserved; artifact identity is byte-identical; `make full` passes.
