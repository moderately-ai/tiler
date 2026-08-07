---
id: survey-what-belongs-in-the-conformance-crate
title: Survey what belongs in the conformance crate
status: done
priority: p2
dependencies: []
related: [admit-the-conformance-crate-to-the-workspace, decide-where-a-device-reaching-conformance-test-may-live]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [research, conformance, architecture]
---
## User-visible outcome

A read-only survey that names, ticket by ticket, what should move into `crates/tiler-conformance` and what should stay where it is — so migration happens as reviewed decisions rather than as drift, and the crate's boundary is derived from the work rather than asserted.

## Why this exists

**Asked for by Tom on 2026-08-07**, in the same answer that admitted the crate: assess what else should move and file it as future tickets. The crate was accepted on the argument that conformance is a *missing component* rather than a homeless file, and that argument makes a claim this ticket has to test — that the work currently scattered across four crates belongs together.

**Fact — the scattering is measurable.** Five open conformance tickets and no two share a scope set: `route-the-contraction-conformance-through-the-staged-oracle`, `route-the-index-region-conformance-through-the-staged-oracle` and `retain-the-selected-semantic-candidate-for-the-conformance-oracle` are `implementation/compiler`; `retain-contraction-conformance-evidence` adds `implementation/reference`, `contracts/numerics` and `research/scheduling`; `conform-the-bf16-vertical-end-to-end` adds `implementation/runtime`. Conformance tests exist under `crates/tiler-compiler/tests/`, `crates/tiler-reference/tests/`, and `crates/tiler-build/tests/`.

## What this must produce

**A classification of every candidate, with the reason, and it must be willing to conclude "stays".** A survey that recommends moving everything it looked at has not discriminated. For each candidate name which of these it is:

- **Cross-layer executed evidence** — a run spanning produce and consume, compared against the reference oracle. Moves.
- **Layer-local evidence** — a test of one crate's own behaviour that happens to use the word conformance. **Stays**, and the survey says so explicitly, because the crate's stated anti-goal is becoming the place tests go when nobody wants to decide.
- **Oracle *plumbing* rather than evidence** — machinery currently inside `tiler-compiler` because there was nowhere else. This is the interesting class and the one the missing-component argument rests on. Decide whether it is genuinely conformance machinery or genuinely compiler machinery; both answers are available and the survey must argue rather than assume.

**Then the harder question, which is the crate's actual long-term shape.** The accepted decision describes conformance as the refuting half of a declaring/refuting pair, whose eventual job includes *producing* support-matrix rows from runs that happened rather than having them hand-asserted in markdown. `AGENTS.md` records that documentation has no automated validator and that a ticket advancing a support-matrix row must remember to file the ledger update — a "must remember" that has already produced measurable drift. Assess, without building anything:

- Which support-matrix and ledger cells could be **derived** from an executed run carrying its host, OS build, toolchain, GPU family, and extent, and which are claims no run can make.
- Whether the maturity ladder (reserved type / architectural seam / implemented support / tested guarantee) and the evidence ladder (`SoundProof` / exhaustive finite / empirical / normative / `Unknown`) can be **stamped** by a harness or must stay a writing convention. Name what each would require.
- What the crate would need to parameterize over as targets multiply — the matrix is `operation family x dtype x contract x target profile x shape class`, and deferred work already names iOS profiles, a CPU vector tier, subgroup tiers, and CUDA. Hand-written per-combination tests do not survive that multiplication; say what shape does.

## Explicit non-goals

**Move nothing.** This is a read-only survey; every migration it recommends is a separate ticket that a later change executes. Do not design the harness API — the crate's public surface is reserved under ADR 0075 and admitting the member accepted no API. Do not re-open where the crate lives; that is decided. Do not build the support-matrix derivation; assess its feasibility and cost.

## Required evidence

Read the candidates in full rather than classifying from titles or grep — the argument for the crate came from reading, and a survey that classifies from names would be weaker evidence than the thing it is checking. Cite each candidate by path and say what reading it established. Where a classification is genuinely uncertain, say so and name what would settle it, rather than picking to keep the table tidy.

## Closes when

