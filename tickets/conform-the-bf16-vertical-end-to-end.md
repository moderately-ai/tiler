---
id: conform-the-bf16-vertical-end-to-end
title: Conform the BF16 vertical end to end against the exact reference corpus
status: in-progress
priority: p2
dependencies: [validate-bf16-at-the-runtime-routing-boundary, carry-a-bf16-subnormal-realization-the-reference-can-be-told, decide-where-a-device-reaching-conformance-test-may-live, wire-the-bf16-reference-to-the-realization-it-is-told, admit-the-conformance-crate-to-the-workspace, decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access]
related: [spike-bf16-through-the-second-dtype-seams, evaluate-bf16-reference-semantics, own-the-dtype-support-maturity-matrix, lower-bf16-to-metal, dispatch-a-tiler-region-on-metal-hardware, wire-the-bf16-reference-to-the-realization-it-is-told]
scopes: [implementation/reference, contracts/numerics, implementation/runtime, implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, conformance, testing]
claimed_from: todo
assignee: agent-bf16-vertical
lease_expires_at: 1786122049
---
## User-visible outcome

One checked conformance run carries a pure-BF16 program from semantic construction to a dispatched device result and compares it against the exact-rational oracle, so a regression anywhere in the vertical is a red test rather than a wrong tensor. Until this exists the layers are each tested against their neighbour and nothing tests the composition.

## Why a per-layer test is not this

**Fact.** Each BF16 child closes on evidence about its own layer. That is correct and it is not sufficient: the U4/F32 vertical is the standing example of a family whose physical carrier, kernel, and lowering are each tested while the composition is not, and `docs/dtype-support.md` records that non-monotone row deliberately.

**Inference.** The composition is where a dtype's *width* assumptions fail — a two-byte element counted as four survives every single-layer test that uses consistent counts on both sides, and only an end-to-end run with a hand-derived expected result catches it.

## Required evidence

- One program — BF16 constant, multiply, add — carried from semantic construction through compile, artifact, runtime routing, and device dispatch, with the exact expected result bits stated in the test rather than read back from the run.
- The corpus covers, in the end-to-end run and not only in the unit oracle: both zeros with their signs, the least positive and least negative subnormals, the greatest subnormal, the least normal, a tie resolved to even, an ordinary rounding, an overflow to infinity, both infinities, and a non-canonical NaN that canonicalizes.
- The **declared flush is applied to the reference before comparison**, and the elements it moves are named. Finding 24 measures BF16 arithmetic flushing subnormals on the macOS row, so bit equality on the subnormal cases would mean the device did not do what was measured — a passing test there is a signal to distrust, not a success.
- An execution witness on a non-subnormal operand, for the reason finding 24 gives: `flushed` and `the arithmetic was optimized away` produce the same observation without one.
- At least one perturbation of the composition itself, observed failing — for instance an element count derived from the wrong width, which every layer-local test would still pass.
- The measurement boundary stated: host, OS build, Metal version, GPU, and family, with no generalization beyond the row that ran.

## Closes when

The end-to-end run passes on the measured macOS row with hand-derived expected bits, the flush is applied and its affected elements named, the execution witness is present, the composition perturbation is observed failing, the boundary is recorded, and the BF16 `Conformance evidence` cell states what this run actually covers rather than the whole family.

## Graph maintenance

- The last of the BF16 implementation children; depends on the runtime boundary, which transitively depends on the rest.
- A host without the measured environment must still run the deterministic reference and structural half and report the unavailable measurement boundary, rather than skipping silently or claiming a pass.
- This closes the BF16 vertical for the **three operations and one target family** it names. It does not promote BF16 generally: contraction, reduction, conversion, mixed precision, and every other family and target stay where the ledger puts them.

## Blocked 2026-08-06, with the derivation

