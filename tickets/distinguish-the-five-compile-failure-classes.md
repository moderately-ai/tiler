---
id: distinguish-the-five-compile-failure-classes
title: Distinguish ADR 0069's five compile failure classes
status: done
priority: p2
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler-api, diagnostics]
---
`CompileFailureClass` has four variants and ADR 0069 requires five distinguishable classes.

## Fact

ADR 0069 states the compiler's "failure classes distinguish at least invalid requests, valid programs lacking a required compilation capability, intrinsically or target-infeasible plans, exhausted bounded search, and failures of compiler-produced IR verification" — five.

`crates/tiler-compiler/src/session.rs` declares four: `Unsupported { rule }`, `NoFeasiblePlan`, `BudgetExhausted`, `InvalidCompilerOutput`. The first two of ADR 0069's list are folded into `Unsupported`, distinguished only by a `&'static str`.

## Fact — the collapse is deliberate and reasoned, not an oversight

`class_of` in the same file carries the argument: "Both are statements about the request rather than about Tiler, and both carry the refusing check's own key, so they classify the same way; the internal distinction between a malformed request and an unsupported capability is preserved in the explain trace." The internal `pipeline::CompileError` does keep `InvalidRequest` and `UnsupportedCapability` apart, so no information is lost — it is merged on the way out.

## Inference — the reasoning is sound about information and wrong about class

Two things pull the other way, and both come from this crate's own conventions.

`CompileFailureClass`'s own doc says the enum exists so "a caller branches on the boundary that refused instead of matching on text". Distinguishing a malformed request from an uncoverable program currently requires matching `rule` against strings, which is the thing the type exists to avoid. ADR 0074 convention 1 makes the same point generally: a variant carries the structured data a caller needs to react, not a preformatted discriminator.

The two also imply different actions. "Your request is malformed" says fix the request. "Your program is valid and no installed capability compiles it" says install a provider or wait for coverage — and that distinction became reachable in practice on 2026-07-27, when out-of-crate capability installation landed and a caller acquired something to *do* about the second.

## Scope

Split the two, preserving the rule key on both. Supersede the `class_of` comment explicitly rather than deleting it: its claim about the explain trace is true and is not what the class is for.

Check the remaining three against ADR 0069's list at the same time rather than assuming they line up.

## Closes when

`CompileFailureClass` distinguishes ADR 0069's five classes; each is reachable by a test that reaches it from the public surface, or is recorded as unreachable with the reason; the superseded reasoning is preserved at its site; and `make full` passes.

## Outcome — five classes, and the reachability of each is now recorded (2026-07-27)

`CompileFailureClass::Unsupported { rule }` split into `InvalidRequest { rule }` and `UnsupportedCapability { rule }`, both keeping the refusing check's key. All five of ADR 0069's classes are now distinct.

**The superseded reasoning is preserved at `class_of`, not deleted**, because it is true about information and wrong about class. Nothing was lost by merging the two — the internal `CompileError` always kept them apart and the explain trace always carried the distinction. What was lost is the caller's ability to *branch*: telling them apart meant matching `rule` against strings, which is what `CompileFailureClass`'s own documentation says the enum exists to avoid, and ADR 0074 convention 1 says a variant carries the data a caller reacts to rather than a preformatted discriminator.

**The remaining three were checked against ADR 0069 rather than assumed to line up**, as this ticket asked. They do: "intrinsically or target-infeasible plans" is `NoFeasiblePlan`, "exhausted bounded search" is `BudgetExhausted`, "failures of compiler-produced IR verification" is `InvalidCompilerOutput`.

### Reachability, which turned up something worth recording

| class | reachable from the public surface |
| --- | --- |
| `UnsupportedCapability` | **yes** — an identity program, tested |
| `NoFeasiblePlan` | **yes** — tested |
| `BudgetExhausted` | only by a program built to exceed a deterministic budget; `RequestError::BudgetExceeded` is its sole source and the governed budgets admit everything this profile compiles |
| `InvalidCompilerOutput` | **no, deliberately** — it reports that Tiler's verifier refused Tiler's output, so reaching it from the public surface would mean shipping the defect it exists to report |
| `InvalidRequest` | **no, today** — see below |

**Fact: `InvalidRequest` cannot currently be produced by any caller.** Its five sources — unsupported schema version, empty target set, duplicated target profile, unverified target selection, unstated numerical contract — are structural facts about the request, and `compile` builds that structure itself through `CompilationRequest::governed_under`. A caller supplies a program, a contract, and capabilities; none can yield any of the five. Verified by reading `From<RequestError> for CompileError` (`pipeline.rs:270-289`), which is the sole classifier.

**Inference: that is a reason to split the class, not to skip it.** It becomes reachable the moment a caller can declare its own target profiles — an empty set and a duplicate are exactly what that admits — which is the live p1 `admit-a-caller-declared-target-profile`. Landing the class now means that ticket adds a construction path rather than also having to widen the failure vocabulary.

The class-distinctness test now compares all five pairwise instead of spot-checking two pairs, so a future collapse fails it; two spot checks would not have noticed.