Every candidate is classified with its reason, the "stays" population is non-trivial or its emptiness is argued, each recommended migration is a filed ticket, the long-term questions above are answered or recorded as deferred with triggers, and the crate's stated boundary is either confirmed by the survey or revised with evidence.

## Graph maintenance

Filed 2026-08-07 by the coordinator on Tom's instruction. Deliberately not blocking [`admit-the-conformance-crate-to-the-workspace`](admit-the-conformance-crate-to-the-workspace.md) or the BF16 vertical: the crate's first content is already decided, and holding it behind a survey would invert the smallest-useful-slice order the admission ticket was scoped for.

## Survey — 2026-08-07

Read-only. Nothing under `crates/`, `docs/`, `spikes/`, or `prototypes/` was edited; no test was moved; `make full` was not run. Every candidate below was read in full with the file reader before it was classified, except where the entry says otherwise and says why.

### The headline finding, which reframes the crate's justification

**Fact — the cross-layer executed run the crate was admitted to hold already exists, and it is outside the gate.** `prototypes/serial-sum-run/src/proof.rs` is 8,159 lines. Its `run()` narrative dispatches one declarative tensor program on the GPU by two independent paths — a direct in-memory `metallib` and an envelope decoded by `tiler-runtime` — and compares both against `tiler-reference`'s evaluation of the same semantic program. `prototypes/serial-sum-run/Cargo.toml` declares `tiler-artifact`, `tiler-build`, `tiler-compiler`, `tiler-ir`, `tiler-metal`, `tiler-metal-aot`, `tiler-reference`, `tiler-runtime`, and a `cfg(target_os = "macos")` `metal` edge: the same eight-plus-one row [ADR 0106](../docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md) gives `tiler-conformance`.

That run is reached only by `cargo run -p tiler-prototype-run -- --artifact <base>`. `Makefile`'s `test` target is `cargo nextest run --workspace --locked` plus `cargo test --workspace --doc --locked`; `full` adds rustdoc, a release run of `tiler-reference` and `tiler-compiler`, `ticketsplease lint`, and `shellcheck`. **No target invokes that binary.** `grep -n "prototype-run" Makefile` returns only the two `--exclude` flags on the Clippy line, which is the second half of the same fact: the `lint` recipe excludes all three prototypes from `cargo clippy -- -D warnings`.

What is gated in that file is `#[cfg(test)] mod tests` from `:5402`, whose own header states it is a loader fixture that "reaches no device", substituting a synthetic payload for a real `xcrun` link. The device half runs nowhere automatic.

**So the missing-component claim is confirmed, and for a stronger reason than ADR 0106 argued.** The record's Context reasons forward from an absence — a device-reaching run "had no legal home". The measurable position is worse than that: the run exists, it carries the corpus's only device observation of a permitted reassociated answer and the only executed match against a retained device digest, and it lives in the one tree the repository deliberately holds to a lower standard, behind a command someone has to remember to type. `carry-the-device-executed-value-proof-into-the-conformance-crate` is that migration.

### Where the stated evidence is weaker than advertised

**The scope-set clause is false as written, and the conclusion survives.** ADR 0106's Context and this ticket's own "Why this exists" both say the five open conformance tickets are such that "no two share one" scope set. Read from the tickets:

| ticket | scopes | shared |
| --- | --- | --- |
| `route-the-contraction-conformance-through-the-staged-oracle` | `implementation/compiler` | `project/tickets` |
| `route-the-index-region-conformance-through-the-staged-oracle` | `implementation/compiler` | `project/tickets` |
| `retain-the-selected-semantic-candidate-for-the-conformance-oracle` | `implementation/compiler` | `project/tickets` |
| `retain-contraction-conformance-evidence` | `implementation/reference`, `implementation/compiler`, `contracts/numerics`, `research/scheduling` | `project/tickets` |
| `conform-the-bf16-vertical-end-to-end` | `implementation/reference`, `contracts/numerics`, `implementation/runtime`, `implementation/conformance` | `project/tickets` |

