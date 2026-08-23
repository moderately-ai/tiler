---
id: route-a-program-inputs-storage-carrier-from-its-own-resolved-value-type
title: Route a program input's storage carrier from its own resolved value type
status: done
priority: p1
dependencies: []
related: [retire-the-gather-kernel-lowering-classification-after-the-body-landed, lower-the-indirect-gather-read-through-the-structured-kernel-body, emit-the-indirect-gather-on-metal, admit-a-storage-carrier-for-integer-program-inputs]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, gather, compiler, program-identity, abi]
---
## User-visible outcome

A program input declared at `tiler::u32@1` is materialized at `StorageScalar::U32` and read through `KernelType::U32`, so a statically proved gather assembles into a kernel program instead of refusing because its index operand was declared `f32`.

## Why this exists

Filed 2026-08-23 by `worker-retireclass` from [`retire-the-gather-kernel-lowering-classification-after-the-body-landed`](retire-the-gather-kernel-lowering-classification-after-the-body-landed.md). That lane retired the `("kernel-lowering", "gather-kernel-body")` classification after [`lower-the-indirect-gather-read-through-the-structured-kernel-body`](lower-the-indirect-gather-read-through-the-structured-kernel-body.md) emitted the indirect body, and the wall that surfaced behind it is this one. It is filed rather than absorbed because a per-input carrier is a separate piece of work with its own identity and ABI consequences, not a detail of retiring a classifier.

**Fact — the compiler's carrier is a total map from the program's arithmetic alone, with no input in it.** `crates/tiler-compiler/src/program.rs` declares `BoundedCarrier::of` under the anchor `The carrier one recognized arithmetic type materializes through`; it matches `ArithmeticType` and nothing else, answering `F32 -> (StorageScalar::F32, KernelType::F32)`, `Bf16 -> (StorageScalar::Bf16, KernelType::Bf16)`, and `ArithmeticType::F16 | ArithmeticType::F64 => None,`. One value is chosen once per program at the anchor `let Some(carrier) = BoundedCarrier::of(request.numerical_contract().arithmetic)` and then reaches every declared value: `fn program_input(` and `fn internal(carrier: BoundedCarrier, role: ValueRole, shape: Shape)` each stamp it into a `MaterializedValueSpec`'s `storage_scalar` and `element_type` together.

**Fact — the ABI byte formula shares that one carrier too, so this is wider than a value-spec field.** `declare_host_abi` pushes a single literal at the anchor `let element_bytes = builder.push_abi_root(AbiRoot::UnsignedLiteral(carrier.element_bytes()))?;` and multiplies *every* input's and internal's element count by it. A per-input carrier therefore needs a per-input width in the ABI expression arena, which is program identity, not a local field swap. The gather fixture happens to hide this — `StorageScalar::U32` and `StorageScalar::F32` are both four bytes wide — so a fix validated only against a `u32` index would leave a narrower integer input sized by the wrong width. That is the failure mode this ticket must discriminate.

**Fact — the refusal, measured rather than predicted.** At base `7d1219ec` with the classifier retired, `gather_program_over([4, 0], [2], 0)` compiles to `InvalidCompilerOutput(Program(CoreConstruction(StageElementType { position: 1, expected: U32, actual: F32 })))`. The refusal is `crates/tiler-ir/src/program/builder.rs`'s, at the anchors `if buffer.element_type != value.element_type {` and `return Err(KernelProgramBuildError::StageElementType {`. Position 1 is the index operand; position 0 is the `f32` source, which agrees. `a_statically_proved_gather_clears_kernel_lowering_and_stops_at_the_program_carrier` in `crates/tiler-compiler/src/request/tests.rs` pins it, and is the test that should change when this lands.

**Fact — the IR half already exists and this is not a duplicate of the ticket that landed it.** `crates/tiler-ir/src/program/model.rs` carries `StorageScalar::U32` at tag `0x04`, four bytes wide, whose own documentation says it is `physical storage and not an integer-arithmetic capability` and that its `natural access type is the exact-width` `KernelType::U32`. [`admit-a-storage-carrier-for-integer-program-inputs`](admit-a-storage-carrier-for-integer-program-inputs.md) landed that pair and is `done`, and its own boundary paragraph states the limit at the anchor `This is a physical program-input carrier and exact access type only` — it admitted the vocabulary and the exact bind check, and deliberately did not wire any compiler-side selection into it. What is missing is the compiler asking the input what it is.

