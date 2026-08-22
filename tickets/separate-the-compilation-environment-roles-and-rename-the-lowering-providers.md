---
id: separate-the-compilation-environment-roles-and-rename-the-lowering-providers
title: Separate the compilation-environment roles and rename the lowering providers
status: in-progress
priority: p1
dependencies: []
related: [package-selected-physical-implementation-provenance-in-artifact-identity, decide-the-artifact-physical-selection-provenance-surface]
scopes: [implementation/artifact, implementation/build, contracts/artifacts, implementation/runtime, contracts/foundation, research/cache, research/artifacts, research/runtime, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, refactor, public-boundary, provenance]
claimed_from: todo
assignee: worker-roles
lease_expires_at: 1787430750
---
## User-visible outcome

`CompilationEnvironment` carries an explicit lowering role and an explicit physical role rather than one undifferentiated set, and every existing selected-provider name says *lowering* — so the physical-selection surface can be added next without a rename tangled into an identity step.

## Why this exists

Split out 2026-08-22 by the coordinator when `package-selected-physical-implementation-provenance-in-artifact-identity` **stopped correctly** rather than leave a half-applied identity migration. That worker mapped the full scope at base `e1ada851`: **216 rename occurrences across 56 files**, 72 `CompilationEnvironment::new` sites, 15 coupled `VariantSpec` literals, roughly 1,500 lines of new surface across 11 core files, plus the `tiler-build` bridge, `docs/artifact-abi.md`, and pin/golden/cache-subject recomputation. It got as far as writing new capacity constants and **reverted them**, because landing a role rename half-applied is exactly the incoherent state an identity migration must never be left in. That was the right call.

**This lane is the half that gates on its own.** It performs no identity step, adds no new surface, and steps no domain. The parent keeps the physical-selection run, the `v21` to `v22` step, its pins, and its perturbations, as one coherent migration.

**Coordinator deviation from the delivering worker's proposal, stated so it is not mistaken for an oversight.** That worker proposed three lanes: rename, then physical-row surface, then pins-and-perturbations. I split it into **two**. Pin and golden recomputation cannot be a lane separate from the domain step that invalidates them — AGENTS.md requires identity-domain changes to stay coherent across owning version, ledgers, and pins, and recomputing them in a later lane would publish a tree whose pins disagree with its bytes. The rename genuinely does gate alone; the pins genuinely do not.

## Facts, verified by the coordinator at `2cc3aefa`

**Fact — a bulk rename would silently corrupt two unrelated APIs, and both are out of scope.** `spikes/program-planning/physical-frontier-budget-calibration/src/program.rs` declares `pub selected_providers: usize` at three sites — a spike-local summary counter with no relation to the artifact surface. `spikes/artifacts/artifact_envelope.rs` carries **8** occurrences of the same name in its own standalone model; it is a `rustc --test` file that does not link `tiler-artifact` at all. Both match the rename pattern. **Neither may be touched.**

**Fact — one packet instruction has no subject.** The accepted packet's section 1 requires that "built **and decoded** `selected_providers()` accessors" be renamed. There is no decoded accessor: `crates/tiler-artifact/src/program/codec/view.rs` contains **zero** occurrences of the string `provider`, and no `pub fn …provider` exists anywhere under the codec directory. Only the model-side accessor exists. Rename the one that exists and **record that the packet's population was wrong** rather than inventing the missing one.

**Fact — the superstring ordering is what makes the rename safe.** `SelectedProviders` is the sole extension of `SelectedProvider`, so renaming the longer symbol first makes the remainder unambiguous. Verify this at your base before relying on it; a second extension appearing would invalidate the ordering.

## Required work

- Re-audit all three Facts at your base and report a per-Fact verdict; re-derive the 216/56 census rather than trusting it, and **say which unit you report** — `grep -c` counts lines, not occurrences.
- Apply the rename with a script that **reads each file and writes it back whole**. This environment blocks stream-editor mutation in place, and that block has already prevented one corruption here, so treat it as a guard rather than an obstacle. Exclude both spike populations above by path, explicitly, and print the excluded set so a reviewer can check it.
- Separate `CompilationEnvironment` into explicit lowering and physical roles: required sets, **no union, no default, and no inference from payload, backend, or profile**. A missing member is a typed artifact-build refusal; never substitute the governed provider.
- Validate existing selected *lowering* rows against the lowering set only. Do not add any physical row, accessor, tag, or byte — that is the parent's work.
- **No identity, schema, domain, pin, golden, or cache subject may move in this lane.** Rederive and state that; if one moves, **stop and report** — that would mean the split boundary is wrong.

