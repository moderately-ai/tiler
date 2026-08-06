---
id: decide-whether-the-appended-explain-event-steps-the-schema-version
title: Decide whether the appended explain event steps the schema version
status: review
priority: p2
dependencies: []
related: []
scopes: [implementation/compiler, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [explain, identity, schema, decision]
claimed_from: todo
assignee: agent-explain-version
lease_expires_at: 1786038888
---

## The fact (doc-sweep audit 2026-08-06, coordinator-verified at source)

Event tag 13 (`SynchronizationRealization`, with renderer v7's `synchronization:` line) landed in `fece761f` under an unmoved `EXPLAIN_SCHEMA_VERSION = 9` and `EXPLAIN_RENDERER_VERSION = 7` (`crates/tiler-compiler/src/explain.rs:35-36`; the append is documented in the ledger comment at the version block and at the tag site near `:2908`). The append is byte-safe — no earlier record's tag or field layout moves — but a v9 trace's event *vocabulary* is no longer decided by its version alone: two v9 traces from different builds can differ in which tags they may contain.

## The question

Whether the explain schema's versioning contract requires a version step for an appended event tag, or whether appends-only tag additions are admissible under one version with per-tag injectivity reasoning (the discipline the schedule/kernel domains use). The version-block comment's own precedent cuts the other way: v7, v8, and v9 each stepped for additive changes, so the landed append is inconsistent with the file's own history — either the append should have stepped to v10, or the versioning rule should be restated so that a reader knows appends do not step it and must not infer vocabulary from version.

## The work

Read the explain schema's stated versioning rule and its consumers (anything that dispatches on `EXPLAIN_SCHEMA_VERSION` or decodes by tag), decide which world is correct against them, and execute it whole: either step the version with the ledger comment moved in the same commit (an identity-domain step, executed completely per AGENTS.md), or restate the versioning rule at the version block so the append discipline is explicit, with the tag-13 comment aligned. Half-measures — a stepped version with unmoved ledger text, or a restated rule that still implies version-decides-vocabulary — are worse than either whole answer.

## Closes when

The version block's stated rule, the tag-13 record, and every version consumer agree, and a reader of a v9 (or v10) trace knows exactly what its version does and does not promise.

## Outcome (2026-08-06) — the version does not step; the rule is now stated

**Decision: no step. `EXPLAIN_SCHEMA_VERSION` stays 9 and `EXPLAIN_RENDERER_VERSION` stays 7, and the rule that makes tag 13's landing correct is now written where a reader will look.** This was not a fork — the elimination left one survivor, derivation below. No identity moved, so no pin moved.

### Scope added

`contracts/optimizer` (`docs/compiler/**`), added autonomously and recorded here rather than escalated: this ticket's "Closes when" requires *every version consumer* to agree with the stated rule, and `docs/compiler/optimizer.md:703` was the contract sentence stating the rule ("a record added without changing the rendering advances only the schema"). Correcting the code comment while leaving that sentence would have produced two contradictory rules — exactly the half-measure the ticket names. Verified no live claim held `contracts/optimizer` at the time of the edit: the only other in-progress tickets were `land-the-conversion-pair-decomposition-adr` (`contracts/decisions, contracts/navigation, research/numerics`) and `widen-the-staged-realization-law-to-the-registered-elementary-families` (`implementation/ir`, read branch-side from `tkt/widen-…`).

### Consumer enumeration, with each one's actual dependence

Every consumer of `EXPLAIN_SCHEMA_VERSION` is inside `crates/tiler-compiler/src/explain.rs`; `grep -rn EXPLAIN_SCHEMA_VERSION crates/` returns no other crate.

1. `ExplainWriter::new` → `push_trace_preamble(&mut identity_prefix, EXPLAIN_SCHEMA_VERSION, &subject)` (`:1246`, encoder at `:2808`). **Value-only.** The number is four big-endian bytes in the identity prefix. It discriminates nothing within a build, because it is a compile-time constant on every trace the build seals.
2. `ExplainWriter::seal` → `VerifiedExplainTrace { schema_version: EXPLAIN_SCHEMA_VERSION, … }` (`:1734`). **Storage only.**
3. `VerifiedExplainTrace::verify` → `self.schema_version != EXPLAIN_SCHEMA_VERSION` (`:1861`). **The only comparison of the number anywhere, `#[cfg(test)]`, against this same build's constant.** It detects a stale identity and cannot observe a vocabulary: it does not select a decoder, does not gate a tag, and the tags it would need to gate are not decoded at all.
4. `explain_vocabulary_is_append_only_and_versioned` (`:3324`) pins `9`/`7`. **Pin only.**
5. `COMPILATION_EXPLAIN_SCHEMA_VERSION` (`:2157`) is a separate number for the composite domain `tiler.explain.compilation.v1` and does not fold the trace schema version; it folds the nested trace *identities*, which already carry it.

Nothing decodes. `grep -rn "tiler.explain.trace.v1" crates/` returns exactly one hit, the encoder at `:2809` — there is no reader, in this workspace or outside it. The trace is never serialized, never embedded in an artifact envelope, and never cached (`session.rs:1171-1201` states all three with their derivations), and it leaves the crate only as an opaque `VerifiedCompilationExplain` (public surface: `semantic_candidate_count`, `render`, `Debug`, `Eq`) and an `ExplainReport` whose only capability is rendering. The rendered header names the *renderer* version, and both `optimizer.md:711` and `session.rs:1216-1223` refuse the rendered text as a parse target. So no consumer breaks or silently misreads when vocabulary extends under one version — and none could, because no consumer reads bytes back at all.

**The pinned request qualifier is not a consumer.** `request_qualifier` is `stable_qualifier(&subject.canonical)` (`:1259`) and the rendered `request=` is `stable_qualifier(&self.compilation_subject.canonical)` (`:1811`); both fold only `canonical_explain_subject_bytes` (`request.rs:2197`), which does not contain the schema version. **So `ce6f9106c1c5933b` does not fold the schema version and would not have moved even under a step** — checked rather than assumed, per the brief. A step would in fact have moved *no* pinned identity in the workspace: no test pins trace-identity bytes or a digest of them (`explain.rs:4237` asserts only non-emptiness; `:5130` compares two identities to each other). Cheapness is therefore not what decided this.

### Precedent, read honestly

The version block's history is **not** uniform, and the ticket's framing ("v7, v8, and v9 each stepped for additive changes") is half right. Commits found with `git log -G'EXPLAIN_SCHEMA_VERSION: u32 = ' --oneline -- crates/tiler-compiler/src/explain.rs`, each diff read:

- **v9 `bef89656` — forced.** The refusing honourability fact was pushed *inside event tag 10's payload*, so every already-encodable unhonourable record's bytes moved. Renderer 6→7 likewise respelled records that already rendered.
- **v8 `0b7e59d3` — unforced, and the exact analogue of tag 13.** A brand-new `DeferredTargetRequirement` variant at fresh event tag `8`, a fresh disposition tag `16`, and a first renderer spelling. Nothing earlier moved. It stepped both numbers anyway.
- **v7 `99c9c421` — unforced.** `Quantity::Bits` at fresh quantity kind `8`, plus its first unit spelling. Nothing earlier moved. It stepped both numbers anyway.
- **v6 `d1046e45` — forced.** The resolved dtype joined tag 10's payload; existing bytes moved. The renderer correctly did *not* step, because v4 already published the nominal dtype spelling — the one historical case proving the two numbers are governed by two different questions.
- **v5 `727cd8b1` — forced.** `bytes.push(arithmetic.tag())` entered tag 10's payload and changed its spelling. The same landing *also* appended subject kinds 13/14 and disposition 15, which alone would have stepped nothing.

So three of five steps were forced by byte-moving changes; two (v7, v8) were pure appends that stepped without needing to. The precedent does not decide the question in either direction — it only shows the file has been inconsistent, which is why the outcome is a stated rule rather than a number.

### Derivation — why "restate the rule" is the sole survivor

Candidate A: **appends do not step; the version is an injectivity domain.** Candidate B: **appends step; the version is a vocabulary census.**

- **B cannot be stated truthfully, which eliminates it before cost is considered.** A version that steps for everything cannot signal anything: the one question a version exists to answer is whether bytes written under an earlier value are still readable as themselves, and a number that moves on every change carries no answer to it. B is also unimplementable as stated — nothing maps a version to a tag set, so stepping to v10 would leave "which tags may a v10 trace contain?" answerable only by the changelog comment, which is the mechanism in *both* worlds. B buys the reader nothing the comment does not already buy, and charges every trace identity in a build for content that did not change.
- **B contradicts the compiler's own sibling rule, one file away.** `request.rs:2199-2208` steps `canonical_explain_subject_bytes` to `v5` and says why in the negative: "this is a domain step rather than an appends-only re-tag: the per-tag injectivity argument that would license the cheaper option does not close, and half a step is worse than none." That sentence presupposes A. The same rule is stated at roughly twenty tag sites in `tiler-ir`'s schedule, kernel, and program models and at `tiler-artifact`'s stage key, whose `v3` note states the converse case exactly ("a `v2` reader handed `v3` bytes would read the framed identity as the following occurrence, so the separator steps rather than the field being appended silently"). Under B, explain would be the only identity domain in the workspace running a second, contradictory rule, and the next reader who has just read the schedule tags would apply the wrong one.
- **A is the rule that separates fail-closed from silent misread, which is the correctness content of the whole question.** A moved field inside a known tag is misread silently by any reader that survives it — that is why byte-moving changes must step. An appended tag is an unframed payload a decoder cannot skip, so it fails closed on the unknown tag. A steps exactly in the case where silence is possible, and declines exactly where the failure is already loud. B cannot make this distinction because it steps in both.
- **AGENTS.md's own wording assumes A**: "the converse claim, that a change is appends-only, is carried by per-tag injectivity reasoning at each encoding site, not by the gate staying green" — a category that only exists if appends-only changes need not step.
- **Neither world is distinguishable by consumer breakage**, per the enumeration above, and neither is distinguishable by cost. So the decision rests entirely on which rule is true and statable, and only A is both.

No candidate survived alongside A, so this was never a decision for Tom; no public boundary moves (`explain` is a private module, both constants are `pub(crate)`), no accepted ADR is touched or superseded, and ADR 0073 states nothing about stepping.

### Executed

- **`crates/tiler-compiler/src/explain.rs`, version block.** Replaced the changelog-only comment with the stated rule and its reasoning: one rule for both numbers ("a version steps when something a reader already had changes, and does not step when something new merely becomes expressible"), the schema version as an injectivity domain with its sibling authorities cited, the renderer version as presentation under the same shape, an explicit statement that **a version does not promise the vocabulary and a reader must never derive a tag set from one**, the complete consumer enumeration with its non-dependence, and the ledger rewritten to mark each historical step *forced* or *unforced* so the file's history is not read as its rule. Also verified and stated: every tag assigned at v4 retains its v4 value and every addition since took a value above the range then in use (checked by diffing the stage, disposition, subject-kind and quantity-kind tables and the event tags against `727cd8b1^`), and event tag 9 is unused because it named the omitted-record summary the complete-or-refused contract removed at v1 — a gap, not a reservation.
- **`explain.rs`, tag-13 encoding site.** Aligned the comment with the schedule/kernel wording and with the rule: tags 1–12 keep their values and layouts, a reader reaching 13 is reading a record the earlier vocabulary could not express, and the renderer did not step for the same reason.
- **`explain.rs`, `explain_vocabulary_is_append_only_and_versioned`.** Added the tag-13 pin — a valid `SynchronizationRealization` event (`validate()` asserted `Ok`) encoded and its leading byte pinned to `13` — so "appended, and the schema did not step" is a checked claim rather than a comment. Confirmed the check can fail: changing the pin to `14` fails with `assertion `left == right` failed: left: 13, right: 14`.
- **`docs/compiler/optimizer.md:703`.** Corrected the contract sentence that said adding a record advances the schema — quoted in place with its correction date, since that sentence is what made tag 13 read as an omission — and restated the rule, the independence of the two numbers for the corrected reason, the refused vocabulary promise, and the absence of any dependent consumer.

### Commands

`cargo fmt --check` (touched files), `cargo check -p tiler-compiler --all-targets`, `cargo nextest run -p tiler-compiler`, `cargo clippy -p tiler-compiler --all-targets -- -D warnings`, `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-compiler`, `tkt lint`, `git diff --check`, `tkt guard --base 3522b5f2 tkt/decide-whether-the-appended-explain-event-steps-the-schema-version`. No encoding changed, so no workspace-wide run was required; one was run anyway because the ticket touches an identity domain's stated rule.
