---
id: declare-metal-numerical-honourability
title: Declare Metal numerical honourability as a target profile fact
status: review
priority: p0
dependencies: [select-numerical-contract-and-compose-feasibility]
related: [draft-target-honourable-numerical-contract-adr, prototype-metal-numerical-realization, express-metal-honourability-in-the-shared-form]
scopes: [implementation/metal, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, numerics]
claimed_from: todo
assignee: agent-metal
lease_expires_at: 1785001280
---
ADR 0076 item 3, on the one target that has a measured unhonourable dimension. This is the ticket that gives the Apple row a positive conformance story for the first time: a flush-tolerant `f32` contract compiles and conforms, a preserving one rejects with a named cause.

## What is implemented today, and why it is not enough

`MetalNumericalGap::SubnormalFlushInArithmetic` records the unhonourable obligation, is written into the generated MSL provenance header, and is enforced by `MetalTranslationUnit::require_declared_realization`, which fails closed with `MetalEmitError::UnrealizableNumericalObligation`. That is correct as far as it reaches and was the honest thing to build at the time. It is insufficient as a durable answer for four reasons, each independently sufficient:

- it is one gap variant that cannot distinguish input flushing from result flushing;
- it names no target-profile identity, so a rejection cannot say who declared the fact;
- emission still succeeds, so a caller that never asks for conformance never sees the rejection;
- nothing above `tiler-metal` can select a contract the target would honour instead, so the only reachable outcomes on Apple are a refused conformance claim or a caller that never asks.

## The work

Express `MetalSubnormalArithmetic` as a per-dimension honourability declaration in the shared form `select-numerical-contract-and-compose-feasibility` establishes, rather than a backend-local target fact — so the compiler can assess it *before* emission rather than discovering it during. Retire `MetalNumericalGap` and `require_declared_realization` in favour of the typed rejection, **or** state precisely why a backend-local conformance step survives alongside the profile declaration. Either is acceptable; leaving both without saying which is authoritative is not.

Keep the measurements recorded on the declaring types. `MetalTargetFacts` already documents its measured basis on the type itself, and that is the pattern to preserve.

## The inference that constrains how you may establish honourability

**Honourability is a stated target fact and must never be a value observed from a probe kernel.** The measurement behind this is worth reading before you design anything: under `-fmetal-math-mode=relaxed`, subnormal operands come back *unchanged* from a `scale 1.0, bias +0.0` kernel, which looks like preservation. It is not. `x * 1.0` folds to a copy under every math mode, the kernel retains exactly one floating-point operation under `safe` (the `+0.0` fadd, unremovable without `nsz`) and zero under `relaxed`, and the surviving `fadd` is what flushes. The same licence that breaks signed zero deletes the operation that would have flushed. So observing preserved subnormals from a compiled kernel is not evidence that the target preserves them — it may be evidence that no arithmetic executed, and the modes where this misleads are exactly the least trustworthy ones.

`MetalTargetFacts::subnormal_arithmetic` already takes the correct approach: a required caller-stated fact with the measurement recorded on the type. Generalize that; do not replace it with anything inferred.

## The contract half

`docs/backends/metal.md` records the strict flag row and states that the compatibility probe "did not observe the numerical behavior these flags request". The re-verified measurement in ADR 0076 closes that gap in one direction and the contract must record it: **the strict row does not deliver subnormal preservation.** `-fmetal-math-mode=safe` emits `air.compile.denorms_disable` alongside `air.compile.fast_math_disable`, under `safe`, `relaxed`, and `fast` alike; no offline flag and no runtime `MTLCompileOptions` setting clears it. Materialization is unaffected — a load-then-store round trip preserves every subnormal — so the limit is a property of arithmetic specifically, and the contract should say that rather than a blanket claim about the target.

ADR 0076 leaves as an open question whether the profile *declaration mechanism* belongs in `docs/backends/metal.md` or in the architecture contract. Recording the measured flag behaviour there is not in question; siting the mechanism is. If you conclude it belongs elsewhere, say so and add the scope rather than writing it where it does not belong.

