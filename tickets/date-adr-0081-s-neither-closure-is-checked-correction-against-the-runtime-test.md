---
id: date-adr-0081-s-neither-closure-is-checked-correction-against-the-runtime-test
title: Date ADR 0081 s neither-closure-is-checked correction against the runtime test
status: done
priority: p3
dependencies: []
related: [correct-the-artifact-abis-claim-that-nothing-asserts-the-kernel-identity-crossing]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, documentation]
---

ADR 0081 item 2's `Correction — 2026-07-26` says "neither closure is checked now. The decision stands; its enforcement is review." A mechanical check has since landed for one of the two closures.

## Facts

**Reported by the worker that found it, not coordinator-verified — check each before editing.** `crates/tiler-runtime/tests/identity_join/main.rs`'s `the_consumer_links_no_compiler_emitter_or_build_provider` parses `Cargo.lock` and mechanically refuses `tiler-build`, `tiler-compiler`, `tiler-cache`, `tiler-metal`, and `tiler-metal-aot` from the transitive closure, with three positive controls.

**Reported: the correction is only half stale, and the surviving half matters.** Still unchecked are the **positive** direction — that the normal direct dependency set is exactly `[tiler-artifact]` — and `tiler-metal-aot`'s empty closure. So ADR 0077's parallel correction reportedly remains accurate, and a repair that reads "this is now checked" without qualification would overstate.

## Fact audit at `96e867a3ec2d370ccac42ebb1273073d45f1effa`

1. **Verified.** `the_consumer_links_no_compiler_emitter_or_build_provider` in `crates/tiler-runtime/tests/identity_join/main.rs` reads and hand-parses `Cargo.lock`, walks `tiler-runtime`'s transitive closure (including development dependencies), proves the three required controls are present, proves `tiler-build` reaches `tiler-compiler`, and rejects reachability to `tiler-build`, `tiler-compiler`, `tiler-cache`, `tiler-metal`, and `tiler-metal-aot`.
2. **Imprecise; repaired above.** The former phrase "the closure is exactly `[tiler-artifact]`" conflated the direct normal dependency set with the lockfile closure the test walks. `crates/tiler-runtime/Cargo.toml` has only `tiler-artifact` in `[dependencies]`, but its test closure intentionally includes `tiler-ir` and `tiler-reference` through `[dev-dependencies]` and transitive dependencies. The test does not assert the exact normal direct set. `crates/tiler-metal-aot/Cargo.toml` still has neither dependency table, and no current test checks that complete empty closure; ADR 0077's correction remains accurate.

## ADR 0081 current-tree claim audit at `96e867a3ec2d370ccac42ebb1273073d45f1effa`

I counted **13** current-tree claim clusters: **6 verified, 3 imprecise, and 4 false**. A cluster is a claim-bearing context paragraph, decision paragraph, consequence bullet, or traceability paragraph that purports to describe live source, manifests, package population, or local records. I excluded normative decisions, alternatives, and expressly historical statements such as the former temporary scope owner. Every table anchor below is a shortest actual source fragment, verified with `grep -F` before commit, not a line-number citation.

