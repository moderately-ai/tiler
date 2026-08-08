---
id: recompute-the-unasserted-bf16-byte-lengths-in-the-dtype-support-matrix
title: Recompute the unasserted BF16 byte lengths in the dtype support matrix
status: done
priority: p2
dependencies: []
related: [carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [dtype, artifact, identity, documentation]
---

`docs/dtype-support.md` states five BF16 artifact byte lengths that **no test asserts**, so nothing goes red when they drift. The derived index-arithmetic step grew the artifact envelope, which makes them stale in a way the gate cannot see.

## Facts, verified 2026-08-08 by the coordinator at the merge that moved the envelope

**The heading read "that grew the envelope" and was corrected 2026-08-08 by the worker, because the direction is the whole point of this ticket.** `FIXED_CONTENT_BYTES` — a golden for the *metal plan's* published envelope in `crates/tiler-build/src/metal_plan.rs` — did grow `65_308 -> 65_313`. The four BF16 figures this ticket is about belong to different artifacts, and every one of them **shrank**, by up to 57,012 bytes; see the Outcome. A worker who read "grew" as licence to add five would have produced numbers wrong by more than half.

**Fact — corrected 2026-08-08 by the worker at `0f319ec8`; the original claimed five numbers and there are four.** `97,060`, `90,806`, `45,457`, and `73,556` live in a single paragraph of `docs/dtype-support.md`, the one whose first sentence is the searchable anchor *"BF16's physical-carrier, ABI, and kernel-vocabulary cells moved on 2026-08-05"*: `97,060` in the carrier-only forged pair's round-trip clause, the other three in the pure-BF16 producer clause anchored at *"carried a pure-BF16 program from semantic construction through verified coverage"*. ~~A fifth, `36,832`, appears in the same region.~~ **False.** `grep -c "36,832" docs/dtype-support.md` returns `0`; that figure is `docs/artifact-abi.md`'s, scope `contracts/artifacts`, and is not reachable from this ticket.

**Fact — two further numbers in the same clause were unaudited and belong to it.** *"differing at exactly forty positions"* is **false** at this base: `DIFFERING_CARRIER_POSITIONS` (`crates/tiler-artifact/src/program/codec/tests.rs`) is `68`, asserted by `a_bf16_artifact_round_trips_and_its_carrier_enters_identity`, and `docs/artifact-abi.md` had already corrected its own copy to 68 while this one stayed at 40. *"the two canonical identities differ at exactly four bytes"* is unasserted; measured, it still holds.

**Fact.** `grep -rn "97060\|90806\|45457\|73556\|36832" crates/ prototypes/` returns **nothing**. No test, golden, or pin asserts any of them. This is the whole reason the ticket exists: a documented measurement with no assertion behind it is a claim that decays silently.

**Fact.** The envelope grew by exactly five bytes in `carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit`: `FIXED_CONTENT_BYTES` moved `65_308 -> 65_313`, and that worker decomposed the +5 as five insertions of the literal `0x01` — one entry-row `resources` plus four embedded kernel identities. `tiler.artifact-program` moved `v15 -> v16` and `MANIFEST_SCHEMA` `(15,0) -> (16,0)` in the same step.

**Inference, and the trap this ticket exists to stop.** It is tempting to add five to each number and call it recomputed. **Do not.** The +5 was measured on one specific envelope; these five figures describe different artifacts (a forged carrier-only pair, a pure-BF16 producer artifact, its identity, and an F32 twin), and an identity length in particular has no reason to track a content-byte delta at all. Each number must be regenerated from the construction that produced it.

## What closes this

Either each figure recomputed on the merged tree from its own construction and restated with the date it moved, **or** — better, and the reason this is `contracts/navigation` rather than a doc typo — a decision that prose should not carry unasserted byte counts at all. If a number is worth stating it is worth pinning; if it is not worth pinning, stating it invites exactly this drift. A worker choosing the second path should say what replaces the figures and confirm nothing else in the document cites them.

Check the surrounding paragraphs for the same shape before closing: this is unlikely to be the only place a measurement was written into prose without an assertion behind it. Report the census either way, so "none found" is distinguishable from "did not look".

## Outcome

**Measurement, at `0f319ec8`, `cargo nextest run -p tiler-artifact -E 'test(a_bf16_artifact_round_trips_and_its_carrier_enters_identity) or test(a_producer_built_bf16_artifact_round_trips_and_re_derives_its_identity)' --no-capture`, with temporary `println!`s added to those two tests and reverted before commit (`git status` clean, `git diff --stat` empty).** The `+5` inference the Facts above warned against would have been wrong by more than fifty thousand bytes in the wrong direction: the envelope did not grow, it **shrank**.

| Figure | As stated | Measured at `0f319ec8` | Delta |
| --- | --- | --- | --- |
| forged pair, BF16 envelope | 97,060 | **40,048** | −57,012 |
| forged pair, F32 envelope | (equal, asserted) | **40,048** | equality holds |
| forged pair, canonical identity | 48,584 (`artifact-abi.md`) | **40,132** | −8,452 |
| forged pair, identity differing bytes | 4 | **4** | unchanged |
| forged pair, envelope differing positions | forty (`dtype-support.md`) | **68** | pinned; the doc copy was false |
| producer, BF16 envelope | 90,806 | **39,781** | −51,025 |
| producer, BF16 identity | 45,457 | **39,865** | −5,592 |
| producer, F32 envelope | 73,556 | **31,212** | −42,344 |
| producer, F32 identity | 36,832 (`artifact-abi.md`) | **31,296** | −5,536 |

**Decision — the second route, and no pin is recommended.** `docs/dtype-support.md` no longer states any absolute byte length that no test asserts. Each struck figure was stated to support a property that is already pinned: the forged pair's **length equality** and its `DIFFERING_CARRIER_POSITIONS` count in `a_bf16_artifact_round_trips_and_its_carrier_enters_identity`, and the producer pair's **length inequality** there plus the identity inequality and the `(12, 24)` binding windows in `the_bf16_artifact_and_its_f32_twin_are_two_artifacts` (`crates/tiler-artifact/src/program/tests.rs`). Pinning the absolute lengths would be actively wrong, and the codec test says so in its own comment: an identity step moves them and they carry no information the two inequalities do not. That prediction was borne out — the `v15 -> v16` step moved all of them, by up to 57,012 bytes, with nothing going red. The one number in the clause that *is* worth pinning was already pinned, and the correct repair for it was to stop copying its value into prose and name the constant instead.

**Census — the same shape, elsewhere in `docs/dtype-support.md`: none found, over the document's full measurement-shaped population.** Every numeral in the file was enumerated (`grep -on "exactly [a-z0-9,]*\|[0-9][0-9,]\{2,\}\|thirty[a-z ]*\|forty[a-z ]*\|..." docs/dtype-support.md`), ADR/ticket identifiers, dates, tags, and toolchain version strings set aside, and each survivor resolved to its construction:

- **Pinned in a workspace test.** `65,536` encodings and `254` NaNs — `assert_eq!((zeros, subnormals, normals, infinities, nans), (2, 254, 65_024, 2, 254))` and the `65_536` sum, `crates/tiler-reference/src/bf16/tests.rs`. `0x7fc0` — `CANONICAL_NAN`, same file and `crates/tiler-conformance/src/bf16_vertical.rs`. `30` hand-derived witnesses — `assert_eq!(total, 30)`. "exactly four named witnesses" — the four names asserted verbatim in `changing_only_the_tie_rule_breaks_the_corpus`. "ten witnesses" — `BF16_WITNESSES`, all ten indices asserted individually in `a_bf16_kernel_agrees_with_the_reference_oracle_bit_for_bit`. "fifteen hand-derived corpus elements" — `assert_eq!(cases.len(), 15)`. "the five subnormal operands" — `the_declared_flush_moves_exactly_the_subnormal_operands` names five positions. `8040 -> 8000` — `NEG_HALF_MIN_NORMAL`, asserted.
- **Not a measurement.** BF16's 8 significand bits against binary32's 24, the six OCP MX schemes, block size 32, and the `65,536`/`128` cardinalities in the F16/F64/F128 dry-run table are format parameters and spec constants.
- **A dated device or research measurement, bounded to its host and labelled `Measurement`.** The F32 proof's thirty cases on one Apple M4 Max; the quantized profile's `0 of 18` greedy-token positions and `196` weighted projections. These are a different evidence class from an artifact byte length: they are bounded to a named host and corpus and cannot be pinned by a workspace test, which is why the document states them as measurements with their boundary rather than as claims a gate holds.

## Out of scope, for a `contracts/artifacts` ticket

`docs/artifact-abi.md`'s *"Measurement, of the carrier-only forged pair"* and *"Measurement, of the producer-path pair"* clauses carry six stale figures: the identity length `48,584` (now 40,132), the four offsets `3,104` / `3,106` / `47,898` / `47,899` — the last two of which are now **past the end of the identity**, so a reader following them lands nowhere — and `90,806` / `45,457` / `73,556` / `36,832`. Its differing-position count and its own account of pinning are correct and should not be touched. The same document's neighbouring paragraph is the precedent for the repair: *"Measurement, and it is now pinned by a test rather than carried as prose here."*