## A knock-on you will hit immediately

`crates/tiler-metal/src/emit.rs` carries irrefutable `let SubnormalMode::Preserve = mode;` bindings in `realization_requirements` and `record_subnormal_obligation`. Those become compile errors the moment `widen-numerical-vocabulary-and-complete-identity` lands, which is the guard working as designed. Handling the new variant is part of this ticket.

The four golden fixtures can then carry a contract the hardware actually honours instead of one it cannot — check whether their compilation through `golden_compilation` should move to the flush-tolerant contract, and say which contract they are governed under either way.

## Inherited from `widen-numerical-vocabulary-and-complete-identity`

That ticket widened `SubnormalMode` to `Preserve | FlushToZero { zero_sign }` and `NumericalPermission` to `Forbidden | Permitted`, which broke both irrefutable `let SubnormalMode::Preserve = mode;` guards in `crates/tiler-metal/src/emit.rs`. They were repaired without a wildcard, and two decisions there are yours to supersede.

**The flag question is answered and needs no further work.** Neither subnormal behaviour names a `MetalNumericalRequirement`, and the two reasons differ: preservation names none because no `-fmetal-math-mode`, `-ffp-contract`, `-fmetal-math-fp32-functions`, or `-O` selection preserves subnormals through `f32` arithmetic — the front end emits `air.compile.denorms_disable` under all of them — and flushing names none because that same measurement makes the flush unconditional, so no selection has to be made to obtain it. `realization_requirements` records both reasons.

**The declaration question was deferred to you and currently fails closed.** `MetalSubnormalArithmetic::FlushesToZero` states *that* the target flushes and not *which zero* it produces, even though the measured Apple flush is sign-preserving (`0x80400000 * 2.0f` returns `0x80000000`). A declared flush is therefore not established by the target fact. `emit::subnormal_gap` is a total comparison of the declared mode against the target fact and yields two new `MetalNumericalGap` variants for the newly expressible cases:

- `SubnormalPreservationInArithmetic` — the contract flushes and the target preserves. Honouring it would mean emitting an explicit flush, which is emulation, which this backend does not express.
- `UndeclaredFlushedZeroSign` — the contract flushes to a stated zero and the target names no zero. This is a placeholder for exactly this ticket: once the profile declares honourability per dimension including the sign, a sign-matching flush becomes a positive conformance claim and only a sign *mismatch* stays a gap. Retire the variant rather than keeping it alongside the declaration.

Behaviour on the governed path is unchanged: the registered contract is still `Preserve`/`Preserve`, so the four `crates/tiler-metal/goldens/*.metal` still record `subnormal-flush-in-arithmetic` and only that. Their identity digests moved, because the scheduled-region and kernel identities were re-baselined; the emitted bodies did not.

## The target fact is under-specified, and that is the second half of the execution blocker

**Fact — verified by reading `crates/tiler-metal/src/emit.rs::subnormal_gap` at `065a9b8`.** The gap rule is total over `(declared, target)` and produces a gap in *three* of its four arms:

| declared | target | result |
| --- | --- | --- |
| `Preserve` | `PreservesSubnormals` | no gap |
| `Preserve` | `FlushesToZero` | `SubnormalFlushInArithmetic` |
| `FlushToZero { .. }` | `PreservesSubnormals` | `SubnormalPreservationInArithmetic` |
| `FlushToZero { .. }` | `FlushesToZero` | `UndeclaredFlushedZeroSign` |

**Inference — making the numerical contract selectable is necessary and not sufficient.** `select-numerical-contract-and-compose-feasibility` lets a caller state a flush-accepting contract instead of the strict one. That moves the governed Apple row from row two to row four of the table above, which is still a gap. `SubnormalMode::FlushToZero` names *which* zero it produces via `FlushedZeroSign`, and `MetalSubnormalArithmetic::FlushesToZero` carries no such field, so the target states that it flushes and not what it flushes to. The emitter is right to refuse: a contract that specifies a zero sign cannot be honoured by a target that does not state one.