| ADR 0081 source anchor | Verdict | Evidence / action |
| --- | --- | --- |
| `the artifact envelope round-trips` | **False** | `prototypes/serial-sum-run/src/proof.rs` retains `dispatch_direct` but its `// ---- the envelope path` calls `DecodedProgram::decode` and `prepare`; a dated correction preserves the earlier observation and records the current route. |
| `missing component is not a Metal component` | **Verified, with historical “missing” tense** | `crates/tiler-runtime/src/lib.rs` states the loader touches no device or platform binding; `Cargo.toml` names only `tiler-artifact` as a normal dependency. |
| `accepted profile withholds a *reusable Metal-runtime* crate` | **Verified** | `docs/architecture.md`'s `This is an unstable prototype packaging profile` still withholds the reusable Metal-runtime crate. Its quoted frontend/cache wording is historical and later amended, not a live omission claim. |
| `device-free half of a Tiler runtime` | **Verified** | `DecodedProgram::decode`, `prepare`, and `route_with_adapter` keep decoding, classification, binding, and one-way routing in the loader while device objects remain on the adapter side. |
| `neither closure is checked now` | **Imprecise** | The 2026-07-26 correction was true on its recorded tree. The runtime test landed later at `ec7fff032dfc8366ea37c60ccded206ec457cf84`; the dated 2026-08-08 correction records its exact negative-reachability coverage and remaining review-only properties. |
| `` `tiler-ir` is absent as a *direct* edge`` | **Imprecise** | It is absent from the normal dependency set, but `crates/tiler-runtime/Cargo.toml` names it in `[dev-dependencies]`; the new correction narrows the sentence accordingly. |
| `item 2 makes it structural` | **Imprecise** | The current test structurally refuses the five named workspace packages, not every possible platform binding or the exact positive normal dependency set; the correction states that enforcement boundary. |
| `admission moves five things together` | **Verified** | Root `Cargo.toml`, `docs/architecture.md`'s `Accepted prototype packaging profile`, `ticketsplease.toml`, and ADR 0081 agree on `tiler-runtime`; `scripts/check_workspace.py` is absent as stated. |
| `workspace carries seven reusable libraries` | **False** | `docs/architecture.md` and `crates/tiler/tests/workspace_population.rs` record twelve reusable libraries, `tiler-conformance` counted apart, and three prototype/integration executables; corrected beside the consequence. |
| `gains a route through the envelope` | **Verified** | The current runner has both the diagnostic direct dispatch and the decoded/validated envelope route; the route remains intentional. |
| `profile still deliberately omits` | **False** | ADRs 0082 and 0088 later admitted the cache and frontend pair. `tiler-candle` and the reusable Metal-runtime crate remain omitted; corrected beside the consequence. |
| `implementation_status: "partial"` | **False as the stated offset reason** | Both offset tickets are `done`; `place_bindings` in `load.rs` evaluates `DecodedBinding::accessible_offset`, `RoutedBinding` carries it, and hosts honour it. The correction retains partial only for the separately evidenced reviewed-draft public-boundary maturity. |
| `## Traceability` | **Verified** | The referenced local records resolve on this base; `make citations` is the final mechanical link check. |

## Cross-scope remainders for coordinator ticketing

Do not edit either owner on this branch. Before this ticket closes, the coordinator should create a separate alignment ticket with both `contracts/foundation` and `implementation/runtime` scopes:

- `docs/architecture.md` says `The exception is the frontier around the frontend` is the first dependency-table property a test can reject, but `the_consumer_links_no_compiler_emitter_or_build_provider` is now a second mechanical dependency-direction check. Reconcile the live packaging-profile enforcement description with both tests.
- `crates/tiler-runtime/Cargo.toml` says ADR 0081 fixes this crate's `dependency closure` at `[tiler-artifact]` while the same manifest declares development dependencies on `tiler-ir` and `tiler-reference`. Reword it to name the normal direct dependency set and distinguish it from the lockfile closure.

## What closes this

The correction dated beside, naming the test and stating precisely which half it discharges and which it does not. **Do not write that the closure is enforced** — the refusing direction is, the asserting direction is not, and the distinction is the whole content.

It was **true when written**, so date beside rather than substitute. Verify with `git show <commit>:<file>` rather than assuming; this is repository practice — several ADRs state it while applying it and none decides it, so cite the practice, not an authority. A retired sentence quoted verbatim stays greppable; say inline that a later hit lands inside your note.

**Do not change what ADR 0081 decides**, and do not touch ADR 0077 — check whether its parallel correction is genuinely still accurate and report, rather than editing a second record on inference.

**Cite by searchable anchor, run its grep before committing, and use `grep -F`.** Anchors fail as absence three ways: a line break inside them, an emphasis marker the source lacks, and unescaped brackets read as a character class.

**Check this ADR's other claims about the tree and name the count.** Sweeps of two sibling ADRs this week found 9 of 17 and 11 of 18 tree-claim clusters false, most predating the landing that prompted the review — so assume the neighbours are unexamined rather than clean.

## Outcome

Landed in `b74e9d31` (`Correct ADR 0081 runtime closure history`, 2026-08-08) and closed in `5d9384dc`. ADR 0081 now preserves the 2026-07-26 statement as historical and adds a dated correction naming exactly what `the_consumer_links_no_compiler_emitter_or_build_provider` enforces: dev-inclusive negative reachability from `tiler-runtime` to the five compiler/emitter/build-side packages. The correction expressly does not claim an exact normal dependency set or an empty `tiler-metal-aot` closure. The same sweep dated the later envelope route, current workspace population and omissions, and implemented byte-offset route without rewriting the accepted record's historical observations.

The two cross-scope remainders above were subsequently completed in `d6fcb5b4` (`Align runtime dependency records`, 2026-08-08). `crates/tiler-runtime/Cargo.toml` now distinguishes its one normal direct dependency from its dev-inclusive resolved closure, and `docs/architecture.md` names the frontend and runtime checks as two bounded checked slices rather than one exhaustive dependency-table authority. No ADR 0077 change was required.