**Fact — the input already knows what it is.** `crates/tiler-ir/src/semantic/gather.rs`'s `gather_index_resolved_type` returns a nominal `ResolvedValueType` over `TypeKey::new("tiler", "u32", 1)` — anchor `the governed gather index key is valid` — and `gather_program_over` declares its index through `builder.input_resolved(...)` with exactly that type. So the resolved value type reaches the semantic program; the question is only how it reaches `build_plan_program`.

## Required work

- Re-audit every Fact above at your own base before editing. The anchors are quoted from source rather than from a rendered view; `natural access type is the exact-width` is deliberately shorter than the sentence it sits in, because that sentence wraps across two `///` lines and a full-sentence anchor returns zero.
- Route a declared input's carrier from its own resolved value type rather than from the program's arithmetic, so a `tiler::u32@1` input materializes at `StorageScalar::U32` / `KernelType::U32` while an unrecognized type refuses by name rather than defaulting to the arithmetic carrier. A silent default here sizes a caller's buffer by a width nobody stated, which is the hazard `BoundedCarrier::of`'s own `None` arm already names.
- Decide and state what happens to the ABI byte formula. A single shared `element_bytes` literal cannot express two widths; whether that becomes one literal per carrier, one per value, or something else is a program-identity choice and must be argued, not picked.
- State every identity domain that steps. **Inference, to be verified rather than trusted:** the *vocabulary* does not step, because `StorageScalar::U32` and `KernelType::U32` already hold tags `0x04` and `0x07`, and no program that this compiler can currently build carries a non-arithmetic input — every such program refuses at `StageElementType` first. So no existing pinned program's bytes should move. Verify that against the pins rather than reasoning to it.
- Test a width that actually differs, not only `u32`. `U32` and `F32` are both four bytes, so every byte-count and alignment path agrees by accident in the gather fixture and a wrong shared width stays invisible there.
- Perturb each behaviour on its own subject and quote the failure text.

## Non-goals

The kernel body, which landed. The Metal emission, which is [`emit-the-indirect-gather-on-metal`](emit-the-indirect-gather-on-metal.md) and refuses `KernelType::U32` at `msl_type` by an accepted named decision. Integer *arithmetic*, conversion, or reinterpretation — this is a carrier for a value the body only reads as an address. Widening `BoundedCarrier::of`'s arithmetic domain: `F16` and `F64` answering `None` is a separate question with its own evidence.

## Fact audit at `a61995c7` (worker-carrier, 2026-08-23)

Every Fact re-read at this branch's own base, each anchor greped against the file the citation names rather than taken from the ticket.

**Fact 1 — the carrier is a total map from the program's arithmetic alone: verified.** `grep -n "The carrier one recognized arithmetic type materializes through" crates/tiler-compiler/src/program.rs` returns `80`; the arm `ArithmeticType::F16 | ArithmeticType::F64 => None,` is at `95`; `let Some(carrier) = BoundedCarrier::of(request.numerical_contract().arithmetic)` is at `1676`; `fn program_input(` at `2181` and `fn internal(carrier: BoundedCarrier, role: ValueRole, shape: Shape)` at `2198`. Read in full: `BoundedCarrier::of` matches `ArithmeticType` and nothing else, and both value constructors stamp `carrier.storage` and `carrier.element_type` together.

**Fact 2 — the ABI byte formula shares that one carrier: verified.** `sed -n '2011p;2022p;2036p'` on the same file returns the single `let element_bytes = builder.push_abi_root(AbiRoot::UnsignedLiteral(carrier.element_bytes()))?;` and its two `element_bytes,` uses. **Its conclusion is imprecise and is repaired below**: a per-input width is a change to what the arena *can* express, but it is not a program-identity step, because `AbiRoot::UnsignedLiteral` carries the number and not the carrier and every value of every program this build could previously assemble has the same width. See **Identity** under Outcome.