Three of the five carry **identical** scope sets, and all three are about one compiler-resident file. The true statement — the five span five distinct scopes with none common to all, and the two that actually scatter are the last two — supports the same conclusion. `correct-the-scope-set-claim-in-adr-0106-s-missing-component-evidence` carries the rewording, because this ticket cannot edit `docs/decisions/`.

**The population is larger than five.** `grep -ril conformance tickets/` returns 289 files, 283 of them tickets; 76 are non-terminal. Twenty-seven carry `conform*` in the title. That does not weaken the argument — it is the same argument with a bigger population — but a reader auditing "five" against the tree will not find five.

### Classification

Cross-layer executed evidence **moves**. Layer-local evidence **stays**. Oracle plumbing is argued individually.

| Path | Class | Verdict |
| --- | --- | --- |
| `prototypes/serial-sum-run/src/proof.rs` — the `run()` narrative and its comparisons | Cross-layer executed | **Moves** |
| `prototypes/serial-sum-run/src/buffer.rs` | Cross-layer executed (the FFI half) | **Moves**, as the precedent for the crate's single unsafe module |
| The six L3 cells' retained `result_sha256` against executed results | Cross-layer executed | **Moves** (one cell exists; five do not yet) |
| `prototypes/serial-sum-run/src/proof.rs` `#[cfg(test)] mod tests` | Layer-local | **Stays out**; relocates to `crates/tiler-runtime/tests/` |
| `crates/tiler-compiler/src/pipeline/conformance.rs` | Oracle plumbing | **Stays** — compiler machinery |
| `crates/tiler-compiler/src/governed/contraction_conformance.rs` | Oracle plumbing | **Stays** — compiler machinery, with one leg transferring later |
| `crates/tiler-reference/src/conformance.rs` | Oracle plumbing | **Stays** — oracle machinery |
| `crates/tiler-reference/src/value_conformance.rs` | Oracle plumbing | **Stays** — oracle machinery |
| `crates/tiler-reference/src/structural.rs`, `src/oracle.rs` | Not a candidate | **Stays** — matched on the word alone |
| `crates/tiler-ir/src/semantic/conformance.rs` | Not a candidate | **Stays** — a different sense of the word |
| `crates/tiler-reference/tests/contraction_conformance.rs` | Layer-local | **Stays** |
| `crates/tiler-reference/tests/structural_conformance.rs` | Layer-local | **Stays** |
| `crates/tiler-reference/tests/slice_conformance.rs` | Layer-local | **Stays** |
| `crates/tiler-reference/tests/concatenate_conformance.rs` | Layer-local | **Stays** |
| `crates/tiler-reference/tests/contraction_profile_cells.rs` | Layer-local, adjacent to a moving claim | **Stays** — see the split below |
| `crates/tiler-compiler/tests/bf16_numerical_contract.rs` | Layer-local | **Stays** |
| `crates/tiler-build/tests/custom_backend/` | Layer-local | **Stays** |
| `crates/tiler-runtime/tests/adapter_route/` | Layer-local | **Stays** — and moving it would destroy its claim |
| `crates/tiler-runtime/tests/identity_join/` | Cross-layer executed, but **stays** | The one genuinely hard case; argued below |
| `crates/tiler/tests/facade/pass/inline_region_executes.rs` and siblings | Layer-local | **Stays** — already decided by ADR 0106 item 3 |
| `prototypes/serial-sum-compile/tests/determinism.rs` | Layer-local (producer) | **Stays** |

### The "stays" population, and why it is not a default

Eighteen of the twenty-one rows stay, and the arguments are not interchangeable.

**Four files stay because they say so themselves, and reading confirmed the disclaimer is accurate.** `crates/tiler-reference/tests/contraction_conformance.rs` (379 lines) states "What a pass here is not. It is evidence about the semantic contract and the host reference evaluator. It is not evidence about any schedule, any lowering, any compiled kernel, any device, or any model-level tolerance; no such thing is exercised." Its imports are `tiler_ir::semantic`, `tiler_ir::shape`, and `tiler_reference` — nothing else is reachable. `structural_conformance.rs` (487), `slice_conformance.rs` (378), and `concatenate_conformance.rs` (381) carry the identical "any compiled or executed realization" exclusion and the identical import set. These four are the crate's third anti-goal stated positively: a test of one layer's own behaviour whose failure names the layer that broke.

