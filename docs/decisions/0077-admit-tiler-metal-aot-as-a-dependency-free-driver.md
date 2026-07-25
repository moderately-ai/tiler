---
schema: "tiler-doc/v1"
id: "ADR-0077"
kind: "decision"
title: "Admit tiler-metal-aot as a dependency-free offline driver"
topics: ["rust", "workspace", "dependencies", "metal", "apple-targets"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "implemented"
applies_to: ["tiler.contract.architecture"]
evidence: ["tiler.research.workspace.prototype-crate-layout-and-msrv", "tiler.research.apple-targets.compatibility"]
supersedes: ["ADR-0056"]
refines: ["ADR-0070"]
ticket: "record-an-adr-for-the-metal-aot-crate-admission"
---

# 0077: Admit tiler-metal-aot as a dependency-free offline driver

**Status:** accepted. This record was unusual in the direction it pointed: the crate, its empty dependency closure, and the development-only edge below were already implemented and mechanically pinned, and what was missing was the decision. Acceptance supplies it, so ADR 0056's clause placing AOT invocation inside `tiler-metal` is superseded and no longer retained text the workspace contradicts.

## Context

**Fact — the crate was admitted by a ticket that could not write its decision.** [`prototype-apple-aot-driver`](../../tickets/prototype-apple-aot-driver.md) created `crates/tiler-metal-aot`, added it to `Cargo.toml`'s `members` and `[workspace.dependencies]`, and pinned it in `scripts/check_workspace.py`'s `EXPECTED_MEMBERS`, `PACKAGE_DESCRIPTIONS`, `PACKAGE_DIRS`, and `EXPECTED_DEPENDENCIES`. It held the `implementation/metal-aot` and `implementation/workspace` scopes and never `contracts/decisions`, so it structurally could not have written the superseding decision in the same change. [`record-metal-aot-in-architecture-crate-profile`](../../tickets/record-metal-aot-in-architecture-crate-profile.md) then recorded the crate, its empty closure, and the development-only edge in the [architecture contract](../architecture.md)'s accepted packaging profile, and said there that the ADR record was still open. This record closes it.

**Fact — ADR 0056's AOT-invocation clause is retained text the workspace has departed from.** [ADR 0056](0056-use-four-libraries-and-two-proof-executables.md) carries `decision_status: superseded`, but its status line supersedes it only for the reusable-crate count and reference-evaluator placement (ADR 0065), for the compiler-to-artifact dependency edge (ADR 0070), and for its lockstep artifact/IR consequence (ADRs 0070 and 0071); it then says "the remaining packaging boundaries are retained". Its Decision states "MSL emission and AOT invocation remain modules in `tiler-metal`". AOT invocation is a separate crate. `AGENTS.md` requires a durable decision to be superseded explicitly rather than silently departed from, so the departure needs this record whether or not anyone would have written the crate differently.

**Fact — ADR 0065 is correct exactly as accepted and is not superseded here.** [ADR 0065](0065-extract-reference-evaluation-from-ir.md)'s "Add a fifth reusable target-independent crate" is an ordinal about the crate that record adds. Its normative content is what `tiler-reference` owns — host reference values, input bindings, evaluator traversal, evaluation diagnostics, and the separately frozen reference-capability registry — and that compiler, artifact, backend, and runtime production crates do not depend on it while proof executables and tests may. A sixth crate neither reverses nor narrows any of that. **Inference.** The ordinal reading is the whole of it, and the arithmetic settles which reading is available: ADR 0056's four are `tiler-ir`, `tiler-artifact`, `tiler-compiler`, and `tiler-metal`, and `tiler-metal` is target-*dependent*, so "fifth" is correct only when the count runs over every reusable crate. "Target-independent" therefore describes `tiler-reference` rather than restricting the set being counted — the tempting stronger argument, that a target-dependent sixth crate falls outside the category the phrase counts, is not available and is recorded here so it is not reached for later. What remains is sufficient: ADR 0065 states which crate it adds and what that crate owns, not how many the workspace may hold. It needs no superseding note and gets none; adding one would supersede a correct decision.

**Fact — ADR 0070's dependency block is incomplete rather than wrong.** [ADR 0070](0070-own-shared-compiler-ir-in-tiler-ir.md)'s Decision writes "The dependency direction is:" and lists five crates. Every edge it lists is still exactly what `scripts/check_workspace.py` pins. What it omits is the sixth crate and both development edges — including `tiler-compiler` → development `tiler-reference`, which predates the driver and which ADR 0065 already authorizes as test consumption. **Inference.** An incomplete enumeration is extended by restating it completely, not reversed by superseding it; superseding ADR 0070 would retire correct edges to add missing ones. This record therefore refines ADR 0070 and supersedes only ADR 0056.

**Fact — what the crate is, at `82254ff`.** [`crates/tiler-metal-aot/Cargo.toml`](../../crates/tiler-metal-aot/Cargo.toml) declares neither a `[dependencies]` nor a `[dev-dependencies]` table. [`crates/tiler-metal-aot/src/driver.rs`](../../crates/tiler-metal-aot/src/driver.rs) reaches Apple's offline toolchain by spawning a launcher (`xcrun` by default, resolved from `PATH`) as `xcrun --sdk <sdk> --find <tool>`, `xcrun --sdk <sdk> <tool> --version`, and the SDK identity queries, then runs the `metal` compile and `metallib` link stages through the same launcher. [`crates/tiler-metal/Cargo.toml`](../../crates/tiler-metal/Cargo.toml) declares normal dependencies on `tiler-artifact` and `tiler-ir` and a development dependency on `tiler-metal-aot`.

## Decision

### 1. `tiler-metal-aot` is the offline Apple Metal compiler driver

**Proposal.** The workspace admits `tiler-metal-aot` as a reusable library whose one responsibility is to turn Metal Shading Language text plus an explicit target and explicit output-affecting flags into `metallib` bytes with full toolchain provenance, failing closed with typed errors. It does not emit MSL, does not assemble the target-neutral artifact bundle, and does not implement the expansion cache or the proc-macro layer.

### 2. Its empty dependency closure is a decided property, not an accident of ordering

**Proposal.** `tiler-metal-aot` has no dependencies of any kind — no workspace crate, no third-party crate, no development dependency — and acquiring one is a change to this contract rather than a use of it. `scripts/check_workspace.py`'s `EXPECTED_DEPENDENCIES` enforces it: that table lists third-party crates alongside workspace ones and pins `"tiler-metal-aot": []`, so the closure is mechanically checked rather than described.

**Why the property is worth deciding.** The crate spawns the Apple compiler. Its value is that the exact invocation — the ordered `-target`, `-std`, `-O`, and numerical flags, the SDK selection, and the tool provenance recorded alongside the bytes — can be read and audited without the lowering stack behind it. A reader auditing what Tiler asks the Metal compiler to do reads one crate with nothing underneath it. That property is destroyed by the first dependency, not degraded by it, which is why it is stated as a decision instead of left as a fact about today's manifest.

**The distinction the block in item 3 does not carry on its face.** `tiler-ir -> []` there means no *intra-workspace* edges; `tiler-ir` has three `crates.io` normal dependencies and one development dependency. `tiler-metal-aot -> []` means the complete closure is empty. The two rows look alike and claim different things.

### 3. The complete packaging profile

**Proposal.** The workspace carries six reusable libraries and two non-published proof executables. Their intra-workspace edges — normal, plus development where marked — are:

```text
tiler-ir        -> []
tiler-reference -> [tiler-ir]
tiler-artifact  -> [tiler-ir]
tiler-compiler  -> [tiler-ir]                  + development [tiler-reference]
tiler-metal     -> [tiler-ir, tiler-artifact]  + development [tiler-metal-aot]
tiler-metal-aot -> []

tiler-prototype-compile -> [tiler-ir, tiler-reference, tiler-artifact, tiler-compiler, tiler-metal]
tiler-prototype-run     -> [tiler-artifact] + planned platform Metal bindings
```

This restates ADR 0070's block completely rather than replacing any edge in it. The executables are named by their exact package names; ADR 0056 wrote them informally as `prototype-compile` and `prototype-run`, and `prototypes/serial-sum-compile/Cargo.toml` and `prototypes/serial-sum-run/Cargo.toml` declare `tiler-prototype-compile` and `tiler-prototype-run`. The runner's platform Metal bindings remain part of the accepted profile rather than a landed edge, because `tiler-prototype-run` is still a stub.

### 4. The `tiler-metal` → `tiler-metal-aot` edge is development-only, and promoting it is forbidden

**Proposal.** The edge exists so `crates/tiler-metal/src/golden_compilation.rs` can compile every checked-in golden through the real driver inside the repository gate, and so `crates/tiler-metal/src/target_correspondence.rs` can see both Apple target vocabularies at once. It stays in `[dev-dependencies]`. Promoting it to a normal dependency requires superseding this record.

**Why, in the two ways it costs.** `tiler-metal` is pure source emission and owns no Apple tool discovery, so a normal edge would put a process-spawning toolchain driver into the build graph of every consumer of source emission in order to serve tests alone. And Cargo permits a dependency cycle through a development dependency while rejecting one through normal dependencies, so keeping this edge out of the normal graph is what preserves the eventual `tiler-metal-aot` → `tiler-metal` production direction that the driver's consumption of emitted source implies. A normal edge would foreclose that direction outright rather than merely making it expensive.

**Not taken now.** That production direction is reserved, not claimed. Taking it today would give the driver `tiler-ir` and `tiler-artifact` transitively and spend the closure item 2 decides, and no component yet needs it: the component that first orchestrates emission and compilation together is the one that will.

### 5. What ADR 0056 retains after this supersession

**Proposal.** This record supersedes exactly one clause of ADR 0056: that AOT invocation remains a module in `tiler-metal`. The rest of that sentence and its neighbours are untouched and remain retained boundaries.

- MSL emission remains a module in `tiler-metal`. Only the invocation half moved.
- Multiple target-independent IRs remain modules in `tiler-ir`, and compiler passes remain modules in `tiler-compiler`. ADRs 0070 and 0073 reinforce both.
- The runner depends on the artifact contract and live Metal bindings, never the compiler.
- No frontend, proc-macro, Candle, generalized cache, or reusable Metal-runtime crate is created for the first proof. **Inference.** `tiler-metal-aot` does not breach this clause and is not an exception to it: it is a build-time compiler driver that never touches a live device, an `MTLDevice`, or a pipeline state, so it is not the reusable Metal-*runtime* crate that clause withholds. A reader must not cite this admission as precedent for admitting one.

## Consequences

- The decision record stops disagreeing with the workspace. Before this, the six-crate profile was written down only in the architecture contract, which said so about itself in the same paragraph.
- The empty closure becomes a boundary a future change must argue against rather than a fact it can erode. Adding one convenience dependency to the driver is now visible as a decision, and `scripts/check_workspace.py` fails until someone makes it deliberately.
- The Apple target vocabulary stays owned twice, and item 2 is why. `tiler-metal` owns the MSL version, artifact family, and deployment minimum that emitted source declares; `tiler-metal-aot` owns the ones a compiler invocation selects; neither record subsumes the other. Collapsing the overlap into a shared type or a dependency edge would spend the driver's closure on three enumerations, which is the argument that decided it in [`choose-one-owner-for-apple-target-vocabulary`](../../tickets/choose-one-owner-for-apple-target-vocabulary.md) — dependency *closure*, not dependency direction. The reasoning and the rejected alternatives live on the types in [`crates/tiler-metal/src/target.rs`](../../crates/tiler-metal/src/target.rs) and [`crates/tiler-metal-aot/src/input.rs`](../../crates/tiler-metal-aot/src/input.rs) and are deliberately not restated here.
- What keeps those two vocabularies in step is not a shared type but a total map. [`crates/tiler-metal/src/target_correspondence.rs`](../../crates/tiler-metal/src/target_correspondence.rs) pairs every variant of each vocabulary with its counterpart through matches exhaustive over both, so a language standard or artifact family added to either crate fails `tiler-metal`'s build until the other gains it. That check can only live in `tiler-metal`, because the development edge in item 4 is the sole edge in the workspace over which both vocabularies are visible at once — which also bounds it to a test rather than a production conversion.
- The two driver enums that map matches across the crate boundary — `MslVersion` and `ApplePlatform` — cannot become `#[non_exhaustive]`. That is ADR 0074's convention 5b and this is a site of it: item 4's edge is what makes the convention bind here, because an out-of-crate exhaustive match is what turns a divergence into a build failure, and `#[non_exhaustive]` would force in a wildcard arm that could only invent a family or a language standard.
- The profile still deliberately omits frontend, proc-macro, Candle, generalized cache, and reusable Metal-runtime crates. What ADR 0056 set was the rule that a crate is admitted when evidence requires one, not a fixed count; admitting a sixth on that basis applies the rule rather than relaxing it.

## Alternatives considered

**Keep AOT invocation as a `tiler-metal` module, as ADR 0056 decided.** This is the smallest change — it requires none — and it is the honest baseline. It is rejected because it puts the lowering stack behind the component that spawns the compiler: `tiler-metal` depends on `tiler-ir` and `tiler-artifact`, so the exact `xcrun` invocation could not be read in isolation, and every consumer of pure source emission would acquire Apple tool discovery in its build graph. ADR 0056 made that choice before any AOT invocation existed to weigh, and the shape the driver actually took is the evidence that changed.

**Promote the development edge to a normal dependency.** Simpler to explain and removes a manifest subtlety. Rejected for both reasons in item 4, the second of which is not a preference: a normal edge makes the reserved `tiler-metal-aot` → `tiler-metal` direction a Cargo error rather than a future decision.

**Give the driver a normal dependency on `tiler-metal` now.** This is the eventual production direction and Cargo permits it today, since the opposing edge is development-only. Rejected because it would spend the empty closure in item 2 before any component needs the direction, and because the obligation belongs to whichever component first orchestrates emission and compilation together rather than to the driver.

**Supersede ADR 0070 rather than refine it.** Attractive because one record would then hold the whole dependency block. Rejected because every edge ADR 0070 states is still correct, so superseding it would retire correct decisions in order to add omitted ones, and would tell a reader that some edge of it had changed when none has.

**Write the Apple target-vocabulary split as its own ADR.** Rejected in [`record-metal-aot-in-architecture-crate-profile`](../../tickets/record-metal-aot-in-architecture-crate-profile.md), whose reasoning this record does not relitigate: the split changes no public surface, so none of ADR 0075's always-ask categories reaches it, and its full argument already lives on three source modules. It appears above as a consequence because it *follows from* item 2 — a shared crate or a dependency edge for three enums is rejected because it costs the driver its closure — which puts it in the indexed decision record without creating a fourth authority over it.

## Implementation boundary

**Fact — nothing in this record is unimplemented.** `implementation_status` is `implemented`: `Cargo.toml` carries the six library members, `scripts/check_workspace.py` pins the member set, the `[workspace.dependencies]` table, and every package's complete normal and development dependency list including third-party crates, `crates/tiler-metal/src/golden_compilation.rs` uses the development edge, and `crates/tiler-metal/src/target_correspondence.rs` enforces the vocabulary correspondence. Acceptance changes the record, not the workspace.

**The edit withheld until acceptance has now been made.** ADR 0056's Consequences already carried an in-body **Retired:** marker where ADRs 0070 and 0071 retired a clause; its Decision paragraph now carries the same marker beside "MSL emission and AOT invocation remain modules in `tiler-metal`". Writing it before acceptance would have asserted a supersession that had not happened, which is why it was withheld rather than landed with the draft.

## Traceability

The [prototype crate layout research](../research/workspace/prototype-crate-layout-and-msrv.md) is the evidence that the crate set is the mechanical enforcement of Tiler's layer separation rather than a packaging convenience, which is what makes admitting one a decision. The [Apple Metal artifact-compatibility research](../research/apple-targets/artifact-compatibility.md) is the evidence behind the target vocabulary the driver names — the SDK-selected artifact families, the measured MSL 3.1 standard, and the deployment minimum — and keeps Mac Catalyst explicitly deferred rather than relabelled. The [architecture contract](../architecture.md) owns the packaging profile this record decides. The work records are [`prototype-apple-aot-driver`](../../tickets/prototype-apple-aot-driver.md) for the crate, [`compile-golden-msl-through-the-aot-driver-in-the-gate`](../../tickets/compile-golden-msl-through-the-aot-driver-in-the-gate.md) for the development-only edge and its two reasons, [`choose-one-owner-for-apple-target-vocabulary`](../../tickets/choose-one-owner-for-apple-target-vocabulary.md) for the vocabulary split, and [`record-an-adr-for-the-metal-aot-crate-admission`](../../tickets/record-an-adr-for-the-metal-aot-crate-admission.md) for this record.
