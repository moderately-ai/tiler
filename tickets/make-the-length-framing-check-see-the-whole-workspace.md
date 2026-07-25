---
id: make-the-length-framing-check-see-the-whole-workspace
title: Make the length-framing check see the whole workspace
status: closed
priority: p1
dependencies: []
related: [finish-consolidating-tiler-ir-length-framing, pin-the-admitted-unsafe-sites-in-the-workspace-gate, implement-boundary-property-model]
scopes: [implementation/compiler, implementation/reference, implementation/metal-aot, implementation/cache, implementation/workspace, implementation/ir, implementation/artifact, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, identity, gate]
---
The canonical length-framing consolidation was never workspace-wide, and the test that enforces it is structurally incapable of noticing.

**Fact — the enforcing test is crate-local.** `crates/tiler-ir/src/identity.rs::length_framing_has_exactly_one_definition_in_this_crate` walks `env!("CARGO_MANIFEST_DIR")/src`. Its own documentation says it exists because "stating a convention did not hold it" after five copies grew inside `tiler-ir`. It cannot see any other crate, and its name says so — but the module doc above it states the rule as though it governed identity derivation generally.

**Fact — `implement-boundary-property-model` found a sixth and seventh copy in `tiler-compiler`** (`frontier.rs`, `selection.rs`) and consolidated them. That work is done; it is cited here as how the gap surfaced, not as remaining work.

**Fact — copies remaining at `8674378`, from `grep -rn -F <pattern> --include='*.rs' crates prototypes`:**

| Crate | Sites | Reaches `tiler-ir`? |
| --- | --- | --- |
| `tiler-compiler` | `encode_len` in `cover.rs`, `fusion_legality.rs`, `legality.rs`, `capability.rs`; `encode_bytes` in those four plus `region.rs`, `explain.rs` | yes |
| `tiler-reference` | `encode_len`, `encode_bytes` in `lib.rs` | yes |
| `tiler-metal-aot` | `push_len` in `identity.rs` and `family.rs`; `push_slice` in `family.rs` | **no** |
| `tiler-cache` | assorted `.len() as u64` in `bundle.rs`, `store.rs`, `collect.rs`, `tests.rs` | **no** |

**Fact — the dependency closures decide which of those are defects.** `crates/tiler-compiler/Cargo.toml` and `crates/tiler-reference/Cargo.toml` both declare `tiler-ir`, so their twelve sites are ordinary duplication and consolidating them is mechanical. `tiler-metal-aot` declares no workspace dependency and `scripts/check_workspace.py` pins `"tiler-metal-aot": []` (ADR 0077 item 2), so **its copies are forced by an accepted decision, not a defect against `crate::identity`** — but it holding *two* copies of one framing is a defect within that crate. `tiler-cache` declares only `tiler-artifact`, and a transitive dependency is not usable in Rust without declaring it.

**Do not sweep `tiler-cache` on the grep.** Read each site: `if bytes.len() as u64 > limit`, `self.entries.len() as u64`, and `selected: selected.len() as u64` are *counting and comparison*, not canonical framing, and rewriting them onto a framing helper would be wrong. `AGENTS.md`: grep counts are not scope assessments, and duplication must be understood before it is consolidated.

**Do not touch the two deliberate independent assertions.** `crates/tiler-ir/src/shape/env.rs:1218` and `crates/tiler-compiler/src/feasibility.rs:1458` spell the eight-byte prefix out by hand *in tests*, and `identity.rs`'s own documentation states that this independence is exactly what would catch the framing width changing — a test that checked the encoder with the encoder's own helper could not. Both are inside `#[cfg(test)]`, which is why the existing crate-local check already skips them; whatever replaces it must skip them for the same stated reason rather than by accident.

## Closes when

The twelve `tiler-compiler` and `tiler-reference` sites use `tiler_ir::identity`; `tiler-metal-aot` holds exactly one definition of its own framing with a comment naming the closure decision that forces it to have one at all; each `tiler-cache` site is classified as framing or counting with the framing ones consolidated onto whatever single definition that crate is permitted; the enforcement sees every workspace member rather than one crate; and the full gate passes.

**The enforcement's shape is the design question.** The established instrument is `scripts/check_workspace.py`, which already owns workspace-wide mechanical predicates and already carries admitted-exception tables with reasons — `ADMITTED_UNSAFE_SITES` pins `(path, item signature, reason)` triples so that adding, moving, renaming or rewording one fails the gate until the pin is updated in the same change. A closure-forbidden crate needs exactly that treatment: its copy is admitted, with the decision that forces it named beside it, so a *second* admitted copy is a diff someone must look at. Decide whether the crate-local test is then retired or kept as a faster inner loop; keeping both is defensible but only if the module documentation stops implying the crate-local one is the authority.

**A caution the work should not repeat.** The existing check reports success by finding nothing, and finding nothing is what a broken search also does — this ticket was itself opened after a first sweep returned six clean results because `zsh` expanded an unquoted `--include=*.rs`. Whatever check lands must name and count its population, so that a scan matching zero files fails loudly instead of passing.

## Outcome

Landed. The rule is now enforced by `scripts/check_workspace.py` over every workspace member, and the crate-local `tiler-ir` test is retired.