**`crates/tiler-compiler/tests/bf16_numerical_contract.rs` (760 lines) stays for the same reason and matters more, because BF16 is the crate's first content.** Its header: "**It is not evidence that BF16 executes.** Nothing below the request boundary realizes BF16 [...] Every positive answer here stops at the recognizer's `dtype-f32` rule, which is asserted rather than avoided precisely so a reader cannot mistake feasibility for support." When `conform-the-bf16-vertical-end-to-end` lands its device run, this file does not follow it: the two answer different questions and the ledger's `Conformance evidence` cell already distinguishes them.

**`crates/tiler-build/tests/custom_backend/` (3,249 lines across five files) stays because it executes nothing and consults no oracle.** Its header states it is "an integration test on purpose. It compiles against `tiler-build`'s public surface alone, so a `pub(crate)` item is unreachable here in exactly the way it is unreachable to a consumer". `grep -rn tiler_reference crates/tiler-build/tests/` returns nothing, and `crates/tiler-build/Cargo.toml` has no `[dev-dependencies]` section at all. It produces, publishes, and re-accepts from cache; a run that never executes is not cross-layer executed evidence.

**`crates/tiler-runtime/tests/adapter_route/` (5,375 lines across four files) stays, and moving it would destroy what it proves.** `crates/tiler-runtime/Cargo.toml`'s comment above `[dev-dependencies]` is the whole argument, and it is checkable: the dev-dependency set is exactly `tiler-ir` and `tiler-reference`, and the comment records that "Neither reaches `tiler-compiler`: a loader that could rebuild a plan instead of validating the one it was handed is the boundary the crate split exists to enforce, and that stays true of its tests." `tiler-conformance` depends on `tiler-compiler` normally, so the same file compiled there would stop being evidence for the thing it is evidence for. ADR 0106's own elimination of `crates/tiler-runtime` as a *home* is the same fact read from the other end.

**`crates/tiler-runtime/tests/identity_join/` (1,849 lines across four files) is the genuinely hard case, and it stays.** By ADR 0106's definition it is cross-layer executed evidence: a producer program (`crates/tiler-build/examples/identity_join_producer.rs`) compiles a plan, translates it, publishes through the cache seam, and writes an envelope; this suite loads those bytes in a different process, executes them, and — its header's words — "routes those bytes to a result it checks against `tiler-reference`". Produce, consume, oracle. It is the pattern.

It stays because its central claim is a *negative* one about its own dependency closure, and that claim is only true where it currently sits. `the_consumer_links_no_compiler_emitter_or_build_provider` proves from the resolved dependency graph that this process links no compiler, no emitter, no AOT driver, and no build-time provider — "it *cannot*", as the header puts it. In `tiler-conformance` it demonstrably can, so the test would either be deleted or would assert something false. The generalizable rule this case yields: **a cross-layer run whose claim includes what the consumer cannot reach belongs in the crate whose manifest makes it unreachable.** That is a second class the crate's three anti-goals do not name, and it is worth carrying into the record.

**Two files matched the grep on the word alone and are not candidates.** `crates/tiler-reference/src/structural.rs` (448) is the reference evaluator for the four structural families; "conformance" appears only in its prose explaining why the declared numerical conformance is deliberately *not* read there. `crates/tiler-ir/src/semantic/conformance.rs` (1,938) is resolved-value binding conformance — the contract, scan, and evidence for whether a bound value implements its declared type. Neither is a test and neither is about cross-layer agreement.

**`crates/tiler-reference/src/conformance.rs` (494) and `src/value_conformance.rs` (298) are oracle plumbing that is genuinely the oracle's.** The first defines `ReferenceNumericalConformance`, the type that tells the evaluator which numerical contract it is computing under, and refuses by name a realization whose result is a set rather than one value. The second is the ledger of conformance proofs one evaluation holds. Both are production code in the crate ADR 0106 item 1 names as the authority. Moving either would be the authority substitution the first anti-goal refuses — it would put the statement of what the oracle computes outside the oracle.

