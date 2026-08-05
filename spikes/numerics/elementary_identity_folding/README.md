---
schema: "tiler-doc/v1"
id: "tiler.spike.numerics.elementary-identity-folding"
kind: "experiment"
title: "Elementary-identity folding probe"
topics: ["numerics", "transcendentals", "metal", "math-modes", "optimizer"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.numerics.elementary-identity-rewrite-dimension"]
entrypoints: ["spikes/numerics/elementary_identity_folding/probe.sh", "spikes/numerics/elementary_identity_folding/probe.metal", "spikes/numerics/elementary_identity_folding/identity_counterexample.py"]
last_verified: "2026-08-05"
ticket: "name-the-elementary-identity-rewrite-dimension"
---

# Elementary-identity folding probe

## The two named questions

**Would rewriting through an elementary function's own identity change the answer in binary32, and does the offline Metal compiler perform such a rewrite?** The identities in question are `exp(a) * exp(b) = exp(a + b)`, `log(a) + log(b) = log(a * b)`, and `sqrt(a) * sqrt(b) = sqrt(a * b)`.

The two halves are separate entrypoints and separate evidence, and each is worthless without the other. If the rewrite changed no result it would need no permission and the compiler's behaviour would not matter; if the compiler already performed it, a permission Tiler withheld would be withheld only on paper.

The questions decide a vocabulary question. [The elementary-identity rewrite dimension](../../../docs/research/numerics/elementary-identity-rewrite-dimension.md) proposes naming this freedom as a dimension of the numerical contract. Every dimension of that contract is a behaviour a target profile is asked whether it honours, and under [Numerical semantics](../../../docs/numerical-semantics.md#per-dimension-honourability-and-how-it-composes-with-feasibility) a profile that says nothing about a dimension leaves it `Unknown`, which never reaches an executable frontier. So a proposal to add the dimension has to say what a profile would declare about it and on what evidence. This directory is that evidence for one compiler.

## What it does and does not establish

**It establishes**, for one pinned offline compiler and six flag sets, which arithmetic each source spelling compiled to. That is a compile-side fact of the kind [the Apple GPU numerical behaviour record](../../../docs/research/apple-targets/numerical-behaviour.md) separates from delivered numerics.

**It establishes nothing about the runtime compiler.** A `.metallib` carries AIR; the AIR-to-GPU-ISA stage runs at pipeline creation and is not exercised here. Finding 30 of the numerical-behaviour record is the standing reason not to extrapolate: the runtime compiler was measured contracting a multiply/add pair whatever the offline selection said. Whether it also folds an elementary identity is `Unknown` after this probe, and the research record says so rather than reading this negative onto it.

**It establishes nothing about values.** No device is opened and no result bit pattern is compared. `air.exp.f32` being emitted twice says the compiler emitted two calls; it says nothing about what either returns.

**It establishes nothing about another compiler build, and it is not on the qualified row's toolchain.** The retained `environment.tsv` names the exact route: `xcode-select` on this host points at Xcode 27.0 build 27A5228h and `xcrun metal` resolves a downloaded `MetalToolchain` asset reporting `metalfe-32023.921`, where [the numerical-behaviour record's](../../../docs/research/apple-targets/numerical-behaviour.md) qualified row is Xcode 26.6 build 17F113 with an offline `metalfe-32023.883`. The neighbouring [transcendental emission probe](../metal_transcendental_emission/README.md)'s retained record is on that older offline build, so its rows and these are **not rows of one table** and a difference between them is not evidence of anything until one of the two is re-run on the other's toolchain. The build string `metalfe-32023.921` also happens to be what finding 8 of the numerical-behaviour record names as the macOS *runtime* compiler; a coincident build string is not the same artifact reached by the same route, and this probe exercised the offline driver alone.

**It installs nothing and mutates no toolchain component.** It reads the compiler already on the host and writes only into this directory.

## Reproduce

From **this directory** (no `make` target reaches `spikes/`):

```sh
./probe.sh                              # the compiler question
python3 identity_counterexample.py      # the arithmetic question
python3 -O identity_counterexample.py   # the same, with assertions stripped
```

### The arithmetic question

`identity_counterexample.py` evaluates `fl(exp(a)) * fl(exp(b))` against `fl(exp(a + b))` over the non-positive integer grid `[-40, 0]` in both arguments — the region the governed softmax's exponential admits, since its arguments are `s_i - m` against the row maximum and `SOFTMAX_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS` is `+0.0`. The exponential is **correctly rounded** to binary32, computed in `Decimal` at 120 digits and rounded once, so the disagreement it finds belongs to the identity rather than to a library's error. Standard library only; no dependency to pin and none claimed.

**502 of 1681 pairs disagree — 29.9%** — and the smallest-magnitude disagreement is at `a = b = -1.0`, which is the ordinary regime rather than an edge case:

| quantity | binary32 bits |
| --- | --- |
| `exp(-1.0)`, correctly rounded | `0x3ebc5ab2` |
| `fl(exp(-1.0) * exp(-1.0))` | `0x3e0a9556` |
| `exp(-2.0)`, correctly rounded | `0x3e0a9555` |

One ulp apart, under the strongest exponential any target could declare. **The freedom is therefore observable and needs a permission**, and no target's accuracy contract can make it unobservable.

**What this grid cannot detect.** Replacing the correctly rounded exponential with a host `math.exp` rounded from float64 to binary32 leaves every verdict unchanged — the two implementations agree on the whole grid — so the run distinguishes the identity's error from a *large* implementation error and not from a one-ulp one. The retained numbers are for a correctly rounded `exp` and must not be read as a bound over declarable ones.

### The compiler question

`probe.sh` compiles [`probe.metal`](probe.metal) to AIR under six flag sets and emits one row per `(flag set, kernel)` whose third column is a canonical **opcode signature**: every emitted AIR callee and every floating-point opcode in that kernel's body, sorted, with counts.

`governed` is the flag set [the workload profile](../../../docs/research/program-planning/first-metal-lm-workload.md) records as the qualified Apple9/F32 baseline. `governed-contracting` is the same set without `-ffp-contract=off`, present because it is where one of the positive controls fires. The remaining four are the compiler default and three non-governed modes, so the governed row is a comparison rather than an isolated observation.

Each kernel isolates exactly one spelling. Statements in one body would have been common-subexpression-eliminated against each other, which would make a spelling's count depend on which other spellings sat beside it — the first draft of this probe did exactly that and its counts were unreadable.

## How to read the record

The identity kernels come in pairs: the spelling that would be rewritten, and the spelling it would be rewritten into. **A fold is the first kernel's signature becoming the second's.** So `exp_product` and `exp_of_sum` differing is the finding, and their agreeing would have been the opposite finding — which is what perturbation 2 below demonstrates rather than asserts.

## Retained record

[`results/2026-08-05-identity-folding-msl4-macos26-metal32023.921/`](results/2026-08-05-identity-folding-msl4-macos26-metal32023.921) holds `record.tsv` from `probe.sh`, `counterexample.tsv` from `identity_counterexample.py`, and the `environment.tsv` that bounds both, including the SHA-256 of all three source files so a row can be attributed to the exact source that produced it. The records are positive claims that outlive their producer: only re-running detects drift from them. `counterexample.tsv` was verified byte-identical between the unoptimized and `-O` runs.

**The result, in one line: no pair folded, in any flag set.** Sixteen kernels across six flag sets, and every rewritable spelling kept the operations its source named. The only flag-dependent differences in the whole record are the precise-to-`fast_` intrinsic substitution the neighbouring probe already measured, and the two positive controls.

**The mechanism, read in the IR rather than inferred from the counts.** `ctl_exp_of_zero` computes `exp(0.0f) + a[t]`, and its signature is `air.exp.f32=1;fadd=1` in every flag set — the compiler does not even constant-fold the exponential of a literal zero. `air.exp.f32` is an opaque AIR intrinsic rather than an LLVM one, so the constant folder and the identity combiners that recognize `llvm.exp.f32` never match it. That is why the negative is uniform: it is structural at this stage rather than a policy the flags express.

## The checks can say no

Six deliberate perturbations were run on 2026-08-05 in scratch copies rather than in place. The unperturbed runs returned exit 0 before and after.

| Entrypoint | Perturbation | Result |
| --- | --- | --- |
| `probe.sh` | The `log_difference` kernel is deleted from `probe.metal` | exit 1, `population mismatch: emitted 90 rows, expected 96 (16 kernels x 6 flag sets)` |
| `probe.sh` | `exp_product`'s body is respelled as `exp(a[t] + b[t])` — the folded form, written by hand | `governed exp_product air.exp.f32=1;fadd=1`, byte-identical to `exp_of_sum`'s row |
| `probe.sh` | `-fmetal-math-mode=bogus` replaces the relaxed flag set | `metal: error: unsupported argument 'bogus'`, then exit 1, `population mismatch: emitted 80 rows, expected 96` |
| `identity_counterexample.py` | `GRID` is shrunk to 20, leaving the declared literal | exit 1, `population mismatch: evaluated 441 pairs, expected 1681` |
| `identity_counterexample.py` | The declared product bits are moved one ulp | exit 1, `smallest counterexample mismatch`, printing both records |
| `identity_counterexample.py` | The correctly rounded exponential is replaced by `math.exp` rounded from float64 | **exit 0** — the two implementations agree on this whole grid |

**The second is the one that matters and it is the reason the compiler probe is evidence rather than a tautology.** It shows what a fold would have looked like in the record: the rewritten spelling's signature becomes the target spelling's, exactly. Since no unperturbed pair does that, the record's negative is a reading of the compiler rather than a property of the counting method.

**The sixth is a limitation and is recorded as one rather than dropped.** It is the perturbation that did *not* fire, and its meaning is stated above under the arithmetic question: this grid separates the identity's error from a large implementation error and not from a one-ulp one.

Both population checks compare against bare literals — `DECLARED_KERNELS=16` in `probe.sh` rather than a count derived from `probe.metal`, and `DECLARED_PAIRS = 1681` rather than `(GRID + 1) ** 2` — following the discipline [`verify-sources.sh`](../../../docs/research/numerics/sources/verify-sources.sh) states at its own top and [the online-softmax bound probe](../online_softmax_bound/README.md) had to repair: a check whose two sides come from one source cannot say no. **The first of them caught its own author on its first execution** — the literal was written as 18 against 16 kernels, and the probe refused to publish.

## Two positive controls that fire, and one negative that explains the rest

- **`ctl_double`** computes `x + x`. Its signature is `fadd=1` under `governed` and `governed-contracting`, and `fmul=1` under all four relaxing sets — the compiler rewrites it to a multiply by two. So the counting method registers an algebraic rewrite, and registers one that is *mode-dependent*, which is the exact shape an identity fold would have had.
- **`ctl_muladd`** computes `x * b + x`. Its signature is `fadd=1;fmul=1` under `governed` and `llvm.fmuladd.f32=1` under `governed-contracting` — so `-ffp-contract=off` is load-bearing in the governed set, and the method registers a contraction too.
- **`ctl_exp_of_zero`** is the negative described above. It is not a control in the sense the other two are; it is the observation that gives the whole record its mechanism.

## Traceability

- **Supported claim:** [The elementary-identity rewrite dimension](../../../docs/research/numerics/elementary-identity-rewrite-dimension.md).
- **The derivation that produced the question:** [Certified rounding-error bounds as rewrite permissions](../../../docs/research/numerics/certified-bounds-as-rewrite-permissions.md), Part 2, which found the online-softmax rescaling fold consuming `exp(a) * exp(b) = exp(a + b)` and named it a freedom no dimension covers.
- **Neighbouring probe, different question:** [Metal transcendental emission probe](../metal_transcendental_emission/README.md) measures which intrinsic a *single* spelling selects. This one measures whether a *composition* is rewritten. Neither answers the other's question, and their retained records are on different compiler builds.
- **Normative owner:** [Numerical semantics](../../../docs/numerical-semantics.md).
- **Work record:** [`name-the-elementary-identity-rewrite-dimension`](../../../tickets/name-the-elementary-identity-rewrite-dimension.md).