**The measurement needed to close it already exists**, recorded on `tiler_ir::schedule::FlushedZeroSign::PreservesSign`: on an Apple M4 Max under macOS 27.0 with Metal 32023.883, an emitted `x * 2.0f` returns `0x80000000` for the operand `0x80400000`, not `0x00000000`. Apple's flush is sign-preserving. What is missing is not evidence but expressiveness — `MetalSubnormalArithmetic::FlushesToZero` needs to carry the zero sign it produces so `subnormal_gap` can compare the declared sign against the declared target sign and return no gap when they agree.

**Consequence for sequencing.** First execution needs *both* changes, and neither alone produces a `metallib`: the contract must become selectable, and the target fact must state its zero sign. This ticket is no longer merely blocked behind `select-numerical-contract-and-compose-feasibility` — the two are the two halves of one gate, and the gate is what every Metal and runtime p0 waits on.

**Do not close the gap by widening the rule.** Making the fourth arm return no gap regardless of sign, or dropping `zero_sign` from `SubnormalMode`, would let a program that specifies positive-zero flushing run on a sign-preserving target and return `0x80000000` where it asked for `0x00000000`. That is a wrong answer, not a relaxed one.

## Progress — the target fact now names its zero; it is still backend-local

**Landed at `a56bff8`.** `MetalSubnormalArithmetic::FlushesToZero` became `FlushesToZero { zero_sign: MetalFlushedZeroSign }`, and `subnormal_gap` compares the declared zero against the target's through an exhaustive `flushed_zero_gap`. Agreement is now a positive conformance claim; only a genuine sign mismatch is a gap, renamed `FlushedZeroSignMismatch`. Previously the target stated *that* it flushes and not *to what*, so a `SubnormalMode::FlushToZero` — which always names a zero — could never be established, and every flush contract failed closed as `UndeclaredFlushedZeroSign`. Four golden MSL provenance headers were rebaselined; the gate recompiles them through `xcrun`.

That change plus a selectable contract is what let a real program reach an Apple M4 Max and return bits identical to the reference oracle.

**Why this ticket is still open.** What landed is a *backend-local target fact*, which is exactly what this ticket exists to replace. `MetalSubnormalArithmetic` still lives in `tiler-metal` and is still consulted only during emission, so the compiler cannot assess honourability *before* emitting — it discovers unhonourability from `require_declared_realization` after a translation unit already exists. The per-dimension honourability declaration in the shared form, expressed so `feasibility` can assess it as a peer of `CheckedTargetProfile`, is not built; that is `compose-numerical-honourability-and-retire-the-strict-boolean`'s peer authority, and this ticket owns the Metal side of it.

Also unchanged: whether `MetalNumericalGap` and `require_declared_realization` are retired in favour of the typed rejection, or whether a backend-local conformance step survives alongside it with a stated reason. The measurements stay recorded on the declaring types either way.

## Contract half landed; the declaration mechanism is what remains

**Done — `docs/backends/metal.md`'s numerical realization section.** The ticket names this half as "not in question", and the contract was stating something the re-verified measurement contradicts: it said the compatibility probe "did not observe the numerical behavior these flags request" without recording that a *different* measurement since has.

Four things are now recorded there, each separated from the others because they fail differently:

- **The strict row does not deliver subnormal preservation.** `-fmetal-math-mode=safe` emits `air.compile.denorms_disable` beside `air.compile.fast_math_disable`, under `safe`, `relaxed`, and `fast` alike, and no offline flag or runtime `MTLCompileOptions` setting clears it. The strict spellings request preservation and do not obtain it.
- **The limit is arithmetic specifically.** A load-then-store round trip preserves every subnormal bit pattern, so materialization is unaffected. Stating this as a blanket property of the target would be wrong in the direction that matters, because a program that only moves subnormals is unaffected.
- **The flush is sign-preserving**, `0x80400000 * 2.0f` → `0x80000000` on an M4 Max under macOS 27.0 with Metal 32023.883 — which is what lets a flush-accepting contract be a positive conformance claim on this row rather than merely a weaker one.
- **Honourability is stated, never probed.** The `relaxed`-mode trap is recorded in the contract itself rather than only in the ticket, because the contract is where someone would otherwise be tempted to close the loop with a probe kernel: preserved subnormals coming back from a compiled kernel can mean no arithmetic executed, since `x * 1.0` folds to a copy and the surviving `+0.0` fadd is the operation that flushes.

