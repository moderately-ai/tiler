---
id: disclose-offered-and-selected-physical-provider-sets-separately
title: Disclose offered and selected physical-provider sets separately
status: done
priority: p1
dependencies: [drive-an-external-physical-implementation-provider-through-compilation]
related: [accept-the-public-backend-provider-composition-boundary]
scopes: [implementation/compiler, research/extensions]
shared_scopes: [contracts/decisions, project/tickets]
paths: []
tags: [implementation, compiler-api, backend-providers, explainability, public-boundary]
---
## User-visible outcome

Explain output distinguishes which providers were *offered* a compilation from which were *selected* for it, and a physical-implementation provider's identity can appear in both — so a caller can see that its installed provider was consulted and lost, rather than reading an empty set and being unable to tell that from never having been asked.

**Read "explain output" as the compilation's disclosure surface, not the sealed explain trace, and the reading is forced rather than chosen.** ADR 0090 item 5's own words are "*A compilation reports* the complete frozen physical-provider environment it was offered, and the providers its retained plan selected, as two sets", and [the composition record](../docs/research/extensions/backend-provider-composition.md)'s sketch of item 5 is two `impl` blocks of accessors. The half that had already landed when this ticket was dispatched, `PlanAlternative::selected_physical_providers`, is an accessor too. The sealed trace's provider vocabulary is per-record — `ProviderRef` in `crates/tiler-compiler/src/explain.rs` references the provider that lowered *one occurrence* — and carries no compilation-wide environment section for an offered set to be a row of, so siting the offered half there would have meant inventing one and leaving the two halves in different surfaces.

## Why this exists

**Fact — the offered set is populated from the lowering registry alone.** `crates/tiler-compiler/src/session.rs:1513` constructs `offered_providers: Arc<[ProviderIdentity]>` from the lowering capability registry and passes it to `into_compilation_batch` (`:1520`); it reaches the compilation through `:1841` and is read back through the accessor at `:761`. No physical-implementation provider contributes to it, so no physical provider's identity can appear in explain output at all.

**Fact — this is item 5 of an accepted ADR, and it is the one item with no ticket.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md):19 records, in the accepted status paragraph, that "the item-2 physical-provider registry and the item-5 disclosure accessors remain unimplemented", and `:143` states that implementation follows item by item. Item 2 has [`drive-an-external-physical-implementation-provider-through-compilation`](drive-an-external-physical-implementation-provider-through-compilation.md) and item 11's orchestration promotion landed on 2026-08-01; item 5 had nothing.

**Inference — the separation is a correctness property of explain, not a convenience.** AGENTS.md concentrates correctness scrutiny on "explain output for accepted and rejected rewrites, candidates, guards, capabilities, and assumptions". Collapsing offered into selected makes a rejected candidate indistinguishable from an absent one, which is precisely the distinction explainability exists to preserve.

## Per-Fact audit at base `aae3da245c79314b09f442342b22b8458b8558e1`

Every Fact above was re-read at this base before any edit. **All six line citations across the two Facts are stale, and one Fact is materially false in its second half.** Nothing above is deleted, so a grep for a retired string lands in this section.

| Claim | Verdict | Evidence, read at this base |
| --- | --- | --- |
| "`session.rs:1513` constructs `offered_providers`" | **imprecise** — claim true, citation stale | The population is `session.rs:2369-2370`. `grep -n "capabilities.0.lowering().providers()" crates/tiler-compiler/src/session.rs` returns one line, which is the anchor to use; `:1513` lands inside an unrelated `}`. |
| "passes it to `into_compilation_batch` (`:1520`)" | **imprecise** — claim true, citation stale | The call is `session.rs:2376`; `:1520` lands on a doc comment about a dimension refusal. |
| "it reaches the compilation through `:1841`" | **imprecise** — claim true, citation stale | The field is set at `session.rs:2767`; `:1841` lands on the heading `# One arithmetic type`. |
| "read back through the accessor at `:761`" | **imprecise** — claim true, citation stale | The accessor is `session.rs:841`. `grep -n "pub fn offered_providers(" crates/tiler-compiler/src/session.rs` returns one line; `:761` lands on a closing brace. |
| "No physical-implementation provider contributes to it" | **verified** | The sole population is `Arc::from(capabilities.0.lowering().providers())`. |
| "so no physical provider's identity can appear in explain output at all" | **FALSE at this base** | `crates/tiler-compiler/src/session.rs "pub fn selected_physical_providers"` returns the providers a retained plan chose, and `crates/tiler-compiler/tests/external_physical_provider.rs` reads an installed provider's identity back out of one from outside the crate. The *selected* half landed with the installed-provider seam on 2026-08-08 under [`drive-an-external-physical-implementation-provider-through-compilation`](drive-an-external-physical-implementation-provider-through-compilation.md). Only the **offered** half was missing, and that is what this ticket does. |
| ADR 0090:19 quotes "…remain unimplemented" | **verified as a string, false as a claim** | The string is present at line 19 and the citation resolves. But lines 21–23 are a dated 2026-08-08 correction recording it as *half* false, so a reader taking line 19 at face value gets a claim the same file already retires four lines later. |
| "`:143` states that implementation follows item by item" | **imprecise** — claim true, citation stale | Line 143 is blank. The anchor is `docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md "This record decided; implementation follows item by item"`, at line 158 before this landing's edits. |
| "it is the one item with no ticket" | **FALSE at this base** | Item 5 has two: this one, and [`disclose-the-physical-provider-environment-a-compilation-was-offered`](disclose-the-physical-provider-environment-a-compilation-was-offered.md), which owns `offered_providers`'s own documented-versus-actual gap. The claim was true when written and had already been overtaken. |

