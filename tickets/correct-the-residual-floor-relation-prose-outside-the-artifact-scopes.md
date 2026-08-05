---
id: correct-the-residual-floor-relation-prose-outside-the-artifact-scopes
title: Correct the residual floor-relation prose the subgroup-threads equality falsified
status: todo
priority: p2
dependencies: []
related: [correct-the-subgroup-threads-route-dimension-meaning, rename-the-route-resource-floor-vocabulary-for-its-corrected-relation]
scopes: [contracts/decisions, research/scheduling, research/extensions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions, naming]
---
## Why this exists

`correct-the-subgroup-threads-route-dimension-meaning` changed `RouteResourceDimension::SubgroupThreads` from a floor to an equality and renamed the field `minimum` to `required`. It held `implementation/artifact` and `contracts/artifacts` only. `rename-the-route-resource-floor-vocabulary-for-its-corrected-relation` then renamed the type and variant across the runtime, the prototypes, the spike, `research/runtime`, and ADR 0092. Neither could reach four further sites that still describe the removed relation, in two documents that are **accepted ADRs** and two research records. Found by sweeping the relation vocabulary rather than only the type name.

**Fact — two sites assert a satisfaction rule that no longer exists.** `docs/research/scheduling/subgroup-execution-tier.md:58` states as **Fact** that "Its satisfaction test is `is_satisfied_by(observed) = self.minimum <= observed` — a floor", and line 171 repeats "`RouteResourceDimension::SubgroupThreads` is a *floor* — `is_satisfied_by(observed) = self.minimum <= observed`". Both are doubly falsified at `f032c0d`: `crates/tiler-artifact/src/program/requirement.rs` has no `minimum` field, and `is_satisfied_by` reads `RouteResourceDimension::SubgroupThreads => self.required == observed`. Reproduce with `grep -n "is_satisfied_by" -A6 crates/tiler-artifact/src/program/requirement.rs`.

**Fact — one site is inside an accepted ADR and byte-paired with a retained span.** `docs/decisions/0094-bind-a-subgroup-combine-to-a-register-transfer-tree.md:28` reads "The one place a subgroup width is named — `RouteResourceDimension::SubgroupThreads` — is a live-device floor over 'threads that execute in lockstep', which every implemented adapter answers `Unrecognized`." `docs/research/scheduling/subgroup-execution-tier.md:377` carries the same sentence byte-identically (verified: `cmp <(sed -n '28p' <adr>) <(sed -n '377p' <record>)` reports no difference), and line 377 sits inside that record's drafted-ADR span, which begins at its heading on line 357. So the ADR 0092 handling applies unchanged: **correct the ADR, record the drift beside the span, re-transfer rather than edit inside.** ADR 0094 item 7 at line 20 and the record's line 344 both describe the change as forthcoming and are correctly historical — they name what item 7 *does*, and they stay.

**Fact — a fourth site is in a second accepted ADR whose twin is out of that ADR's scope.** `docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md:69` reads "`Unrecognized`, a wrong-shaped answer, and an unmet floor stay three distinct rejections". `docs/research/extensions/backend-provider-composition.md:70` carries a near-identical (not byte-identical) sentence. Neither is a type name, and the decision each states — that an adapter reports and never adjudicates — is untouched; what is stale is "floor", which now names a relation the vocabulary does not have for the one dimension it carries.

**Why this is one ticket and not four edits.** Two of the four sites are byte-paired across a scope boundary. `contracts/decisions` reaches the ADR halves; `research/scheduling` and `research/extensions` reach the other halves. Correcting only the in-scope half of a verbatim pair forks it, which is exactly the failure the retained-span convention exists to prevent — so the three scopes have to be held together, and that is the whole reason this was filed rather than absorbed.

## What closes this

No sentence in an accepted ADR or a research record states a floor relation, a `minimum` field, or a `self.minimum <= observed` satisfaction test for `RouteResourceDimension::SubgroupThreads`; every corrected sentence carries a dated marker naming this ticket in the `correct-adr-0074-driver-vocabulary-consumers` shape; the ADR 0094 / subgroup-execution-tier pair is either byte-identical again or its divergence is recorded beside the span with the re-transfer stated; and no `decision_status` moves, because no decision changes meaning.

Distinguish the two uses of "floor" before editing: a sentence *rejecting* a floor is the live derivation and stays — `requirement.rs`'s "Why the relation is an equality and not a floor", `docs/artifact-abi.md:308`, and the regression test's "which a floor accepted" are all correct as written and must not be swept. Only a sentence *asserting* the row is a floor is stale.
