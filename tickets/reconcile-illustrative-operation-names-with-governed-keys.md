---
id: reconcile-illustrative-operation-names-with-governed-keys
title: Reconcile illustrative operation names in the IR contract with governed keys
status: done
priority: p2
dependencies: []
related: [own-operation-family-support-matrix]
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, coherence]
---
The illustrative built-in list in `docs/ir.md` spells operation families as `Constant`, `Cast`, `Reindex`, `Broadcast`, `FloatAdd`, `WrappingAdd`, `CheckedAdd`, `Multiply`, `SaturatingAdd`, `WideningAdd`, `Gelu`, and `Reduce`. None of those spellings is a governed key. The standard semantic registry in `crates/tiler-ir/src/semantic/registry.rs` registers exactly four operations — `tiler::constant-f32@1`, `tiler::multiply-f32@1`, `tiler::add-f32@1`, and `tiler::strict-serial-sum-f32@1` — and the contract text itself later refers to `tiler::add-f32@1` and `tiler::multiply-f32@1` by their real keys when describing the rank-zero scalar admission. So the same document uses two naming systems for the same operations without saying which is which.

`docs/ir.md` does say the list is "illustrative rather than a closed Rust enum", so this is not a false support claim. It is a spelling-coherence hazard: a reader comparing the list against the [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix), which records that no `Cast`, `Reindex`, or `Broadcast` key exists, has to infer that `FloatAdd` and `add-f32` are the same family and that `Reduce` is not `strict-serial-sum-f32` but a family that has no key at all.

Decide and record one convention: either give the illustrative list the canonical `namespace::name@version` spelling for families that have a key and a clearly marked hypothetical spelling for families that do not, or state explicitly that illustrative names are prose placeholders that never denote a key. Do not silently rename the governed keys.

Scope note: `docs/ir.md` is `contracts/foundation`. The support matrix in `docs/roadmap.md` is `contracts/navigation` and must be updated in the same change only if this ticket changes a fact the matrix cites.

## Outcome

**Done.** `docs/ir.md` now records the convention immediately after the illustrative list, in two labelled paragraphs: an illustrative name is prose naming an operation *family* and is never a key, while a governed key is always `namespace::name@version`; and this document states what an operation means, never which operations exist.

**The ticket's second option was chosen, and the first was rejected for a specific reason.** Option (a) — canonical spellings for the families that have a key — would have turned an illustration of the operation *design space* into a half-inventory, listing three real keys beside nine hypothetical families. The corpus already has an authority for that inventory: `docs/design-map.md` routes "which operation families are actually supported?" to the roadmap's maturity matrix, and `docs/open-questions.md` cites that matrix as the owner. Making `docs/ir.md` a second one is precisely the duplicated authority `AGENTS.md` forbids. Option (b) preserves the list's actual job and adds a pointer instead of a rival.

**Bare option (b) would have left the real question unanswered, so the change goes one step further.** Telling a reader that the names denote nothing does not tell them where the real thing is. The second paragraph therefore names both authorities — the compilation request's frozen operation registry for what is registered, and the support matrix for per-family status — and states that a placeholder is not a support claim, not a reservation, and not a commitment to that spelling.

**Fact — verified spellings.** Exactly four governed operation keys exist outside test modules, all constructed through one private helper at `crates/tiler-ir/src/semantic/operation.rs:59`: `constant-f32@1`, `multiply-f32@1`, `add-f32@1`, `strict-serial-sum-f32@1`, each in the `tiler` namespace. The `namespace::name@version` rendering the contract now cites is not a documentation convention invented here — it is what `Display` produces, at `crates/tiler-ir/src/semantic/types.rs:195`, reached by `OpKey` through `TypeKey`. Reproduce: `grep -rn "governed_op(" crates/tiler-ir/src/` returns the four call sites and the helper.

**One finding beyond the ticket's framing.** The ticket treats the illustrative names as pure placeholders. Three of them are not quite that — they are near-misses of real public identifiers. `FloatAdd` and `Multiply` sit beside `F32Add` and `F32Multiply`, which are real typed authoring facades at `crates/tiler-ir/src/semantic/standard_operations.rs:133` and `:51` and which this same document already names at the rank-zero admission. A reader is therefore looking at three spellings for one thing: a prose family name, a facade type, and a key. The convention paragraph names all three relationships explicitly rather than only denying that the first is a key, because denying it would leave the reader to guess whether `FloatAdd` was meant to be `F32Add`.

**Renaming the placeholders was considered and rejected.** Aligning `FloatAdd` to `F32Add` would collide with the real facade type, and aligning it to `add-f32` would assert that the family and the key are the same thing. They are not: `FloatAdd` names a floating-point addition family whose siblings in the same list are the integer overflow families, while `tiler::add-f32@1` realizes that family for exactly one dtype. The contract now says so.

**No roadmap change was required.** The matrix cites two facts about this document — that it lists `Cast` among illustrative built-ins with no `Cast` key existing, and that it names `Gelu` illustratively and requires an admitted key to pin its formula. Both remain true verbatim; nothing was renamed and no key was added or removed, so the `contracts/navigation` scope this ticket does not hold was not needed.

**Split out.** `F32Add` and `F32Multiply` denote two implemented, public, same-crate constructs — the semantic authoring facades and `BinaryOp::F32Add`/`::F32Multiply` in the structured-kernel vocabulary at `crates/tiler-ir/src/kernel/model.rs:198` and `:200`. That is the `disambiguate-select-across-ir-layers` defect class again, and the most severe instance found so far because both senses are real rather than one being unimplemented. It is added to `disambiguate-operation-names-shared-across-expression-layers` with its evidence rather than resolved here, since the resolution is a convention decision touching `implementation/ir`.

**Evidence.** `uv run --locked python scripts/docs.py render` passes at 181 records; full repository gate green.
