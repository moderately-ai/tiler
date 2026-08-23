---
id: decide-whether-tiler-reference-publishes-a-bit-extraction-convenience
title: Decide whether tiler-reference publishes a bit-extraction convenience
status: awaiting-decision
priority: p3
dependencies: []
related: [decide-the-backend-provider-conformance-harness-public-surface]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [reference, public-boundary, decision, needs-tom]
---
## User-visible outcome

Either `tiler-reference` publishes one small evaluation-to-bits convenience with an accepted signature, or the decision to leave every caller writing it is recorded with its reason.

## Why this exists

Filed 2026-08-22 by `worker-packet`. The second re-derivation on `decide-the-backend-provider-conformance-harness-public-surface` itemized the independent backend fixture and found exactly one candidate export that is both genuinely reusable and genuinely non-self-certifying — the caller cannot manufacture an oracle. It is deliberately **not** folded into that public-boundary answer, because it belongs to `tiler-reference` rather than to any conformance facade, and because bundling a small unrelated surface into a facade decision is how an unaccepted item rides along.

**Fact — what the helper is.** `crates/tiler-conformance/tests/independent_backend/workload.rs` defines `reference_bits`, which builds a dense `Tensor` from `f32` bit patterns, evaluates the same `SemanticProgram` through `ReferenceEvaluator::standard()`, destructures `TensorPayloadView::Dense`, and returns the output element bits. Its own header states the property that makes it worth having: `Nothing in this file states an expected value`.

**Fact — it is currently hand-written per caller.** Re-audit the population at your base before proposing anything; do not assume the fixture is the only site.

## Required work

- Census the callers that already do this by hand. State the spellings searched for and why that set is complete; a census anchored on one phrasing under-counts silently.
- If the population is one, the honest answer is probably to publish nothing, and recording that is the outcome.
- Any published signature is a `tiler-reference` public boundary under ADR 0075. Treat it as a labelled draft until Tom accepts its exact included and excluded surface, and do not let it default a bit order, a payload view, or an evaluator profile.

## Closes when

Either an accepted signature exists with its unsupported cases named, or the no-publication answer is recorded with the census that supports it.

## Coordinator census at `1cb2a09e`, 2026-08-22 — the "population is one" hypothesis is very likely wrong, but 36 is a ceiling, not an answer

**Fact 1 verified.** `reference_bits` is at `crates/tiler-conformance/tests/independent_backend/workload.rs`, declared `pub(crate) fn reference_bits(program: &SemanticProgram) -> Vec<u32>`.

**The Required work says "if the population is one, the honest answer is probably to publish nothing." Do not start from that assumption.** A co-occurrence census — tracked `.rs` files mentioning **both** `ReferenceEvaluator` and `TensorPayloadView::Dense`, counted as *files*, run from a Python file rather than a shell one-liner — returns **36**.

**But 36 is an upper bound on candidates, not a count of duplicated helpers, and handing it on as a population would be the exact error AGENTS.md names.** Co-occurrence in a file is not the same as hand-writing this helper, and three of the 36 are disqualified by construction: `crates/tiler-reference/src/evaluate.rs`, `quantization.rs`, and `structural.rs` are the reference crate's **own internals** — they define the evaluator and cannot be duplicate callers of a convenience over it. The crate's own `src/tests.rs`, `src/bf16/tests.rs`, and its `tests/` files are in-crate and would not need a *published* surface to reach it either.

**The population that actually bears on a publication decision is the out-of-crate one.** By crate, the 36 break down as: `tiler-reference` itself 13 (internals plus in-crate tests, mostly disqualified), `tiler-compiler` 5, `tiler-conformance` 4, `tiler-runtime` 2, `prototypes/` 2, `spikes/` 4 — roughly **17 out-of-crate files** worth reading. Read each and classify it; a file may use both symbols for entirely unrelated reasons.

**So the census the ticket asks for is a reading task, not a grep.** State the spellings you searched and why that set is complete — mine is one vocabulary and a floor: a caller that reaches bits through a different accessor, or evaluates without naming `ReferenceEvaluator` directly, lands outside it. `grep -c` counts lines and I counted **files**; say which unit you report.

