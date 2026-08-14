---
id: publish-occurrence-bound-selected-physical-implementation-evidence
title: Publish occurrence-bound selected physical implementation evidence
status: in-progress
priority: p1
dependencies: [disclose-the-physical-provider-environment-a-compilation-was-offered, accept-the-installed-physical-provider-public-surface]
related: [accept-the-installed-physical-provider-public-surface, disclose-offered-and-selected-physical-provider-sets-separately, carry-complete-access-alignment-requirements-on-physical-proposals]
scopes: [implementation/compiler, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, provenance, identity, public-boundary]
claimed_from: todo
assignee: worker-selected-physical-evidence
lease_expires_at: 1786723107
---
## User-visible outcome

A neutral artifact assembler can forward the compiler's exact selected physical authority for every covered region without reconstructing private selection state or collapsing a mixed plan to a provider set.

## Required delivery

- Add one compiler-minted, borrowed or owned projection for every selected cover-region implementation, in canonical region-occurrence order.
- Carry the canonical occurrence binding, the exact `ImplementationProposalIdentity`, the readable `ProviderIdentity`, and the closed proposal-kind code. Do not expose body internals, cost, rejected alternatives, the offered provider environment, or provider installation order.
- Derive every field from the retained `RegionSelection` / `AdmittedImplementation`; callers and physical providers must not construct or replace the authority.
- Keep the projection occurrence-bound. A deduplicated provider set, provider-plus-kind set, or order-only list is insufficient for a plan that mixes providers or selects one provider more than once.
- Add a subject perturbation showing that changing only provider authority or only the occurrence association changes the projected evidence while the structural body fixture remains unchanged.
- Record the exact public included/excluded surface under ADR 0075 and update the artifact contract language that consumes it.

## Non-goals

Packaging the artifact, exposing private implementation bodies, serializing offered providers, changing provider installation precedence, or changing selection policy.

## Closes when

The build layer can consume complete compiler-owned selected physical evidence without re-derivation, its population and ordering are pinned, and independent review confirms no private selection authority was widened beyond the four required subjects.

## Exact-base Fact audit — 2026-08-14

Audited at `506b700b6d71461fca981c852575aa20d344e78f` before editing. Files read in full were `AGENTS.md`, this ticket, both dependency tickets, ADRs 0075 and 0090, `docs/artifact-abi.md`, `crates/tiler-compiler/src/physical_provider.rs`, and `crates/tiler-compiler/tests/external_physical_provider.rs`. The relevant construction and consumption paths were also read at the anchors named below in `session.rs`, `selection.rs`, `region.rs`, `frontier.rs`, `physical.rs`, `pipeline/planning.rs`, and `lib.rs`.

- **Verified — existing public traversal.** `PlanAlternative::selected_physical_providers` already returns borrowed `SelectedImplementation` views over `SelectedPlan::selections`. `assemble_plan`, at `ordered.sort_by`, establishes canonical whole-occurrence-byte order before any public view exists.
- **Verified — retained occurrence authority.** `RegionSelection` retains the private `RegionOccurrenceIdentity` beside its exact `AdmittedImplementation`; `RegionOccurrenceIdentity` states that it is region content plus graph-local members, boundary inputs, and retained outputs.
- **Verified — retained proposal authority.** `AdmittedImplementation::identity` returns the private `ImplementationProposalIdentity` minted by `encode_proposal_identity` from the verified body subject, host-stamped provider, closed proposal kind, applicability, derived boundary, and deferred feasibility. Cost and proposal enumeration order are absent.
- **Verified — readable provenance already existed.** `SelectedImplementation::provider`, `provider_explain_subject`, and `proposal_kind` read the retained admission; no provider-supplied identity or label is trusted to reconstruct it.
- **Verified — construction stays private.** `selection` remains a private module and `SelectedImplementation` remains a public tuple struct with a private field and no production constructor or mutator.
- **Corrected — the separate identity fields do not prove a public admitted neighbour exists.** The audit initially treated the separately framed occurrence and proposal inputs in `encode_plan_identity` as also implying that the current public compilation population could hold an equal proposal identity at two distinct occurrences. That implication was imprecise. Separate framing proves two independent identity fields; it does not prove a population inhabitant. Runtime probes found no constructible public positive in the bounded operation set: two separately published identical branches refuse at `output-partition-overlap`; combining two identical post-reduction branches refuses at `operation-set`; and both an unused-prefix operation and a renamed semantic interface compile but move the proposal identity too. This correction does not change the ticket's purpose. The implementation evidence below uses a `cfg(test)` crate-private perturbation of a genuinely admitted selection and does not represent it as a public admitted neighbour.
- **Verified — artifact packaging remains downstream.** `docs/artifact-abi.md`, at `a selected capability provider row carries two independent revisions`, describes semantic capability-provider provenance, not occurrence-bound physical selection. No current manifest row or artifact builder consumes the four physical subjects.

