---
id: execute-the-doc-drift-sweep-the-audit-enumerated
title: Execute the doc drift sweep the audit enumerated
status: done
priority: p2
dependencies: []
related: []
scopes: [implementation/compiler, implementation/ir, implementation/reference, implementation/build, implementation/metal-aot, implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The work (drift audit 2026-08-06; A4/A1/B spot-verified by the coordinator)

The ~35 mechanically-verifiable doc corrections the cross-cutting sweep enumerated, as one commit: section A's load-bearing claims (A1 two-account admission, A3 multi-output justification, A6 allowed-authority closed-set vs builtin bypass, A7 access-list shape, A14 eight-field count, A15 four destructure sites, the A12/A13 # Errors lists); B's four phantom-boundary sentences (private mods with zero pub items claiming reviewed-draft public boundaries); C's ten outgrown enumerations; D's link levels, byte counts, the orphaned CONTRACT_KEY_DOMAIN block + policy.rs:106 reference, the tensor_role_tag rename, the test-only import. Plus the vocabulary audit's two cooperative.rs corrections (the unreachable phase-0 example; the StagedProducer rationale narrowed for rounds>1 with the third broader-space bullet), the AxisDecode non-closure-under-composition paragraph (the einops '(a b)->(b a)' counterexample — verify the map before writing it), the caps audit's F7/F8 one-liners (cover.rs:1698/:1684 pointing at their documented twins), and the FIRST_INPUT five-vs-ten count line on the fold ticket. A4/A5/A8-A11 are EXCLUDED — they belong to the three filed tickets (reconcile, restate-sequence, renderer-fork).

Every correction is VERIFIED against source before written — the audit is a claim; one prior audit correction in this corpus was itself false. Any item that fails verification is left unchanged and recorded.

## Closes when

The sweep lands as one commit with a per-item verify-then-correct table in the Outcome, full gate green, and no excluded item touched.

## Outcome

**33 of 35 enumerated items verified and corrected; 2 failed verification and were left byte-unchanged.** Every item was checked by reading the site and its construction/consumption context in full before writing. Two audit claims were themselves wrong about magnitude (A15, C8) and were corrected to the verified number rather than the claimed one. A4/A5/A8–A11 and all `docs/` paths were untouched.

### Section A — load-bearing claims

| item | site | check | verdict |
| --- | --- | --- | --- |
| A1 | `tiler-compiler/src/program.rs:356` | `tiler-ir/src/program/verify.rs:407-428` admits two accounts; `program.rs:861` gives a publishing region's last stage empty coverage and `:875` declares the copy | corrected — was "the one documented exception", named only `push_partial_reduction` |
| A3 | `tiler-compiler/src/physical.rs:2188` | `request.rs:3812` "Ordered multi-output programs are admitted"; tests destructure `[product, sum]` | corrected — "the only one there is" was stale |
| A6 | `tiler-compiler/src/explain.rs:1222` vs `:1334` | `push` admits `ProviderRef::builtin` *before* the `allowed_providers` membership test | corrected — the list is not the closed set |
| A7 | `tiler-ir/src/schedule/model.rs:633` | `verify_intrinsic` (`builder.rs:413-460`) gives 4 / variable / 2 / 3 accesses per scalar-program family | corrected — "one read followed by one owning write" holds for no family but the serial reductions |
| A12 | `request.rs` `# Errors` ×4 | `recognize_elementwise` omitted `resolve_elementwise`'s rules; `mint_elementwise` omitted `elementwise-operand` and `elementwise-arity`; `recognize_pointwise` claimed `elementwise-rank`/`operation-set` it never reports and omitted `ShapeProductOverflow`; `recognize_epilogue` omitted `ShapeProductOverflow` and `recognize_epilogue_producer`'s rules | corrected ×4 |
| A13 | `tiler-ir/src/index/refinement.rs` `verify` | the fn returns `SemanticRealizationLawRefused` under `staged-law-requires-region-sequence` before any listed class | corrected (audit cited `:1478`; the heading is at `:1504`) |
| A14 | `schedule/model.rs:1018` | `ResourceRequirements` carries 8 numerical fields | corrected four → eight |
| A15 | `schedule/model.rs:1262` | **audit said three other sites; four verified** — `push_schedule`, `verify_cooperative_semantics`, `verify_reduction`, `cooperative_plan` | corrected to four |

### Section B — phantom public boundaries

`boundary.rs:87`, `frontier.rs:66`, `fusion_legality.rs:37`, `selection.rs:59`. Check: all four are `mod x;` (private) in `lib.rs`, `grep -nE "^\s*pub [a-z]"` returns nothing in any of them, and no `pub use` anywhere in the crate names them. Corrected to internal-authority wording that keeps the draft discipline and says where the acceptance is actually owed — not deleted.

### Section C — outgrown enumerations (10/10 corrected)

metal-aot `lib.rs:16` two → four bullets; macros `lib.rs` heading "four modules" → "three modules and this file" and its map gained `aot` and `family_cfg` (11 modules declared); reference `lib.rs` table gained `conformance` and `quantization` (18 declared, 16 listed); ir `lib.rs` slices sentence now names the layers it actually ships; `program/model.rs:3` accessor list gained execution order, split reductions, publishing copies, and the ABI trio; `program/verify.rs:1` phase list gained split, output, ABI, and routing-commit; `schedule/builder.rs:1` duty list gained the cooperative participant space, staging dataflow, and synchronization realization; `law.rs:243` — "nine" matched neither the 9 variants (8 single-region) nor the 11 constructors, rewritten to be unambiguous against `realizes_region_sequence`; `explain.rs:22` ledger gained event tag 13; `planning.rs:67` allow-reason gained the lowering-resolution stage.

**Finding worth a reader's attention (C9).** `SynchronizationRealization` (event tag 13) and its renderer spelling landed in `fece761f` with `EXPLAIN_SCHEMA_VERSION = 9` and `EXPLAIN_RENDERER_VERSION = 7` *unchanged* — verified with `git show fece761f:crates/tiler-compiler/src/explain.rs` against its parent. No previously encoded trace's bytes move, so the append is byte-safe, but a v9 trace's event vocabulary is not decided by the version alone. The ledger now records this; moving a version is behaviour and was out of scope here.

### Section D

| item | check | verdict |
| --- | --- | --- |
| D1 link levels | every other `docs/decisions/` link is file-relative and correct; `tiler-build/src/{lib,metal_plan}.rs` were the only two `../../` at `crates/<crate>/src/` depth, where siblings `payload_cache.rs`/`plan_artifact.rs` use `../../../` | corrected ×2 |
| D2 byte count | `push_synchronization_subject` pushes 4 tags + 2 flags = 6 bytes | corrected "Five tag bytes" → "Six bytes: four tags and the two fence flags" |
| D3 arithmetic | staged law = 2-access fold + 3-access pointwise = 5, constant bounds 6 | corrected; the constant is untouched and now stated as a margin |
| D4 orphan block | `CONTRACT_KEY_DOMAIN` and `canonical_contract_key` exist nowhere; the 16-line block had been absorbed as the head of `MAX_NUMERICAL_CONTRACT_PREFERENCES`'s doc | orphan removed; `policy.rs:106` repointed at `tiler_ir::schedule::F32_NUMERICAL_CONTRACT_KEY_DOMAIN` |
| D5 `refinement.rs:5526` | **failed verification** — the cited site is a test assertion block; `grep -n import refinement.rs` returns nothing, and all 35 intra-doc link targets in the file resolve | left byte-unchanged |
| D6 `request.rs` ×3 | `TooManyNumericalContracts` no longer bounded by "distinct public contracts"; the contraction arm is one of four, not "a third … neither existing arm"; `governed`'s only callers are tests, contradicting its own `dead_code` reason two lines below | corrected ×3 |
| D7 `tensor_role_tag` | no such function; the real total maps are `push_tensor_role` (selection/frontier) and `push_tensor_role_name` + `tensor_role_name_len` (call_registry) | corrected |
| D8 `builder.rs` ×3 | only 2 of 4 arms carry a prologue (3 bind `FIRST_INPUT`); the distinctness array holds 11 realizations, not six; `partitions: 3, contributors_per_partition: 1` covers three, not four | corrected ×3, plus the same test's "two cases" over four cases |
| D9 `refinement.rs:2533` | **failed verification** — the cited site is a well-formed `Display` impl; no reproducible drift at or near it | left byte-unchanged |

### Vocabulary, AxisDecode, caps, and the fold ticket

- `cooperative.rs:575` — the "read in phase 0, rewrite in phase 2" example is **unreachable**: `StagedProducer` (`builder.rs:1837`) requires every read's writer in a strictly earlier phase, so phase 0 never reads. Replaced with a reachable pair and the reason phase 0 cannot appear.
- `builder.rs` `StagedProducer` rationale — narrowed for `rounds > 1`; now states why a loop-carried tile does not widen it.
- `cooperative.rs:100` third broader-space item — `StagedSpan` *already* carries a per-dimension stride vector across participants; what is absent is a stride *within* one participant's contiguous run. Corrected.
- `AxisDecode` non-closure — counterexample verified by hand before writing: `split(0,[2,2]) → permute([1,0]) → merge([0,1])` on `[4]` sends `0,1,2,3` to `0,2,1,3`, which no single `(linear / divisor) % modulus` reproduces over an operand axis of extent 4. Paragraph states the non-closure and that `recognize_structural_read` refuses the chain at `structural-operand` (`is_leaf` admits only a declared input or the staged value).
- caps F7/F8 — `cover.rs` `.unwrap_or(1)` and the duplication-refusal skip now point at their documented twins (`refused_duplication`'s doc and the anchored loop's comment).
- `admit-a-fold-over-any-declared-input-…` — the "five places" are five *categories*; the sites are ten. Added one Fact line with the category reading and its reproducing command. Three further non-test `FIRST_INPUT` uses bind the strict-affine `u4` component reads and decide no fold, which is why they are out of scope.

### Verification that the checks can fail

The rustdoc step was perturbed deliberately. A broken intra-doc link in a **private** fn's doc (`push_synchronization_subject`) passed `RUSTDOCFLAGS="-D warnings" cargo doc` with exit 0 — rustdoc never resolves links in undocumented private items, so that check is vacuous there. The same link on a **public** field (`IndexRegion::accesses`) failed with `error: unresolved link to NoSuchItemHere` and exit 101. Both perturbations were reverted and the file re-verified at its corrected diffstat.

### Commands run

`cargo fmt --check`; `cargo check --workspace --all-targets`; `cargo clippy -p tiler-ir -p tiler-compiler -p tiler-build -p tiler-macros -p tiler-metal-aot -p tiler-reference --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` over the same six packages; `tkt lint`; `git diff --check`; `tkt guard`; `make full`.

### Scope

All edits fall in `implementation/{ir,compiler,build,frontend,metal-aot,reference}` plus `project/tickets`. No `docs/` path, no behaviour change, no public signature change: every edit is a doc comment, an ordinary comment, an `#[allow]` reason string, a relative link path, or ticket prose.

`project/tickets` was added to `shared_scopes` autonomously after `tkt guard` reported it under-declared. It is required because the assigned work includes the FIRST_INPUT count line on `tickets/admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary.md`, and because this ticket's own file carries the claim and this Outcome — the guard does not treat either as implicitly shared. It belongs in `shared_scopes` rather than `scopes` for the reason the guard's own output shows: every open ticket declares it, so an exclusive claim would collide with all of them. Guard is clean on the re-run below; the remaining `implementation/{artifact,cache,metal,runtime}` scopes in its report are reverse-dependency expansion of the declared crates, not files this branch touched.