The compiler-provenance section's cross-reference, which described the subnormal behaviour as still unobserved and pointed at this ticket, was corrected in the same pass rather than left to contradict the section above it.

**Siting, as the ticket required stating.** The measured flag behaviour went in `docs/backends/metal.md`, which the ticket says is not in question. The profile *declaration mechanism* is not written there, so ADR 0076's open question about where it belongs stays open and is not answered by omission.

**What remains, and why the ticket stays open.** The two substantive halves are untouched: expressing `MetalSubnormalArithmetic` as a per-dimension honourability declaration in the shared form so `feasibility` can assess it before emission rather than discovering it after, and deciding whether `MetalNumericalGap`/`require_declared_realization` retire in favour of the typed rejection or survive alongside it with a stated reason. Both wait on `compose-numerical-honourability-and-retire-the-strict-boolean`'s peer authority, which this ticket already lists as the shared form it must adopt.

**Evidence.** `uv run --locked python scripts/check_repository.py` passes. The run also caught a broken link in the first draft of this amendment — `0076-target-honourable-numerical-realizations.md` against the real `0076-declare-target-honourable-numerical-realizations.md` — which is the gate doing its job on a hand-written cross-reference.

## Outcome

Two of the three open questions are settled and recorded; the third — expressing `MetalSubnormalArithmetic` in the shared honourability form — is split into `express-metal-honourability-in-the-shared-form`, which depends on `compose-numerical-honourability-and-retire-the-strict-boolean`. This ticket goes to `review`, not `done`: its title names the half that moved out.

### Decided — the backend-local conformance step survives, and the profile declaration is authoritative

The ticket allowed either arm and forbade leaving both without saying which is authoritative. **The profile declaration is the authority on what the target honours.** `MetalNumericalGap` and `require_declared_realization` are kept, and they are not a second authority on that fact. Three reasons, the third decisive:

- **Fact — the Metal fact is declared exactly once.** `MetalTargetFacts::subnormal_arithmetic` is the sole statement of it, and every arm of `emit::subnormal_gap` and `emit::flushed_zero_gap` is derived from that one value. A second checkpoint reading one declaration cannot diverge from it; two declarations of the same property could, which is the failure mode being avoided.
- **Inference — the two checkpoints answer different questions.** A profile declaration is a claim about a *target and a contract*, answerable before emission. A gap is a claim about *the operations one translation unit actually emitted*. The measurement makes that distinction load-bearing rather than pedantic: the limit is `f32` arithmetic specifically, and a load-then-store round trip preserves every subnormal. A single checkpoint sited before emission would refuse a materialization-only kernel this target does honour; a single checkpoint at admission would let emission proceed unchallenged. Collapsing them produces a wrong answer in one direction or the other.
- **Fact — the dependency graph makes the backend step non-optional.** `tiler-metal` depends on `tiler-ir` and `tiler-artifact` and deliberately not on `tiler-compiler`; the two are siblings over the IR. Verified with `grep -n 'tiler-' crates/tiler-metal/Cargo.toml crates/tiler-compiler/Cargo.toml` at `94fb26e`. A compiler-side rejection is therefore unreachable from `emit_translation_unit`, which is a public entry point a caller can drive from `tiler-ir` alone — the crate's own doctest does exactly that. Retiring the backend step would leave that path emitting source under a refused contract with no conformance claim in reach.

Recorded on the declaring types (`crates/tiler-metal/src/record.rs`, `crates/tiler-metal/src/lib.rs`) and in the contract (`docs/backends/metal.md`, "Two conformance checkpoints, one declaration of the fact").

### Decided — the goldens stay under the strict declared realization

The ticket anticipated that the four fixtures could "carry a contract the hardware actually honours". They are **not** moved, and the reasoning is worth stating because the anticipation is reasonable and the evidence goes the other way.

