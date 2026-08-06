---
id: correct-the-silu-subnormal-fact-that-covers-only-the-negative-tail
title: Correct the SiLU subnormal fact that covers only the negative tail
status: done
priority: p2
dependencies: []
related: [apply-the-declared-numerical-conformance-on-every-reference-evaluation-path, derive-the-oracle-for-a-permitted-divergence-candidate, recompute-the-explain-request-qualifier-for-the-silu-subnormal-fact]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, semantics, subnormals]
---
## User-visible outcome

`tiler::silu-f32@1`'s declared subnormal fact says what the operation does over its whole domain, so a reader deciding whether a target's flush is observable for this family gets the right answer instead of one that holds only where the fact was measured.

## Why this exists

**Fact — the declared value overstates its own measurement.** `SILU_F32_FACT_SUBNORMALS` (`crates/tiler-ir/src/semantic/silu.rs:110`) resolves to `preserved-and-unreachable-no-binary32-silu-result-or-intermediate-is-subnormal`, and its doc justifies "unreachable" from one region of the domain: `silu(-88.7228)` is `0x82b173cc`, a normal value, and `silu(-88.73)` is already `0x80000000`, so "the result drops from normal straight to `-0.0` with no subnormal band between them".

**Measurement — the claim is false near zero, where the reference is `x / 2`.** Evaluated at `nightly-2026-07-19` on the reference's own pinned formula, `silu(0x007fffff)` is `0x00400000` and `silu(0x00400000)` is `0x00200000` — both subnormal results, from subnormal operands. Reproduce in one line:

```text
python3 -c "import struct,numpy as np
f=lambda i: np.float32(struct.unpack('>f',struct.pack('>I',i))[0])
b=lambda x: struct.unpack('>I',struct.pack('>f',np.float32(x)))[0]
x=f(0x007fffff); print(format(b(np.float32(x/np.float32(np.float32(1.0)+np.float32(np.exp(np.float64(-x)))))),'08x'))"
```

**Inference — the negative tail is a real measurement and the domain claim is a generalization of it.** The tail argument is about `x / (1 + e^{-x})` for large negative `x`, where the numerator's magnitude collapses faster than the subnormal band; it says nothing about small positive `x`, where the quotient is approximately `x / 2` and a subnormal argument therefore has a subnormal image.

**Fact — the reference no longer relies on the claim.** [`apply-the-declared-numerical-conformance-on-every-reference-evaluation-path`](apply-the-declared-numerical-conformance-on-every-reference-evaluation-path.md) applies both declared subnormal dimensions inside `silu_f32_under` and records the counterexample at that function's definition rather than trusting the fact. So this is a declaration defect with no live incorrect *evaluation* behind it, which is why it is filed rather than fixed in that branch: the fix is in `crates/tiler-ir`, outside its scopes.

## What this ticket must produce

- A declared value that is true over the whole binary32 domain. The obvious candidate is a `preserved` spelling with no reachability claim, but the wording is this ticket's to derive — a fact that says "unreachable" where a case exists is the failure mode being repaired, and a replacement that overstates in the other direction is the same defect.
- The doc comment carrying the counterexample bits beside the tail measurement, so the next reader sees which region each claim covers.
- A check that would have caught it: a case at the subnormal boundary of the argument domain, watched failing against the current spelling.
- The sweep for siblings. `SOFTMAX_F32_FACT_SUBNORMALS` and `RMS_NORM_F32_FACT_SUBNORMALS` both state a *reachable* divergence and are not suspect on this pattern; `BF16_FACT_SUBNORMALS` states preservation without a reachability claim. Read each in full and record the verdict rather than assuming from the shape.

## Explicit non-goals

Changing what the reference computes; changing the accuracy contract; any device measurement.

## Closes when

The declared fact is true over the domain it quantifies, the counterexample is recorded where the old claim was, and the new case has been watched failing against the old spelling.

## Graph maintenance

Filed by [`apply-the-declared-numerical-conformance-on-every-reference-evaluation-path`](apply-the-declared-numerical-conformance-on-every-reference-evaluation-path.md), which found the counterexample while deciding whether the SiLU family could be documented immune to both subnormal dimensions rather than made to apply them. It could not, and the fact is why the question was asked.

## Outcome

**The corrected fact, and what grounds each clause.** `SILU_F32_FACT_SUBNORMALS` now registers `preserved-by-this-contract-and-reached-as-a-result-near-zero-where-the-reference-is-x-over-two-and-as-the-subordinate-exponential-for-large-positive-arguments-and-flushed-on-a-declared-flushing-realization-a-recorded-divergence`, in the idiom the two sibling *reachable* facts already use (`SOFTMAX_F32_FACT_SUBNORMALS`, `RMS_NORM_F32_FACT_SUBNORMALS`, both read in full). Clause by clause: `preserved-by-this-contract` is the unchanged policy, and it is what `silu_f32_under` implements at the strict reading; `reached-as-a-result-near-zero-where-the-reference-is-x-over-two` and `as-the-subordinate-exponential-for-large-positive-arguments` are the two Measurements below; `flushed-on-a-declared-flushing-realization-a-recorded-divergence` is the ADR 0076 framing the siblings carry, and it is live rather than hypothetical because [the conformance-threading landing](apply-the-declared-numerical-conformance-on-every-reference-evaluation-path.md) applies both dimensions inside `silu_f32_under`. The word `unreachable` is gone and no replacement claim quantifies over a region that was not measured.