**What this changed about the ticket, stated because a repair that changes what a ticket is *for* has to stop.** It does not. The outcome is unaltered — the two sets are disclosed separately, and the distinction is preserved rather than collapsed. What moved is the remaining work: half of it had already landed, so this ticket delivered the **offered** half only. The two Facts were written before the selected half existed and neither was ever an argument for building it here.

**A defect found while auditing, in a doc comment rather than in a ticket.** `CompileRequest::with_physical_providers` read "It may then lose on cost, which [`PlanAlternative::selected_physical_providers`] is what tells apart from never having been asked." That is the conflation this ticket exists to prevent, asserted as though already discharged: the selected half alone gives a provider that lost and a provider that was never installed the same reading. Corrected in the same commit to name the *pair* of accessors, which is what actually draws the distinction.

## Public boundary — draft, do not self-accept

The accessors are a public boundary. ADR 0090:19 states it in terms: "every concrete public surface named here — the provider registry and its installation method, **the offered-versus-selected disclosure accessors**, the promoted `assemble_artifact` boundary — still comes to Tom at implementation time under [ADR 0075]". [`accept-the-public-backend-provider-composition-boundary`](accept-the-public-backend-provider-composition-boundary.md) is `done` and routed exactly these to him. Land a reviewed draft with an out-of-crate fixture exercising it, and put the exact accessor shapes, their ownership, and their naming to Tom before acceptance.

## Closes when

An installed physical-implementation provider that was consulted and not selected is visible as offered-and-not-selected in explain output from an out-of-crate caller; a provider never installed is distinguishable from one installed and rejected, with a check observed failing when the two are conflated; ADR 0090's status paragraph is corrected to stop naming item 5 as unimplemented; and the accessor shapes have gone to Tom.

**Worker report, 2026-08-08, against each of the four.** Closure is the coordinator's; this records what was done, not that it is enough.

1. **Done.** `crates/tiler-compiler/tests/external_physical_provider.rs "fn a_consulted_provider_that_won_nothing_is_not_reported_as_never_installed"` compiles one program twice under environments differing *only* in whether a silent provider is installed, and classifies it `NeverInstalled` against `OfferedAndNotSelected`. A third provider that does win is the positive control, without which "not selected" would be satisfied by a seam selecting nothing.
2. **Done, with the failure observed rather than asserted.** Removing the installed identities from the offered environment makes that test fail with `left: NeverInstalled` against `right: OfferedAndNotSelected` — the conflation itself, in the check's own words. Removing the *governed* identity instead leaves that test green and reddens four others, which is what shows the two properties are guarded separately rather than by one assertion.
3. **Done, and it needed more than the status paragraph.** Line 19's clause was already carrying a dated 2026-08-08 correction from [`record-the-landed-physical-provider-seam-in-adrs-0078-and-0090`](record-the-landed-physical-provider-seam-in-adrs-0078-and-0090.md) that recorded item 5 as *half* landed. Four sites in ADR 0090 asserted the offered half was open — the status correction, the four-obligation Fact, item 5's own correction, and the implementation boundary — and all four are corrected in this landing's commit, each quoting its retired string so a grep lands inside the correction.
4. **Routed, not decided.** The accessor is added to [`accept-the-installed-physical-provider-public-surface`](accept-the-installed-physical-provider-public-surface.md)'s included set as an **additive** change, with `InstalledPhysicalProviders::offered_identities` excluded by stated reason, and a fourth question added for Tom: whether `offered_providers` should be renamed `offered_lowering_providers` for symmetry, which is **breaking** and therefore not an implementing agent's to do.

## Outcome and current boundary — 2026-08-09

Commit `788b0c03` delivered the missing offered half as
`Compilation::offered_physical_providers`, populated from the same complete
`InstalledPhysicalProviders` environment the frontier receives. It remains a
different subject from the accepted lowering-only `offered_providers`; selected
providers remain per-alternative through `selected_physical_providers`. The
out-of-crate test distinguishes never installed, offered and not selected, and
selected, and the count-neutral conflation perturbation failed with
`NeverInstalled` versus `OfferedAndNotSelected`. ADR 0090 and the composition
record were corrected in the same landing. Commit `059fedbd` closed this ticket.

The public accessor is implemented and still a labelled draft. Its acceptance
and the possible breaking rename remain correctly parked at
`accept-the-installed-physical-provider-public-surface`, which is currently
`awaiting-decision`; this completed implementation ticket does not imply that
Tom accepted either spelling.