**The evidence table above was incomplete, and it is measurable by how much.** Running the landed scan against a `git archive` of base `736350f` reports **42 recognized sites** with no scan errors; HEAD reports **9**, all admitted. The table named 15 of the 42. The reproduction is one line: `scan_length_framing_sites(<tree>, PACKAGE_DIRS)` from `scripts/check_workspace.py`, against an extracted base tree.

The table found sites by grepping four helper names. The check as landed recognizes a *shape* — a `&mut Vec<u8>` sink plus one `usize`/`&[u8]`/`&str` payload, or one statement that both reads a length and writes it as fixed-width bytes. The four classes a name grep cannot reach:

- `tiler-compiler`'s `region.rs` named its length helper `encode_count`, which no listed name matched, and `fusion.rs` and `request.rs` each held a whole helper — `encode_evidence_bytes`, `encode_explain_bytes` — under a name nobody would guess. Fifteen helper definitions moved onto `tiler_ir::identity` in all, not twelve.
- **Thirteen copies had no helper at all**, writing `u64::try_from(x.len()).unwrap_or(u64::MAX).to_be_bytes()` inline: five in `explain.rs`, five in `request.rs`, and **three inside `tiler-ir` itself** in `index/integer.rs` and `semantic/operation.rs`. The crate-local test was supposed to prevent exactly those three; its `.len() as u64` pattern cannot see this spelling. That incompleteness is measured rather than asserted, and it is why the test was retired instead of kept as a faster loop — two recognizers for one rule is the divergence this module exists to prevent.
- `tiler-cache` held a genuine framing pair, `push_count`/`push_run` in `expansion/subject.rs`, that the table did not mention. The table listed only that crate's *counting* sites, and its caution against sweeping those was correct — but the crate's real framing question was elsewhere, and it is the one that decided the dependency question.

**`tiler-metal-aot` held two framings, not two copies of one.** `family.rs` framed with `u32::try_from(...)` — **four bytes** — while `identity.rs` framed with `u64` — eight. Both carried the same doc sentence, "Writes a fixed-width big-endian count before a repeated run", so nothing but the `u32` literal distinguished them. This contradicts the premise that the copies differ only above `u64::MAX`.

**Canonical bytes: exactly one subject moved.** Every `tiler-compiler`, `tiler-reference`, and `tiler-ir` consolidation is byte-identical. The three replaced spellings are `u64::try_from(v).unwrap_or(u64::MAX)`, `u64::try_from(v).expect(...)`, and `count(v)` (itself the first), and on a 64-bit host `u64::try_from(v: usize)` is total and returns `v`, so all three equal `(v as u64).to_be_bytes()`. `scripts/check_rust.py:435` rejects any host that is not 64-bit little-endian. Behaviour differs only above `u64::MAX`, where the new form panics instead of saturating — the fail-loud direction, and the one `identity.rs` already documents.

`ArtifactFamilySelection::canonical_bytes` in `tiler-metal-aot` **does** move, from four-byte to eight-byte prefixes. Nothing observable moves with it: the method is `pub(crate)` inside a private `mod family`, `grep -rn canonical_bytes crates prototypes spikes` returns no non-test caller, no test pinned exact bytes, and the crate computes no digest (ADR 0077 item 2). A new test, `the_family_count_carries_the_eight_byte_framing`, now spells the eight-byte prefix out by hand — independently of the encoder, for the reason `shape/env.rs` and `feasibility.rs` do.

**Two crates keep an admitted copy, each forced by an accepted decision.** `tiler-metal-aot` under ADR 0077 item 2, whose closure is pinned empty. `tiler-cache` under ADR 0082 item 2, which decides its closure is exactly `tiler-artifact` and says in terms that `tiler-ir` is "an edge this record decides the crate does not have" — so declaring it was eliminated rather than chosen against. Each crate now holds exactly one framing definition, documented with the decision that forces it, and the pin's citation field is checked against that documentation.

**Three sites are classified as *not* framing and pinned as such.** `tiler-cache`'s `bundle.rs` section count and `tiler-artifact`'s `encode.rs` section count are fixed-offset fields of decodable containers — written at named offsets, read back by `read_u64`/`cursor.u32()`, with content located by explicit descriptor offsets rather than by following a prefix, and never digested into an identity. `tiler-artifact`'s is four bytes because the envelope's tables are `u32`-indexed; widening it would change the artifact ABI. `tiler-ir`'s `semantic/identity.rs::encode_string` is a `&str` adapter that delegates to `push_slice`. All three are pinned rather than exempted by shape, so a body that stopped delegating is a diff someone must look at.

**The check's failure path is reachable.** `validate_length_framing_pins` names its population — packages scanned and production files per package — and fails when no package was scanned, when any package contributed zero files, or when the admitted table is empty. `test_an_empty_population_fails_rather_than_reporting_success` and `test_a_scan_over_no_package_at_all_fails` exercise both. Test-only code is excluded by resolving `#[cfg(test)] mod name;` declarations to their files (nine such files exist) and by balancing each `#[cfg(test)]` item's braces rather than truncating at the first one, so production code after a test module is still scanned. The two deliberate independent assertions in `shape/env.rs` and `feasibility.rs` are skipped for that stated reason.

**Stated bounds.** A framing helper written as a method, or over a sink type other than `&mut Vec<u8>`, is not recognized. A length bound to a local by one statement and written by another is not recognized; `bundle.rs` writes its section-descriptor lengths that way and they are classified with the section count that *is* recognized.
