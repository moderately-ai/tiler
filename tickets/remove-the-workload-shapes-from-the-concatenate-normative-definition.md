---
id: remove-the-workload-shapes-from-the-concatenate-normative-definition
title: Remove the workload shapes from the concatenate normative definition
status: in-progress
priority: p1
dependencies: []
related: []
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: coord
lease_expires_at: 1786188969
---
## Worker per-Fact audit, 2026-08-08, at base `aae3da245c79314b09f442342b22b8458b8558e1`

Every Fact below was re-read in full at this base before any edit. Citations are searchable anchors, each confirmed to resolve exactly once with `grep -rF`. **The ticket's central identity claim is false**, and correcting it shrinks the blast radius from a predicted domain step plus five pins across four crates to one pin in one crate.

| # | Fact as written | Verdict |
| --- | --- | --- |
| 1 | The definition contains the `[8, 0, 128]` sentence | **Verified.** It is one `concat!` fragment, `"[8, 0, 128] with [8, T, 128] on axis 1 yields [8, T, 128] whose elements are the second operand's, "`, and the preceding fragment ends in `Joining `. Quoting it as a sentence is right; grepping for it as one contiguous source line would have failed |
| 2 | The text reaches identity through `push_slice(output, definition.normative_definition().as_str().as_bytes())` | **Verified**, and it is two sites, not one: `encode_operation_definition` for operations and `push_slice(output, definition.normative_definition.as_str().as_bytes());` in `encode_type_definition` for value types. **Imprecise on which identity** — see 5 |
| 3 | Concatenate is the only family with concrete shapes, "scanned every `*_NORMATIVE_DEFINITION`", six-row table | **Conclusion verified, population imprecise.** A scan for constants named `*_NORMATIVE_DEFINITION` reaches six families; `NormativeDefinitionRef::new` is called from thirteen non-test registration sites in `crates/tiler-ir/src/semantic/`, because `contraction`, `rms_norm`, `softmax`, `silu`, `bf16`, `quantization`, and `catalog`'s scalar rows pass their text inline with no such constant behind it. Those seven were outside the stated scan. I read all thirteen: none names a tensor shape, so the ticket's conclusion holds on a population it did not actually cover. The nearest miss is `silu`, which names the binary32 constant `0xc2b00000` — an argument at which two spellings differ, not a workload extent |
| 4 | An identical illustration already lives in a doc comment further up the same file | **Verified.** `concatenate_result_shape`'s doc comment: `` `[8, 0, 128]` with `[8, T, 128]` on axis 1 therefore yields `[8, T, 128]`, ``. Doc comments are not encoded, so it stays |
| 5 | "Changing a normative definition changes the operation's canonical identity, which **steps the semantic-graph identity domain**" | **False, on two independent counts.** (a) `compute_graph_identity` in `crates/tiler-ir/src/semantic/identity.rs` never reads a normative definition. After `bytes.extend_from_slice(GRAPH_DOMAIN);` it encodes input keys, resolved types and shapes, operation keys and attributes, operand and result ids, and output keys — nothing else. The semantic-graph subject does not observe this edit at all. (b) Even for the subjects that *do* fold the text, a domain separator steps when the encoding **grammar** changes, not when a **value** inside it moves. This edit changes neither field set nor framing, so a different input yielding a different digest is the encoding working, not a reinterpretation of retained bytes |
| 6 | "The step is `tiler.semantic-graph.v3 → v4`, not `v2 → v3`. `v3` is taken" | **False.** `v3` is indeed live — `const GRAPH_DOMAIN: &[u8] = b"tiler.semantic-graph.v3\0";` and `b"tiler.semantic-graph.v3\0",` in the `crates/tiler-ir/src/domains.rs` census — but **no step is required**, so the correction corrects a claim that should not have been made. `domains.rs` stayed green throughout, which is the census agreeing |
| 7 | "The pin population is five", naming `ARTIFACT_IDENTITY`, `CACHE_SUBJECT`, `FIXED_CONTENT_BYTES`, the explain qualifier, and `DIFFERING_CARRIER_POSITIONS` | **False. Measured population is one.** `cargo nextest run --workspace` on the edited tree: 3,219 tests, 3,218 passed, **1 failed** — the explain request qualifier. The other four held unchanged |
| 8 | `DIFFERING_CARRIER_POSITIONS` "moved by chance … expect it to move again", and its `docs/artifact-abi.md` twin needs a carrier ticket | **Not fired.** `const DIFFERING_CARRIER_POSITIONS: usize = 68;` is unchanged and its test passes. No carrier ticket is needed, and `contracts/artifacts` is not required |
| 9 | "`tiler.compiler.request-subject` and `tiler.program-alternative` do not step … a value move inside them stays injective" | **Verified** — and this is the reasoning that refutes 5 and 6. The ticket applied it correctly one layer up and not to the layer it was actually editing |
| 10 | The ticket "needs `implementation/compiler`, `implementation/artifact`, and `implementation/build` beside `implementation/ir`" | **One third true.** Only `implementation/compiler` is needed, for one constant. `implementation/artifact` and `implementation/build` are not |

