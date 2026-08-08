---
id: refresh-the-forkless-physical-provider-spike-against-the-landed-seam
title: Refresh the forkless physical-provider spike against the landed seam
status: in-progress
priority: p2
dependencies: [drive-an-external-physical-implementation-provider-through-compilation]
related: [prototype-a-forkless-custom-metal-physical-provider, record-the-landed-physical-provider-seam-in-adrs-0078-and-0090, disclose-offered-and-selected-physical-provider-sets-separately, disclose-the-physical-provider-environment-a-compilation-was-offered]
scopes: [research/extensions]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, spike, evidence]
claimed_from: todo
assignee: coord
lease_expires_at: 1786184079
---
## User-visible outcome

The forkless physical-provider spike either compiles and drives the landed seam end to end from a genuinely out-of-tree crate, or its retained compile-fail goldens state the boundary that actually holds — so the spike stops asserting a blocker that was removed.

## Why this exists

`drive-an-external-physical-implementation-provider-through-compilation` landed the installable seam on 2026-08-08 and did not hold `research/extensions`, so the spike was not touched. The spike's probe takes `tiler-compiler` by `path` (`spikes/extensions/forkless-physical-provider/probe/Cargo.toml`), so it tracks the live tree rather than the commit its Measurement is pinned to.

**Fact, verified at `750b29e0`.** `spikes/extensions/forkless-physical-provider/probe/tests/ui/fail/no_physical_provider_installation_seam.stderr` pins `error[E0599]: no method named \`with_physical_providers\` found for struct \`CompileRequest<'a>\``. That method now exists — `grep -n "fn with_physical_providers" crates/tiler-compiler/src/session.rs` returns one line — so the golden cannot be reproduced. The fixture's own header comment also cites `crates/tiler-compiler/src/pipeline/planning.rs:171` and `request.rs:542` as the blockers, both stale.

**Fact.** Spikes are run manually and gate nothing (`AGENTS.md`, Research), so this is a stale artifact rather than a red gate. Nothing in `make full` exercises it.

## Fact audit at base `cb62784c`, 2026-08-08

Each verdict is from reading the named file in full at this base and from one recorded run of the spike as it stood, whose log is quoted below.

**Verified, and understated — the installation golden.** `grep -n "fn with_physical_providers" crates/tiler-compiler/src/session.rs` returns exactly one line, and the golden pins the `E0599`. The understatement is what the run reports: `cargo nextest run --workspace` in the spike at `cb62784c` failed **three** of five compile-fail fixtures, not one. The installation fixture's actual output is not "the method appeared" but `error[E0308]: mismatched types ... expected \`InstalledPhysicalProviders<'_>\`, found \`[&ProviderIdentity; 1]\``, because the landed method takes an installation type rather than a provider list.

**False as a blocker claim, verified as a visibility claim — `frontier_provider_vocabulary_is_private`.** `mod frontier;` is still private in `crates/tiler-compiler/src/lib.rs`, so the ticket's implicit premise that only one golden had moved is wrong on both counts: this one had also moved, and for the opposite reason. `crates/tiler-compiler/src/physical_provider.rs` publicly re-exports the vocabulary, so rustc now emits `help: consider importing this struct instead: tiler_compiler::physical_provider::…` for six of the eight errors, and the byte comparison reports `mismatch`. The module is private; the *finding* that the vocabulary is unreachable is refuted.

**False, and with a third stale citation of its own — `provider_inputs_are_private`.** All four modules it names are still private, so it would have been easy to record this fixture as unaffected. It also failed. `pipeline::compile` no longer exists: the actual output leads with `error[E0432]: unresolved import \`tiler_compiler::pipeline::compile\` … no \`compile\` in \`pipeline\``, ahead of the four `E0603`s. The fixture's *claim* — that the provider seam's transitive closure is private and therefore blocking — is also refuted: none of those four modules is needed, because `ImplementationContext` supplies the target profile, the resolved numerical realization, the region subject, and the host's own baseline spelling of it.

**Verified — the header comment's two citations.** `crates/tiler-compiler/src/pipeline/planning.rs` no longer contains the one-element provider literal, and `request.rs:542` names unrelated code. Both stale, as stated.

