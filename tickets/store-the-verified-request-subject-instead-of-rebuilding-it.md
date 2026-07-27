---
id: store-the-verified-request-subject-instead-of-rebuilding-it
title: Store the verified request subject instead of rebuilding it
status: done
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

## Outcome

Done. **Request-subject reconstructions per compile fell from 57 to 12**, and the 12 that remain are the verification rather than waste.

**One method was serving two roles.** `subject()` was both the accessor every reader used and the reconstruction the tamper check needed, so every reader paid the verifier's price. They are now separate:

- `subject()` returns `&VerifiedRequestSubject` — a borrow of the stored `authority`. The subject is a pure function of fields that are private and never mutated after `for_target` verified them, so rebuilding it per call reproduced a value the type already holds.
- `reconstructs_its_authority()` re-derives and compares. Same behaviour as the old `is_authoritative()`, same cost, and now named so a caller choosing it is choosing the cost.

**Nothing was removed.** The forged-request tests that assert the check catches tampering still pass unchanged — they now call the method that actually performs the check. Deleting the reconstruction entirely would have been faster still and would have removed a tested guarantee, which is why it was split rather than dropped.

The remaining 12 are `reconstructs_its_authority` at the sites that genuinely verify, plus `for_target` computing the authority in the first place.

**The maintainability half.** The naming is the point. `subject()` reading as free and costing a deep clone of the semantic identity, both shapes, the members and the contract preference is precisely how the count reached 57 — every new reader took the obvious-looking method. A borrow cannot be misused that way, and a caller who wants the check now has to name it.

`physical.rs` had the sharpest instance: it called `subject()` and then `is_authoritative()` back to back, building the value twice, once per proposal per region per cover.

Gate: `make full` green (982 nextest + 11 doc-tests, rustdoc, release numerical tests, `tkt lint`, shellcheck). Artifact identity unchanged.
