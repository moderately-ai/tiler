---
id: draft-target-honourable-numerical-contract-adr
title: Draft a proposed ADR for target-honourable numerical contracts
status: in-progress
priority: p1
dependencies: []
related: [prototype-metal-numerical-realization, prototype-artifact-program-model, own-operation-family-support-matrix]
scopes: [contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [documentation, decisions, numerics, feasibility]
claimed_from: todo
assignee: agent-draft-target-honourable-numerical-contract-adr
lease_expires_at: 1784915749
---
Record, as a **proposed** ADR, how a numerical contract expresses what a target
can actually honour — so a real target can be conformant rather than permanently
refused.

## What forced this (measured, not theorised)

Apple GPU `f32` arithmetic **flushes subnormals to zero unconditionally**.
Compiling with `-fmetal-math-mode=safe` — the strictest mode, which explicitly
disables fast math — still emits `air.compile.denorms_disable` alongside
`air.compile.fast_math_disable` in the AIR. No offline flag and no runtime
`MTLCompileOptions` setting clears it. Materialization is unaffected: load/store
round-trips preserve every subnormal. So the limit is specific to arithmetic.

Our strict profile declares `SubnormalMode::Preserve` for inputs and results,
because the profile's correctness claim is bitwise equality with the CPU
reference evaluator, and the CPU preserves subnormals. On Apple that claim is
therefore unsatisfiable for any computation that produces a subnormal.

## The reframe this ticket exists to record

This is **not** a restriction on what Tiler can do. It is a specification input:
it tells us which knobs the numerical contract must expose. Today
`tiler_ir::schedule::SubnormalMode` has exactly one variant, `Preserve`, and
`StrictF32NumericalContract::governed()` is a hardcoded constant no caller
chooses. So there is no flush-tolerant contract to select and no way for a caller
to ask for one — which is why refusal was the only available answer. A vocabulary
that cannot describe real hardware forces every real target to be non-conformant.

The architectural line to preserve while fixing it: a numerical contract is a
**target-neutral semantic declaration** of what the program means; whether a given
target can deliver it is a **feasibility** question. A target's limitation must
never silently redefine what the program means. Under that line, refusing
strict-preserve on Apple is the feasibility authority working correctly — the gap
is only that no feasible alternative is expressible.

## What the ADR should settle

- **Vocabulary.** What `SubnormalMode` must offer beyond `Preserve` (at minimum a
  flush-to-zero mode), and whether input and result subnormals need independent
  settings, since hardware can differ on each.
- **Selection.** How a caller states the contract it needs, replacing a hardcoded
  `governed()` constant — and what happens when a caller states nothing.
- **Target capability declaration.** How a target profile declares which
  numerical realizations it can honour, so feasibility can *select* a conformant
  contract rather than only reject an unsatisfiable one. This is the piece that
  turns a refusal into a choice.
- **Artifact record.** How the delivered realization is recorded so a consumer
  knows what it actually got, rather than inferring it from the request. A
  consumer comparing GPU output against a CPU oracle must be able to tell which
  contract the artifact honours.
- **The honesty rule.** What must happen when no available contract is honourable
  on a target: an explainable rejection, never a silently downgraded one.

Note the same shape applies beyond subnormals — contraction, reassociation, and
NaN payload behaviour are all places where a target may not honour the strict
reading. Decide whether this ADR states a general model or only the subnormal
case, and say which.

## Boundaries

Proposed only: `decision_status: "proposed"`, `ticket` pointing here, open
questions left explicit. Change no code and no other contract; implementation
follows as its own tickets across `tiler-ir` (vocabulary), `tiler-compiler`
(selection and feasibility), `tiler-metal` (capability declaration), and
`tiler-artifact` (delivered-realization record). Prose is not hard-wrapped. Run
`scripts/docs.py render` and the documentation gate before completion.

## Outcome

Recorded 2026-07-24. [ADR 0076](../docs/decisions/0076-declare-target-honourable-numerical-realizations.md) is written as **proposed**; acceptance is Tom's separate step and no code changed.

### The framing needed one correction, and it changed the record's job

**Fact — the vocabulary is already decided, so the record must not invent one.** This ticket states that "a vocabulary that cannot describe real hardware forces every real target to be non-conformant." That is accurate about the implemented enums and inaccurate about the design. ADR 0019 is accepted and already resolves subnormal input and result handling independently with preservation or explicit flush-to-zero on each. `docs/numerical-semantics.md` spells `SubnormalContract { inputs: Preserve | FlushToZero, results: Preserve | FlushToZero }`, already names the four backend outcomes `SupportedExactly`/`SupportedWithExactEmulation`/`SupportedOnlyUnderDeclaredRelaxation`/`Unsupported`, and already states that target defaults cannot expand the program's permissions. ADR 0011 already decides the selection model as a program ceiling. The conformance matrix already requires all four preserve/flush combinations as adversarial coverage. Inventing a vocabulary would have created a second authority over the same terms. The ADR therefore *binds the implemented subset* to those accepted records and decides only what they leave open, and it says so explicitly so a later reader cannot mistake it for the origin of the vocabulary.

**Fact — "feasibility selects a conformant contract" is the wrong verb, and this ticket's own line is why.** If any planning authority chooses the numerical contract, a target's limitation has redefined what the program means — the line this ticket says to preserve. `docs/artifact-abi.md` already forbids the neighbouring case: routing "never chooses between different accuracy meanings." The ADR's model is that the *caller* states the contract, optionally as an ordered preference list, and feasibility only assesses; nothing downstream may narrow it. That is recorded as a deliberate correction rather than a paraphrase.

**Fact — the genuine open question was different from the five items as stated.** Two accepted four-outcome vocabularies exist — ADR 0043's `Proven`/`Deferred`/`Rejected`/`Unknown`, which classify how well a predicate is known, and the numerical contract's four, which classify by what means an obligation is honoured — and nothing states how they compose. That composition is the substance of ADR 0076 §3.

### The five items, as settled

1. **Vocabulary.** No new terms; grow `SubnormalMode` and `NumericalPermission` to what ADR 0019 and ADR 0011 already accept. `NumericalRealization` already carries `input_subnormals` and `result_subnormals` as independent fields, so independence needs preserving rather than adding. New requirement: a flush behaviour must state which zero it produces, because the measured Apple flush is sign-preserving.
2. **Selection.** The resolved contract is a required typed compile input with no `Default` and no ambient fallback; stating nothing is a compile error naming the unstated contract, not a silent strict or relaxed compile. A caller may state an ordered preference list resolved deterministically by stated order, never by cost.
3. **Target capability declaration.** A per-dimension honourability report using the contract's four means-outcomes, as a distinct authority from `CapabilityAxis` — because `SupportedWithExactEmulation` has no representation as a bound comparison and would be lost. The composition into ADR 0043's outcomes is stated, including that an unenumerated dimension is `Unknown` and therefore fails closed. `supports_strict_f32` and `CapabilityAxis::StrictF32Arithmetic` are retired rather than extended.
4. **Artifact record.** A readable delivered-realization record complete over the dimensions, with each dimension's means and the declaring profile identity — not inferable from the request and measurably not inferable from compiler flags.
5. **Honesty rule.** Stated in both directions; the converse of the existing rule is new: no authority may narrow the caller's stated contract to make a target feasible. Consequence: the numerical contract is never a cost-ranked search dimension.

A sixth item was added that the ticket did not ask for and that the evidence forced: the realization's identity encoding must be complete and fail closed on a widened vocabulary. See below.

**Scope.** Generalised, not subnormal-only, with the reasoning stated in ADR 0076 §0: every mechanism question has the same answer in every dimension; the measured evidence already spans subnormals, signed zero, and contraction, so a subnormal-only record would fail to cover evidence in hand; and ADR 0011/0019 are already general. What is *not* generalised is the claim of unhonourability — that is measured for one dimension on one target row and the record says so.

### A defect found by reading, not stated in the ticket

**Fact — the two sibling identity encoders disagree and only one fails closed.** `tiler_ir::kernel::model::push_numerical` encodes both subnormal fields through `push_subnormal`/`push_permission` whose matches are exhaustive, so widening the enum is a build error. `tiler_ir::schedule::model::push_numerical` encodes the profile key, the NaN bits, and the two `permits_*` booleans and encodes **neither subnormal field**; because those accessors are `!matches!(…)` expressions, widening compiles silently and two realizations differing only in subnormal treatment would receive the same `CanonicalScheduledRegionIdentity`. `crates/tiler-metal/src/emit.rs` guards itself a third way with irrefutable `let SubnormalMode::Preserve = mode;` bindings. Widening the vocabulary is the single change that converts the omission from unobservable to a cache-and-artifact correctness defect, so ADR 0076 §6 requires the two to land together.

**Fact — `derive_requirements` ignores the subnormal dimensions entirely.** It computes `requires_strict_f32: !permits_reassociation() && !permits_contraction()`. Once the vocabulary widens, a subnormal-preserving contract that permits contraction and reassociation derives `requires_strict_f32 == false`, and `Relation::Implies` makes the predicate vacuously satisfied, so it would be *admitted* on a target declaring no strict-`f32` support. Unobservable today because one variant per enum makes every realization identical.

**Fact — `docs/ir.md` does not list the numerical realization in `IndexRegion` identity.** It enumerates "iteration and reduction domains, typed tensor boundaries, access maps, scalar operations and values, constraints, and ordered outputs", and its layered summary reads "canonical iteration/scalar/access content". The implemented `IndexRegion` carries a `numerical` field, and `ScalarProgram`'s variants separately carry the canonical NaN bits and a contraction flag, so part of the realization is inside "scalar content" and the region-level declaration is outside it. The contract and the implementation disagree about where the realization sits, which is *why* the encoder can omit it. ADR 0076 §6 requires the IR contract to gain the sentence rather than only the encoder to be repaired.

**Fact — the request boundary is more closed than the ticket states.** `StrictF32NumericalContract` is `pub(crate)`, `CompilationRequest` is `pub(crate)`, `pipeline::compile` is `pub(crate)`, and the only assembly site is a `#[cfg(test)]` constructor. There is no non-test path by which any caller states a numerical contract at all, so the selection decision is being made before the boundary is public — which under ADR 0075 is the right time and makes this a shaping decision rather than a change to a published surface.

### Re-verified measurements

Everything below was re-measured on this host on 2026-07-24, not carried over: Apple M4 Max, macOS 27.0, `Apple metal version 32023.883 (metalfe-32023.883)`, `xcrun --sdk macosx metal -target air64-apple-macos13.0 -std=metal3.1`, dispatch through a purpose-built Objective-C Metal host.

**Measurement — reproduced as reported.** `!"air.compile.denorms_disable"` is emitted under `safe`, `relaxed`, and `fast`, and under `safe` it appears alongside `!"air.compile.fast_math_disable"` with no fast-math flag on any emitted `fmul`/`fadd`. The `-0.0` divergence reproduces exactly: `MultiplyThenAdd { scale 1.0, bias +0.0 }` returns `0x00000000` under `safe` and `0x80000000` under `relaxed`/`fast`. The contraction divergence reproduces exactly: `0x3fc58f9e` under `-ffp-contract=off` and `on`, `0x3fc58f9d` under `fast`. A load/store round trip preserves every subnormal under every mode.

**Measurement — sharper than the existing record.** Input and result flushing are separable and each was isolated. `x * 2.0f` returns `0x00000000` for `0x00400000`, where preserving the operand gives the *normal* `0x00800000` — the correct result is not subnormal, so this isolates **input** flushing. `x * 0.5f` returns `0x00000000` for the normal `0x00800000`, where the correct result is the subnormal `0x00400000` — the operand is normal, so this isolates **result** flushing. Both hold at `-O0` and `-O2` under `safe` and `fast` without variation.

**Measurement — new.** The flush is sign-preserving: `0x80400000 * 2.0f` returns `0x80000000`, not `0x00000000`. This is why ADR 0076 §1 requires a flush behaviour to state its zero.

**Measurement — new, and a trap for any measurement-based design.** A relaxed mode can appear to honour a strict contract by deleting the arithmetic. In the `scale 1.0, bias +0.0` kernel, subnormals are returned unchanged under `relaxed`/`fast` and flushed under `safe`. Counting floating-point operations in the emitted LLVM IR explains it: `x * 1.0` folds to a copy under every mode, and the kernel retains exactly one floating-point operation under `safe` — the `fadd` of `+0.0`, unremovable without `nsz` — and zero under `relaxed`. The surviving `fadd` is what flushes. The same licence that breaks signed zero deletes the operation that would have flushed. **Inference.** Observing preserved subnormals from a compiled kernel is not evidence that a target preserves them; it may be evidence that no arithmetic executed. Honourability must be a stated target fact, which is what `MetalTargetFacts::subnormal_arithmetic` already does and what ADR 0076 generalizes.

**Measurement — new, and why item 4 exists.** Under `relaxed` the module records `!"air.compile.fast_math_disable"` while every floating-point operation in it carries `reassoc nsz arcp contract afn`. The module-level flag is not a faithful summary of the licences applied, so an artifact reader inferring the delivered realization from it reads the opposite of the truth.

**Measurement boundary.** One host, one toolchain build, one target row. The probe harness was built in a scratch directory and is **not checked in and not in the repository gate**, which ADR 0076 records as an open question against `AGENTS.md`'s requirement that a reproducible experiment live under `spikes/`.

### Frontmatter

`decision_status: "proposed"`, `implementation_status: "not-started"`, `catalog_group: "numerical-operations"`, `refines: ["ADR-0011", "ADR-0019"]`, `depends_on: ["ADR-0043"]`. `applies_to` names `tiler.contract.numerical-semantics` and `tiler.contract.ir` as strong edges and `tiler.contract.metal-backend` and `tiler.contract.artifact-abi` as weaker ones, with the weakness recorded as an open question rather than hidden. `tiler.contract.correctness-and-testing` is deliberately *excluded*: the honesty rule guarantees declared and delivered never diverge for any artifact that exists, so that contract's oracle rule stands unchanged. `evidence` cites the conformance matrix and the physical feasibility model as design evidence, plus the Apple compatibility probe for its explicit disclaimer that it "did not observe the numerical behavior these flags request" — cited for that boundary, not as evidence about subnormals.

### Implementation successors

Named in ADR 0076 so each inherits a concrete answer, and ordered rather than parallel because widening the vocabulary without the identity encoding is a correctness defect and widening it without the profile declaration leaves the variant unreachable: `tiler-ir` (vocabulary and identity together, plus `derive_requirements`), `tiler-compiler` (required contract input, honourability authority, composition, retire the boolean axis, rejection shape), `tiler-metal` (profile declaration; subsume `MetalNumericalGap`), `tiler-artifact` (the delivered-realization record — the crate is a four-line shell, so this creates its first content and is a new public namespace Tom reviews under ADR 0075).

### Gate results

`uv run --locked python scripts/docs.py render` passed (178 records; the ADR catalog and chronology blocks regenerated). `uv run --locked python scripts/check_repository.py` passed (`complete repository validation passed`, exit 0). `git diff --check` clean. `ticketsplease guard tkt/draft-target-honourable-numerical-contract-adr` reported no scope escape.