## Worker Fact audit at base `0e28564a` — 2026-08-22

Unit note: every count below is **occurrences**, produced by `str.count` over the whole file (not `grep -c`, which counts lines). Where the two differ it is stated.

**Fact 1 — verified, with one imprecise sub-claim repaired.** `spikes/program-planning/physical-frontier-budget-calibration/src/program.rs` carries **6** occurrences of `selected_providers` on 6 lines. The coordinator's brief said it "declares `pub selected_providers: usize` (I saw it at three sites)"; the *declaration* occurs once, at the `pub selected_providers: usize` field, and the other five are struct-literal initializers and one assignment. The claim that matters — that this is a spike-local summary counter unrelated to the artifact surface — is verified. `spikes/artifacts/artifact_envelope.rs` carries exactly **8** occurrences of `selected_providers` on 8 lines, and is verified standalone: its header states `rustc --edition 2021 --test spikes/artifacts/artifact_envelope.rs`, it has no `use tiler_artifact`, and it defines its own `Digest`, `ProgramId`, and envelope constants. Both files are on the explicit deny list and neither was opened for rewrite.

**Fact 2 — verified, and the packet's population is wrong as the coordinator said.** `crates/tiler-artifact/src/program/codec/view.rs` contains **0** occurrences of the lowercase string `provider`. It contains exactly one case-insensitive match, `ArtifactCodecError::InvalidProviderIdentity`, which is an error-variant spelling and not an accessor. No `pub fn` ending in `provider` exists anywhere under `crates/tiler-artifact/src/program/codec/`; the three functions that come closest are `pub(super) fn read_providers`, a private `fn provider`, and `pub(crate) fn providers`. The whole tree has exactly one definition of the accessor, the model-side one, spelled `pub fn selected_providers` at this ticket's base and `crates/tiler-artifact/src/program/model.rs "pub fn selected_lowering_providers"` after this lane — the citation is pinned to the live spelling on purpose, because the base-time spelling no longer exists anywhere and a citation to it would read as false absence. **The accepted packet's section 1 instruction to rename "built **and decoded** `selected_providers()` accessors" therefore names a decoded accessor that does not exist and never did in this tree.** Only the built accessor was renamed; nothing was invented to satisfy the missing half.

**Fact 3 — verified.** `SelectedProviders` (5 occurrences in `.rs`) is the sole identifier extending `SelectedProvider` (66 occurrences); the census enumerated every `SelectedProvider[A-Za-z0-9_]*` match and found no third form. The superstring ordering is therefore sufficient, and it was applied.

**Fact 4, added by the worker — a superstring hazard the brief did not name.** `selected_provider` (singular) is a live prefix of `selected_provider_identities`, a **physical**-provider test helper defined in `crates/tiler-compiler/tests/external_physical_provider.rs` and `spikes/extensions/forkless-physical-provider/probe/tests/composition.rs`. Renaming it to say *lowering* would have been actively false. Both paths are on the deny list.

**Fact 5, added by the worker — the packet's version numbers are stale, which does not affect this lane.** The packet's section 5 says to step `ARTIFACT_DOMAIN` from `tiler.artifact-program.v18` to `v19` and `MANIFEST_SCHEMA` from 18.0 to 19.0. At this base the live domain is `tiler.artifact-program.v21`, observed directly in the captured identity bytes (`74696c65722e61727469666163742d70726f6772616d2e763231` decodes to `tiler.artifact-program.v21`). The coordinator's `v21` to `v22` framing is the correct one. **No domain was stepped in this lane.**

### Re-derived census

The rename population is **177 token occurrences across 46 files**, plus **71 `CompilationEnvironment::new` call sites across 33 files** that the two-argument constructor required, for a whole-edit population of **51 files**. This does not reproduce the previous worker's *216 occurrences across 56 files*, and the difference is not a contradiction: that figure was taken at `e1ada851` against a token set that is not enumerated in the report, and it was a scope estimate for the whole migration rather than for this lane, which adds no physical row. The unit is occurrences.