**The public-boundary constraint is unchanged and is the binding one.** Any published signature is a `tiler-reference` public boundary under ADR 0075, stays a labelled draft until Tom accepts its exact included and excluded surface, and must not default a bit order, a payload view, or an evaluator profile. If the reading says publish, **stop and produce a packet for Tom rather than publishing** — the ticket's Closes-when admits "an accepted signature", and acceptance is Tom's.

## Worker-bits packet at `99ec35f5`, 2026-08-22 — reading says publish; the census is ~20 independent copies, not one

**Correction to the coordinator's own count.** The coordinator's census said `tiler-reference` itself contributes 13 of the 36 co-occurrence files. Re-run from a file (not a shell one-liner) at this same base: `tiler-reference` actually contributes **19** (`src/evaluate.rs`, `quantization.rs`, `structural.rs`, `tests.rs`, `bf16/tests.rs`, plus 14 files under `tests/`), not 13. This does not change the decision: every one of those 19 is still in-crate by the same construction argument the coordinator gave, so the out-of-crate population is unaffected. Flagged because a stale sub-count is exactly the kind of thing that should not be silently carried forward. `5 (tiler-compiler) + 4 (tiler-conformance) + 2 (tiler-runtime) + 2 (prototypes/) + 4 (spikes/) = 17` still matches `19 + 17 = 36` exactly.

**Census method.** Vocabulary: tracked `.rs` files (`git ls-files '*.rs'`, 764 files at this base) containing both the literal substrings `ReferenceEvaluator` and `TensorPayloadView::Dense`. Unit: **files**, not lines. Run from a Python file, not a shell one-liner, so no shell interpolation can corrupt the pattern:

```python
import subprocess, pathlib
repo = "."  # repository root
files = subprocess.run(["git", "-C", repo, "ls-files", "*.rs"], capture_output=True, text=True, check=True).stdout.splitlines()
needle_a, needle_b = "ReferenceEvaluator", "TensorPayloadView::Dense"
hits = []
for rel in files:
    try:
        text = (pathlib.Path(repo) / rel).read_text(encoding="utf-8")
    except (UnicodeDecodeError, FileNotFoundError):
        continue
    if needle_a in text and needle_b in text:
        hits.append(rel)
print(len(files), len(hits))
```

Result: 764 tracked `.rs` files, **36** co-occurrence files — reproducing the coordinator's number exactly at this base.

**Why this vocabulary is a floor, and what I did to bound the gap.** The brief itself names the risk: a caller reaching bits through a different accessor, or evaluating without naming `ReferenceEvaluator` directly, lands outside a two-term co-occurrence census. I checked this rather than asserting it: `comm -23` between `git grep -l ReferenceEvaluator` and `git grep -l TensorPayloadView::Dense` finds 8 files that use the evaluator without the literal destructure string. Read each:

- `crates/tiler-compiler/src/pipeline/conformance.rs` — imports and calls `tensor_bits`/`bits_of`, helpers **defined in the sibling file `crates/tiler-compiler/src/pipeline/tests.rs`** (already inside the 36 — see below). This is the floor firing exactly as warned: a real caller of the pattern, invisible to the two-term grep because the crate already factored its own private copy out from under it.
- `crates/tiler-conformance/src/envelope.rs` — the string `ReferenceEvaluator` appears only in a doc comment naming an unrelated constant (`iteration_step_allowance`); it does not evaluate or destructure a tensor here. Correctly outside the population.
- `crates/tiler-reference/src/{contraction.rs,lib.rs,rms_norm/tests.rs,softmax/tests.rs,value_conformance/tests.rs}` — all in-crate, already covered by the construction argument.
- `spikes/extensions/operation-api/src/lib.rs` — defines its **own** unrelated `trait ReferenceEvaluator` (a hypothetical operation-extension seam), not `tiler_reference::ReferenceEvaluator`. A same-name false positive, correctly outside the population.