### The oracle-plumbing class, argued

Both compiler-resident files are `#[cfg(test)]` modules (`crates/tiler-compiler/src/pipeline.rs:2299` and `crates/tiler-compiler/src/governed.rs:2246`), so neither ships. Both call the reference oracle. Both would be plausible conformance content on their names alone. Both stay, for different reasons.

**`crates/tiler-compiler/src/pipeline/conformance.rs` (1,618 lines) is compiler machinery.** Read in full. It is the target-neutral optimizer conformance gate: it drives the ordinary `compile()` entry point, registers an out-of-crate semantic provider and an out-of-crate lowering provider through the public `capability` surface, and asserts complete legal covers, occurrence identities, explain dispositions, and refusal rules. Where it reaches the oracle — `a_published_and_consumed_intermediate_compiles_and_agrees`, `outputs_reading_input_subsets_compile_and_bind_the_inputs_they_read`, `the_activation_compiles_and_matches_the_reference_bit_for_bit` — the *execution* is `interpret_fused`, a KIR interpreter defined `pub(super)` at `crates/tiler-compiler/src/pipeline/tests.rs:633` beside a `KirMachine` at `:266`. No device, no artifact, no runtime.

Two things follow. Its subject is the compiler's own output, checked one layer below where a target exists; and moving it would require exporting the KIR interpreter, `crate::cover`, `crate::region`, `crate::explain`, `crate::request`, and `crate::physical` from the compiler — a public namespace expansion that is Tom's under ADR 0075, bought for a relocation that adds no evidence. `docs/correctness-and-testing.md` names this module by name as the optimizer conformance gate and cites six of its test functions as the discharged precondition for accepting the public compiler facade; the gate and the crate that it gates are the same crate on purpose.

**`crates/tiler-compiler/src/governed/contraction_conformance.rs` (763 lines) is compiler machinery today, with one leg that transfers later.** Read in full. It compares three computations of the same contraction: the region `refine_index_region` actually returned for the occurrence, executed by the reference's index-region oracle; the registered reference evaluator; and the retained `result_sha256` of the L3 probe's `direct` kernel, measured on an M4 Max.

The third leg is the interesting one and it is the reason this file is the strongest argument *against* its own placement — and still not enough. It is a comparison against a **transcribed** device measurement, not an executed one; the file reconstructs the probe's operands from its `SplitMix64` stream and hand-writes a SHA-256 because "adding `sha2` to this crate would edit `Cargo.lock`, which this work does not own". That is exactly the "refuted by hand in a spike and transcribed into the declaration manually" pattern ADR 0106's Context offers as the worked example of the missing half.

It stays anyway, and the discriminator is what it needs to run: `crate::legality::refine_index_region`, `super::governed_lowering_capabilities`, `super::governed_scalars`, `super::governed_realization_laws`, and `crate::capability::LoweringSignature` — all crate-private. Its subject is *what the governed lowering emitted*, which only the compiler can produce. What transfers, when `route-the-realization-conformance-half-into-the-conformance-crate` lands, is the retained-digest leg: once a dispatched device comparison exists, comparing an emitted region against a transcribed digest stops being the best available evidence and becomes a duplicate of a better one. The file's own module doc already says so — "Closing that gap needs a dispatched device comparison rather than a third host implementation."

**Uncertainty, stated.** I am confident about the first file and less so about the second. What would settle it is the shape `route-the-index-region-conformance-through-the-staged-oracle` chooses: if that ticket concludes the compiler declines the cost of comparing the emitted region at the remaining cells, the file's residue is a lowering-refinement test with no digest in it and the question closes as "stays, whole". If it concludes the comparison is worth paying for, the file acquires more retained-digest surface and the case for splitting it strengthens. That ticket should be worked before anyone acts on this row.