**Fact 3 — the refusal: verified by rerun, not by reading.** `cargo nextest run -p tiler-compiler a_statically_proved_gather_clears_kernel_lowering_and_stops_at_the_program_carrier` at `a61995c7` reports `1 test run: 1 passed`, and that test's assertion is exactly `StageElementType { position: 1, expected: U32, actual: F32 }`. The IR anchors resolve: `if buffer.element_type != value.element_type {` at `crates/tiler-ir/src/program/builder.rs:1410` and `return Err(KernelProgramBuildError::StageElementType {` at `1411`. The pin was at `crates/tiler-compiler/src/request/tests.rs:7073`, as the brief stated.

**Fact 4 — the IR half exists and this is not a duplicate: verified.** `StorageScalar::U32` is at `crates/tiler-ir/src/program/model.rs:361` with `Self::U32 => 0x04` and `Self::F32 | Self::U32 => 4`. Both short anchors resolve — `physical storage and not an integer-arithmetic capability` at `358` and `natural access type is the exact-width` at `359` — and the ticket's warning about the wrapped sentence is correct. [`admit-a-storage-carrier-for-integer-program-inputs`](admit-a-storage-carrier-for-integer-program-inputs.md) is `status: done` and its boundary anchor `This is a physical program-input carrier and exact access type only` is at line `150` of its file.

**Fact 5 — the input already knows what it is: verified.** `crates/tiler-ir/src/semantic/gather.rs:203` defines `gather_index_resolved_type`, and `205` holds `TypeKey::new("tiler", "u32", 1).expect("the governed gather index key is valid")`. `gather_program_over` at `crates/tiler-compiler/src/request/tests.rs:5437` declares its index through `builder.input_resolved(..., gather_index_resolved_type())`. The route to it from `build_plan_program` is `SemanticProgram::value(input.value())?.resolved_type()` — `ValueRef::resolved_type` at `crates/tiler-ir/src/semantic/operation.rs:1933`.

**The ticket's "Inference, to be verified" is verified, and the reason is stronger than the one it offers.** It argued from the tags. The tags are indeed unmoved — this branch changes no file under `crates/tiler-ir/` at all — but that alone would not settle it, because the *value spec* of a `u32` input does change. What settles it is that no program carrying one could previously be assembled: see **Identity**.

## Outcome (worker-carrier, 2026-08-23)

### What was routed, and how

`BoundedCarrier::of_input(&ResolvedValueType) -> Option<Self>` answers for a declared input, beside `BoundedCarrier::of`, which keeps answering for the arithmetic every region computes at. It asks the two authorities that already state the admitted identities rather than restating either: `crate::request::recognized_arithmetic` for `tiler::f32@1` and `tiler::bf16@1`, routed onward through `of` so an arithmetic-typed input provably gets the same answer it got before; and `tiler_ir::semantic::gather_index_resolved_type` for `tiler::u32@1`. Anything else is `None`, refused in `build_cover_core` as `ProgramError::Structure { rule: "program-input-carrier-type" }` rather than defaulted.

`build_cover_core`'s input list became a named `DeclaredInput { key, shape, extents, carrier }`, so the four sites that need the width — the external allocation's capacity, its alignment guarantee, the value's storage scalar and element type, and the ABI byte formula's scale factor — read one answer instead of four independent ones. Internal values keep the program carrier, which is correct rather than conservative: nothing outside the program binds one, and an internal is by construction a value some region computes at the program's arithmetic.

**The ABI byte formula: one literal per value, deduplicated, and this is argued rather than picked.** `declare_host_abi` no longer hoists a shared `element_bytes` node; each declared input pushes its own carrier's width and each internal pushes the arithmetic carrier's. The three candidates were one literal per program (what was there, and cannot express two widths at all), one per *carrier* (a two-entry table threaded through the function), and one per *value*. The last dominates the middle because `KernelProgramBuilder::push_abi_root` already deduplicates by structural equality — its own documentation says so at the anchor `an identical expression returns the handle already minted for it` — so a per-value push *is* a per-distinct-width table, computed by the arena rather than by a second structure this function would have to keep in agreement with it. It also removes the one node the old spelling could emit that nothing named.

### Identity: what steps, and what does not

**Nothing steps.** Stated per domain, and derived rather than inferred from the green gate:

