---
id: record-the-accessmode-total-mapping-site-under-convention-5b
title: Record the AccessMode total-mapping site under ADR 0074 convention 5b
status: done
priority: p3
dependencies: []
related: [implement-boundary-property-model, harden-public-enums-non-exhaustive]
scopes: [contracts/decisions, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [contract, adr, api-conventions]
---
ADR 0074 convention 5b governs public enums that are exhaustively matched from outside their defining crate, and enumerates the sites where that holds. `implement-boundary-property-model` created a new one and could not record it, holding only `implementation/compiler`.

**Fact.** `tiler_ir::schedule::AccessMode` is now mapped totally onto identity
tags at two out-of-crate sites:
`crates/tiler-compiler/src/selection.rs::access_mode_tag` and
`crates/tiler-compiler/src/frontier.rs::access_mode_tag`.

**Fact.** It carries no `#[non_exhaustive]`, and under convention 5b it must not gain one: a total mapping from outside the crate is exactly what `#[non_exhaustive]` would break, and the mapping is deliberate — an identity encoding must be exhaustive with no wildcard arm, so that adding a variant is a compile error at every encoder rather than a silently mis-encoded subject.

**Inference.** This is the *reason* convention 5b exists rather than an exception to it, so the correct action is to add the site to the ADR's enumeration, not to reconsider the enum's attributes.

**Check before writing.** Confirm both call sites still exist and are still total, and confirm the ADR's enumeration is a normative list rather than an illustrative one — if it is illustrative, adding a site is not sufficient and the ADR needs to say how a reader finds the complete set. `AGENTS.md`: a failed search is evidence the search was wrong until the file has been read, and a doc comment that overstates what is enumerated makes unreachable work look reachable.

**Related but distinct:** `harden-public-enums-non-exhaustive` decides which growth-expecting public enums *gain* `#[non_exhaustive]`. `AccessMode` is a case that must not, so the two tickets must not contradict each other; whichever lands second should check the first.

## Closes when

The site is enumerated in ADR 0074, the must-not-gain-`#[non_exhaustive]` reason is stated where a future editor will see it, any catalog block quoting ADR 0074 is updated by hand in the same change, and `make full` passes.

## Outcome — enumerated, with the reason verified rather than asserted (2026-07-27)

**Both call sites confirmed present and total.** `access_mode_tag` at `crates/tiler-compiler/src/selection.rs:1640` and `crates/tiler-compiler/src/frontier.rs:1425` each match `Read => 1` and `Write => 2` with no wildcard. `AccessMode` at `crates/tiler-ir/src/schedule/model.rs` carries no `#[non_exhaustive]`.

**The enumeration is illustrative, not normative, and the ADR already said so.** The check this ticket demanded returns the answer it warned about: the paragraph above the new entry states outright that "the enumeration above is a snapshot", and the clause's closing test makes the classification "a property of the consumers that exist", with the ticket adding a consumer owning the re-check. So adding a site is *not* sufficient on its own, exactly as this ticket anticipated. The ADR now says so in its own voice: a new paragraph records that nothing in the repository enumerates 5b sites, that no command produces the complete set, and that the list is worked examples plus evidence the convention is live — while pointing a reader at the decidable per-type test, which is what they can actually run.

**The must-not reason lives at the definition, and it is now measured.** The doc comment on `AccessMode` states why the attribute must not be added. Rather than assert that, the attribute was added and the build watched: `cargo check -p tiler-compiler` fails with two `E0004`s naming `frontier.rs:1426` and `selection.rs:1641`, each reporting that `AccessMode` "is marked as non-exhaustive, so a wildcard `_` is necessary" and offering `_ => todo!()`. That wildcard is precisely the failure the comment describes — an arm that would have to invent an identity tag the variant alone determines. The probe was reverted and the crate re-checked clean.

**One process note worth keeping.** The first attempt at that probe silently did not modify the file, and `cargo check` passed — which read as "the attribute is harmless" and would have refuted the comment I had just written. The edit was only trusted once it printed whether the attribute was actually present. A verification whose subject was never applied is indistinguishable from a verification that passed.

**No contradiction with `harden-public-enums-non-exhaustive`.** That ticket is `done` and classified `AppleSdk`, `OptimizationLevel`, `ArtifactProvenance`, `CompiledArtifact`, and `IndexExprClass`. It never touched `AccessMode`, so nothing there says this type should gain the attribute.

**Catalogs.** `docs/decisions/README.md` carries two lines for ADR 0074, both title-and-status only; neither moved, so no catalog block needed editing. The ADR keeps `decision_status: accepted` and its rationale.