**`crates/tiler-reference/tests/contraction_profile_cells.rs` (1,265 lines) stays, and the split is worth naming precisely.** Read in full. It runs both reference oracles over the six L3 cells and compares each against the retained `direct` digest. It reads as cross-layer evidence and is not: every computation in it is a *host* computation, and the device appears only as a 64-character constant. It is the reference crate proving its own staged oracles reach cells its unstaged bound refuses — which is layer-local, and its `#[ignore]` cost annotations (10.8 s dev for six cells, 64.4 s for the staged region walk) are the reference crate's costs to bear. `retain-contraction-conformance-evidence`'s two halves land on exactly this line: its *reference conformance* half is this file and stays; its *realization conformance* half — the same digests against an **executed** result, declining on a non-matching environment row — is the filed migration.

### Confirming the three anti-goals

All three survive the survey, and one gains a fourth sibling.

- **Not a second semantic authority** — confirmed, and it is what keeps `crates/tiler-reference/src/{conformance,value_conformance,oracle,structural}.rs` where they are. It is also the reason the target-multiplication answer refuses per-target expected-value tables.
- **Not a benchmark harness** — confirmed and untested by this survey; nothing in the candidate set is a benchmark. Worth noting that the migrating run *carries* wall-clock prints (`println!` of ms and ns/step in the profile-cell tests and the L3 comparisons); those are cost annotations attached to `#[ignore]` decisions, not measurements, and they must not become the seed of a benchmark suite in the new home.
- **Not a home for layer-local tests** — confirmed, and it is doing most of the work: eighteen of twenty-one rows stay under it.
- **Proposed fourth, from the `identity_join` case**: *not a home for a run whose claim is about what its consumer cannot reach.* Such a run is cross-layer and executed and still belongs in the crate whose manifest makes the negative true. This is not a revision of the three — it is the case the third anti-goal's wording does not cover, because `identity_join` is not layer-local.

### The long-term half

**Which ledger cells a run can derive.** One column, and it is the one with a measured drift record. `docs/dtype-support.md`'s physical/execution matrix has nine columns; eight report what authority exists at a layer, which is a fact about source and accepted decisions that no run observes — a run can only fail when one is missing. The ninth, `Conformance evidence`, reports whether a checked run composed the layers, which is what a cross-layer comparison observes about itself. That is also the cell `decide-whether-the-bf16-conformance-evidence-cell-overstates` found reading a bare `tested guarantee` — the same two words as the `f32` cell, which rests on a device-executed thirty-case comparison — while nothing had dispatched a BF16 kernel. A run additionally carries three qualifiers every tested cell is supposed to have and several do not: the operation set it covered, the environment row it is bounded to, and whether the measured half was available at all.

**Whether the ladders can be stamped.** The maturity ladder cannot and should not: three of its four rungs describe states with no run at all — a reserved type and an architectural seam are precisely the cases where nothing executes — and the top rung's own rule is a scoping judgement ("a tested guarantee must not cover an untested composition") rather than a fact about a test passing. It also has three spellings in the corpus (`AGENTS.md:56`'s four claims; `docs/dtype-support.md:21`'s five values with `implemented mechanism` for `implemented support`; `docs/roadmap.md:441`'s seven R1–R7 rungs) and no Rust representation anywhere. What a harness *can* do is refuse to let a `tested guarantee` cell exist with no run naming it — a comparison, not a stamp.

The evidence ladder can be *reported* but must not be *assigned*, and cannot yet be either. It already exists as typed values the producing authorities mint, but as four disagreeing types: `ConformanceEvidenceClass` (`crates/tiler-ir/src/semantic/accuracy/evidence.rs:50`, `pub`, five variants, top rung spelled `FormalProof`), `FusionEvidenceClass` (`crates/tiler-compiler/src/fusion_legality.rs:90`, `pub(crate)`, `SoundProof`, two variants reserved and never constructed), `IndexDomainEvidence` (`crates/tiler-ir/src/index/predicate.rs:89`, no `NormativeGuarantee` rung, `Empirical` documented as never emitted), and `EvidenceBasis` (`crates/tiler-compiler/src/explain.rs:379`, `pub(crate)`, seven variants including `CheckedInvariant` and `Assumption`). A conformance run reporting the class its inputs carried is sound; a run *choosing* a class would be the second authority the first anti-goal refuses. Reconciling the four, or declaring their differences deliberate, is a prerequisite and a public-boundary question under ADR 0075. `derive-the-conformance-evidence-ledger-cells-from-executed-runs` carries all of this, deferred with a trigger.