**Verified — the spike gates nothing.** The `Makefile` has no target under `spikes/`, its header says so in terms, and the root `Cargo.toml` member list is explicit and reaches no spike, so `cargo check --workspace --all-targets` never builds this workspace. `check-citations.sh` reads `tickets/**`, `docs/**`, and root `*.md` only, so the spike's own README links are outside the citation gate as well.

## Outcome, 2026-08-08

**This is the second of the ticket's three cases and then some: the spike probed something the landing made obsolete, and the question underneath it is now answerable positively.** It is not spent. [`docs/operation-extensions.md`](../docs/operation-extensions.md) names it as the artifact that would upgrade the physical-provider row — `docs/operation-extensions.md "The spike is the artifact that would upgrade it"` — because the landed evidence lives inside the defining package. So the repair is a re-run that flips the recorded answer, not a retirement and not a regenerated golden.

**The spike now answers yes, with a boundary.** `spikes/extensions/forkless-physical-provider/results/2026-08-08-macos-arm64.json` records the run: `cargo nextest run --workspace` from that directory, `8 tests run: 8 passed`. A provider crate in a separate workspace implements `PhysicalImplementationProvider` against the public surface, installs through `CompileRequest::with_physical_providers`, has each body re-verified by the host, and is retained as an additional plan alternative naming it as its authority — one alternative under the governed environment, two with the provider installed, both `acme::simdgroup-pointwise-metal@4` and `tiler::prototype-serial-sum-physical@1` named. Its bodies emit through stock `tiler-metal` unchanged.

**What the retained goldens pin now.** The two fixtures whose absences ended were retired rather than regenerated; a golden re-pointed at whatever the compiler now says would pin nothing anyone chose. Five fixtures pin the subjects the host reserves — the verified request, cost-model attribution, the region subject's members, the enumeration entry point, and the three unproposable bodies. Four of the five are also `compile_fail` doctests carrying their exact error code in `crates/tiler-compiler/src/physical_provider.rs`; the fifth, the proposal-body restriction, is stated in prose on `ImplementationProposal::scheduled_kernel` and this fixture is the only check over it. Each was shown red by a separate perturbation of `tiler-compiler` in an out-of-repository copy, recorded in the results file, with the three `pass/` contrasts staying green in every run.

**One absence remains, narrower than the two retired.** `Compilation::offered_providers` is still lowering-only, so an installed provider that reached no retained plan is invisible in the offered half while the selected half names it. Measured as `offered-provider-set-is-still-lowering-only` in the results file. That is [`disclose-the-physical-provider-environment-a-compilation-was-offered`](disclose-the-physical-provider-environment-a-compilation-was-offered.md)'s and [`disclose-offered-and-selected-physical-provider-sets-separately`](disclose-offered-and-selected-physical-provider-sets-separately.md)'s subject; both are `todo` and both carry stale line citations, and the second's outcome is already half-satisfied by the landed selected accessor. Neither is edited here.

**Not done here, and out of scope.** ADR 0090's dated correction still says its present-tense evidence sentence no longer reproduces; replacing it with this re-run is `docs/decisions/**` work this ticket may not touch. The `crates/tiler-compiler` doc comment that calls the lowering set "the complete frozen provider set offered to this compilation" is still there and its consequence has now met its trigger condition. Both are reported rather than reached for.

## Closes when

The spike builds at a recorded commit, its `README.md` and `results/` state what the landed seam does and does not admit, every retained compile-fail golden is reproduced or replaced with the boundary that actually holds, and ADR 0090's dated correction is updated to cite the re-run rather than the absence.

## Graph maintenance

- Do not delete a compile-fail fixture whose boundary still holds; four bypasses are pinned as `compile_fail` doctests in `crates/tiler-compiler/src/physical_provider.rs` and the spike should state the same boundary from outside the tree rather than a different one.
- If the spike finds the out-of-tree path blocked by something the integration fixture cannot see, that is a defect in the landed surface and belongs in its own ticket rather than in a spike note.
- The remaining ADR 0090 correction needs a carrier ticket holding `contracts/decisions`; this ticket holds `research/extensions` and `project/tickets` only.