No ticket Fact was false in a way that changed the requested outcome. The one imprecise inference above is repaired here rather than silently replaced.

## Public boundary record — ADR 0075

Accepted authority is the dependency record `Accepted subject boundary 2026-08-11` in `disclose-the-physical-provider-environment-a-compilation-was-offered` together with the current `tiler_compiler::session` surface accepted by `accept-the-installed-physical-provider-public-surface`.

**Included, exactly:** the existing borrowed `PlanAlternative::selected_physical_providers` traversal in canonical occurrence order; the whole opaque canonical region-occurrence identity bytes; the whole opaque compiler-minted implementation-proposal identity bytes; the existing readable `ProviderIdentity`; and the existing closed proposal-kind code. These are four subjects per retained selection: occurrence, proposal, provider, and kind. Only the compiler constructs the borrowed view.

**Excluded, exactly:** a new public type, constructor, mutator, owned selection record, identity parser, presentation label substituted for canonical bytes, proposal body or constructor, structural cost, rejected alternatives, offered-provider environment, installation order, selection-policy change, and any artifact manifest row, schema/version step, serialization, or packaging implementation.

## Implementation evidence — 2026-08-14

- `SelectedImplementation::region_occurrence_identity` and `SelectedImplementation::implementation_proposal_identity` return borrowed whole canonical byte runs directly from the retained checked selection. They do not copy, label, parse, or reconstruct either identity.
- `selected_implementation_evidence_preserves_population_order_and_multiplicity` observes a real multi-region public plan, requires strict whole-occurrence-byte order, proves every row carries both identities, and proves repeated selection of one provider remains repeated rows rather than becoming a set.
- `provider_authority_moves_selected_evidence_without_moving_the_body` installs two providers that clone the same verified baseline body with the same specialization and cost. For one shared occurrence and proposal kind, only provider authority and compiler-minted proposal identity move.
- `selected_evidence_tracks_an_occurrence_that_cannot_be_rebound` is the controlled occurrence subject perturbation. A `cfg(test)`-only crate-private seam wraps one genuinely admitted `RegionSelection`; the test clones it, replaces only its retained occurrence, proves the entire `AdmittedImplementation` remains equal, and observes only the occurrence projection move while proposal identity, provider, and kind remain fixed. Independently, swapping only the two occurrence associations in the checked plan is refused by reassembly as `binding` / `member-mismatch`. This is negative tamper evidence, not a claim that a second public admitted neighbour exists.
- The `SelectedImplementation` compile-fail example requires `E0423` when an external caller attempts its private tuple constructor. Production exposes no constructor or replacement path.
- **Provider-authority negative control:** temporarily omitting `encode_provider` from production `encode_proposal_identity` made `provider_authority_moves_selected_evidence_without_moving_the_body` fail with `both equal-body providers should be retained for at least one shared occurrence`; the identity collision collapsed the second provider before selection. The production encoding was restored before the positive gates.
- **Occurrence-projection negative control:** temporarily returning proposal bytes from production `region_occurrence_identity` made `selected_evidence_tracks_an_occurrence_that_cannot_be_rebound` fail with `moving the retained occurrence did not move the projected occurrence subject`. The production accessor was restored before the positive gates.

## Verification — 2026-08-14

- `cargo nextest run -p tiler-compiler` — 928 passed, 1 skipped.
- `cargo test -p tiler-compiler --doc` — 2 ordinary doctests and 12 compile-fail doctests passed, including the `SelectedImplementation` constructor boundary.
- `cargo check -p tiler-compiler --all-targets` — passed.
- `cargo clippy -p tiler-compiler --all-targets -- -D warnings` — passed.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-compiler --no-deps` — passed.
- `cargo fmt --check`, `git diff --check`, and `tkt lint` — passed.
- `make citations` — 1,170 pinned citations and 6,561 local links resolved; zero untracked resolutions.
