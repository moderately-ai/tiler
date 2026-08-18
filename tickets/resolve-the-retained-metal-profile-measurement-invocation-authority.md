---
id: resolve-the-retained-metal-profile-measurement-invocation-authority
title: Resolve the retained Metal profile measurement invocation authority
status: in-progress
priority: p1
dependencies: [decide-the-compilation-selection-provenance-public-and-wire-surface]
related: [carry-required-compilation-selection-identity-on-compile-profile-contexts, split-metal-profile-measurement-sources-by-compilation-selection]
scopes: [implementation/build, implementation/metal-aot, research/target-profiles, contracts/numerics, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [metal, target-profiles, provenance, measurement, identity, decision, needs-tom]
claimed_from: todo
assignee: worker-invocation-authority
lease_expires_at: 1787088563
---
## User-visible outcome

Every retained measured row in the authoritative Metal compile profile either cites a checked exact compilation selection that produced it or is removed fail-closed. No row inherits a neighbouring run's flags, reconstructs an unavailable harness by inference, or lets an unchecked caller self-certify compiler arguments.

## Why this is a decision prerequisite

This is a consequential `decision` / `needs-tom` ticket, not implementation detail. Remeasuring a support row, withdrawing an authoritative support row, and admitting a new recorded-invocation authority each change what evidence Tiler treats as sufficient. It is blocked on the parent public/wire decision. The parent may be accepted independently; the implementation carrier depends on both and therefore remains structurally blocked until this ticket is accepted and completed.

No production implementation, queue edit, or status transition is authorized here. A read-only inspection and a rerun on an already-qualified, byte-for-byte matching evidence environment are ordinary bounded research; choosing the disposition and changing the authoritative profile still requires Tom. Installing, selecting, upgrading, or substituting any host, OS, Xcode, SDK, compiler, linker, or device component changes the evidence environment and separately requires Tom's authorization.

## Exact-base audit — 2026-08-17 at `a01e78b7c99ea8ee00a7e2e58894094587da9def`

The production declaration makes sixteen measured declaration operations and produces twenty-four canonical profile rows, found from the complete `BoundMetalCompileDeclaration::declare` path, both dtype projection helpers, and `declare_measured_subnormal_dimension` in `crates/tiler-compiler/src/target.rs`:

- one grid-axis extent;
- one saturated-parallel-fold-steps cost;
- one workgroup-tree-width policy;
- two dtype-dispatchability rows;
- four dtype/subnormal dimension declarations, input and result for F32 and BF16, each expanding to a complete three-row table (twelve canonical rows); and
- seven remaining F32 numerical rows: contraction, the forbidden and permitted reassociation contracts, permutation, signed zero, NaN assumptions, and infinity assumptions.

`let measured = measured_source(rows)?` currently creates one source and every one of those sixteen operations / twenty-four rows receives that source or a clone. Equal compiler-build and execution-environment fields do not establish equal compilation selections.

### Population A — grid axis: exact source, non-production selection

**Fact.** `compile_probe` in `spikes/target-profiles/metal-grid-axis-extent/src/main.rs` invokes `xcrun --sdk macosx metal` with `-std=metal4.0`, `-target air64-apple-macos26.0`, `-c`, and `-o`; it then invokes `xcrun --sdk macosx metallib <AIR input> -o <metallib output>`. After the linker tool/SDK selection there are no additional linker flags; the AIR input and output paths are excluded from selection. The compiler invocation passes no optimization or numerical-selection flags. The source was introduced with the fixture at `aa17fe8afba0468418c1bc51bf7bc052f1b96742` and is byte-identical at this base, SHA-256 `360eea9e7644e5ef9fcf08c8f410ca13d46665cb9b631b6be0d48374d1655266`. The retained `extent.tsv` positively records standard and target, not a statement that the unlisted flag remainder is complete; completeness is recovered from the retained source. The result carries no harness hash or repository revision binding that source to the run, so the matching repository history does not prove that source produced the record and a recorded-invocation disposition must close that gap.

**Fact.** Every `tiler_metal_aot::CompileRequest::compile_flags` invocation contains target, standard, one optimization flag, and all three numerical flags. Its driver selects `metallib` with `xcrun --sdk <sdk> --find metallib`, invokes the resolved binary with AIR input and `-o` output, and `link_flags` is the empty counted run of additional flags after that tool/SDK selection. There is no lower Metal selection constructor: the full `CompilationIdentity::new` is private and includes source/toolchain, while output `ArtifactProvenance` cannot identify a request-only selection. The grid invocation therefore is not representable by the proposed `CompileRequest`-derived selection without claiming flags that did not run.

### Population B — saturated cost: pinned source unavailable

**Fact.** `spikes/program-planning/reduction-dispatch-crossover/results/2026-08-07-apple-m4-max-macos27.0-26A5388g/environment.tsv` pins repository base `01f140237f3617a5d415dbc0a67182a83ac8d139` and `sweep.harness_sha256=d76fcd2fb74ecfe00b492c3042c0d1be58d88a6420f91b5cdb1940555bf9e27b`. The spike path does not exist at that base. The checked-in and introduction-revision `src/main.rs` hash is `9952f2efa414fecabd6a1cfaa11b6d2197e4bf2afc013d5f87ceb2c170e2edca`, not the pinned hash, and the exact unreachable-blob scan below found no source with SHA-256 `d76fcd2fb74ecfe00b492c3042c0d1be58d88a6420f91b5cdb1940555bf9e27b` while its known reachable control was detected.

**Fact.** The available later source uses `CompileRequest` with `OptimizationLevel::Default`, the declaration's strict numerical realization, and the empty additional-linker-flag selection after `xcrun --sdk <sdk> --find metallib` has selected the linker. That proves the later harness's selection, not the unavailable harness's. Assigning O2/safe/precise/contract-off to the retained cost result would be inference.

### Population C — tree width: content-recovered production selection

**Fact.** The retained 2026-08-07 partition-calibration environment pins `main.rs=a99a7372bcfa483da5036f9abef4c19a514c57e680bffb61efc48407c9c4c0f3`, `regions.rs=1a7d4f492fef25d0ea719c8e47bfccac36e9b724e7bf79f0e2873c1b5d61ab96`, `buffer.rs=a3f8303eb323c147c21b2ef2547d89e8e1cb0b0f205be66bb33b96b9c08b03fe`, and `regret.rs=031b051b5dbc4d4601c540c8c723f59303cb24ce94827d526e5c0aef1b3db432`. Although its stated base predates the path, all four contents are recoverable exactly at introduction commit `80d9949114efe39b425f9aa1e2042186463c02c4`. The recovered `prepare` path constructs `CompileRequest` with `OptimizationLevel::Default`, the bound declaration's strict numerical realization, and no additional linker flags after tool/SDK selection. Its exact selection is therefore reconstructable as macOS/MSL 4.0/O2/safe/precise/contract-off with the empty additional-linker-flag run.

### Population D — dispatchability and numerics: retained exact production selection

**Fact.** The unified 2026-08-02 F32+BF16 records pin repository base `0fcc952ac8f548f462eff6b204386253e65d2522` and `probe.harness_sha256=17b8b8ddc7731ba1a11f6e971e17cf3fa874ff4153a52de95c634331693a9bb6`; the file at that base and the current `numerical_probe.py` both match. `Profile::offline_flags` emits target, standard, optimization, math mode, fp32-functions mode, and contraction mode in a complete exact order; `Toolchain::link` invokes `xcrun --sdk <family.sdk> metallib <AIR input> -o <metallib output>` with no additional linker flags after tool/SDK selection. The rows the declaration consumes are the witnessed macOS/MSL 4.0/O2/safe/precise/contract-off cases. Their selection is reconstructable and equals Population C's selection byte for byte.

**Inference, bounded.** Populations C and D may share a canonical source only after an implementation assertion independently derives both retained selections and the current production request and observes equality. Shared environment text alone is not that assertion. Populations A and B cannot join them at this base.

### Reproduction commands run at the packet base

```sh
rg -n 'declare_measured_' crates/tiler-build/src/metal_declaration.rs
sed -n '329,380p' spikes/target-profiles/metal-grid-axis-extent/src/main.rs
sed -n '1,16p' spikes/target-profiles/metal-grid-axis-extent/results/2026-08-04-apple-m4-max-macos27.0-26A5388g/extent.tsv
git log --diff-filter=A --format=%H -- spikes/target-profiles/metal-grid-axis-extent/src/main.rs
git show aa17fe8afba0468418c1bc51bf7bc052f1b96742:spikes/target-profiles/metal-grid-axis-extent/src/main.rs | shasum -a 256
git cat-file -e 01f140237f3617a5d415dbc0a67182a83ac8d139:spikes/program-planning/reduction-dispatch-crossover/src/main.rs
git show 54d2c9e61730409096576f4b02fe386a8a9a27e4:spikes/program-planning/reduction-dispatch-crossover/src/main.rs | shasum -a 256
git cat-file -e 6ed456f4a70d75a3e9ae6c1f78ccee602a4bf3db:spikes/program-planning/reduction-partition-calibration/src/main.rs
git show 80d9949114efe39b425f9aa1e2042186463c02c4:spikes/program-planning/reduction-partition-calibration/src/main.rs | shasum -a 256
git show 0fcc952ac8f548f462eff6b204386253e65d2522:spikes/apple-targets/numerical_probe.py | shasum -a 256
target_digest=d76fcd2fb74ecfe00b492c3042c0d1be58d88a6420f91b5cdb1940555bf9e27b; control_digest=9952f2efa414fecabd6a1cfaa11b6d2197e4bf2afc013d5f87ceb2c170e2edca; control_blob=$(git rev-parse 54d2c9e61730409096576f4b02fe386a8a9a27e4:spikes/program-planning/reduction-dispatch-crossover/src/main.rs); { printf '%s control\n' "$control_blob"; git fsck --full --no-reflogs --unreachable --no-progress 2>/dev/null | awk '$2 == "blob" { print $3, "unreachable" }'; } | while read -r object_id object_class; do object_digest=$(git cat-file blob "$object_id" | shasum -a 256 | awk '{ print $1 }'); if [ "$object_digest" = "$target_digest" ]; then printf 'target %s %s\n' "$object_class" "$object_id"; fi; if [ "$object_class" = control ] && [ "$object_digest" = "$control_digest" ]; then printf 'control %s %s\n' "$object_class" "$object_id"; fi; done
```

The log prints `aa17fe8afba0468418c1bc51bf7bc052f1b96742`; the grid hash prints `360eea9e7644e5ef9fcf08c8f410ca13d46665cb9b631b6be0d48374d1655266`. The two `cat-file` commands fail because the paths do not exist at the recorded bases. The remaining three hashes print, respectively, `9952f2efa414fecabd6a1cfaa11b6d2197e4bf2afc013d5f87ceb2c170e2edca`, `a99a7372bcfa483da5036f9abef4c19a514c57e680bffb61efc48407c9c4c0f3`, and `17b8b8ddc7731ba1a11f6e971e17cf3fa874ff4153a52de95c634331693a9bb6`. The final scan prints exactly `control control 607944f6929a5170620dd44c4fe4fb1a8c24bb89` and no `target` line. The known reachable blob and digest make an empty or broken blob traversal visible; absence of a `target` line is therefore evidence from the scanned reachable-control plus unreachable-blob population, not from a command that visited nothing.

## Exact decision frontier

The choice is per unresolved population; Tom need not give grid and cost the same disposition.

### 1. Remeasure through production `CompileRequest` — recommended when the exact environment remains available

Rerun Population A and/or B through the production request authority, retaining the request inputs, canonical selection bytes, harness/source digests, environment, raw outputs, validation, and resulting profile value. Reconcile the grid bound or cost value and every dependent test/ledger pin if it changes. The provenance identity comes from the same request whose compilation executes, never from copied flags.

Strongest counterargument: a rerun can move a support or cost row because the old harness shape cannot necessarily be reproduced through the production path. Reversal evidence: proof that the production request cannot drive the same measured subject, or that the exact qualified environment is no longer present. Negative controls independently change the request's reachable platform/target, standard, optimization, and each numerical dimension and quote the build-owned mismatch; a real future linker input gets its own control, while today's zero-count linker run gets a census assertion only.

### 2. Remove the unresolved row fail-closed

Remove only the affected grid guarantee and/or cost preference. Grid capacity then resolves `Unknown` for the unsupported dispatch population; removing the cost row leaves feasibility unchanged and removes only that measured preference. Do not replace either with a representability ceiling, governed floor, copied later value, or default.

Strongest counterargument: grid removal makes every plan needing a compile-profile grid guarantee unavailable; cost removal loses a measured selection preference. Reversal evidence: recovery or reproduction of the exact producing invocation. Negative controls prove the absent row resolves `Unknown` (grid) or leaves the feasible plan population unchanged while removing the preference (cost).

### 3. Admit checked recorded-invocation authority — only behind a separately accepted exact boundary

This can apply to Population A because its source is retained, and to Population B only if source SHA-256 `d76fcd2fb74ecfe00b492c3042c0d1be58d88a6420f91b5cdb1940555bf9e27b` is recovered. The boundary must name who records the complete ordered compiler and linker selections, bind them to the retained harness/source/environment, prove the list is complete rather than merely positive, and let Metal validate a canonical grammar before provenance construction. An arbitrary public `from_bytes`, `from_flags`, or independently supplied facts-plus-source adapter is producer self-certification and is eliminated.

This candidate does not pretend the historical grid invocation equals today's `CompileRequest`: the compilation-selection identity is provenance and does not select or rewrite a production request. No current resolver conditions a profile fact on selection, so provenance recovery alone would still apply the no-optimization/no-numerical grid result under O2/safe/precise/contract-off without authority. The parent packet admits only request-derived identity plus exact production equality. Therefore this disposition must specify both (a) a sealed `tiler-metal-aot` evidence producer returning the same opaque identity and (b) an accepted population-specific transfer/selection-independence rule with a typed refusal when its premises do not hold. If either needs a new public type, function, constructor, or error, it amends the parent packet before implementation. Until both are accepted, option 3 is research/decision work rather than an implementation shortcut.

Strongest counterargument: the recorder and validator may merely restate the same invocation and create a second Metal authority beside `CompileRequest`, while even perfect provenance does not prove the fact transfers to a different production selection. Reversal evidence requires both an independently retained invocation transcript whose completeness is mechanically checked against process execution and owner-level evidence establishing exactly which selection differences leave this population valid. Negative controls delete, reorder, and alter each real recorded argument, then independently perturb every premise of the transfer rule and the production optimization and numerical selections; each must quote the owning typed refusal.

### Eliminated candidates

- Inferring Population A's missing optimization/numerical flags or Population B's flags from current code silently invents authority.
- Giving all sixteen declaration operations / twenty-four canonical rows one selection because their compiler/environment strings match repeats the defect.
- Adding an unchecked opaque byte field to `LedgerRows` lets the producer self-certify the proposition under test.
- A test-only linker option does not exercise production authority.
- Deferring this ticket while implementing the carrier leaves no compiling, truthful production partition and is not an implementation candidate.

## Required atomic delivery after Tom's decision

- Represent each retained measurement population separately in the private ledger. Rows may share one source only after complete canonical selection equality is asserted.
- Under the currently proposed parent packet, retain an independently derived expected selection for each population, construct its source contexts from it, and compare it with `CompileRequest::compilation_selection_identity()` before building the complete profile descriptor. A mismatch fails as `BoundMetalDeclarationError::CompilationSelectionMismatch { population: MetalProfileMeasurementPopulation }` with the exact per-population production-request display texts fixed there. If Tom instead chooses option 3 for a differing selection, amend the parent packet with the accepted transfer/applicability vocabulary, enforcement, and exact refusal before the carrier starts; context equality alone is insufficient.
- Keep the generic IR/compiler layers opaque. Metal/build owns the comparison and cannot delegate semantic validation of Metal bytes to a generic compiler.
- Preserve the current empty production additional-linker-flag run honestly after `xcrun --sdk <sdk> --find metallib` tool selection and the required AIR input/`-o` output. Encode and count zero additional flags; add no product input solely to satisfy a perturbation request.
- Follow the parent packet's Tom-selected adapter branch exactly. Retention keeps `declare_metal_f32_subnormal_behaviour` explicitly caller-vouched and non-authenticating; retirement must not recreate it under another name. Neither branch may use it as invocation authority.

## Unsupported population until closure

- The retained grid row has no currently accepted exact selection authority; its raw invocation is not representable by `CompileRequest`.
- The retained saturated-cost row has no recoverable exact producing source/selection.
- Any host/toolchain substitution is outside the evidence population.
- Any future nonempty linker selection remains unsupported until a real request field and retained measurement exist.

## Closes when

Tom has accepted a disposition for both unresolved populations; each retained row has reproducible exact selection authority equal to the selection in its compile-profile context and is either equal to the production request or covered by an accepted enforced population-specific transfer rule, or is removed with its unsupported population stated; Populations C and D have an independently checked equality; and the carrier can land without historical inference or public self-certification.
