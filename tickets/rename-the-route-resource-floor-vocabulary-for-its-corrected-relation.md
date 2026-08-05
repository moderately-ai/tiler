---
id: rename-the-route-resource-floor-vocabulary-for-its-corrected-relation
title: Rename the route resource floor vocabulary for its corrected relation
status: review
priority: p2
dependencies: []
related: [correct-the-subgroup-threads-route-dimension-meaning]
scopes: [implementation/artifact, implementation/runtime, implementation/candle, research/runtime, contracts/decisions, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, runtime, naming, public-boundary]
claimed_from: todo
assignee: agent-floor-rename
lease_expires_at: 1785954211
---
## User-visible outcome

The type and variant carrying a live-device route resource requirement are named for what they are — a required quantity compared by the relation its dimension fixes — rather than for the floor relation `correct-the-subgroup-threads-route-dimension-meaning` removed.

## Why this is separate from the correction that created it

**Fact.** `correct-the-subgroup-threads-route-dimension-meaning` changed `RouteResourceDimension::SubgroupThreads` from a floor to an equality and corrected every name it could reach inside `crates/tiler-artifact`: the private field and public accessor `minimum()` became `required()`, and `RouteRequirementError::VacuousFloor` became `ZeroResourceQuantity`. Two names it could not reach remain, and both state the removed relation:

- `RouteResourceFloor` — the struct.
- `RouteRequirement::ResourceFloor` — the enum variant.

**Fact — the exact reason they were not renamed.** Both are named outside the `implementation/artifact` and `contracts/artifacts` scopes that ticket held. Reproduce with `grep -rn "ResourceFloor" --include="*.rs" --include="*.md" . | grep -v "^./crates/tiler-artifact/"`:

| Site | Scope |
| --- | --- |
| `crates/tiler-runtime/src/load/route.rs` (2 match arms) | `implementation/runtime` |
| `crates/tiler-runtime/tests/adapter_route/adapter.rs`, `crates/tiler-runtime/tests/identity_join/adapter.rs` | `implementation/runtime` |
| `prototypes/serial-sum-run/src/proof.rs` (a match arm and a `RouteResourceFloor::new` call) | `implementation/runtime` |
| `prototypes/candle-metal-adapter/src/adapter.rs` | `implementation/candle` |
| `spikes/runtime/inline-dispatch/src/adapter.rs` | `research/runtime` |
| `docs/research/runtime/backend-scoped-route-requirement-answers.md` (3 sentences) | `research/runtime` |
| `docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md` (2 sentences) | `contracts/decisions` |

The last is the one that makes this a boundary rather than a sweep: ADR 0092 is **accepted**, and its Decision item 8 and its open-questions paragraph both name `ResourceFloor` as a live type. A rename leaves accepted text naming a type that does not exist.

## What to decide

- The replacement name. `RouteResourceRequirement` matches the neighbouring `BackendFeatureRequirement` and the `RouteRequirement` enum it sits in; the cost is that `RouteRequirement::ResourceRequirement` reads redundantly, so the variant may want a different spelling from the struct.
- Whether accepted ADR 0092's two sentences are corrected in place with a marker (the `correct-adr-0074-driver-vocabulary-consumers` precedent, which corrects falsified factual claims inside an accepted ADR against measured source) or left with a note that the type was renamed after acceptance.

## Non-goals

Changing the relation, the wire tags, or any encoded byte — all of that landed with the correcting ticket, and this is a pure rename. Adding a relation to the wire. Adding a dimension.

## Closes when

No name in the workspace states a floor relation for a row the dimension compares by equality, the accepted ADR's text agrees with the types it names, and no encoded byte or artifact identity moved.

## Outcome

**Decision — both names, by elimination with one survivor each.** The struct is `RouteResourceRequirement` and the variant is `RouteRequirement::Resource`. `RouteResourceQuantity` was eliminated because the type carries a dimension and a satisfaction test as well as the number, and `ZeroResourceQuantity` already spends "quantity" on the scalar alone; `RouteResourceRow` introduces a third noun with no `Row`-suffixed type anywhere in the crate; `RouteResourceMinimum` and `RouteResourceBound` restate the relation being removed; `RouteResourceConstraint` collides with the shape-constraint vocabulary in `tiler-ir` and `tiler-compiler`. The surviving name keeps the `RouteResource*` prefix it shares with `RouteResourceDimension`, and the `Route` prefix is load-bearing rather than decorative — it separates the row from the entry-level `ResourceRequirements` the same module's documentation names. For the variant, the ticket's own redundancy objection to `ResourceRequirement` holds, and `Resource` earns more than brevity: `RouteRequirement`'s variants are now exactly `RouteRequirementSubject`'s (`Resource`, `BackendFeature`), so the `subject()` match reads as the bijection it is, and the pair `Resource(RouteResourceRequirement)` / `BackendFeature(BackendFeatureRequirement)` is symmetric in both halves.

