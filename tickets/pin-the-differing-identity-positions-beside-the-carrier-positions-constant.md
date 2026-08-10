---
id: pin-the-differing-identity-positions-beside-the-carrier-positions-constant
title: Pin the differing identity positions beside the carrier positions constant
status: done
priority: p2
dependencies: []
related: [replace-the-stale-artifact-abi-byte-figures-with-the-properties-tests-pin, recompute-the-unasserted-bf16-byte-lengths-in-the-dtype-support-matrix, retire-the-false-unasserted-identity-difference-prose-after-the-pin]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, identity, tests]
---

Two documents carried a BF16-versus-F32 identity difference of four bytes as prose, with nothing asserting it. Both retired the figure because it was unasserted and its neighbours had rotted. **The property was worth keeping — it needed to live in a test rather than in a sentence.** That pin has landed; the work order below is historical.

## Facts

**Reported by the worker that retired the prose figures, not coordinator-verified at open.** `a_bf16_artifact_round_trips_and_its_carrier_enters_identity` in `crates/tiler-artifact/src/program/codec/tests.rs` already derived both identities, so the assertion had no new setup cost. **The open proposal was** `DIFFERING_IDENTITY_POSITIONS: usize = 4` beside the existing `DIFFERING_CARRIER_POSITIONS: usize = 68`, plus an assertion that the two identity runs are equal in length before comparing them. **Delivered** — see Outcome.

**Coordinator-verified:** `DIFFERING_CARRIER_POSITIONS = 68` is asserted in that file, anchored by the constant name. The measured identity difference reproduced at **4** in two independent regenerations, one per document, measured from separate constructions.

## The design point, and it is the reason this is its own ticket

**Keep the two constants separate subjects.** 68 covers two digests and can move by coincidence — a prior landing saw it return to 68 for reasons that were *not* a revert, and the worker who found that checked rather than assuming. 4 is exactly the two tag pairs and nothing else. Folding them into one assertion, or deriving one from the other, would lose the property that makes 4 meaningful.

**Do not pin the byte offsets.** The retiring worker deliberately declined to propose that, and the evidence is decisive: across one identity step the four offsets moved in **opposite directions** — component tags forward by 7, binding-row offsets backward by 8,452. An offset is a position in a layout that is free to move; the count of differing positions is a statement about what a carrier change *means*. Pin the second, not the first.

## What closed this

The constant added, the length-equality precondition asserted, and the count asserted against it. **Perturb the subject and quote the failure text** — change the carrier so a third position differs, or make the identities unequal in length, and show what each assertion said. Perturb the two properties **separately**; a perturbation that reddens both cannot show which one is load-bearing, and this ticket exists precisely because two counts were conflated.

Do not restate either count in prose anywhere as an unasserted measurement. Both documents retired their copies; reintroducing a bare number that a test owns is how this drifted the first time. Live contracts that still claim the pin has not landed are out of this ticket's scopes and owned by the remainder named in the Outcome.

## Outcome

Landed at `b03f2b81` (`Pin the differing identity positions beside the carrier positions constant`), touching only `crates/tiler-artifact/src/program/codec/tests.rs`.

- `const DIFFERING_IDENTITY_POSITIONS: usize = 4` sits beside `const DIFFERING_CARRIER_POSITIONS: usize = 68` as a separate subject, deliberately neither folded into nor derived from the carrier constant.
- In `a_bf16_artifact_round_trips_and_its_carrier_enters_identity`: equal identity lengths are asserted first (`a carrier enters the identity as a tag, so the two identity byte runs must be equal in length`), then the positional count is asserted against `DIFFERING_IDENTITY_POSITIONS`.
- Offsets of the four differing identity positions are deliberately not pinned.
- Watched failing, separately, by temporary perturbation of the identity encoder in `program/model.rs` (reverted in the landing): writing the component carrier tag twice yields `left: 5, right: 4` on the identity count while length equality and `DIFFERING_CARRIER_POSITIONS` still pass; appending a Bf16-only identity byte yields `left: 40133, right: 40132` on length equality before the count runs.

**Contracts were out of scope** (`implementation/artifact` + `project/tickets` only). Live false "unasserted" / "What is left unpinned" prose remains in `docs/artifact-abi.md` and `docs/dtype-support.md` until [`retire-the-false-unasserted-identity-difference-prose-after-the-pin`](retire-the-false-unasserted-identity-difference-prose-after-the-pin.md) rewrites those paragraphs under `contracts/artifacts` and `contracts/navigation`. Do not re-open this implementation ticket for those files.

Status: `done` for the code pin. Closed on the landing above; this Outcome records what the status flip omitted.