Net effect: the floor gap is real but small and already inside the read population once `pipeline/conformance.rs` is attached to the `pipeline/tests.rs` site it calls. No further out-of-crate site was found reaching bits through an unlisted accessor. I did not check for a third possible gap — a caller that decodes bits from raw sidecar bytes without going through `Tensor`/`TensorPayloadView` at all (e.g. `tiler-conformance/src/envelope.rs`'s `decode_f32_bits`, which reads proof-sidecar payloads, not reference-evaluator output) — because that is a structurally different operation (decoding a stored proof artifact, not extracting bits from a freshly evaluated reference tensor) and out of the ticket's stated scope (the fixture-shaped helper).

**Per-site classification of the 17 out-of-crate files.** All 17 read as genuine, independent, load-bearing reimplementations of the same core shape: evaluate a `SemanticProgram` through `ReferenceEvaluator`, destructure one output's `TensorPayloadView::Dense`, map each element's `as_bytes()` through a fixed-endianness `fromXX_be_bytes`, collect. None is a same-symbol-different-purpose false positive.

| File | Local name(s) | Shape | Width |
|---|---|---|---|
| `tiler-compiler/src/governed/attention_conformance.rs` | `result_bits` | leaf decode only | u32 |
| `tiler-compiler/src/governed/contraction_conformance.rs` | `result_bits`, `element_bits` | leaf decode only | u32 |
| `tiler-compiler/src/normalize.rs` | inline ×3 | full evaluate+decode, **re-derived three separate times inside its own test module**, never even locally factored | u32 |
| `tiler-compiler/src/pipeline/tests.rs` | `tensor_bits`, `bf16_tensor_bits` | leaf decode only, `pub(super)`, reused by `pipeline/conformance.rs` | u32 **and** u16 |
| `tiler-compiler/tests/contraction_topology_witness.rs` | `result_bits` | leaf decode only | u32 |
| `tiler-conformance/src/bf16_vertical.rs` | `reference_bits` → `reference_encodings` | full evaluate+decode | **u16 only** |
| `tiler-conformance/src/publication/proof.rs` | `reference_bits` | full evaluate+decode | u32 |
| `tiler-conformance/src/serial_sum.rs` | `dense_bits`, `reference_bits` | leaf decode + evaluate wrapper, factored into two fns (closest existing shape to a two-piece public API) | u32 |
| `tiler-conformance/tests/independent_backend/workload.rs` | `reference_bits` | full evaluate+decode, fixture-embedded operands (the ticket's own Fact site) | u32 |
| `tiler-runtime/tests/adapter_route/main.rs` | `reference_bits`, `pointwise_reference_bits` | full evaluate+decode ×2 in one file | u32 |
| `tiler-runtime/tests/identity_join/main.rs` | `reference_bits` | full evaluate+decode | u32 |
| `prototypes/serial-sum-compile/src/sidecar.rs` | `reference_bits` | full evaluate+decode | u32 |
| `prototypes/serial-sum-run/src/proof.rs` | `dense_bits`, `reference_bits` | leaf decode + evaluate wrapper (same two-piece shape as `tiler-conformance/src/serial_sum.rs`) | u32 |
| `spikes/program-planning/reduction-dispatch-crossover/src/main.rs` | inline | full evaluate+decode | u32 |
| `spikes/program-planning/reduction-partition-calibration/src/main.rs` | inline | full evaluate+decode | u32 |
| `spikes/runtime/backend-provider-portfolio/src/semantic.rs` | `reference_bits` (`pub fn`) | full evaluate+decode | u32 |
| `spikes/target-profiles/scalar-cpu-vertical/src/vertical.rs` | `reference_bits` | full evaluate+decode | u32 |

**Bit order is consistent in practice, not by accident of the search.** `FloatBitOrder` has two variants; I checked whether `LeastSignificantByteFirst` appears anywhere in the 17 (or the 8-file floor check) — it does not. It appears only inside `tiler-reference`'s own implementation (`src/tensor.rs`) and its own test of that conversion (`src/tests.rs`). Every out-of-crate site, without exception, encodes with `FloatBitOrder::MostSignificantByteFirst` and decodes with `from_be_bytes`. A published helper that fixes big-endian decode would match every current call site exactly; it would still be *a* published choice, not a rediscovery of one already forced by the type.

**Finding: this is not a population of one, and the ticket's own hypothesis is wrong.** It is closer to twenty near-identical hand-written copies (17 out-of-crate, plus the crate-internal `tiler-compiler` pair `pipeline/tests.rs`/`pipeline/conformance.rs`, plus `tiler-conformance`'s own four in-crate copies the ticket already named as disqualified-by-construction but still real duplication *within* that crate). One file (`normalize.rs`) re-derives the block three times without even a local helper. Two independent sites (`tiler-conformance/src/serial_sum.rs`, `prototypes/serial-sum-run/src/proof.rs`) have already converged, unprompted, on the same two-function split (`dense_bits(tensor) -> Vec<u32>` plus a thin `reference_bits(...)` evaluate wrapper around it) — which is independent evidence for where the natural seam sits.

### Decision-packet readiness gate

1. **Re-audit.** Done above: the coordinator's 36 reproduces exactly; its `tiler-reference` sub-count (13) was stale and is corrected to 19 with no change to the out-of-crate population or the decision. `reference_bits` at `tiler-conformance/tests/independent_backend/workload.rs` re-verified unchanged at this base.
2. **Options enumerated.**
   - **Status quo** — publish nothing; ~20 copies stay independently maintained.
   - **Narrow leaf-decode convenience** — `tiler-reference` publishes a function that takes an already-evaluated `Tensor` (or one `ReferenceOutput`) and returns its dense element bits. Does not touch evaluator construction or program/input assembly.
   - **Full evaluate+decode convenience** — publishes a function that also calls `ReferenceEvaluator` and picks an output index, folding in the boilerplate every full-shape site repeats.
   - **Complete replacement** — publish a convenience *and* migrate all ~20 existing sites to it in this change.
   - **Further bounded research** — prototype 2–3 signatures against a few real call sites before asking Tom.
   - **Deferral** — park behind `decide-the-backend-provider-conformance-harness-public-surface`. Explicitly excluded by this ticket's non-goals; the two decisions were deliberately split apart because this helper belongs to `tiler-reference`, not to any conformance facade.
3. **Eliminated.**
   - *Full evaluate+decode* is eliminated: it cannot avoid defaulting an evaluator profile. `bf16_vertical.rs` calls `ReferenceEvaluator::under(registry, conformance)`; every other site calls `ReferenceEvaluator::standard()`. A convenience that picked one would silently narrow or misrepresent the other caller's contract — exactly the "must not default … an evaluator profile" constraint this ticket states. It survives only if it takes the evaluator as an explicit argument, at which point it saves little over the narrow option and stops being a materially distinct choice.
   - *Complete replacement* is eliminated for this ticket: this is a decision ticket, not an implementation one, and its own non-goals say "editing `crates/` beyond what the decision needs, which for a no-publication answer is nothing" — migrating ~20 call sites is downstream implementation work that depends on Tom's accepted signature, not a prerequisite to deciding it. It becomes a follow-up ticket once a signature is accepted.
   - *Further bounded research* is eliminated: the open question is not empirical (no measurement would change the answer), it is which public shape Tom wants — a call I am not authorized to make per ADR 0075 and the Tom-retained-decisions list ("consequential public crate, module, trait, type, or call-site boundaries").
   - *Deferral* is eliminated by this ticket's own non-goals.
   - *Status quo* is not eliminated — it is dominated (see below) but remains presentable as the fail-closed baseline.
4. **Survivors compared.**

   | Dimension | Status quo | Narrow leaf-decode convenience |
   |---|---|---|
   | Correctness | No shared code to get wrong, but no shared code to fix either — a bit-order or width bug in one of ~20 copies stays local to it. | One decode path; a fix or a caught bug propagates everywhere at once. No behavior change for any existing site (every site's own logic is reproduced, not altered). |
   | Fail-closed strictness | Each site's own `panic!`/`.expect()` on non-Dense payload, uniformly enforced today by convention, not by a shared contract. | Same panic-on-non-Dense contract, now stated once; whether it stays a panic or becomes a typed error is itself part of the signature question below — not defaulted here. |
   | Maintainability | ~20 near-identical bodies across 6 crates/dirs (one, `normalize.rs`, re-derives it 3× in one file without even a local helper); two sites (`tiler-conformance/src/serial_sum.rs`, `prototypes/serial-sum-run/src/proof.rs`) already independently converged on the same two-function split. | Collapses ~20 bodies to 1 definition plus call sites; the width fork (u32 vs u16) is the one place it does not fully collapse without a signature choice (see Tom's question). |
   | Host runtime/memory | No difference — every site already allocates and iterates identically. | No difference. |

   Status quo is not worse on strictness or runtime, but the narrow convenience strictly dominates it on maintainability with no offsetting cost anywhere — this is not a close call.
5. **Frontier.** Two candidates survive elimination and the comparison narrows to one dominant answer: **publish a narrow leaf-decode convenience.** No trade-off remains between it and the status quo, so per the gate ("when one option dominates, recommend or take that option") this packet recommends publish rather than presenting it as an open choice. What remains open, and is Tom's alone under ADR 0075 ("must not default a bit order, a payload view, or an evaluator profile"), is the exact signature.
6. **Survivor's counterargument, reversal evidence, and perturbation.**
   - **Strongest counterargument.** 15 of the 17 out-of-crate sites are test/prototype/spike code, not shipping surface; a permanent `tiler-reference` public commitment (ADR 0075 review, a labelled draft Tom must accept and later un-default) is a heavier instrument than the roughly 6–10 lines it saves per call site, and non-shipping duplication is cheaper to carry than a public surface is to govern.
   - **Evidence that could reverse it.** If Tom judges that test/prototype/spike duplication is cheap enough to keep by policy — i.e., that `tiler-reference`'s public surface should stay at its current minimum until an external consumer exists (echoing this file's own ADR-0075-adjacent reasoning about compatibility not yet being a real cost) — the correct answer flips to status quo despite the maintainability numbers above, because the *decision* being made is about surface governance cost, not line count.
   - **Perturbation performed.** I checked whether the 17 sites are actually heterogeneous enough that "one convenience" is the wrong frame — different output-count handling, different error contracts, structurally different beyond decode. They are not: 16 of 17 share the exact `u32`/big-endian leaf shape verbatim, and the sole width outlier (`bf16_vertical.rs`) is mirrored by a second, independent bf16 copy inside `tiler-compiler/src/pipeline/tests.rs`, so the outlier is itself a small recurring case rather than a one-off that would argue against a shared helper.

### The one question for Tom

Given 16 of 17 out-of-crate sites share one exact `u32`/big-endian leaf-decode shape and 2 sites (one in `tiler-conformance`, one in `tiler-compiler`) independently need a `u16`/bf16 variant of the same shape: should `tiler-reference` publish

- **(A)** a monomorphic `pub fn dense_f32_bits(tensor: &Tensor) -> Vec<u32>` covering the 16 f32 sites verbatim, leaving the two bf16 call sites to keep their existing hand-written `u16` copies unconverted; or
- **(B)** a pair (or a width-generic form) that also covers the bf16 shape, at the cost of a second public item (or a const-generic/byte-array return type that pushes the final `from_be_bytes`/`from_le_bytes` interpretation back onto the caller so the helper itself never encodes a bit-order choice)?

Both take `&Tensor` (or one `ReferenceOutput`), not a `SemanticProgram` plus bindings — the full evaluate+decode shape was eliminated at step 3 above because it cannot state an evaluator profile without defaulting one. Whichever shape is accepted remains a labelled draft until Tom accepts its exact included/excluded surface per ADR 0075; migrating existing call sites onto it is separate follow-up work, not part of this decision.

**This ticket is not closed by this packet.** Per ADR 0075 and this repository's Tom-retained-decisions list, acceptance of a public `tiler-reference` signature is Tom's; no code was published or edited under `crates/tiler-reference/**` by this worker. Status moved to `awaiting-decision` so the packet is dispatchable to Tom's review queue.