**Measurement — the near-zero region, exact bits, `nightly-2026-07-19`.** For `|x| <= 0x33000000` (`2^-25`) the correctly rounded `e^-x` is exactly `1.0`, because `e^-t > 1 - t` at every `t > 0` puts it strictly above `1 - 2^-25`, which is the rounding midpoint below `1.0` (the predecessor of `1.0` is `1 - 2^-24`). The divisor is therefore exactly `2.0` and the activation is `fl(x / 2)` over the whole region. Every argument from `0x00000002` to `0x00fffffe` in magnitude has a subnormal result. Boundaries: `silu(0x007fffff) = 0x00400000` (the filing's counterexample, reproduced); `silu(0x00800000) = 0x00400000` — a subnormal result from a **normal** operand, which the filing did not state and which is the stronger form of the defect; `silu(0x00fffffe) = 0x007fffff`, the largest subnormal result, with its successor `silu(0x00ffffff) = 0x00800000` back in the normal range; and `silu(0x00000001) = 0x00000000`, round-to-nearest ties-to-even landing on zero rather than a flush.

**Measurement — a second false clause the filing did not name.** The old value claimed no *intermediate* is subnormal either, and that is false in its own region: the subordinate `e^-x` is subnormal from `0x42aeac50` (`87.3365478515625`) through `0x42cff1b4` (`103.97207641601562`), and exactly `+0.0` above. It is unobservable in the result — `fl(1 + subnormal)` is exactly `1.0`, so the activation returns `x` — but it is a value an arithmetic unit produces, which is the site a flush policy acts on. The corrected fact names it; a spelling that repaired only the near-zero region would have generalized from one region exactly as the old one did, which is why the new test asserts both clauses separately.

**Measurement — the tail claim was true, and is now bounded instead of sampled.** The last argument with a finite divisor is `0xc2b17217`, the negation of `SILU_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS`, giving `0x82b1726d` (about `2.6e-37`, over twenty times the minimum normal); its successor `0xc2b17218` overflows the exponential and gives exactly `0x80000000`. The filing's cited pair reproduces: `silu(0xc2b17213) = 0x82b173cc` and `silu(0xc2b175c3) = 0x80000000`. Two samples cannot establish a region, so the retained test states the bound rather than the samples: a finite divisor is `fl(1 + e)` for finite non-negative `e` and is therefore at most `f32::MAX`, so no argument at or beyond the ceiling can have a subnormal image. Both measurements were computed twice by independent routes that agree — exact-rational (`mpmath` at 400 bits, `fractions.Fraction`, correctly rounded binary32 with explicit ties-to-even) and host `f64` exp narrowed to binary32.

**The version question, settled by reading each owning site rather than assumed. No encoding version moved.** `tiler.semantic-definition-projection.v5` (`crates/tiler-ir/src/semantic/registry.rs:1784`) and `tiler.semantic-registry.v7` (`:2673`) count *rendering* revisions; this change adds, removes, and reorders nothing. `encode_operation_definition` (`:2828`) writes `definition.canonical_facts().value().encode(output)`, and `CanonicalValueData::Utf8` encodes as tag `7` followed by `push_slice` (`crates/tiler-ir/src/semantic/types.rs:996-999`), a self-delimiting length-prefixed payload — so a payload of any length stays injective under the unchanged rendering. The standard semantic provider stays at revision 7 on its own documented rule (`registry.rs:2240-2255`): the revision moves only "for a change this registry's *content* encoding cannot already carry", the projection already folds every definition's facts, and "bumping it for a change the projection already carried would invalidate every pinned provenance for an authority change that did not happen".

**One pinned identity moves, it is known and measured, and this branch deliberately leaves it unmoved.** `explain::tests::deterministic_trace_is_sealed_and_rendered_separately`'s request qualifier (`crates/tiler-compiler/src/explain.rs:4134`) moves `f3244b2242ebcb5c` → `6dd42be71c6745fe`. Two independent runs report the identical `left`, and it is the only failure in a full-workspace run: `2727 tests run: 2726 passed, 1 failed, 7 skipped`. Reproduce with `cargo nextest run -p tiler-compiler -E 'test(deterministic_trace_is_sealed_and_rendered_separately)'` and read the reported `left`. **This branch therefore leaves the workspace gate red at exactly that one assertion and nothing else**, which is a stated condition of the dispatch rather than an unfinished step: `crates/tiler-compiler/**` is `implementation/compiler`, held exclusively by the live claim `reach-a-verified-kernel-through-the-structural-families` (`assignee: agent-structural-r6`, read from its own branch with `git show tkt/reach-a-verified-kernel-through-the-structural-families:tickets/reach-a-verified-kernel-through-the-structural-families.md`). The disjointness verdict against that holder is **vacuous, not clear** — the branch has zero commits (`git rev-list --count $(git merge-base main tkt/reach-a-verified-kernel-through-the-structural-families)..tkt/reach-a-verified-kernel-through-the-structural-families` → `0`), so its diff evidences nothing about what it will touch. The recompute is filed as [`recompute-the-explain-request-qualifier-for-the-silu-subnormal-fact`](recompute-the-explain-request-qualifier-for-the-silu-subnormal-fact.md), following the shape [the BF16 law worker used](recompute-the-explain-request-qualifier-for-the-bf16-realization-rows.md); the integrator sequences the two and recomputes the value on the merged tree rather than copying this one.

**The pin population was surveyed before editing, and the full run confirms it.** `grep -rnE '"[0-9a-f]{16}"|request=[0-9a-f]{16}' crates/ --include='*.rs'` returns four hits, three of which are hex-digit alphabet tables (`delivery.rs:519`, `region.rs:1897`, `digest.rs:152`) and one of which is the qualifier above. `grep -rlE '\b[0-9a-f]{64}\b' crates/ --include='*.rs'` returns eight files, all external specification digests, tensor result payload digests, Metal source digests, or SHA-256 test vectors — none folds the semantic snapshot. The whole-workspace run is the check that no other pin folds these bytes, and exactly one did.

**Sibling check — the find-one-check-all sweep, each read in full, no second defect found.** `SOFTMAX_F32_FACT_SUBNORMALS` (`softmax.rs:329-337`, value at `:611`) states a *reachable* divergence with a measured subnormal exponential `0x00b33687` and is not suspect on this pattern. `RMS_NORM_F32_FACT_SUBNORMALS` (`rms_norm.rs:141-150`, value at `:449`) likewise states a reachable divergence, justified from a row of `1e-40` squaring to `+0.0`. `BF16_FACT_SUBNORMALS` (`bf16.rs:104`) carries two values, `preserved-every-subnormal-encoding-denotes-a-distinct-constant` for the constant and `preserved-operands-and-results-in-the-bf16-subnormal-range-are-not-flushed` for the arithmetic; both state preservation with **no** reachability claim, so neither can overstate in the way this ticket repaired. Verdict: SiLU was the only fact in the family asserting unreachability, and it was the only one wrong.

**The check watched failing, four perturbations applied, run, and reverted.** Each fired at its intended assertion rather than at an opaque diff, which is why the region assertions are ordered before the exact-value one.

1. Fact reverted to `preserved-and-unreachable-no-binary32-silu-result-or-intermediate-is-subnormal` → `FAIL … the band is reached in two regions, so no spelling may declare it unreachable: preserved-and-unreachable-…`.
2. Fact replaced with a **one-region-only** spelling that repairs the near-zero region and drops the exponential one → `FAIL … the subordinate exponential is itself subnormal for large positive arguments: preserved-by-this-contract-and-reached-as-a-result-near-zero-…`.
3. The near-zero divisor set to `1.0`, modelling the wrong reading that the reference near zero is `x` → `FAIL … 1.0 + 1.0 is exact, so the divisor is exactly 2.0  left: 1.0  right: 2.0`.
4. `SILU_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS` moved to `0x3f800000` so `ceiling / f32::MAX` falls inside the subnormal band → `FAIL … the smallest magnitude the tail can produce is 2.938736052218037e-39 …`. This is what shows the tail bound is doing work rather than holding tautologically.

**A check that could not say no, caught by perturbing it, and repaired.** The exponential-band clause was the one part of the corrected fact that nothing in the tree could refute, so `the_subordinate_exponential_enters_and_leaves_the_subnormal_band_at_the_stated_bits` was added to bracket both ends against `126 ln 2` and `150 ln 2` — the exact inequalities that decide the band — using `std::f64::consts::LN_2` rather than a host exponential, whose `~1e-17` error sits thirteen orders of magnitude below the `7.6e-6` binary32 gap being separated. **Its first draft passed a perturbation it should have failed:** naming both ends of each bracket as independent literals asserts only that the threshold lies *somewhere* between them, so moving the lower end outward by one ULP (`0x42aeac4f` → `0x42aeac4e`) still passed. Repaired by deriving each neighbour from the boundary under test (`bits - 1`, `bits + 1`), which makes widening either end move the boundary and break the other side. Re-perturbed in all four directions and each now fails at the named assertion: `0x42aeac50` → `0x42aeac4f` and → `0x42aeac51`; `0x42cff1b4` → `0x42cff1b3` and → `0x42cff1b5`. This is the AGENTS.md "prove every check can say no" step finding a real weakness rather than confirming one, and it is recorded because the first draft is the shape to avoid.

**Scope.** The branch touched `crates/tiler-ir/src/semantic/silu.rs`, `crates/tiler-ir/src/semantic/silu/tests.rs` (`implementation/ir`, exclusive, held) and three files under `tickets/` (`project/tickets`, shared). `implementation/compiler` was deliberately **not** declared, because nothing under `crates/tiler-compiler/**` was edited — declaring a scope this branch does not touch would collide with a live exclusive holder for no work.