### Excluded set, printed by the rename script

Two files by name, because both match the rename pattern and neither may move:

- `spikes/program-planning/physical-frontier-budget-calibration/src/program.rs` (12 matching occurrences, untouched)
- `spikes/artifacts/artifact_envelope.rs` (17 matching occurrences, untouched)

Six subtrees, because they carry either dated records or physical-provider vocabulary: `docs/research/documentation/ticket-audit-2026-08-10/`, `docs/research/extensions/`, `crates/tiler-compiler/`, `spikes/extensions/forkless-physical-provider/`, `spikes/program-planning/`, and `tickets/`.

### Scopes added, and why

The accepted packet mandates a **two-argument** constructor with no one-argument overload. That makes every `CompilationEnvironment::new` call site a compile error until it states both roles, and those call sites are not confined to the three declared scopes. Under AGENTS.md ("Adding scopes required by authorized work is scheduling metadata; add and explain them") this ticket now also declares:

- `implementation/runtime` — `crates/tiler-runtime/tests/adapter_route/fixture.rs` (2 sites) and `prototypes/serial-sum-run/src/proof.rs` (1 site). Both are workspace members, so without them `cargo nextest run --workspace` cannot be green.
- `contracts/foundation` — `docs/operation-extensions.md` names `SelectedProvider::capability` in live present-tense contract language. Left alone it would become a false statement about the boundary. The edit is the spelling plus a dated correction note; the accepted surface is unchanged.
- `research/cache`, `research/artifacts`, `research/runtime`, `research/target-profiles` — six spike harnesses construct a `CompilationEnvironment` against the real crate. They are not workspace members and so do not gate, but leaving them uncompilable would rot cited research.

### Identity neutrality, rederived rather than asserted

Nothing encoded is derived from a Rust identifier. `PROVIDER_KEY_DOMAIN` is the byte literal `b"tiler.artifact-program.provider.v3\0"` and is **not** stepped; `ARTIFACT_DOMAIN` stays at v21 and `MANIFEST_SCHEMA` is untouched; the offered sets were never serialized before this change and are not serialized after it, so splitting one into two cannot reach a byte. The renamed `ArtifactEntityKind`, `ArtifactLimitKind`, `OrderedSubject`, and `ArtifactDiagnostic` spellings reach only `Debug`/`Display` diagnostic text — `ArtifactDiagnostic::rule()` is consumed by assertions and error reporting and by no encoder. No pin, golden, or cache subject moved, and none needed recomputation.

**Negative control.** A dump of canonical identity bytes plus encoded envelope bytes for four fixtures — `default_artifact`, `partial_window_artifact`, `strict_affine_u4_dequantize_artifact`, and a two-offered/one-selected artifact — was captured at `0e28564a` before any edit and again on the finished tree. Both sides hash to `d97c8d68a90865c90d44d156d942ba068ad7f7eabcd10ec140bc5d2da4e9e60e` and `diff` reports no difference across roughly 480 KB of hex. The control was then shown reachable: perturbing `PROVIDER_KEY_DOMAIN` from `.v3` to `.v4` moved the dump to `69ae21a8e2a5a3e6d5488fbde0592158d06d587b131d72af4a9baae15850c284`, and reverting restored the original hash exactly. The dump harness was temporary and is not in the commit.

## Evidence

- Perturb the subject separately for each new refusal and quote the failure text: an absent lowering member, and a cross-role member supplied to the wrong set.
- One negative control that a correct environment still builds and its artifact bytes are **byte-identical** to the pre-rename tree. That control is the whole justification for calling this lane identity-neutral.
- Before trusting any new check, state what it would take for it to say *no*, and confirm that case is reachable.

## Non-goals

The physical-selection run, its encoding, the `v21` to `v22` step, pins, goldens, and cache subjects — all belong to [`package-selected-physical-implementation-provenance-in-artifact-identity`](package-selected-physical-implementation-provenance-in-artifact-identity.md). Touching either spike population. Inventing the decoded accessor the packet names but the tree lacks.

## Closes when

Roles are separated with cross-role and absent-member refusals watched firing, every surviving selected-provider name says *lowering*, both spike populations are demonstrably untouched, artifact bytes are unchanged against the pre-rename tree, and the full repository gate is green.