Dispatched as an implementation ticket on `implementation/reference` + `contracts/numerics` at base `afdac9c9`. Neither required half is reachable from those scopes, and one of the two is not reachable from *any* scope set a worker may assemble. Nothing was descoped: no code landed, and the two blocks are structure below rather than a shortened evidence list.

### Block 1 — the device dispatch has no host anywhere in the test suite, and its obligation has been relayed twice

**Fact.** Exactly one workspace member can reach a Metal device. `grep -rn 'metal\.workspace\|^metal = ' --include=Cargo.toml .` returns the workspace pin, `prototypes/serial-sum-run/Cargo.toml:35`, and two *out-of-workspace* spikes (`spikes/runtime/inline-dispatch`, `spikes/target-profiles/metal-grid-axis-extent`). `prototypes/serial-sum-run` maps to `implementation/runtime`. `crates/tiler-reference` depends on `tiler-ir` alone, so adding a device edge there would put a live backend under the target-independent oracle — the dependency inversion the crate exists to prevent, and an architectural change that is Tom's.

**Fact — the same member is otherwise exactly right, which is why the placement question is narrow rather than open.** `prototypes/serial-sum-run` already depends on `tiler-artifact`, `tiler-build`, `tiler-compiler`, `tiler-ir`, `tiler-metal`, `tiler-metal-aot`, `tiler-reference`, `tiler-runtime`, and (macOS-gated) `metal`; its `[[bin]]` carries `test = true`, so a `#[test]` there is reached by `cargo nextest run --workspace` and therefore by `make full`. It is the only place in the repository where "a regression anywhere in the vertical is a red test" is presently constructible.

**Fact — the obligation was relayed here rather than scheduled here.** `lower-bf16-to-metal` closed at a revised offline boundary and assigned dispatch, the flush-applied comparison, and the execution witness to its two dependents. `validate-bf16-at-the-runtime-routing-boundary` then closed holding `implementation/runtime` — the one scope that could have dispatched — and recorded "**No BF16 kernel was dispatched.** Nothing in this branch touches a device", satisfying its own "routes and executes" item with a synthetic scalar-host image whose measurement boundary states "No case here claims BF16 executes". So the obligation arrived at a ticket declaring strictly weaker scopes than the one that had already declined it. That is the graph defect, not a worker-level inconvenience: `docs/dtype-support.md`'s BF16 `Backend execution` cell names this ticket and `validate-bf16-at-the-runtime-routing-boundary` as the owners of the dispatch, and neither held a dispatchable scope at the time it closed.

**Scope added, and why it belongs here.** `implementation/runtime` is added as declaration and scheduling metadata under the already-authorized outcome: the ticket's own `User-visible outcome` says "to a dispatched device result", and that sentence cannot be executed from any other scope. Verified free of any live claim on 2026-08-06 — the four in-progress tickets hold `implementation/compiler`, `implementation/ir`, `contracts/navigation`, and `research/numerics`. Three were read branch-side (`git show tkt/<id>:tickets/<id>.md`); `retire-the-corrected-softmax-fact-quotations` has no `tkt/` branch, so there is no branch-local scope addition to read and the integration copy is the only source that exists for it. None of the four holds `implementation/runtime`, `implementation/reference`, or `contracts/numerics`.

**The placement fork that remains, and is the coordinator's or Tom's rather than a worker's.** Adding a BF16 pointwise conformance run to `prototypes/serial-sum-run` repurposes the member whose manifest describes it as the "runner for Tiler's serial-Sum value proof", and whose `run()` is one linear F32 narrative. The alternative is a sibling prototype member, which is crate admission — `implementation/workspace` and `implementation/cargo-lock` for the manifest and lockfile, a new `[scopes]` entry in `ticketsplease.toml` (a path under `prototypes/` matching no existing glob is a guard scope-escape, exit 6), and Tom's decision under the scaffolding boundary. This ticket declares the scopes the first arm needs and deliberately does not pre-declare the second arm's, because declaring them would pre-commit the fork.

### Block 2 — the declared flush cannot be applied to the reference at all

