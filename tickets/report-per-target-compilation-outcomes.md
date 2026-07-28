---
id: report-per-target-compilation-outcomes
title: Report per-target compilation outcomes instead of aborting on the first refusal
status: awaiting-decision
priority: p2
dependencies: []
related: [prototype-public-compiler-api]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api]
---
## Decision needed (2026-07-28)

**Is a multi-target compilation one all-or-nothing outcome, or a per-target outcome set?**

**Read this next to `admit-a-caller-declared-target-profile`, and decide them in that order.** The defect here is latent, not live: `CompilationRequest::governed` declares exactly one target profile, so the loop has one iteration and the two shapes are indistinguishable to every caller that exists. It becomes live the moment a second profile is admitted or the request is exposed — which is `admit-a-caller-declared-target-profile`, itself `awaiting-decision`. So these are two parked tickets where one gates the other, and answering this one first spends a decision on a shape whose only consumer may never be admitted; answering it second means the shape is chosen against a known caller. If the profile ticket is declined, this question may not need answering at all.

| Option | Enables | Prevents |
| --- | --- | --- |
| **1. Keep all-or-nothing.** | Costs nothing now, because one declared profile makes the two shapes indistinguishable. No public type moves. | Fails the stated need the moment a second profile is admitted: a caller cannot learn which target refused and cannot use the targets that were feasible. |
| **2. Outer `Result`, inner per-target outcomes.** The outer `Err` carries whole-compilation refusals *and* compiler-state defects like `UnverifiedTargetSelection`; the inner sequence carries one Ok-or-refusal per declared target, each with its own sealed explain trace. | Satisfies both constraining facts by construction — fact 1 needs no reclassification, and fact 2's defect never enters a per-target slot. Declaration order is preserved, so a caller can index results against the profiles it asked for. | Makes a caller that declares **one** target — every caller today — unwrap twice to reach a compilation it could previously get in one step. The cost is paid by the common case to keep two error classes apart in the type for a case that does not yet exist. |
| **3. A success list plus a separate rejection list.** | Equivalent in information to option 2 and flatter to consume. | Loses the declared order, so a caller cannot index results against the profiles it asked for without matching on identity. |

**Recommendation: option 2.** It is the only one that keeps the two error classes apart in the type rather than by convention, and it preserves declaration order.

**Its counterpoint, which the earlier write-up omitted:** the double unwrap is not free, and today it is pure cost — every caller declares one profile, so every caller pays an extra level of matching to express a distinction none of them can currently observe. The question to answer is whether the type-level separation of "the compilation failed" from "this target refused" is worth that, or whether the separation should arrive with the second profile rather than ahead of it. A convenience accessor for the one-target case would blunt the cost and would also be the first place someone reintroduces the conflation, so it is not a free repair.

**Why it cannot proceed without the decision.** It changes `CompilationProduct`, the public return type of `session::compile_governed`, and what "the compilation failed" means to every caller — ADR 0075's always-ask category. The ticket's own scope says so.

### Both constraining facts, verified

1. **Whole-compilation refusals genuinely precede per-target work — confirmed.** `compile_verified` (`crates/tiler-compiler/src/pipeline.rs:448-463`) takes an already-`VerifiedCompilationRequest` and an already-computed `NormalizationOutcome`, so request verification, semantic output typing, numerical-contract resolution, and normalization have all completed before the loop exists. Any per-target shape can keep them distinguishable simply by leaving them on the outer error; nothing has to be reclassified.
2. **`for_target` fails with `RequestError::UnverifiedTargetSelection` — confirmed**, at three sites in `request.rs` (`:1105`, `:1108`, `:1127`). **And it is called *inside* the map**, at `pipeline.rs:458`, which sharpens the constraint rather than restating it: in the shape that exists today a compiler-state defect and a target's own refusal already arrive at the same place, indistinguishable to the caller. Whatever is chosen must separate them, and the current code does not.

**Fact — one target's refusal discards every other target's result.** `crates/tiler-compiler/src/pipeline.rs::compile_verified` maps each declared target profile through `compile_target` and gathers them with a fallible collect, so the first `Err` short-circuits and no `TargetCompilationProduct` survives. `CompilationProduct` has no per-target failure record and no partial form. The exact check is to read `compile_verified` — the `map(...).collect::<Result<_, _>>()?` at `pipeline.rs:457-461` is the whole mechanism.

## Consequence at the public boundary

`tiler_compiler::session::compile_governed` returns `Result<Vec<Compilation>, CompileFailure>`, so a caller that declared N targets gets either N successes or one failure. It cannot learn which target refused, and it cannot use the targets that were feasible. `prototype-public-compiler-api` closed the adjacent question — a failed compilation now carries the complete explain trace the compiler sealed — but that trace is scoped to *one* target request, so on a multi-target compilation it explains one refusal and says nothing about the others.

**Why it was split rather than folded in.** It is a different question from the seven `prototype-public-compiler-api` owns. Those are about what a boundary exposes *of a report*; this is about the cardinality of the result itself — whether a compilation is one all-or-nothing outcome or a per-target outcome set — and answering it changes `CompilationProduct`, the public return type, and what "the compilation failed" means. Folding it in would have widened an approved surface on a question nobody had asked.

## Scope

Decide and implement whether a multi-target compilation returns per-target outcomes. Two facts constrain the answer, and both were checked before the options above were written:

- A refusal that happens *before* per-target work — request verification, semantic output typing, numerical-contract resolution, normalization — is genuinely whole-compilation and has no target to attribute to. Any per-target shape must keep that case distinguishable from a target-specific refusal, or a caller will read a malformed request as N target rejections.
- `verified.for_target(target)` itself fails with `RequestError::UnverifiedTargetSelection`, which is a compiler-state defect rather than a target's assessment, so it does not belong in a per-target rejection either.

The shape is a public-boundary change and is Tom's to accept.

## Closes when

A compilation declaring several target profiles reports each target's outcome separately, a whole-compilation refusal stays distinguishable from a per-target one, a test covers a request in which one target succeeds and another is refused, and `make full` passes.

Parked 2026-07-27; the question, the verified facts, and the elimination were hoisted to the top of this ticket on 2026-07-28 and are no longer restated here.