### Why exactly one pin moved, read rather than inferred

The text enters two subjects and neither is the graph:

- **`tiler.semantic-registry.v7`** — `let mut bytes = b"tiler.semantic-registry.v7\0".to_vec();` folds **every registered operation**, so concatenate's text is in it for any program at all.
- **`tiler.semantic-definition-projection.v5`** — `let mut bytes = b"tiler.semantic-definition-projection.v5\0".to_vec();` folds only the closure `project_program_authority` computed from the program's own value types and operations.

The compiler's request subject folds both: after `bytes.extend_from_slice(b"tiler.compiler.request-subject.v6\0");` it writes `self.semantic_identity.registry_snapshot().as_bytes(),`. **That is the whole mechanism of the one failure.**

The artifact never carries the registry snapshot. `crates/tiler-artifact/src/program/codec/encode.rs` writes `push_slice(&mut bytes, semantic.reached_definitions.as_bytes());` — the closure-scoped projection — and the graph identity, and no `registry_snapshot` appears anywhere in `tiler-artifact`, `tiler-build`, or `tiler-cache` sources. The `tiler-build` fixture's program is two constants, a multiply, an add, and a strict serial sum; concatenate is not in its closure, so its text never reaches those three pins. **That is why they held, and it is a structural reason rather than a lucky measurement.**

## A pinned-workload shape reaches canonical operation identity

`CONCATENATE_F32_NORMATIVE_DEFINITION` in `crates/tiler-ir/src/semantic/concatenate.rs` contains the sentence:

> `[8, 0, 128] with [8, T, 128] on axis 1 yields [8, T, 128] whose elements are the second operand's, `

Those are the pinned `Qwen3-0.6B` decoder's **8 KV heads and 128 head dimension**. The string is not documentation — it is encoded into identity: `encode_operation_definition` in `crates/tiler-ir/src/semantic/registry.rs` does `push_slice(output, definition.normative_definition().as_str().as_bytes())`.

**So a change to the pinned workload rewrites a registered operation's identity.** That is the leak `AGENTS.md` forbids when it says to use examples to exercise general machinery rather than specialize semantics around one case, and to keep the compiler core independent of any one consumer.

### It is the only family that does this — verified exhaustively

The coordinator scanned every `*_NORMATIVE_DEFINITION` in `crates/tiler-ir/src/semantic/`:

| family | concrete shapes in its normative definition |
| --- | --- |
| `broadcast` | none |
| `catalog` (complex) | none |
| **`concatenate`** | **`[8, 0, 128]`, `[8, T, 128]`** |
| `gather` | none |
| `reindex` | none |
| `slice` | none |

Every sibling states its zero-extent and empty-operand rule **abstractly**. Concatenate is the outlier, so the fix is to match the established pattern rather than to invent one.

## What to do