**Fact.** `ReferenceNumericalConformance` reaches every capability through `ReferenceEvaluationRequest::conformance`, but applies its two subnormal dimensions with `apply_to_operand(f32) -> f32` and `apply_to_result(f32) -> f32` (`crates/tiler-reference/src/conformance.rs:234,240`). The three BF16 capabilities perform no binary32 arithmetic and read the conformance nowhere: `grep -n conformance crates/tiler-reference/src/bf16.rs` returns two hits, both in the module header explaining the gap. So this ticket's third required-evidence item — the declared flush applied to the reference before comparison — is presently unstatable, and a comparison run without it would assert bit equality on exactly the subnormal cases finding 24 measures the device moving.

**Fact — that gap is owned, its trigger has now fired, and its first deliverable is Tom's.** `carry-a-bf16-subnormal-realization-the-reference-can-be-told` owns it. Both of its triggers fired and were logged on 2026-08-06 (the authoritative profile now declares measured BF16 flush rows; a registered contract now resolves subnormal dimensions per format), and it moved from `deferred` to `todo`. Its first deliverable is a declaration naming which format a subnormal resolution speaks about, which that ticket states is a public boundary and not self-accepted — and the derived-versus-declared fork recorded there is a genuine one. Added to `dependencies:`, because this ticket cannot state its central comparison until that lands.

### Landed on this branch