- **`tiler.kernel-program.v13`, the program identity domain** (`crates/tiler-ir/src/program/model.rs`, `PROGRAM_IDENTITY_DOMAIN`), together with its four sub-key domains `tiler.kernel-program.{value.v1,allocation.v1,view.v1,stage.v3}` and the arena's `tiler.kernel-program.abi-arena.v1`: **does not step.** A domain steps when a previously encodable program's bytes move. None do. Every declared input is output-reachable — `SemanticProgramBuilder`'s `compact_to_outputs` drops an input whose value is not, so an unreachable input cannot survive into `inputs()` — hence every input is a value `recognized_program_arithmetic` walks. That walk admits exactly one recognized arithmetic per program, with one exemption: a value in `gather_address_operands`. So an input is either at the program's arithmetic, where `of_input` routes through `of` and answers byte-identically to the code it replaced, or it is a gather address operand at `tiler::u32@1` — and **no program containing one could be assembled before this change**, because it refused at `StageElementType` first. The encodable population is strictly extended, not remapped.
- **The ABI arena's node sequence: unchanged for every such program.** `AbiRoot::UnsignedLiteral` carries the width as a number, not as a carrier, and within any program that could previously be assembled every value's width is equal — `4` throughout an `f32` program, `2` throughout a `bf16` one — so the first value still pushes the same literal at the same arena position and every later value interns to it.
- **The `StorageScalar` and `KernelType` vocabularies: do not step and do not widen.** `git diff a61995c7 -- crates/tiler-ir/` is empty; no variant, tag, or field position is touched.
- **`tiler.compiler.request-subject.v6`, the semantic, schedule, kernel, index, and artifact domains: untouched**, for the same reason — this branch modifies three files, all under `crates/tiler-compiler/src/`, and none of them encodes an identity.
- **No ledger row or pinned digest moves.** The whole workspace suite, which holds the identity pins, is green: `cargo nextest run --workspace` reports `4067 tests run: 4067 passed, 8 skipped`, with the only pre-rewrite failure being the one test this ticket owns.

Because no domain steps, there is no cascade and nothing to recompute on the merged tree. The base `a61995c7` is three commits behind `origin/main` at `46184f8c`, and those three commits touch only `docs/` and `tickets/` (`git diff --stat a61995c7..origin/main`), so no `crates/` content differs between this branch's tree and the merge result and the counts above are the merged tree's counts too. Two of those three commits do falsify documentation this landing invalidates again — see **Follow-ups**.

### The differing-width case

**No program this build can assemble has two inputs of different widths, and the derivation is closed rather than a failure to find one.** Every input is a value `recognized_program_arithmetic` walks (above); the walk's one exemption is a gather address operand; the gather family admits exactly `tiler::u32@1` there (`GatherError::UnadmittedIndexType`) and requires a `tiler::f32@1` source and result (`GatherError::SourceNotF32`); `u32` and `f32` are both four bytes; and the only other recognized arithmetic, `bf16`, has no gather family at all. So every assemblable program is uniformly four bytes or uniformly two, and the gather fixture genuinely cannot witness a wrong shared width — the trap this ticket names is real.

The routing is therefore tested at the level where a differing width *is* constructible, which is `declare_host_abi` itself: `two_inputs_of_differing_widths_scale_by_their_own_and_equal_ones_still_share` declares three boundaries of eight elements each at carriers `4, 2, 4` and pins each byte expression against a hand-rebuilt `width * count`. The oracle is the arena's own content deduplication, so this pins the exact literal rather than merely that two handles differ, and the third boundary is the control that the arena does share when the width is equal. Reverting `declare_host_abi` to one shared literal reddens exactly the two-byte row:

```
assertion `left == right` failed: the two-byte input's accessible range was scaled by a width that is not its own
  left: AbiExprId { owner: ProgramBuilderId(1), index: 2 }
 right: AbiExprId { owner: ProgramBuilderId(1), index: 5 }
```

Note that the four-byte rows still pass under that perturbation, which is the point: a fixture with only four-byte inputs cannot discriminate.

`a_declared_inputs_carrier_is_its_own_resolved_types_and_an_unadmitted_one_is_refused` pins the three admitted rows by width and pins the refusal for `tiler::i32@1` — a registered governed scalar a frontend can really declare an input at, so the refusal arm is about a reachable population and not a hypothetical.