**What survives target multiplication.** Not hand-written per-combination tests, and not a combinatorial generator either — the matrix is sparse and non-monotone by design, so enumeration would produce mostly refusals and would move the knowledge of which cells are meaningful out of the declaration and into the harness. What survives is a run declared as a *value*: operation set, dtype, contract key, shape class, and the environment row it is bounded to, driven by one executor with the target profile supplied rather than baked in. Three properties make it work — the environment row is an operand so "this does not apply here" is a stated outcome rather than a `#[cfg]`; the oracle stays singular, with the per-target variation being the declared numerical realization applied to the reference before comparison rather than a per-target expectation table; and a refusal is a recorded case outcome, so an unimplemented cell is reported by the corpus rather than by absence from it. `shape-the-conformance-corpus-for-target-multiplication` carries it, deferred against the admission of a target profile outside the macOS Apple9 Metal family.

### Filed

- [`carry-the-device-executed-value-proof-into-the-conformance-crate`](carry-the-device-executed-value-proof-into-the-conformance-crate.md) — todo, p1, `implementation/conformance` + `implementation/runtime` + `implementation/workspace` + `implementation/cargo-lock`, depends on the BF16 vertical.
- [`route-the-realization-conformance-half-into-the-conformance-crate`](route-the-realization-conformance-half-into-the-conformance-crate.md) — todo, p2, `implementation/conformance`, depends on the above.
- [`derive-the-conformance-evidence-ledger-cells-from-executed-runs`](derive-the-conformance-evidence-ledger-cells-from-executed-runs.md) — deferred with a trigger and a check log.
- [`shape-the-conformance-corpus-for-target-multiplication`](shape-the-conformance-corpus-for-target-multiplication.md) — deferred with a trigger and a check log.
- [`correct-the-scope-set-claim-in-adr-0106-s-missing-component-evidence`](correct-the-scope-set-claim-in-adr-0106-s-missing-component-evidence.md) — todo, p3, `contracts/decisions`.

Plus a comment on [`route-the-two-hand-rolled-test-hashes-through-the-digest-crate-or-record-why-not`](route-the-two-hand-rolled-test-hashes-through-the-digest-crate-or-record-why-not.md) rather than a duplicate ticket: its population is three, not two — `prototypes/serial-sum-run/src/proof.rs:4132` is a third transcription its `crates/`-only closing grep does not reach — its scopes are short by `implementation/runtime`, and `tiler-digest` cannot absorb any of the three because its charter refuses an undomained-bytes entry point while all three reproduce a bare `CC_SHA256` over a device result's raw bytes.

### What this survey did not do

- Did not move anything, edit any crate or document, or run `make full`. `AGENTS.md`'s delta rule carries the latest green gate: the diff touches `tickets/` only.
- Did not design the crate's API. Every filed ticket names ADR 0075 where a surface would be involved and leaves it to an acceptance node.
- Did not re-open where the crate lives, and did not pre-empt the unsafe-lint decision — which is `done`, and whose answer is `deny` with named per-site `#[allow(unsafe_code)]` exceptions concentrated in a single narrow module, never a crate-level allow, admitted only for FFI memory management with Metal. `carry-the-device-executed-value-proof-into-the-conformance-crate` restates that rule rather than reopening it.
- Did not read `crates/tiler-reference/src/oracle.rs` (2,752 lines) or `crates/tiler-ir/src/semantic/conformance.rs` (1,938 lines) in full. Both were read far enough to establish they are production authority code rather than tests — the first is the generic slow oracle for verified index regions, the second is resolved-value binding conformance — and neither is a migration candidate under any reading of the crate's charter. Full reads were spent on the candidates whose classification was in doubt.
- Did not edit `retain-contraction-conformance-evidence` to narrow it to its reference half. That is outcome mutation on another open ticket and belongs to the coordinator; the recommendation is recorded in the filed ticket that supersedes its other half.

