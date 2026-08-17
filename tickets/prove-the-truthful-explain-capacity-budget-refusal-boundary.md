---
id: prove-the-truthful-explain-capacity-budget-refusal-boundary
title: Prove the truthful explain-capacity budget-refusal boundary
status: in-progress
priority: p1
dependencies: [implement-the-truthful-explain-capacity-budget-refusal]
related: []
scopes: [implementation/compiler, implementation/frontend, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [evidence, correctness, explain, public-boundary]
claimed_from: todo
assignee: sol-explain-proof
lease_expires_at: 1786933390
---

## User-visible outcome

Retained regression evidence proves the accepted explain-capacity refusal is truthful, arm-specific, prefix-lower-bound accurate, and request-wide atomic without weakening genuine compiler-defect reporting.

## Required delivery

- Reproduce the retained valid seven-specialist public request. Assert `BudgetExhausted`, the correct explain resource/limit/reported/provenance, and the unchanged terminal `compiler-failure:explain-detail-capacity` record.
- Count the complete exercised population so an earlier refusal cannot masquerade as the intended subject.
- Independently force the record and byte arms one unit below their exact attempted prefix. Force both on one record and prove record-first precedence.
- Perturb resource, limit, reported, and `ConstructionLowerBound` independently and quote each unchanged assertion's failure.
- Produce a real verifier/ledger/compiler-output defect and prove it remains `InvalidCompilerOutput`.
- Exercise multiple semantic candidates, a viable fallback numerical contract, and multiple targets with reachability counters. Explain capacity in earlier work must prevent every later path, and an earlier target success must not survive as partial output.
- Retain exact commands, failure text, population counts, and the exact implementation/evidence commits. Do not replace source evidence with a fixture that can mint the expected public payload directly.

## Non-goals

Choosing new capacity values, measuring provider support policy, changing trace contents, benchmarking unrelated compiler work, or proving an unbounded provider population fits the retained ceilings.

## Closes when

Every accepted public field and atomicity guarantee has an independent subject perturbation, the seven-specialist public reproduction is green, genuine compiler defects remain separately classified, and an identity-sensitive independent review reports no correctness or evidence finding.

## Exact-base obligation audit

Audited on 2026-08-16 at exact base `b757936b6620620a321c3c9ba43ec75ca4376599`. This ticket stated no `## Facts` section, so each Required-delivery claim was treated as a current obligation. The accepted implementation commit `b474fd01e339396ab7779c6ae0ae9e58631a7856` is an ancestor of that base.

- **Fact — verified.** The unmodified public seven-specialist request is constructible through source anchors `request_boundary`, `compile_request_diagnostic`, `CompileRequest::preferring`, and `compile(request)`. It reaches every installed-provider outcome before refusing: 7 specialists × 17 invocations = 119, partitioned into 21 proposals and 98 declines.
- **Fact — false as a claim of existing retained coverage.** `detail_capacity_arms_are_independent_and_record_first` called `detail_capacity` directly. `capacity_aborts_before_a_later_candidate_and_contract_fallback` and `outer_capacity_discards_an_earlier_target_outcome_and_skips_the_later_target` constructed `capacity_error()` at orchestration seams. They verified the implementation shape but did not reach the accepted public mapper from production explain construction.
- **Fact — verified but incomplete.** `explain_capacity_maps_exactly_and_other_explain_errors_remain_defects` preserved direct typed carrier fields and classified the real `verify_semantic_output_type` error as `InvalidCompilerOutput`; it did not drive capacity through public compile orchestration.
- **Fact — verified.** Source anchors `EXPLAIN_SCHEMA_VERSION` and `EXPLAIN_RENDERER_VERSION` remained 11 and 9. The accepted boundary required no public API, identity, schema, renderer, trace-content, provider-admission, cap, or policy movement.

No stale statement required a ticket repair, and the audit did not change the ticket's purpose or accepted public surface. `research/program-planning` was added because the retained public reproduction is owned by `spikes/program-planning/physical-frontier-budget-calibration`.

## Retained evidence

Implementation commit: `b474fd01e339396ab7779c6ae0ae9e58631a7856`. Evidence commit: `fa97753ded6b5f72f4a2587034fcd764c09f811e`.

`seven_specialists_reach_the_public_explain_byte_refusal` constructs the public semantic program, seven separately named installed providers, target profile, numerical contract, and `CompileRequest`; it then calls public `session::compile`. It cannot mint a failure class. The retained population is exactly 119 invocations = 21 proposals + 98 declines, with 21 baseline-bearing and 98 coverless/unspellable subjects. It returns zero successes and zero alternatives with:

```text
BudgetExhausted { resource: ExplainDetailCanonicalBytes, limit: 1048576, reported: 1048698 }
```

The failure retains 2,258 rendered record lines and 643,313 bytes. Its unchanged terminal record is:

```text
2257 target-feasibility compiler-failure rule=compile.failure@1 provider=compiler:tiler.compiler@1 subject=region:program-alternative:f10d1b8bfd323115/region:0 event=compiler-failure:explain-detail-capacity causes=2256
```

The exact-arm controls use a private `#[cfg(test)]` writer control selected by target key and target-local writer ordinal. It records only raw `retained_records`, `retained_bytes`, and `attempted_record_bytes`, and substitutes only the two limits passed to the unchanged production `detail_capacity` function. It cannot construct `ExplainError`, `ExplainCapacityError`, or `CompileFailureClass`. The same public `CompileRequest` and `compile` path first observes the ninth attempted detail prefix as exactly 9 records and 2,795 canonical bytes, then proves:

- record limit 8 reports `ExplainDetailRecords { limit: 8, reported: 9 }`;
- byte limit 2,794 reports `ExplainDetailCanonicalBytes { limit: 2794, reported: 2795 }`;
- both limits on that same record report the record arm; and
- all three failures retain the same terminal `compiler-failure:explain-detail-capacity` record.

The public orchestration counters cover three independent populations:

- the reassociating add-chain has two semantic candidates and opens three writers when complete (two candidates plus the portfolio); capacity in writer zero leaves exactly one opening;
- the preferred flush contract resolves successfully, the lower-preference strict contract compiles independently and is therefore viable, and capacity in the preferred writer leaves exactly one opening with no fallback writer; and
- three targets complete in caller order in the control. With capacity on the middle target, openings are exactly `[earlier-success, earlier-success, capacity-stop]`: the earlier candidate and portfolio completed, the stop target opened once, the later target never opened, and public `compile` returned `Err` rather than a partial batch.

The genuine-defect negative builds a `u8` semantic output, obtains `ProgramError::Structure { rule: "semantic-output-type" }` from the real `verify_semantic_output_type` verifier, and proves the public class remains `InvalidCompilerOutput`.

### Exact positive commands

```sh
cargo run --quiet --manifest-path spikes/program-planning/physical-frontier-budget-calibration/Cargo.toml -- request-boundary 7
cargo test --manifest-path spikes/program-planning/physical-frontier-budget-calibration/Cargo.toml seven_specialists_reach_the_public_explain_byte_refusal -- --nocapture
cargo test -p tiler-compiler --lib public_explain_capacity -- --nocapture
cargo test -p tiler-compiler --lib explain_capacity_maps_exactly_and_other_explain_errors_remain_defects -- --nocapture
```

The first command prints the complete 1-through-7 control. Its terminal row is `specialists=7 successes=0 invocations=119 proposals=21 declines=98 raw=119 alternatives=0 explain_record_lines=2258 explain_bytes=643313` followed by the exact class and terminal line above. The retained tests report 1/1, 4/4, and 1/1 green respectively.

### Production-subject perturbations

Each edit below was applied alone, the named unchanged test command was run, and the edit was restored before the next perturbation. The first five use:

```sh
cargo test -p tiler-compiler --lib public_explain_capacity_is_exact_and_record_first_at_one_below -- --nocapture
```

| Production subject perturbation | Exact unchanged failure |
| --- | --- |
| At `fn detail_capacity`, change the record arm's `resource` from `ExplainDetailRecords` to `ExplainDetailCanonicalBytes`. | `the public resource did not preserve the production detail-capacity arm` — left `ExplainDetailCanonicalBytes`, right `ExplainDetailRecords` |
| Change the record arm's `limit` from `u64::from(record_limit)` to `u64::from(record_limit).saturating_add(1)`. | `the public limit did not preserve the production construction limit` — left `9`, right `8` |
| Change the record arm's `reported` from `attempted_records` to `attempted_records.saturating_add(1)`. | `the public reported value did not preserve the production attempted prefix` — left `10`, right `9` |
| At `BudgetResource::refusal`, change the two explain resources from `ConstructionLowerBound` to `SearchLowerBound`. | `the production explain resource lost ConstructionLowerBound provenance` — left `SearchLowerBound`, right `ConstructionLowerBound` |
| Move the byte-limit comparison before the record-limit comparison in `detail_capacity`. | `the public resource did not preserve the production detail-capacity arm` — left `ExplainDetailCanonicalBytes`, right `ExplainDetailRecords` |

Candidate and contract atomicity share one load-bearing production subject: temporarily admit `CompileError::ExplainCapacity(_)` beside candidate-local `NoFeasiblePlan`/`BudgetExhausted` in `compile_candidate_target`. Run these separately:

```sh
cargo test -p tiler-compiler --lib public_explain_capacity_stops_later_semantic_candidates -- --nocapture
cargo test -p tiler-compiler --lib public_explain_capacity_stops_a_viable_contract_fallback -- --nocapture
```

Each fails unchanged with `detail capacity returned a partial or complete compilation batch`; the perturbed compiler retried later work and returned output instead of the required outer refusal.

For target/request atomicity, change the `TargetCompileFailure::Outer(error)` arm in `target_compilation_outcome` from `Err(error)` to a target-local `TargetCompilationOutcome::Rejected`, then run:

```sh
cargo test -p tiler-compiler --lib public_explain_capacity_discards_earlier_targets_and_stops_later_targets -- --nocapture
```

It fails unchanged with `detail capacity returned a partial or complete compilation batch`; the perturbed compiler retains a partial batch and reaches the later target.

For the genuine compiler-output negative, change the `CompileError::InvalidCompilerOutput(_)` public projection to `NoFeasiblePlan` and run:

```sh
cargo test -p tiler-compiler --lib explain_capacity_maps_exactly_and_other_explain_errors_remain_defects -- --nocapture
```

It fails unchanged with `a genuine compiler-output verification failure must not become a budget refusal` — left `NoFeasiblePlan`, right `InvalidCompilerOutput`.

### Identity and surface non-movement

```sh
git diff --name-only b757936b6620620a321c3c9ba43ec75ca4376599..fa97753ded6b5f72f4a2587034fcd764c09f811e
git show b757936b6620620a321c3c9ba43ec75ca4376599:crates/tiler-compiler/src/explain.rs | rg '^pub\(crate\) const EXPLAIN_(SCHEMA|RENDERER)_VERSION'
git show fa97753ded6b5f72f4a2587034fcd764c09f811e:crates/tiler-compiler/src/explain.rs | rg '^pub\(crate\) const EXPLAIN_(SCHEMA|RENDERER)_VERSION'
```

Only compiler tests/test-only writer control, the retained spike regression, and this ticket move. Both revisions print schema 11 and renderer 9. The writer control and every added compiler carrier are `pub(crate)` under `#[cfg(test)]`; no non-test or public item is added, and no request, artifact, cache, schema, renderer, or trace-content identity input changes.