Only the reachable in-scope correction, in `contracts/numerics`: [Correctness and testing](../docs/correctness-and-testing.md#semantic-authority) asserted without qualification that "The comparison follows the declared numerical contract and conformance level". One registered family cannot follow it, and nothing said so — an overstated contract sentence that makes this ticket's evidence list look reachable. The exception is now stated with its reproducing check, its directional consequence (the oracle is the side that would be called wrong), and its owning ticket, which is instructed to retire the paragraph when it closes.

### Not done, and not to be read as partially done

No conformance run, no corpus, no perturbation, no execution witness, no measurement boundary claimed, and no cell of any maturity ledger moved. `docs/dtype-support.md` is `contracts/navigation`, which this ticket does not hold and which was live-claimed on 2026-08-06 by `re-read-the-bf16-and-elementary-support-rows-against-source`; the BF16 `Conformance evidence` cell must stay exactly as it is, because this branch produced no run for it to describe.

### To unblock

Resolve the placement fork in Block 1, land `carry-a-bf16-subnormal-realization-the-reference-can-be-told`, then move this to `todo` and redispatch with `implementation/reference`, `contracts/numerics`, and `implementation/runtime`.

## Dependency note, 2026-08-07 — both dependencies are now `done`, and this stays blocked

Both edges are satisfied: `validate-bf16-at-the-runtime-routing-boundary` and [`carry-a-bf16-subnormal-realization-the-reference-can-be-told`](carry-a-bf16-subnormal-realization-the-reference-can-be-told.md) both read `done`, the latter closed on 2026-08-07 when Tom decided its parked fork (arm A — the format derived at the point of use).

**That does not unblock this ticket, and the status is deliberately left at `blocked`.** The two blocks recorded on 2026-08-06 are structural and neither is a dependency edge:

- **Block 1 is unchanged and is Tom's.** Only `prototypes/serial-sum-run` can reach a Metal device, and putting a device edge under `crates/tiler-reference` would invert the dependency the target-independent oracle exists to prevent. Where a device-reaching conformance test may live is an architecture decision, not a scope a worker may assemble.
- **The flush half is now closer but not landed.** This ticket's evidence list requires "the declared flush is applied to the reference before comparison". The reference can be *told* a realization, and after Tom's decision the route that tells it is settled — but the wiring itself is [`wire-the-bf16-reference-to-the-realization-it-is-told`](wire-the-bf16-reference-to-the-realization-it-is-told.md) and has not landed. Until it does, applying the flush here would mean stating the realization by hand, which is what the module header describes and not what this ticket asks for.

Add a dependency on the wiring ticket when it is claimed; it is left as a relation for now so this node is not re-pointed at work that may land before Block 1 is answered either way.

## Unblocked 2026-08-07 — every block is discharged, and here is each one

Moved `blocked` → `todo` by the coordinator after checking each dependency against the board rather than inferring from the count. All six read `done`.

**Block 1 — no host for a device-reaching test — is discharged.** Tom decided on 2026-08-07 that such a test lives in a proper crate, not in `prototypes/`; [`admit-the-conformance-crate-to-the-workspace`](admit-the-conformance-crate-to-the-workspace.md) landed `crates/tiler-conformance` as a workspace member with the whole vertical as normal dependencies and `metal` behind `cfg(target_os = "macos")`. **This ticket's device half belongs there**, and `implementation/conformance` is added to its scopes for that reason. The crate holds no items yet — this is its first content.

**The unsafe obstacle the crate admission exposed is discharged, with a rule narrower than the precedent.** See the coordinator comment on this ticket and [`decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access`](decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access.md): `deny` with named sites, **never a crate-level allow**, FFI memory management with Metal as the only admitted justification, and unsafe isolated into one narrow module so the conformance logic itself contains none.

**The flush half is discharged.** This ticket's evidence list requires the declared flush applied to the reference before comparison. [`wire-the-bf16-reference-to-the-realization-it-is-told`](wire-the-bf16-reference-to-the-realization-it-is-told.md) landed that: `<Bf16BinaryReference as ReferenceOperation>::evaluate` reads the conformance it is handed and evaluates under it, so a flushing contract now returns the flushing answer through the registered dispatch rather than requiring the realization be stated by hand.

**One thing to carry rather than rediscover.** No capability yet checks that the conformance it was handed was stated about its own format — `ReferenceNumericalConformance::from_realization` discards the subject and has no caller, so every conformance in the tree is `strict()`. That window is unreachable today and is owned by [`give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject`](give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject.md). **This ticket must state the realization it compares under explicitly** rather than assuming a route supplies it.

The 2026-08-06 "Blocked" section below is retained as the dated record of what blocked this and how each block was eliminated; read it as history, not as current truth.

## Outcome, 2026-08-07 — the vertical executes on the measured row, and one leg of it is unreachable

**The device half exists.** `crates/tiler-conformance` gained its first content: a pure-BF16 `(x * 1.5) + 0.0` program carried from semantic construction, through the exact-rational oracle, through the schedule and kernel vocabularies, through `bfloat` MSL emission against the authoritative macOS Apple9 declaration, through the real Apple offline toolchain, to a dispatch on this host's GPU and a bit comparison against the oracle under the declared flushing conformance. It passes. This is the first BF16 arithmetic this workspace has executed on a device.

### Measurement boundary, and nothing generalizes past it

Apple M4 Max reporting `MTLGPUFamilyApple9`; macOS 27.0 build `26A5388g`; `arm64`; offline `Apple metal version 32023.921 (metalfe-32023.921)` and `AIR-LLD 32023.921`; macOS SDK 27.0 build `26A5388f`; profile `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`; AOT target `air64-apple-macos26.0` under `metal4.0`; linked metallib 3,619 bytes. Three operations — `tiler::constant-bf16@1`, `tiler::multiply-bf16@1`, `tiler::add-bf16@1` — one target family, one contract. No iOS family, no other Apple family, no other OS/SDK/compiler row, no contraction, reduction, conversion, or mixed precision.

### The leg that is not crossed, and why it is not a descope

**`compile()` cannot take a BF16 program at this commit.** `select_supported_strategy` (`crates/tiler-compiler/src/request.rs:4206`) refuses every program carrying a non-`f32` value under the rule `dtype-f32` before a subject is normalized, and three files assert that wall on purpose: `crates/tiler-compiler/tests/bf16_numerical_contract.rs`'s `a_flush_accepting_bf16_contract_reaches_the_recognizer_dtype_wall`, the same file's `the_accepted_bf16_contract_schedules_and_lowers_a_region_the_request_cannot_reach`, and `crates/tiler-compiler/src/pipeline/tests.rs`'s BF16 vertical, whose `bf16_scheduled_region` records the identical boundary in the same words. So **the optimizer, the artifact envelope, and the runtime routing commit are not crossed by this run**: nothing can produce the `PlanAlternative` all three consume. The region is assembled through `tiler-ir`'s public builders, which the ticket's own required-evidence sentence ("through compile, artifact, runtime routing") cannot reach from any scope this ticket holds — `crates/tiler-compiler/` is a stop condition on this dispatch and was live-claimed by a parallel worker. Nothing else on the list was trimmed.

### Evidence, item by item

- **The program and the hand-derived bits.** Fifteen corpus elements, each with its operand and *both* expected encodings (preserving and declared) derived by hand from BF16's parameters — sign 1, exponent 8 with bias 127, trailing 7, quantum `2^-133` — and the round-to-nearest-ties-to-even rule, stated in `corpus()` and never read back from any run. `the_hand_derived_corpus_agrees_with_the_oracle_under_both_readings` holds the oracle to them; the device is then compared against the same column.
- **Corpus coverage**, asserted class by class in `the_corpus_covers_every_class_the_ticket_names`: both zeros with their signs (`0x0000`, `0x8000`); least positive and least negative subnormals (`0x0001`, `0x8001`); greatest subnormal (`0x007f`); least normal (`0x0080`); a tie to even (`0x3f81 -> 0x3fc2`, `193.5` quanta resolving to the even significand 194); an ordinary rounding decided by nearness (`0x3fff -> 0x403f`, `191.25` quanta of `[2, 4)`); an overflow to infinity (`0x7f7f -> 0x7f80`, `382.5 * 2^120` above the `255.5 * 2^120` midpoint); both infinities (`0x7f80`, `0xff80`); and a non-canonical NaN that canonicalizes (`0x7fc1 -> 0x7fc0`). Finding 24's two measured input-flush operands `0x0040` and `0x8040` are in the corpus by their measured encodings.
- **The declared flush is applied to the reference before comparison, and the moved elements are named.** The oracle runs under `ReferenceEvaluator::under` at the conformance `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16` resolves to. The flush moves **five** elements — indices 2, 3, 4, 5, 6: `0x0001`, `0x8001`, `0x0040`, `0x8040`, `0x007f`, every subnormal operand and no other. The run requires each to return its flushed answer **and not** the preserving one, so bit equality against the preserving reading is a failure rather than a success. The least normal `0x0080` at index 7 is the unmoved neighbour, one quantum above the greatest subnormal.
- **Execution witnesses on non-subnormal operands.** Multiply: `0x3f80 -> 0x3fc0`. Add: `0x8000 -> 0x0000` — finding 27's measured `+0.0` shape, whose `fadd` survives `safe` because removing it needs `nsz`, so the returned `0x0000` is what separates "the add ran" from "the add was deleted". Both asserted individually and printed.
- **The composition perturbation, observed failing.** An operand payload strided at the neighbouring `f32` carrier's four bytes while the kernel keeps addressing at `tiler::bf16@1`'s two — one mis-derived site, which is the asymmetry a real typo produces. Watched failing at 8 of 15 elements on the device, and the *symmetric* version is asserted to round-trip the corpus unchanged, which is the statement of why every layer-local test passes this defect.
- **A host without the environment.** `MeasuredHalf::{Ran, Unavailable, Failed}`; a non-macOS host reports `Unavailable` naming the missing environment, and `TILER_REQUIRE_METAL_CONFORMANCE` turns that into a failure. Both halves watched: with `xcrun` off `PATH` the run reported `MEASUREMENT BOUNDARY UNAVAILABLE — no qualified Apple Metal toolchain resolved: … could not run xcrun` and passed; with the variable set it failed at that assertion. `cargo clippy -p tiler-conformance --all-targets --target x86_64-unknown-linux-gnu -- -D warnings` is clean, so the non-Apple branch compiles rather than being aspirational.
- **A strict BF16 contract is refused on this row, observed failing.** `a_strict_bf16_region_is_refused_on_the_measured_row` builds the same region under `NumericalContract::STRICT_BF16`, watches `require_declared_realization` return `unrealizable-numerical-obligation`, and then emits the flush-accepting region on the same target to show the refusal is about the contract rather than blanket.

### The unsafe rule, executed

`crates/tiler-conformance/Cargo.toml` no longer inherits `[workspace.lints]`; it states `unsafe_code = "deny"` with the workspace's other lints restated, and the comment above `[lints]` is replaced by Tom's decision rather than the open fork it described. **The population is two sites, both in `src/device_buffer.rs`**: `write_bytes` and `read_bytes`, each with a reasoned `#[allow(unsafe_code)]`, a bounding assertion against the buffer's own reported length, and a `SAFETY` comment naming the invariant and why `metal::Buffer::contents` forces it. **There is no crate-level allow.** The module's interface is `&[u8]`, so every width, stride, and element count stays in safe code — which is what makes the perturbation above expressible at all — and the conformance logic contains no `unsafe`. `the_unsafe_site_population_is_the_two_named_ones` walks every `.rs` file under `src/`, requires exactly two blocks and two allows, requires both to be in `device_buffer.rs`, and refuses to count fewer than five files, so an added site or an added file is a red test rather than an absorbed addition.

### Landed outside `implementation/conformance`

`docs/correctness-and-testing.md` (`contracts/numerics`) gains two paragraphs after the per-dtype-contract-refusal pair: what this run establishes, and what it does not cross with the `dtype-f32` derivation. Nothing else in that document moved; the 2026-08-07 paragraph the wiring ticket left at "Semantic authority" is already correct and was not touched.

### Not done, and not to be read as partially done

- **No pinned identity moved**, and none was expected to: the standard Metal artifact identity `357f0676…`, cache subject `c626e43b…`, fixed content 65,242 bytes, and descriptor length 2,099 are unchanged, confirmed by `cargo nextest run --workspace` passing `tiler-build`'s `the_standard_metal_path_publishes_its_recorded_identities` and `metal_declaration`'s descriptor pin.
- **`docs/dtype-support.md` was not edited.** It is `contracts/navigation`, which this ticket does not hold. The BF16 `Conformance evidence` cell is supported by this run bounded to *three operations, one target family, one measured row, and a vertical that does not cross the optimizer, the artifact, or the runtime routing commit* — and the `Backend execution` cell is likewise supported at exactly that bound. Both need a `contracts/navigation` holder.
- **No public item was added.** Every module is private and `#[cfg(test)]`; every item in them is `pub(crate)`. The crate exports nothing, so no ADR 0075 acceptance node is owed.
- **This run does not separate a sign-preserving flush from an always-positive one.** The trailing `+ 0.0` maps every zero to `+0`, so a flushed subnormal's sign does not survive to the output. That was the deliberate trade for the add's execution witness, is stated in the module header and in the contract document, and stays evidenced by finding 24's measured `8040 -> 8000` row and by `tiler-reference`'s `the_flushed_zero_sign_is_read_on_both_dimensions`.
- No contraction, reduction, conversion, mixed precision, iOS family, or second target profile.

### Remainder to schedule

- **The compile/artifact/routing leg.** Carrying a BF16 semantic program through `compile()` needs the recognizer's `dtype-f32` rule widened, which is `implementation/compiler` and is not owned by any live ticket — `establish-bf16-optimizer-legality` holds legality keying, not recognition. A ticket for it, and a follow-up conformance run that crosses the artifact envelope and the routing commit once it exists, are both unfiled.
- **The `contracts/navigation` cells** named above.
