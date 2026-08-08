---
id: correct-the-every-ir-domain-opens-tiler-ir-premise-in-two-places
title: Correct the every-ir-domain-opens-tiler-ir premise in two places
status: in-progress
priority: p2
dependencies: []
related: [correct-the-coverage-graph-digest-domain-s-eight-count-and-hyphenated-artifact-prefix]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [identity, digest, documentation]
claimed_from: todo
assignee: coord
lease_expires_at: 1786182999
---

A no-prefix argument rests on the premise that every domain the shared IR admits opens `tiler.ir.`. **Most do not.** The conclusion survives; the reasoning does not.

## Facts

**Reported by the worker that repaired the sibling comment, not coordinator-verified — check each before editing.** `crates/tiler-artifact/src/domains.rs` states the premise, and the same sentence appears in `docs/artifact-abi.md` (scope `contracts/artifacts`, **not this one** — report it, do not reach). **24 of 38** `tiler-ir` domain strings do not open `tiler.ir.`, including `EXPR_DOMAIN = b"tiler.artifact-program.abi-expr.v1\0"`, which opens the **same prefix** as `tiler-artifact`'s program-identity container.

**Reported: no collision results.** All 684 cross-crate pairs are clean, so this is a reasoning defect rather than a correctness one. **Verify that independently** — the sibling re-derived it from source literals rather than inheriting it, and found the cross-crate set is the one the argument actually ranges over.

### Per-Fact audit, 2026-08-08, at base `acc26984`

- **Both sites carry the premise — verified.** `crates/tiler-artifact/src/domains.rs` and `docs/artifact-abi.md`. `grep -rn -F 'every domain the shared IR admits opens' crates/ docs/` returns both. The second is `contracts/artifacts` and was not touched.
- **`EXPR_DOMAIN` — verified.** `crates/tiler-ir/src/program/abi.rs`, anchor `const EXPR_DOMAIN`, spelled `tiler.artifact-program.abi-expr.v1\0`. It is the only `tiler-ir` spelling inside `tiler.artifact`, and it shares 23 bytes with this crate's `ARTIFACT_DOMAIN` (`tiler.artifact-program.`).
- **"24 of 38" — false, in both numbers.** The authoritative `tiler-ir` population is `PINNED_IDENTITY_DOMAINS` in `crates/tiler-ir/src/domains.rs`, which that crate's own header calls "the sixty spellings pinned below". It has **60** rows; **14** open `tiler.ir.`, so **46** do not. Neither 24 nor 38 appears in any source.
- **"684 cross-crate pairs" — false.** 684 = 18 × 38. The real product is 18 × 60 = **1080**.
- **"No collision results" — verified, and it is the one claim that survives.** 0 prefix relations across all 1080 cross-crate pairs, and 0 across the 153 pairs within `tiler-artifact`. Within `tiler-ir` there are 3, all from the terminator-free `tiler.scalar` against its own `tiler.scalar-*` spellings; that is `tiler-ir`'s business, not this obligation's, and it does not reach `tiler-artifact`.
- **"703 within `tiler-ir`" — false.** 703 = C(38,2). Over the real 60 rows it is C(60,2) = **1770**.
- **The prescribed repair shape is itself false.** "First differing byte after `tiler.`" does not separate the sets: both sides use `a` and `p` as that byte. Seven `tiler-artifact` domains agree with `EXPR_DOMAIN` for 23 bytes, and the four sidecar domains agree with `tiler.prepared-entry-target-requirement.v1` for 8. `docs/artifact-abi.md` **already** carries this shape ("the two sets diverge at the first byte after the shared `tiler.`"), so it is a second false claim at that out-of-scope site, not a fix for the first.
- **"8 of 18 appears in no source file" — false.** It is in the commit message of `96dfe333`: "It covered 8 of the crate's 18 governed domains." Both figures are real and describe the same retired array under different populations: the retired test was named `no_governed_domain_of_either_container_prefixes_another`, and envelope (7) + sidecar (4) = 11, of which it listed 8; the crate's whole admitted set was 18, of which it listed 8. The module header stated only the first without its scope, which understates the gap by the artifact program's seven domains. Disambiguated in place rather than replaced.
- **Ever-true verdict: never true at any commit.** `d1a95e18` (2026-07-25) relocated `EXPR_DOMAIN` into `tiler-ir`; the `docs/artifact-abi.md` sentence landed at `d48a33af` (2026-08-06) and the `domains.rs` sentence at `96dfe333` (2026-08-08). At `d48a33af` 43 of 57 `tiler-ir` NUL-terminated spellings did not open `tiler.ir.`; at `96dfe333`, 44 of 58. Treatment is therefore substitution with the retired wording quoted, not a date-beside.

### Neighbouring-claim census

**18 claims** read across the module header and the repaired test's doc comment. One false (the premise), one imprecise (`8 of 11` without its scope), one unsound as stated (see below); the other 15 verified. The unsound one: "no crate can hold the union — `tiler-artifact` depends on `tiler-ir` and not the reverse" cites the dependency direction that would *permit* the depending crate to hold a union check. The real obstruction is that `tiler-ir` exports no enumeration of its domains — `PINNED_IDENTITY_DOMAINS` is a private `const`, and the crate declares no `pub` `&[u8]` domain constant. Repaired with the conclusion intact and the reason corrected.

## Why it matters despite no live collision

An argument that reaches the right answer by a false route will keep reaching it only by luck. The premise says the two crates' namespaces are disjoint by construction; they are not, and `EXPR_DOMAIN` is the counterexample sitting inside the very prefix the argument claims separates them. The next domain added under that prefix has nothing but coincidence keeping it clean.

## What closes this

The premise restated so it says what actually separates the domains — the sibling's repair rests on the **first differing byte after `tiler.`** rather than on a prefix quantifier, which is the shape to follow.

**Do not restate a count.** Prose cannot size itself from a type; delegate to `GovernedDomain` as the sibling did. A bare "eighteen" or "thirty-eight" here rots on exactly the schedule the retired "eight" did — and note that "8 of 18", which several tickets have repeated, **appears in no source**: the module header says the retired check covered **8 of 11**. Take counts from source, not from other tickets.

**Establish whether this was ever true** with `git log -S` before choosing a treatment: a claim true when written is dated beside, one never true is substituted with the retired wording quoted. That is repository practice, stated in several ADRs while applying it and decided by none — cite the practice, not an authority. A retired sentence quoted verbatim stays greppable, so say inline that a later hit lands inside your note.

**Cite by searchable anchor and run its grep before committing to it.** The sibling had an anchor fail because it spanned an 80-column break and caught it before shipping; doc comments here wrap.

Check the neighbouring claims and **name the count** — every sweep of these files this week found more than it was sent for.