## Outcome — 21 candidates classified, 3 move, 18 stay, 2026-08-07 at `3f073476`

**The "stays" population is the majority and it is argued, not defaulted.** The four `*_conformance.rs` reference tests each state their own exclusion — "not evidence about any schedule, any lowering, any compiled kernel, any device" — and their import sets confirm it. `custom_backend/` executes nothing: it has no dev-dependencies at all and reaches `tiler_reference` nowhere. `adapter_route/` would be **destroyed** by moving, since `tiler-runtime`'s dev-dependency comment records that its tests must not reach `tiler-compiler`, which `tiler-conformance` depends on normally.

**The hard case was named rather than smoothed.** `identity_join/` *is* cross-layer executed evidence by ADR 0106's own definition, and it stays anyway — because `the_consumer_links_no_compiler_emitter_or_build_provider` proves a **negative about its own dependency closure** that is only true where it sits. That yields a proposed fourth anti-goal the crate's three do not cover: **not a home for a run whose claim is about what its consumer cannot reach.** Worth adopting.

**Genuine uncertainty was flagged rather than resolved to keep the table tidy:** `governed/contraction_conformance.rs`'s retained-digest leg is the transcribe-by-hand pattern ADR 0106 names, and what settles it is whichever shape `route-the-index-region-conformance-through-the-staged-oracle` chooses.

### The verdict on the missing-component claim — confirmed, and my evidence for it was partly false

**Stronger than argued.** The cross-layer executed run *already exists*: `prototypes/serial-sum-run/src/proof.rs` is 8,159 lines, declares the same dependency row the new crate does, dispatches on GPU by two paths and compares both against `tiler-reference` — and is reached **only** by `cargo run`. No `Makefile` target invokes it; the only mentions are two Clippy `--exclude` flags. It holds the corpus's only device observation of a permitted reassociated answer and the only executed match against a retained device digest, in the one tree the repository deliberately holds to a lower standard.

**And partly false, which is the coordinator's error.** I told Tom, and ADR 0106 records, that five conformance tickets have scope sets such that "no two share" one. **Three of the five carry identical scope sets** and are about one compiler file. The real population is 283 conformance-mentioning tickets, 76 non-terminal — not five. The conclusion survives on the stronger evidence above; the stated evidence did not, and [`correct-the-scope-set-claim-in-adr-0106-s-missing-component-evidence`](correct-the-scope-set-claim-in-adr-0106-s-missing-component-evidence.md) repairs the record rather than leaving a false claim inside an accepted ADR.

### The long-term answers

- **Derivable ledger cells: exactly one of nine** — `Conformance evidence`. The other eight report what *authority exists at a layer*, which no run observes. A run also supplies three qualifiers most tested cells lack: operation set, environment row, and measured-half availability.
- **The maturity ladder cannot be stamped, and should not be.** Three of its four rungs describe states with no run at all, and the top rung's rule is a scoping judgement. It has three different spellings across `AGENTS.md`, `dtype-support.md` and `roadmap.md`, and no Rust representation. A harness can only *compare* — refuse a `tested guarantee` cell with no run naming it.
- **The evidence ladder is reportable, not assignable, and not yet either.** Four disagreeing Rust types exist, differing in variants and visibility; `ConformanceEvidenceClass` spells the top rung `FormalProof`. Assigning would be the second authority the first anti-goal refuses.
- **Target multiplication** needs a run declared as a *value* — operation set × dtype × contract key × shape class × environment row — with one executor, the profile supplied rather than baked in, the environment row as an operand so unavailability is a stated outcome, and refusal recorded as a case outcome. Explicitly **not** a combinatorial generator: the matrix is sparse and non-monotone by design.

Five tickets filed, two of them `deferred` with triggers, reproducing commands and dated check-log entries. A comment rather than a duplicate ticket was added to the sha256 ticket, whose population is three rather than two — its closing grep is `crates/`-only and never reaches the third transcription.

**One stated deviation:** two large production files were read far enough to establish they are authority code rather than migration candidates, not in full; full reads went to candidates whose classification was in doubt. Recorded in the survey body as well as here.