State the rule without the instance — something of the shape "a zero-extent operand is admitted and contributes no coordinate; the result takes the other operand's elements". **An identical illustration already lives in the doc comment further up the same file**, which is the correct home for it: doc comments are not encoded.

## The consequence that makes this more than a wording fix

**Falsified 2026-08-08 by the audit above, rows 5 through 8 — read that table before acting on anything in this section or the next. Both are retained rather than deleted so a reader who saw the original can find the correction, and neither is restated in new words.** The sentence below is wrong twice: the semantic-graph domain does not observe a normative definition at all, and a value moving inside an unchanged encoding grammar is not a domain step. No domain steps for this change. The measured pin population is **one**, not five, and it is the explain request qualifier alone.

Changing a normative definition **changes the operation's canonical identity**, which steps the semantic-graph identity domain and moves every pinned identity that folds it. So this is not a cosmetic edit:

- **Recompute every pinned identity on your own merged tree** and report which moved and which did not. Do not carry pin values from any ticket body — two branches moved shared pins from different bases on 2026-08-07 and neither's values survived.
- Coordinate with `carry-a-sourced-shape-on-semantic-values`, which is **live right now** and itself steps `tiler.semantic-graph.v2 → v3`. Two independent identity-domain steps in flight is exactly the shared-identity collision `AGENTS.md` says to serialize. **Check whether that ticket has landed before starting, and if it has not, say so and stop** rather than racing it.

## `carry-a-sourced-shape-on-semantic-values` landed first, 2026-08-07 — three corrections to the item above

**Falsified in part 2026-08-08; see audit rows 6 through 8 and 10.** Its serialization conclusion stands and its `v3` observation is accurate. Its step, pin population, and scope list do not.

Recorded by that ticket's worker, from its own measured tree. The serialization question is settled: it went first, so this ticket follows it rather than racing it.

- **The step is `tiler.semantic-graph.v3 → v4`, not `v2 → v3`.** `v3` is taken.
- **The pin population is five, not the three this ticket's neighbours name.** Measured by running the suite on the landed tree, not hypothesized: `ARTIFACT_IDENTITY`, `CACHE_SUBJECT`, and `FIXED_CONTENT_BYTES` in `crates/tiler-build/src/metal_plan.rs`; the explain request qualifier in `crates/tiler-compiler/src/explain.rs`; and `DIFFERING_CARRIER_POSITIONS` in `crates/tiler-artifact/src/program/codec/tests.rs`. The last two were **absent from every earlier enumeration**. `crates/tiler/src/route/tests.rs`'s `IDENTITY_DOMAIN`, the `index/law.rs` pins, and the `schedule/builder.rs` pins did **not** move and are expected not to move here either. So this ticket needs `implementation/compiler`, `implementation/artifact`, and `implementation/build` beside `implementation/ir` — its current single scope cannot reach three of its five pins.
- **`DIFFERING_CARRIER_POSITIONS` is a *measured* count with a doc-comment twin in `docs/artifact-abi.md` (`contracts/artifacts`), which this ticket does not hold.** It moved by chance rather than structurally when the graph bytes changed, so expect it to move again for no structural reason, and expect the doc half to need a carrier ticket exactly as the landing did.
- **`tiler.compiler.request-subject` and `tiler.program-alternative` do not step for this change.** They stepped for the *subject set*, which a normative-definition edit does not change; a value move inside them stays injective.

## Worker outcome, 2026-08-08 — complete, gated at `make full` exit 0

**Delivered inside `implementation/ir` and `implementation/compiler`.** The instance is gone from `CONCATENATE_F32_NORMATIVE_DEFINITION`; the rule is now stated over the operands, keeping the bit-preservation claim the illustration carried and losing nothing normative, because the two sentences around it already state admission abstractly. The illustration stays in `concatenate_result_shape`'s doc comment. `the_normative_reference_states_the_zero_extent_rule_and_the_extent_refusal` no longer pins the removed text and asserts the general spelling instead.