Two records are called a contract here and they are decided separately: the *declared realization* the program states, baked into the emitted bytes, and the *compiler realization* the driver selects. The goldens are governed under `tiler.test.strict-f32` (preserving on both subnormal dimensions) and the strict driver baseline. Reasons:

- **Nothing about the compiler evidence would change.** Neither subnormal mode names a compiler selection, so the flag row is identical either way, and the emitted bodies are identical too — no operation is emitted to realize a flush, because this backend expresses no emulation. Rebaselining would change every entry-point symbol, since the canonical kernel identity encodes the profile key and both subnormal dimensions, and buy no coverage for four wholesale file rewrites plus an `xcrun` recompile.
- **Under strict they are the only checked-in artifacts pinning the non-empty unrealizable-obligation provenance block**, which is what a caller keeping only the emitted text reads.
- **Decisive — there is no flush-accepting contract this crate can name.** The governed one, `tiler.flush-f32.v1`, is registered in `tiler-compiler`, which `tiler-metal` must not depend on. A "flush golden" would carry a crate-local key that merely resembles it; writing the registered key as a string literal would duplicate a versioned identity across a boundary with no compile-time link, so a rename on the owning side would leave a golden silently claiming the wrong contract.

A consequence that was implicit and is now stated in `crates/tiler-metal/src/golden_compilation.rs`: the units the gate compiles are ones `require_declared_realization` refuses. That is intentional and is itself evidence — it shows the refusal is a Tiler conformance decision about an unhonourable contract, not a compiler rejection of the source.

### Landed — the honourability rule became a tested guarantee

`a56bff8` made a sign-matching flush a positive conformance claim, but the crate owning that rule tested one of its four `subnormal_gap` arms and neither `flushed_zero_gap` arm. The only coverage of an honoured flush lived in `prototypes/serial-sum-compile`, outside `tiler-metal` entirely. Reserved-in-the-type-system, implemented, and tested-guarantee are three different claims, and this was the second. Five tests in `crates/tiler-metal/src/tests.rs` close every uncovered arm:

- a flush the target delivers is honoured over real arithmetic — with an explicit non-vacuity assertion, because the comparison is only reached from emitted `f32` arithmetic and a "no gap" result over a materialization-only kernel would be evidence of nothing;
- a flush to the other zero is refused as `FlushedZeroSignMismatch` — the arm that must never be widened away;
- an always-positive flush is honoured by an always-positive target, so agreement rather than the one measured value is what the rule turns on;
- a flush contract on a preserving target is refused as `SubnormalPreservationInArithmetic`;
- the two subnormal dimensions are compared independently — declared as a *mismatched* flush on one dimension and preservation on the other, so one kernel yields two *different* gaps, which a pair of cases producing the same gap could not distinguish from a coupled comparison. It also pins the documented rejection order.

### Not decided, deliberately

ADR 0076's open question on where the profile *declaration mechanism* is sited stays open. `docs/backends/metal.md` records the relationship between the two checkpoints, which is a Metal backend contract matter, and says explicitly that it does not site the declaration. `express-metal-honourability-in-the-shared-form` carries that decision, along with the finding that forces it: neither `tiler-metal` nor `tiler-compiler` can hold both the shared form and the Metal fact, so the siting is a real choice between `tiler-ir`, a consumer-constructed translation, and a third crate.

### Nothing retracted

No prior claim in this ticket was found to be wrong. The Progress and contract-half sections stand as written.

### Evidence

`uv run --locked python scripts/check_repository.py` passes, and the five new tests run within it.

The gate captures test output, so toolchain resolution is not readable from its log and was confirmed separately rather than inferred: `cargo nextest run -p tiler-metal --lib -E 'test(golden_compilation)' --no-capture` resolves `metal`/`metallib` `32023.883` (`metalfe-32023.883`) against macOS SDK 26.5 build 25F70, links all four fixtures (3683/3715/3747/3859 bytes) and the four-entry-point portfolio (14716 bytes). These tests self-skip where no qualified toolchain resolves, so that command is the check to repeat on another host.
