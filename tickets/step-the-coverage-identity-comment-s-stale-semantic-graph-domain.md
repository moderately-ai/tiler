---
id: step-the-coverage-identity-comment-s-stale-semantic-graph-domain
title: Step the coverage identity comment s stale semantic graph domain
status: in-progress
priority: p2
dependencies: []
related: [repair-the-records-the-sourced-semantic-shape-falsifies, pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [identity, documentation]
claimed_from: todo
assignee: coord
lease_expires_at: 1786182058
---

A doc comment names a semantic-graph domain that has since stepped. **The interesting part is where the repair must land**: an accepted ADR quotes this comment verbatim, so the ADR is faithful and the source is what drifted.

## Facts, coordinator-verified at the merge that found it

**Fact.** `crates/tiler-ir/src/index/refinement.rs`, on `IndexRefinementExecutableCoverageIdentity`, names `tiler.semantic-graph.v2`. The live constant is `tiler.semantic-graph.v3`, declared in `crates/tiler-ir/src/semantic/identity.rs`.

**Fact — and this is why the repair is here and not in the ADR.** `docs/decisions/0104-…md` contains exactly **one** occurrence of `tiler.semantic-graph.v2`, and it sits inside a verbatim quotation of this comment. Editing the ADR would make it misquote its own source. Repair the comment; the ADR's quotation then becomes accurate again by construction. **Do not touch `docs/decisions/**`** — `contracts/decisions`, not this scope.

> **Corrected 2026-08-08 by the worker at base `670e7a31` — the count is false and the "accurate by construction" reasoning is inverted.** See the per-Fact audit below. The conclusion the Fact draws — repair here, not in the ADR — is nonetheless correct, on different reasoning.

**Fact — a prior ticket claimed two occurrences at two locations. There is one, and there was one at the ADR's landing commit too.** The second cited location carries `request-subject.v5`, an unrelated domain. Do not go looking for a second site.

> **Corrected 2026-08-08 by the worker — the historical half is verified, the present-tense half is false.** There is now genuinely a second site, added after this Fact was written. See the audit below.

## Why p2 rather than p3

The comment describes what a coverage identity folds. A reader who takes `v2` at face value concludes the coverage identity is pinned to a superseded graph domain, which is exactly the kind of wrong premise that produces a wrong identity argument downstream. It is one line and no behaviour, but the claim is load-bearing where it is read.

## What closes this

The comment naming the live domain. **Check whether the sentence is still true once the name is corrected** — a domain step is not always a pure rename, and a comment that was right about `v2` is not automatically right about `v3`. Read `semantic/identity.rs` and confirm what the coverage identity actually folds before changing the digit.

**Then re-read the ADR's quotation** and confirm it now matches the source byte for byte. If it does not, the quotation was already inexact in some other way and that is a separate finding — report it rather than editing the ADR.

**Cite by searchable anchor, not line number.** Doc comments here wrap at 80 columns, so an anchor spanning a line break greps as **absent** — the failure mode `AGENTS.md` records and which has bitten three tickets this week. Run your anchor's grep before committing to it.

**Check this file for other domain names while you are in it, and name the count.** A sibling audit found no test asserts any identity domain string anywhere in the tree, so a stale domain name in prose has nothing catching it — see `pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate`, which is live in this same scope and may land first. Coordinate rather than duplicating: if that work lands a census, reuse it.

## Worker's per-Fact audit, re-read at base `670e7a318a461f36210dce24f8f62075c1d11c38`

| Ticket Fact | Verdict | Evidence at this base |
| --- | --- | --- |
| `refinement.rs` on `IndexRefinementExecutableCoverageIdentity` names `tiler.semantic-graph.v2`; live constant is `v3` in `semantic/identity.rs` | **verified** | The comment read `are not re-encoded: \`tiler.semantic-graph.v2\``; `GRAPH_DOMAIN` reads `b"tiler.semantic-graph.v3\0"` |
| ADR 0104 contains **exactly one** occurrence of `tiler.semantic-graph.v2` | **false** | `grep -o … \| wc -l` on the ADR returns **2**. The second is in the `**Superseded — 2026-08-08 …**` header note, which reads "stepped `tiler.semantic-graph.v2` to **`v3`**" — a correct historical statement, not a quotation of this comment |
| "Repair the comment; the ADR's quotation then becomes accurate again by construction" | **false, and exactly inverted** | The ADR's quotation is faithful *because* the source said `v2`. Repairing the source is what makes it stale. The ADR states this itself in the note beside the quotation: "the doc comment … **still says `v2`**, so rewriting the quote here would make this record misquote its own source" |
| Prior ticket claimed two occurrences; "there is one, and there was one at the ADR's landing commit too" | **half verified, half false** | Historical half **verified**: the ADR carried exactly 1 at `e5bfaba4` and `a067735e`. Present-tense half **false**: `980d0a93` (2026-08-08, `repair-the-records-the-sourced-semantic-shape-falsifies`) added the second. The prior ticket's confusion is now real — the header note carries `tiler.semantic-graph.v2` *and* two `tiler.compiler.request-subject.v5` on one line |
| "Check whether the sentence is still true once the name is corrected" | **the claim survives the step** | Read `compute_graph_identity` in full: under `v3` it still writes the operation key and attributes via `encode_operation`, the ordered operand/result signature, and each result's boundary shape, once per operation over `traversal.operation_order`. `SourcedShape::encode` changed how an extent is *spelled*, not which subjects the graph covers. `derive` fixes the occurrence via `SemanticOccurrence::new(program.canonical_operation_ordinal(operation_ref))` |

**Fact — the claim was true when written, so this is drift and not an authoring error.** `git log -S 'already writes each of them for every operation in canonical'` on `refinement.rs` returns exactly one commit, `6d143a01` (2026-08-04). `git show 6d143a01:crates/tiler-ir/src/semantic/identity.rs` reads `const GRAPH_DOMAIN: &[u8] = b"tiler.semantic-graph.v2\0"`. The step to `v3` landed later, at `26157836` (2026-08-07). `git merge-base --is-ancestor 26157836 6d143a01` returns non-zero, confirming the order. The repair is therefore **dated beside**, matching `docs/artifact-abi.md`'s treatment of the same domain step and this file's own claim-6 precedent, rather than the substitution its `COVERAGE_GRAPH_DIGEST_DOMAIN` sibling used for a claim that was never true.

**Fact — the retired spelling stays greppable, deliberately.** The note quotes `tiler.semantic-graph.v2`, so the file still returns 2 hits for it, both inside the note. The note says so inline. The ADR's *full* quoted sentence does not grep in `refinement.rs` and never did — the doc comment wraps it across three lines at 80 columns, verified 0 hits both at this base and after the edit.

## Out-of-scope finding, for `contracts/decisions`

**ADR 0104's note beside the quotation is what this repair makes stale.** `docs/decisions/0104-…md` carries, after the rejected alternative it quotes: "the doc comment on `IndexRefinementExecutableCoverageIdentity` … **still says `v2`** … **The stale text is the source comment, not this record**, and it is not repairable from this ticket's scopes; it is filed rather than fixed in passing." That was accurate when written and is now false in its present tense — the source comment says `v3` and carries a dated correction. The quotation itself at the same location stays faithful to the retired wording, which the source still quotes verbatim, so **only the note needs the update, not the quotation**. Scope is `contracts/decisions`, not this ticket's.

## Domain-name census for `crates/tiler-ir/src/index/refinement.rs`

27 `tiler.`-spelled occurrences: **13 declarations** (byte-string literal constants, every one pinned in `crates/tiler-ir/src/domains.rs`) and **14 prose occurrences** across 13 doc-comment lines. Reduced to distinct checkable prose domain-name claims, there are **six**:

| # | Prose claim | Verdict |
| --- | --- | --- |
| 1 | The `COVERAGE_GRAPH_DIGEST_DOMAIN` prefix argument over `tiler.ir.` / `tiler.artifact` / `tiler.proof-sidecar.` | already repaired by `correct-the-coverage-graph-digest-domain-…`; re-read, correct as it now stands |
| 2 | `tiler.artifact.route-requirement.v1` | correct — appears only inside that ticket's correction note |
| 3 | "The governed domains are `tiler.contract.f32.v2`…" | **verified true** — matches `F32_NUMERICAL_CONTRACT_KEY_DOMAIN` in `crates/tiler-ir/src/schedule/numerics.rs` |
| 4 | "…and `tiler.contract.bf16.v1`" | **verified true** — matches `BF16_NUMERICAL_CONTRACT_KEY_DOMAIN`, same file |
| 5 | "`tiler.semantic-graph.v2` already writes each of them" | **false** — repaired here to `v3`, dated beside |
| 6 | "named by digest rather than restated, as of `v2` and ADR 0104" | **verified true** — this `v2` is the *coverage* tag, live at `b"tiler.ir.index-refinement-executable-coverage.v2\0"`, not the graph domain. Correctly not swept |

Claims 3, 4, and 6 were not covered as domain-name claims by the sibling's eight-claim cross-crate census and are checked here independently; all three hold.

**The source census does not and should not cover this comment.** `crates/tiler-ir/src/domains.rs` pins `b"tiler.semantic-graph.v3\0"` and scans *literals*: its header records that "a literal inside a `//` comment is skipped, because prose quotes domains constantly and a doc comment is not a declaration". This edit changes prose *about* a domain and no domain spelling, so the census is silent on it by design — correctly, because extending it to prose would flag every dated correction that quotes a retired spelling verbatim, which is the convention these repairs follow.