**Decision — accepted ADR 0092 is corrected in place, and the source record's retained span is not.** The `correct-adr-0074-driver-vocabulary-consumers` precedent supplies the shape (a dated italic marker naming the ticket and what the earlier text said), but it does not settle the harder half, because ADR 0092's decision item 8 is *transferred verbatim* from `docs/research/runtime/backend-scoped-route-requirement-answers.md` and the two were byte-identical at the base (verified with `cmp` on line 46 against line 378). That record answers it itself: after the same condition arose there on 2026-08-01 it recorded the rule beside its own span — the ADR is the authority, a drift is recorded beside the span rather than repaired inside it because editing forks the transfer, and the span is re-transferred from the ADR rather than corrected first. Applied unchanged. ADR 0092's item 8 and its open-questions paragraph both carry the new spelling with markers; the span at line 380 deliberately still reads `ResourceFloor` and now carries a note saying so. The open-questions paragraph is this record's own text with no twin, so it needed no such care.

**Fact — this is a pure code-level rename and not an identity-domain step, established by reading both codec halves.** Every encoded byte comes from a value the rename does not touch: `RouteRequirement::tag()` writes the literal `0x01` from an exhaustive match on the variant, `RouteResourceDimension::tag()` writes `0x01` for `SubgroupThreads`, the required quantity is `u64::to_be_bytes`, and `canonical_bytes` leads with `ROUTE_REQUIREMENT_DOMAIN`, the byte string `b"tiler.artifact.route-requirement.v1\0"`, which names no type. `RouteResourceDimension::as_str()` still returns `"subgroup-threads"`, and `RouteResourceDimension` itself was not renamed, so the `TagSubject::RouteResourceDimension` diagnostic subject is unmoved. No schema version, manifest version, or pinned digest moved, and none could — confirmed empirically by the artifact suite's identity goldens passing unchanged.

**Fact — the ticket's site table drifted and the drift is in one direction.** The table says `docs/research/runtime/backend-scoped-route-requirement-answers.md` carries "3 sentences"; four passages there name the old vocabulary, and a fifth uses the bare relation word. The four sites are the design's item 6 heading (line 171), the structural-precondition paragraph under question 4 (247), that question's elimination bullet (251, which spelled the row kind "a neutral floor" and said quantitative CPU facts "should be floors"), and the measurement-boundary bullet (336) — plus item 8 inside the retained span (378/380), which is the one the count most likely excluded. All four outside the span were corrected; the elimination bullet also gained the clause that makes it correct under the new model, that each dimension states its own relation so a CPU floor and a subgroup equality coexist. Every other row of the table was accurate.

**Fact — the sweep is complete in code and the residue is deliberate.** `grep -rn "ResourceFloor\|route_floor\|PROBE_FLOOR\|floor_row_offset" --include="*.rs" . --exclude-dir=target` returns **0**; the same grep for `RouteResourceRequirement` returns 14, which is what makes the zero a result rather than a broken command. Renamed beyond the two public names: the local bindings `floor` → `resource` at six match sites, the shared test helper `route_floor` → `route_resource`, the codec fixture constant `PROBE_FLOOR` → `PROBE_QUANTITY`, and its locator `floor_row_offset` → `resource_row_offset`. Four doc comments that stated the removed relation were corrected — the module doc's `[RouteResourceFloor]` reference, `TagSubject::RouteResourceDimension`'s "one route floor bounds", `model.rs`'s "neutral quantitative floors", and the struct's own doc, whose stale-name disclosure was deleted (it is now performed) and whose "the original floor-only shape" rationale was rewritten to state the invariant without the change-history. The spike's refusal string "a subgroup-threads floor" became "a subgroup-threads row"; it is pinned in no golden (`grep -rn "subgroup-threads floor"` returns nothing).

**Deliberately not swept: a sentence rejecting a floor is the live derivation.** `requirement.rs`'s "Why the relation is an equality and not a floor", its `ZeroResourceQuantity` doc contrasting vacuity under a floor with unsatisfiability under an equality, the regression test's "which a floor accepted", and `docs/artifact-abi.md:308` are all correct as written and are the evidence for the correction. Only a name or a claim *asserting* the row is a floor was stale.

**Filed rather than absorbed.** [`correct-the-residual-floor-relation-prose-outside-the-artifact-scopes`](correct-the-residual-floor-relation-prose-outside-the-artifact-scopes.md) — four further sites assert the removed relation, two of them in accepted ADRs: `docs/research/scheduling/subgroup-execution-tier.md:58` and `:171` state as **Fact** a satisfaction test `is_satisfied_by(observed) = self.minimum <= observed` that is doubly false (no `minimum` field exists, and the comparison is `==`); ADR 0094:28 calls the row "a live-device floor" and is byte-identical to that record's line 377, which sits inside its own drafted-ADR span; and ADR 0090:69 plus `docs/research/extensions/backend-provider-composition.md:70` describe the rejection as "an unmet floor". Two of the four are byte-paired across a scope boundary, so correcting only the reachable half would fork a verbatim pair — which is why it is one ticket holding three scopes rather than the two edits this ticket could have reached. [`re-transfer-the-adr-0092-span-for-its-drifted-prototype-referent`](re-transfer-the-adr-0092-span-for-its-drifted-prototype-referent.md) — the source record flagged a drifted alternatives-entry sentence for the ADR 0092 acceptance sweep on 2026-08-01, the sweep did not reach it, and no node carried it; item 8's spelling is now queued behind the same re-transfer, so the two corrections land as one act.

**Public boundary — not self-accepted.** `RouteResourceRequirement` and `RouteRequirement::Resource` are a public type and a public variant of `tiler-artifact`. The elimination above has one survivor and the ticket authorized the rename, but the exact spelling is Tom's under ADR 0075 and AGENTS.md, which is why this stops at `review`.