**The closing condition is now a test rather than a re-run of a manual scan**, which is the substantive addition. `no_registered_normative_definition_names_a_concrete_shape` in `crates/tiler-ir/src/semantic/registry.rs` walks **every** registered operation and value-type definition and refuses a bracketed run whose comma-separated tokens are all digits or a single uppercase extent symbol. It covers the seven families the ticket's stated scan could not reach, and a family added later inherits it. It states what it cannot claim: a shape spelled `(8, 0, 128)` or `8x128` passes.

**A population floor was written here and then removed, and the removal is a finding.** The convention for an untypeable population is to assert a floor so "nothing ran" cannot look green. Perturbing the subject to test it — dropping four `register_standard_*` calls — does not produce a small registry. `freeze` refuses with `IndexRealizationLawWithoutOperation { operation: … "reindex-f32" }`, because the realization laws registered beside them stop naming a registered operation. `FrozenSemanticRegistry::standard()` is all-or-nothing, so no reachable state gives this scan a shrunken standard registry and a floor beside it would have been a guard that cannot fail. The reason is recorded in the test's doc comment rather than the floor.

### The one dependent pin, blocked on 2026-08-08 and completed the same day

One constant, `"tiler-explain-v7 request=…\n",` in `crates/tiler-compiler/src/explain.rs`. **It had to move in the same commit as the IR edit** — the registry-snapshot identity moves the moment the definition text does, so any tree carrying one without the other is red. It was therefore never eligible to be a follow-up ticket; splitting it would have left `main` failing between the two landings.

**Why it was deferred, and for how long.** `implementation/compiler` was held exclusively by a live claim, `disclose-offered-and-selected-physical-provider-sets-separately`, which itself edits compiler provider disclosure and was a plausible mover of this same qualifier. Two branches rebaselining one pin from different bases is the composition failure `metal_plan.rs` records: *"a pinned identity is recomputed on the tree the step lands into, never taken from either side."* That ticket landed, the coordinator added `implementation/compiler` to this ticket's scopes, and this branch was rebased onto `580e8c1fdb263b35c5165355a4904bde9272a320`.

**Recomputed on the merged tree, not carried.** `940c09e0821665a6` → **`4e10437fec85d7b1`**, regenerated with

```text
cargo nextest run -p tiler-compiler -E \
  'test(deterministic_trace_is_sealed_and_rendered_separately)'
```

and taking the `left` value. **The recomputed value coincides with the pre-rebase branch-local one, and that coincidence is a measurement rather than a licence to have skipped the step.** It coincides because none of the intervening merges moved the registry-snapshot identity — the pin still read `940c09e0821665a6` on the new base before this edit, so the base never moved either. A sibling on this repository measured four offsets across one step that moved in *opposite* directions, so no coincidence of this kind generalizes.

**That this one line was the whole remainder is measured, not argued.** `make full` on the branch before it exited 2, failing at `test` with `3219 tests run: 3218 passed, 1 failed, 8 skipped` on that assertion alone, with `citations`, `fmt`, `build`, and `lint` passing ahead of it. With the line applied on the rebased tree it exits **0** across every stage.

**Scope note, for the record.** This ticket needed `implementation/compiler` beside `implementation/ir` — not the three extra scopes its own body predicted. `implementation/artifact`, `implementation/build`, and `contracts/artifacts` were not required, because none of those pins moved.

### One documentation site goes stale on landing, outside this ticket's scopes

`docs/roadmap.md` states the concatenate family `states the zero-extent rule at L5`'s `[8, 0, 128]` prefill shape. That becomes false when this lands. `docs/roadmap.md` is `contracts/navigation`, held exclusively by a live claim, so it is left for the coordinator to route.

## Closes when

No normative definition in `crates/tiler-ir/src/semantic/` contains a concrete tensor shape — re-run the scan above and report the table; the concatenate rule reads as generally as its five siblings; every pinned identity is recomputed on the merged tree with moved/unmoved stated per pin; and the identity-domain step is coherent with whatever `carry-a-sourced-shape-on-semantic-values` did.
