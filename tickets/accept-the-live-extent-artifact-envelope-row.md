---
id: accept-the-live-extent-artifact-envelope-row
title: Accept the live-extent artifact envelope row
status: in-progress
priority: p1
dependencies: [associate-live-extent-operands-with-symbolic-semantic-interface-axes]
related: [carry-live-extent-operands-through-the-artifact-envelope, bind-frozen-live-extent-bytes-at-declared-backend-transports]
scopes: [contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
claimed_from: todo
assignee: worker-envelope
lease_expires_at: 1787425502
---
## User-visible outcome

Tom accepts or revises the exact included and excluded public/schema surface of the live input-extent artifact envelope row, so the labelled draft on `DecodedExtentOperand` / `DecodedEntry::extent_operands` / `EntryRef::extent_operands` can stop being a draft.

## Decision boundary

[ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) routes every concrete public surface to Tom. [`carry-live-extent-operands-through-the-artifact-envelope`](carry-live-extent-operands-through-the-artifact-envelope.md) added the envelope row that [`accept-the-live-extent-operand-public-surface`](accept-the-live-extent-operand-public-surface.md) deliberately excluded. This node is not implementation work. Only Tom closes it.

## The surface, as drafted

**Included — decoded dispatch record.** `tiler_artifact::program::DecodedExtentOperand` with `key()`, `axis()`, and `value_type()`. `DecodedEntry::extent_operands` returns those rows in canonical `(key, axis)` order. Empty for every entry whose kernel declares no live extent. The live *value* is not on the row.

**Included — verified artifact view.** `EntryRef::extent_operands` returns the same `DecodedExtentOperand` rows. Construction derives them from the bound kernel's `InputExtentParameter` list through the stage access that names the program-interface key. Callers do not supply a second list.

**Included — encode / decode / validate.** A nonempty list is appended after the backend entry key under presence tag `0xfe`. Empty writes nothing, so previously encodable artifact bytes do not move and the artifact domain did not step for this row. *(Repaired 2026-08-22: this read "and `tiler.artifact-program.v16` does not step", which was true when written. The domain is `tiler.artifact-program.v21` at the gate re-run's base; the no-step property is about this row, not about the tree.)* Validate refuses a missing interface key, an axis outside the named input's rank, a non-unsigned type, a non-canonical or duplicated list, a transport count other than `bindings + extents`, and an extent transport that is not `binding_count + ordinal`. **It does not refuse a row whose symbolic axis is not the source-bearing root, which construction does refuse** — see "Finding 2" in the gate re-run below.

**Excluded, each by a stated reason rather than by omission.** The live extent *value*. A second caller-supplied scalar list. Baking the bound value into artifact, payload, library, or pipeline identity. Consuming reserved schedule tag `0x36`. `N = 14` / `N = 15` payload and pipeline execution. Schedule-verified `LiveContraction` end-to-end.

## The questions that are genuinely Tom's

1. **Accept `DecodedExtentOperand { key, axis, value_type }` as the public envelope row?** The alternative is folding the operand into `DecodedBinding`, which would force a scalar extent through buffer storage, encoding, and range fields that do not apply.
2. **Accept empty-writes-nothing as the row's presence spelling?** The alternative is an unconditional length that steps the domain to move every previously encodable artifact. *(Repaired 2026-08-22: this named `tiler.artifact-program.v16`, five steps stale. The question is about the spelling, not about a version; the current domain is stated in the gate re-run's figure table.)*
3. **Accept the extra payload transports after the tensor table as the backend placement?** That is the Metal `eN` ABI Tom already accepted, packaged so a decoder can bind without reconstructing the kernel.

## Recommendation

Accept all three as drafted. **Strongest counterpoint:** publishing `value_type` on a row that only admits `Unsigned` today freezes a field that a later signed or narrower extent might want to mean something else.

## Options eliminated before ranking

Inventing a second caller-supplied scalar list, baking the live value into artifact identity, or self-accepting this draft, can silently give one `S` two meanings or release dependents against an unaccepted boundary. Those are defects, not candidates.

## Closes when

Tom accepts, accepts with named exclusions, or revises.

## Decision hold — semantic source unresolved 2026-08-13

Do not answer the three questions above yet. Exact-base review found that the draft row is derived while artifact construction still rejects symbolic semantic interfaces, and the passing two-N fixture attaches the row to a fixed `[2,3]` semantic axis before executing extents 14 and 15. `{ key, axis, value_type }` may remain sufficient once the semantic source is carried, or the row/schema may need to name additional source identity; that is not yet established.

This packet now depends on [`associate-live-extent-operands-with-symbolic-semantic-interface-axes`](associate-live-extent-operands-with-symbolic-semantic-interface-axes.md). Its release trigger is that ticket's independent derivation of the minimum complete row, including coverage, identity, schema, and unsupported-population consequences. Re-run the decision-packet readiness gate and replace this hold with the exact reviewed surface before presenting it to Tom.

## Released to ready — 2026-08-22, both trigger conditions fired

Verified by the coordinator at `f61c0786`. This packet's hold named two conditions and **both are now met**:

1. The release trigger was `associate-live-extent-operands-with-symbolic-semantic-interface-axes`'s independent derivation of the minimum complete row. That ticket is `status: done` and its verdict is already transcribed below: `{ key, axis, value_type }` remains minimal once the semantic source is carried.
2. The note below states the drafted surface's population stays empty "until `package-the-admitted-live-schedule-into-a-symbolic-kernel-program` lands the packaged symbolic interface representation", and that this packet "should be presented after or beside it". **That ticket is `status: done`** — merged and gated on 2026-08-22 — so the population exists.

**This gates a p0.** `bind-frozen-live-extent-bytes-at-declared-backend-transports` (p0) depends on exactly two tickets; the other is `done`, so this packet is its only remaining blocker.

**Facts below are stale and must be re-audited before the readiness gate is re-run.** The note records "question 2's premise moved with the domain (`tiler.artifact-program.v20` / manifest 20.0 now)". At this base those are **`tiler.artifact-program.v21` / manifest `(21, 0)`**, stepped by the packaging landing. Re-derive every domain, byte count, and pin at your own base; do not carry a figure from this ticket.

**Also carry forward, verified at source:** the accepted ADR 0108 packet states the schedule bounds-proof tags are `0x01`/`0x02` and assigns a new one `0x03`. All three are false — the real values are `TAG_LINEAR_RANGE = 0x11` / `TAG_REDUCTION_DOMAIN = 0x12`, and `0x03` is `TAG_SCALAR_BROADCAST` in the nibble-partitioned access-map space. If this packet reasons about tag space, use `0x13` and the reserved `0x0C`, and do not restate `0x03`.

**Scheduling.** Declares `contracts/decisions`, held by the live gather-remainder lane. **Release trigger: that lane merges or stops at a gated boundary.**

## Release-trigger input — semantic-source association delivered 2026-08-18

The association ticket's derivation is in its delivery record (section "Envelope-row sufficiency re-audit"), and its verdict is: **`{ key, axis, value_type }` remains the minimum complete row once the semantic source is carried, and no additional public or schema field is required.** The artifact carries exactly one shape-environment subject, and construction now proves a row valid only when the named interface axis is symbolic with its symbol rooted at exactly that `(key, axis)` in that one environment — so the carried environment subject plus the named axis determine the governing symbol, and a symbol-identity field on the row would be a second authority over one fact. The two-N fixture this hold cited no longer exists: a fixed semantic axis with a live operand refuses at construction (`ExtentOperandStaticAxis`) and at decode, so the drafted surface's population is empty until [`package-the-admitted-live-schedule-into-a-symbolic-kernel-program`](package-the-admitted-live-schedule-into-a-symbolic-kernel-program.md) lands the packaged symbolic interface representation. Before re-running the readiness gate for Tom, note that question 2's premise moved with the domain (`tiler.artifact-program.v20` / manifest 20.0 now; empty-writes-nothing still holds and no step landed with the association) and that the packaged-interface representation — the one place a genuinely new public/schema surface can arise for this family — is the packaging packet's own question, so this packet should be presented after or beside it.

## Readiness gate re-run — 2026-08-22, base `489cc3553965ef87d053cc15a11279a9e00b4ab4`

Worker `worker-envelope`. **This section supersedes the 2026-08-13 "Recommendation" and "Options eliminated before ranking" above**, which are retained as the draft's history. The three questions are unchanged in subject; questions 2 and 3 now have a dominant answer, question 1 has a genuine surviving trade-off, and one new prerequisite was found that neither the hold nor the release note anticipated.

### Stop condition: did not fire — the population exists end to end

The dispatch brief instructed a stop if a live-extent artifact still cannot be constructed and decoded. It can, on both sides, and each half was read to its source rather than inferred from the packaging ticket's summary.

- **Construction is production-reachable.** `SymbolicInterfaceExtent` is gone; `read_semantic_interface` publishes a declared `ShapeEnv` symbol by name and refuses only a source kind the grammar has no tag for (`UnpublishableInterfaceExtent`). `a_symbolic_semantic_program_publishes_its_symbol_by_name` (`crates/tiler-artifact/src/program/tests/provenance.rs`) opens `ArtifactProgramBuilder::new` on a symbolic program and reads the published symbol back by name.
- **The chain that mints a row is wired throughout.** `admits_source_bound_live_schedule` (`crates/tiler-compiler/src/physical.rs`) admits the population; `live_pointwise_region` writes `LogicalAccess::LiveRowMajorSource`; `live_input_extents` (`crates/tiler-ir/src/schedule/model.rs`) turns that marker into the region's runtime operand; lowering declares an `InputExtentParameter`; `derive_extent_operands` maps it onto the interface key; `payload_metadata` (`crates/tiler-build/src/metal_assembly.rs`) declares `buffers.len() + ordinal` transports; `emit.rs` writes `constant ulong& e{ordinal} [[buffer({index})]]`. `a_symbolic_region_delivers_and_its_retired_walls_stay_retired` (`crates/tiler-macros/src/aot/tests.rs`) delivers an artifact family from that population.
- **Decode admits a row.** `a_well_placed_live_extent_row_over_a_symbolic_axis_is_admitted` encodes and decodes an envelope carrying a nonempty `input_extents` run. Run at this base: passes.

So the gate is answerable. What the release note did *not* anticipate is that the decode side's admitted set is now wider than the construction side's — Finding 2.

### Per-Fact verdict, re-audited at this base

Every row was read in the file it names at `489cc355`; nothing is carried from the ticket, the release note, or the dispatch brief.

| # | Fact as this ticket states it | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | `DecodedExtentOperand` with `key()`, `axis()`, `value_type()` | **verified** | `crates/tiler-artifact/src/program/codec/view.rs`, anchor `pub struct DecodedExtentOperand` — three accessors, no more |
| 2 | `DecodedEntry::extent_operands` returns those rows in canonical `(key, axis)` order, empty when the kernel declares none | **verified** | same file, anchor `fn extent_operands`; order re-proven by `check_extent_operands` |
| 3 | The live *value* is not on the row | **verified** | `ExtentOperandData` is `{ key, axis, value_type }` (`program/model.rs`); the value is frozen at `bind_extent_parameters` (`crates/tiler-runtime/src/load.rs`) |
| 4 | `EntryRef::extent_operands` returns the same rows | **verified, and see Finding 3** | `program/model.rs`, anchor `pub fn extent_operands`. It is the only `Decoded*` type reached from that module, and has zero call sites repo-wide |
| 5 | Construction derives the rows from the kernel's `InputExtentParameter` list through the stage access; callers supply no second list | **verified** | `fn derive_extent_operands` (`program/builder.rs`) |
| 6 | Nonempty list appended after the backend entry key under presence tag `0xfe`; empty writes nothing | **verified in both encoders** | `INPUT_EXTENT_BLOCK_TAG` and `fn push_input_extents` (`program/model.rs`, identity); the same run in `codec/encode.rs` guarded by `if !entry.input_extents.is_empty()` |
| 7 | `0xfe` cannot be re-read as the following length | **verified** | `push_len` (`crates/tiler-ir/src/identity.rs`) writes a big-endian `u64`; high byte `0xfe` needs a length ≥ `0xfe00_0000_0000_0000` |
| 8 | Validate refuses a missing key, out-of-rank axis, non-unsigned type, non-canonical or duplicated list, wrong transport count, wrong transport slot | **verified, all six** | `check_extent_operands` and `check_entry_mappings` (`codec/validate.rs`) |
| 9 | `tiler.artifact-program.v16` does not step for this row | **stale, repaired in place** | the no-step property holds; the domain is `v21`. See the figure table |
| 10 | Q1's alternative would force a scalar through buffer storage, encoding, and range fields | **verified** | `DecodedBinding` publishes eleven accessors — `target`, `kind`, `storage_scalar`, `access_type`, `component_role`, `storage_encoding`, `address_space`, `access`, `alignment`, `accessible_offset`, `accessible_bytes` — of which at most `slot` applies to a scalar |
| 11 | Q3's placement is the Metal `eN` ABI Tom already accepted | **verified** | `crates/tiler-metal/src/emit.rs` writes `constant ulong& e{ordinal} [[buffer({index})]]`; `MetalEntryPoint::input_extent_count`'s doc names the same ABI; `check_entry_mappings` proves `binding_count + ordinal` |
| 12 | Excluded: consuming reserved schedule tag `0x36` | **verified** | `0x36` is still reserved for `CooperativeContractionSplit`; `TAG_REDUCTION_LIVE_CONTRACTION` is `0x38` (`crates/tiler-ir/src/schedule/model.rs`) |
| 13 | Excluded: `N = 14` / `N = 15` payload and pipeline **execution** | **verified, imprecise as worded** | `one_live_extent_library_and_pipeline_subject_at_two_n_when_a_toolchain_resolves` (`crates/tiler-metal/src/golden_compilation.rs`) does cover library and pipeline *subjects* at two N and that no N is baked; what is absent is execution. Read the exclusion as execution only |
| 14 | Excluded: schedule-verified `LiveContraction` end to end | **verified** | the contraction fixture refuses at construction (`a_live_contraction_operand_over_a_fixed_semantic_axis_refuses`); `prove-one-live-extent-artifact-payload-and-pipeline-at-two-n` is `todo` |
| 15 | Release note: association ticket `done`, its verdict transcribed faithfully | **verified** | `tickets/associate-live-extent-operands-with-symbolic-semantic-interface-axes.md`, anchor `Envelope-row sufficiency re-audit` — the transcription matches |
| 16 | Release note: packaging ticket `done`, so the population exists | **verified** | `status: done`; and the chain above |
| 17 | Release note: brief's carried-forward tag values | **verified** | `TAG_LINEAR_RANGE = 0x11`, `TAG_REDUCTION_DOMAIN = 0x12`, `TAG_SCALAR_BROADCAST = 0x03`, `0x0C` reserved for `GatherSource`. Not load-bearing here: this packet reasons about no schedule tag except the `0x36` exclusion |
| 18 | Scheduling: `contracts/decisions` held by the live gather-remainder lane | **verified, still true** | `admit-the-selected-data-dependent-index-representation` is `in-progress` under `worker-gather-remainder` and declares `contracts/decisions`. **Contradicts the dispatch brief**, which listed three live lanes and did not name this one. Not blocking: this packet needs no `docs/decisions/` edit, so the exclusive scope was never entered |

### Recomputed figures — previous value → value at this base

| Subject | Ticket / note said | At `489cc355` | Source |
| --- | --- | --- | --- |
| `ARTIFACT_DOMAIN` | `v16` (body), `v20` (note) | `tiler.artifact-program.v21` | `crates/tiler-artifact/src/program/model.rs` |
| `MANIFEST_SCHEMA` | `20.0` (note) | `(21, 0)` | `crates/tiler-artifact/src/program/codec/encode.rs` |
| `PROVIDER_KEY_DOMAIN` | not stated | `tiler.artifact-program.provider.v3` | `program/model.rs` |
| `STAGE_KEY_DOMAIN` | not stated | `tiler.artifact-program.stage.v4` | `program/model.rs` |
| `PROGRAM_DOMAIN` | not stated | `tiler.kernel-program.v13` | `crates/tiler-ir/src/program/model.rs` |
| `INPUT_EXTENT_BLOCK_TAG` | `0xfe` | `0xfe`, unchanged | `program/model.rs` |
| extent transport rule | `binding_count + ordinal` | unchanged | `codec/validate.rs`, `crates/tiler-build/src/metal_assembly.rs` |
| row wire shape | `{ key, axis, value_type }` | unchanged | `push_input_extents`, `codec/encode.rs`, `codec/decode.rs` |

**No figure this packet depends on requires a step.** Every option below is refusal-side or accessor-side; none adds a byte to either encoder, so no identity, manifest schema, cache subject, or pin moves. That is a claim the implementing ticket must rederive rather than copy.

### Finding 1 — the association verdict survives, and the packaged interface does not widen the row

The release trigger's verdict was that `{ key, axis, value_type }` stays minimal because the carried environment subject plus the named axis determine the governing symbol, so a symbol-identity field would be a second authority. That was written before the packaged symbolic interface existed. Tested against the interface that now exists:

`InterfaceEntryData::extents` is `Vec<SourcedExtent>`, published as `ArtifactInputRef::extents` / `DecodedInput::extents`, with `static_shape() -> Option<Shape>` beside it. The symbol governing `(key, axis)` is therefore *readable directly off the interface run* — `envelope.inputs()[i].extents[axis].symbol()`. Adding a symbol field to the operand row would restate a value the same envelope already carries in a length-framed run the identity encoder folds. The verdict is unchanged and is now stronger than when it was written: before v21 the symbol was reachable only through the retained environment, and it is now spelled twice already (interface run and environment root), with `check_interface_symbol_coherence` existing precisely to refuse those two spellings disagreeing. A third spelling on the row would need a third coherence check.

**The packaged interface introduced no new surface this packet must absorb.** `SourcedExtent` is `tiler-ir`'s, accepted under [`accept-the-sourced-extent-vocabulary-at-its-shape-module-paths`](accept-the-sourced-extent-vocabulary-at-its-shape-module-paths.md) (`done`); the artifact re-exports rather than mirrors it.

### Finding 2 — decode does not re-prove the association, and the one positive decode case is outside the construction-admitted population

**This is new, and it is the reason a follow-up ticket is mandatory rather than optional.**

Construction proves four arms (`fn check_extent_operand_association`, `program/builder.rs`): static axis, symbol not bound by the retained environment, symbol rooted at a non-input source, and symbol rooted at a *different* input dimension. Decode proves one. The codec error vocabulary has no `ExtentOperandSourceMismatch`, `ExtentOperandForeignSymbol`, or `ExtentOperandUnsourcedSymbol`; `grep -rn "ExtentOperandSourceMismatch\|ExtentOperandForeignSymbol\|ExtentOperandUnsourcedSymbol" --include="*.rs" crates/` returns `program/builder.rs`, `program/error.rs`, and `program/tests/extent_operands.rs` only.

The gap opened at v21 and nothing carried the arms across: `check_extent_operand_static_axes` used to refuse *every* row, because every decoded interface axis was literal, so the three missing arms were unreachable. The per-axis narrowing made them reachable.

**Demonstrated, not inferred.** `a_well_placed_live_extent_row_over_a_symbolic_axis_is_admitted` puts the row on `input` axis 1, sets `inputs[0].extents[1]` to `forged_symbol("T")`, and installs `forged_retained_environment()`, which binds `T` to `BindingSource::Static(Extent::new(2))`. That is exactly the combination `a_live_operand_on_a_symbol_without_an_input_source_refuses` pins as `ExtentOperandUnsourcedSymbol` at construction. Both tests pass at this base, so the tree simultaneously asserts that a builder must refuse this row and that a decoder must admit it. Two source comments — in `codec/validate.rs` and `codec/error.rs` — state that a symbolic row "passes to the association checks"; there are no association checks after that point.

Consequence: `bind_extent_parameters` freezes `facts.input_extent(key, axis)` onto the `eN` transport, and preflight's only shape check is `evaluate_retained_shape_relations`, which evaluates declared `SemanticInputConstraint`s and nothing else. A decoded artifact whose row names a non-root axis binds an unrelated quantity into the kernel's loop bound with no refusal anywhere. Filed as [`re-prove-the-live-extent-operand-association-at-decode`](re-prove-the-live-extent-operand-association-at-decode.md), which should become a dependency of `bind-frozen-live-extent-bytes-at-declared-backend-transports`.

**This does not change the row's fields.** Root-ness is derivable from the envelope's retained environment; the missing thing is a check, not a column. So Finding 2 constrains *when* the accepted row may be consumed, not *what* it should be.

### Finding 3 — three of the five drafted public items have no consumer, and one has none at all

`grep -rn "extent_operands()" --include="*.rs" --include="*.md" .` outside `tickets/` returns exactly two lines, both in `crates/tiler-runtime/src/load.rs`, both on the *decoded* view.

| Drafted item | Call sites repo-wide |
| --- | --- |
| `DecodedEntry::extent_operands` | 2 |
| `DecodedExtentOperand::key` | 2 |
| `DecodedExtentOperand::axis` | 2 |
| `DecodedExtentOperand::value_type` | **0** |
| `EntryRef::extent_operands` | **0** |

Sibling entry accessors are all exercised (`launch_preconditions` 6, `zero_work_skips_dispatch` 9, `backend_entry_key` 6, `kernel_identity` 6, `payloads` 44), so the model view is not "complete regardless of consumers" — these two are anomalous. Two further facts bear on them:

- `value_type()` returns a value `check_extent_operands` proves is always `AbiType::Unsigned`. Published today, it is an accessor for a constant. This is the ticket's own "strongest counterpoint" made concrete.
- `EntryRef::extent_operands` is the only place `program/model.rs` hands out a `Decoded*` type; every other model/codec pair is `XRef` / `DecodedX` (`BindingRef`/`DecodedBinding`, `ArtifactInputRef`/`DecodedInput`), and where the type is layer-neutral the model view returns the neutral type (`EntryRef::numerical -> NumericalRealization`). So the name is a small mislabel as well as an unconsumed item.
- The p0 that consumes this family does **not** need either. `RoutedExtentParameter` (`crates/tiler-runtime/src/load/route.rs`) carries `transport_slot`, `value`, `parameter_bytes` and — unlike `RoutedBinding`, which publishes its `DecodedBinding` by documented intent — carries no operand reference at all. A backend implementing `bind-frozen-live-extent-bytes-at-declared-backend-transports` reads only those three.

### Options eliminated before ranking

| Option | Ground for elimination |
| --- | --- |
| Widen the row with a symbol-identity field | Second authority over one fact. The symbol is already spelled twice (interface run, environment root) with a coherence check refusing disagreement; a third spelling needs a third check. It also does **not** close Finding 2, because root-ness is not derivable from a symbol name |
| Carry the root `(key, axis)` on the row | Same defect, one level worse: it restates the retained environment's binding, which artifact identity already folds |
| Fold the operand into `DecodedBinding` | Forces a scalar through eleven accessors of which ten do not apply — storage scalar, encoding, access type, address space, alignment, offset, byte range. Silently invites a reader to size a scalar as a buffer |
| Unconditional length instead of the `0xfe` presence tag | Steps `ARTIFACT_DOMAIN` and `MANIFEST_SCHEMA` and moves every artifact's bytes for zero correctness gain. The v21 interface tag is unconditional for a reason that does **not** transfer: a symbolic axis had *no honest empty spelling* (a zero-extent literal misreports it), whereas an empty operand run is unambiguous and the tag is provably unreadable as the following `push_len`. Dominated |
| Let the payload mapping declare arbitrary extent transport slots | Removes the artifact's ability to prove the accepted `eN` ABI; a mapping could place a scalar anywhere and validate could not tell. Invents authority |
| A second caller-supplied scalar list | Two authorities for one `S`; silently gives one extent two meanings |
| Bake the live value into artifact, payload, library, or pipeline identity | Conflates identities. Three landed negative controls pin the opposite: `a_compiled_plan_does_not_fold_a_bound_extent_value`, `baking_neighbouring_extents_mints_distinct_artifact_subjects`, `invocation_bindings_do_not_enter_artifact_identity` |
| Self-accept the draft | ADR 0075 routes a concrete public surface to Tom |
| Further bounded research | Nothing here is unknown. The population exists, the association is derived, and Finding 2 is a defect with a ticket, not a question |
| Defer again / status quo | Costs the p0 its only remaining blocker and buys nothing: Finding 2 is orthogonal to the row's shape and is now in the graph. Dominated by every surviving option |

### The nondominated frontier

Questions 2 and 3 have one surviving answer each and are **recommended rather than asked**: accept the `0xfe` empty-writes-nothing spelling, and accept `binding_count + ordinal` as the backend placement. Every alternative is eliminated above.

Question 1 has a real trade-off, and it is not about the row's fields — those are settled by Finding 1 — but about how much of the read surface is published now.

| | **A — accept as drafted** | **B — accept the row, publish only what has a consumer** |
| --- | --- | --- |
| Included | `DecodedExtentOperand{key, axis, value_type}`, `DecodedEntry::extent_operands`, `EntryRef::extent_operands` | `DecodedExtentOperand{key, axis}`, `DecodedEntry::extent_operands` |
| Withheld | — | `value_type()` and `EntryRef::extent_operands`, each returning to Tom as a labelled draft when a consumer names it |
| Wire / identity | unchanged; `value_type` stays an encoded, identity-folded field either way | identical — B withholds an *accessor*, not a column |
| Correctness | top tier | top tier |
| Fail-closed strictness | top tier | top tier, marginally better: nothing published is unexercised |
| Maintainability | one approval covers the family; no second round trip if the p0 or a later reader wants either item | no unconsumed or untested public item; the `Decoded*`-on-`EntryRef` mislabel is not frozen |
| Host runtime / memory | identical | identical |
| Unsupported population | identical | identical |

Neither dominates. A pays nothing now and risks freezing an accessor for a constant plus a name the layer convention contradicts; B pays one future approval round trip if a consumer appears, against evidence that the nearest consumer (the p0) needs neither item.

**Recommendation: B**, on the evidence that `RoutedExtentParameter` gives a backend everything the p0 requires and that both withheld items have zero call sites at this base. **Strongest counterargument:** `value_type()` is how a backend would confirm the ABI type before writing eight little-endian bytes, and withholding it may push that confirmation into a `debug_assert` or into nothing at all; if Tom expects the p0's adapter to read the operand rather than only the routed parameter, A is right and B costs a round trip on the repository's highest-priority ticket. **Evidence that would reverse B:** a p0 design that threads `DecodedExtentOperand` into `RoutedExtentParameter` the way `RoutedBinding` threads `DecodedBinding` — which its own documented rationale ("published beside the decoded binding they came from rather than instead of it") would support. **Perturbation that tests it:** delete both items on a scratch branch and build the workspace; if nothing outside `tiler-artifact` stops compiling, B's premise holds, and if the p0's first adapter sketch needs them back, A's does.

**A rider available to either answer, and cheap under this repository's pre-production rule:** rename `DecodedExtentOperand` to a layer-neutral spelling shared by both views, so `EntryRef` stops handing out a `Decoded*` type. Under B the rename is nearly free because only the decoded view survives; under A it removes the one naming asymmetry in `program/model.rs`. Tom may reject it as churn — that is the only argument against it, and the workspace has no external consumer.

### The one concrete question for Tom

**Does the accepted surface publish `DecodedExtentOperand::value_type()` and `EntryRef::extent_operands` now (A), or withhold both until a consumer names them (B)?** Questions 2 and 3 are recommended as drafted and need no answer unless Tom disagrees with an elimination above.

### Conditions to record with whichever answer is accepted

1. Acceptance is of the row's fields, spelling, and transport placement. It is **not** a statement that a decoded row is associated — [`re-prove-the-live-extent-operand-association-at-decode`](re-prove-the-live-extent-operand-association-at-decode.md) must land before `bind-frozen-live-extent-bytes-at-declared-backend-transports` makes a backend consume the frozen bytes.
2. The draft labels are removed on acceptance from `crates/tiler-artifact/src/program/codec/view.rs` (two), `crates/tiler-artifact/src/program/model.rs` (one), and `docs/artifact-abi.md` (the 2026-08-13 draft paragraph, already carrying a dated correction from this re-run). That sweep is `implementation/artifact` plus `contracts/artifacts` and is not this ticket's edit.
3. Under B, the removals are part of the same sweep, and each withheld item returns as a labelled draft owned by whichever ticket first needs it.

### Follow-up tickets this packet requires

- **Created:** [`re-prove-the-live-extent-operand-association-at-decode`](re-prove-the-live-extent-operand-association-at-decode.md) — p0, `implementation/artifact` + `contracts/artifacts`. Adds the three missing decode arms, corrects two false source comments, and repairs the positive decode case to one a builder could produce.
- **Recommended graph edge, for the coordinator:** `bind-frozen-live-extent-bytes-at-declared-backend-transports` gains a dependency on that ticket. Not applied here; this worker does not mutate another ticket's state.
- **Already open and unaffected:** [`prove-one-live-extent-artifact-payload-and-pipeline-at-two-n`](prove-one-live-extent-artifact-payload-and-pipeline-at-two-n.md) still owns the withdrawn execution evidence, and [`deliver-an-artifact-family-from-a-symbolic-region`](deliver-an-artifact-family-from-a-symbolic-region.md) still needs the re-audit the packaging landing's outcome asked for.

### Commands run, at this base

- `git rev-parse HEAD` → `489cc3553965ef87d053cc15a11279a9e00b4ab4`; `git rev-list --left-right --count 489cc355...HEAD` → `0 0`; clean status.
- `cargo nextest run -p tiler-artifact -E 'test(a_well_placed_live_extent_row_over_a_symbolic_axis_is_admitted) or test(a_live_operand_on_an_inferred_equal_axis_refuses) or test(a_live_operand_on_a_symbol_without_an_input_source_refuses)'` → **3 passed, 343 skipped**. This is Finding 2's demonstration: the refusal arms and the contradicting admission are green together.
- `grep -rn "extent_operands()" --include="*.rs" --include="*.md" .` outside `tickets/` → 2 lines, both `crates/tiler-runtime/src/load.rs`.
- `grep -n "ExtentOperand" crates/tiler-artifact/src/program/codec/error.rs` → five variants, none of them an association arm.
- `tkt lint` → ok. `make citations`, `git diff --check`, `tkt guard --base 489cc355 tkt/accept-the-live-extent-artifact-envelope-row` → recorded with the delivery commit.

### Unverified, and stated as such

- **Not measured:** the wrong-result consequence in Finding 2 is derived by reading `bind_extent_parameters`, `evaluate_retained_shape_relations`, and the Metal `eN` binding path. No forged artifact was executed on a device, because no backend binds the transport yet — that is the p0's own work. The *validation* gap itself is not an inference: it is two green tests that contradict each other.
- **Not audited:** `spikes/` is outside the workspace and was not read. The packaging landing already records that three spikes call the retired `DecodedInput::shape()`.
- **Contradicting the dispatch brief:** the brief named three live lanes; there are four, and `admit-the-selected-data-dependent-index-representation` holds `contracts/decisions`, which this ticket also declares exclusively. No `docs/decisions/` file was touched, so nothing collided.