### What a proved gather does end to end

`gather_program_over([4, 0], [2], 0)` is recognized, lowered through the governed gather capability, refined, statically proved on the vacuous closed argument, spelled as `RegionSpellingKind::Gather`, verified, admitted, lowered to a structured kernel body, **and assembled into a verified kernel program**. `crate::pipeline::compile` returns `Ok`. In the packaged program the `source` boundary is `StorageScalar::F32` / `KernelType::F32` and the `index` boundary is `StorageScalar::U32` / `KernelType::U32` at eight required bytes — two elements at four. It earns no refusal at all: the wall this ticket was filed against is gone rather than moved.

The test that pinned the refusal is now `a_statically_proved_gather_compiles_with_its_index_at_its_own_carrier`, keeping the vacuous-bounds premise its predecessor uniquely carried.

### Perturbations

Four, each on the subject rather than the assertion, each quoted in the test documentation and reproduced here. Every one was reverted and the tree re-verified green.

- Answering `StorageScalar::F32` / `KernelType::F32` in `of_input`'s `gather_index_resolved_type` arm restores the old wall: `the governed gather compiles once its index carries its own type, got Program(CoreConstruction(StageElementType { position: 1, expected: U32, actual: F32 }))`.
- Answering `StorageScalar::F32` beside `KernelType::U32` — a pair no carrier names — reports `got Program(CoreConstruction(StorageAccessType { scalar: F32, encoding: Unpacked, expected: F32, actual: U32 }))`. This is why the test's `storage_scalar` assertion is documented as a readout: the IR proves the pair, so that assertion cannot be the one that catches a split carrier.
- Adding one to `input.carrier.element_bytes()` in `declare_host_abi` reports `got Program(CoreConstruction(AccessibleBytesDisagreement { position: 1, expected: 8, actual: 10 }))`, so the ABI width really is checked against the declared view rather than merely declared.
- Inverting the spelling proof gate to `gather_bounds_proof(lowering, normalized.member).is_none()` in `crates/tiler-compiler/src/physical.rs` reports `a vacuously proved gather is spelled by the governed vocabulary: GatherIndexBoundsUnproved`, so the premise is load-bearing.
- Making `of_input` fall back to `Self::of(ArithmeticType::F32)` for an unrecognized type reports `an input type this build materializes no carrier for must refuse rather than default to the program's arithmetic width`.

**One assertion was removed rather than kept green.** The first draft also asserted the index boundary's `alignment()` against `AlignmentRequirement::natural_for(StorageScalar::U32)`. `natural_for` answers four bytes for `U32` and `F32` alike, so no perturbation of the cause this test exists to catch could redden it — it could not say *no*, and it was deleted rather than left as decoration.

### Test-count delta, derived

Baseline `4065` workspace and `1350` release. This branch renames one test and rewrites its body (net `0`) and adds two in `crates/tiler-compiler/src/program/tests.rs` (`+2`); `git diff -- crates/ | grep -c '^+.*#\[test\]'` returns `2` and the `^-` form returns `0` — the renamed test's attribute line is untouched, so only the two additions appear — for a net `+2`. Both new tests are in `tiler-compiler`, which the release run covers (`cargo nextest run --release -p tiler-reference -p tiler-compiler`), so the release count moves by the same `+2`. Expected `4067` / `1352`.

### Follow-ups

- `docs/compiler/optimizer.md` and `docs/roadmap.md` on `origin/main` at `46184f8c` both state the wall this landing removes as live fact, and the optimizer document additionally names the test by its retired name. Neither file is in this ticket's scopes, so [`restate-the-gather-standing-after-the-per-input-carrier-landed`](restate-the-gather-standing-after-the-per-input-carrier-landed.md) is filed for them rather than edited here.
- `crates/tiler-compiler/src/frontier.rs`'s `boundary_carrier` derives the caller-facing boundary contract from the *region's* program rather than per input, so a gather region's index input is handed `bounded_requirements(StorageScalar::F32)`. There is no defect today — `AlignmentRequirement::natural_for` answers four bytes for `U32` and `F32` alike and every other property in the set is width-independent — so it is recorded as the sibling of this pattern rather than changed under a ticket whose outcome is about the program layer.
