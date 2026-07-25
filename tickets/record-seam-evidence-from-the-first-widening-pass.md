---
id: record-seam-evidence-from-the-first-widening-pass
title: Seam evidence from the first widening pass, and what it does not yet show
status: done
priority: p1
dependencies: []
related: []
scopes: [project/tickets]
shared_scopes: []
paths: []
tags: [planning, process, identity]
---
The widening method's stated payoff is that an axis needing no change to a seam is evidence the seam was right, and one forcing a redesign is a finding. The first pass produced both. Recorded here rather than left inside four commit messages, because evidence spread across commits is not evidence anyone will find.

## The strongest finding: the compiler-to-artifact identity seam is systematically incomplete

Three independent values were needed at that boundary in one session, and **all three failed the same way**: the producer had a governed key and not the identity that makes the key evidence.

| Value the artifact requires | What the compiler had | Ticket |
| --- | --- | --- |
| `SelectedProvider::capability` | provider and revision; no capability key | `name-the-resolved-lowering-capability` (done) |
| `TargetProfileRef::descriptor` | profile key; no descriptor | `carry-the-target-profile-descriptor-identity-into-the-plan` (done) |
| `FeasibilityRuleSetRef::key` | a rule *version*; no rule name | `split-profile-and-feasibility-rule-identity` (done) |

Three instances of one shape is not three coincidences. The seam was designed so that `tiler-artifact` states what identity it needs and `tiler-compiler` supplies it, and the compiler was built without ever being asked for the second half of any of them — because until an assembler existed, nothing asked. **This is the redesign-forcing finding the method predicts**, and it was invisible for as long as the artifact layer had only synthetic fixtures to satisfy.

What it does *not* show: whether the pattern continues. All three are now closed — the third, `split-profile-and-feasibility-rule-identity`, landed after this paragraph was written — and no check exists that would catch a fourth. Whether that check should exist — a test that every artifact-required identity has a compiler-side source — is worth deciding rather than assuming; it is the difference between having fixed three bugs and having closed the class.

## A second finding, from a different axis, of the same kind

`decide-whether-storage-encoding-is-a-missing-boundary-property` found the optimizer contract's boundary-property list incomplete: ADR 0047 and the transfer taxonomy already accepted `RepackEncoding` as an enforcer, and the property that enforcer supplies was never named. Again a seam where one side had been extended and the other not, and again found only because something asked the question from outside.

The two findings are independent — one is compiler/artifact, one is optimizer/transfers — which is the axis-independence the method assumes, observed rather than asserted.

## The positive evidence, and why it is weak

The shapes authority consumed `AvailabilityPhase` from `program::abi` and the canonical framing from `tiler_ir::identity` **without either needing a change**. Both were built for other consumers — ADR 0043's feasibility ladder and digest framing — and a genuinely new consumer used them as-is.

That is real but weak evidence, and it should not be cited as more. Both were written in the same session, by the same author, with those seams in mind; the method's own standard is that "a narrow choice only proves itself when something it did not anticipate passes through unchanged", and an author who knows the seam is not that something. The evidence gets strong when a consumer written without knowledge of the phase ladder needs it — the index-bindings work is the first candidate.

## What the pass does not show

No axis has been widened far enough to stress a seam under load. One authority half, one contract amendment, two decisions, and two glossary rows are early work. In particular nothing yet exercises: a second dtype through emission, a symbolic extent through a guard, a second target family through feasibility, or the public boundary through reuse. The findings above came from *asking* at seams, not from *running* across them, and those are different strengths of evidence.

## Closes when

The identity-completeness question above is decided — either a check exists that every artifact-required identity has a compiler-side source, or the reason one is not warranted is stated — and the positive seam evidence is re-taken against a consumer that did not anticipate the seam.

## Decision — no mechanical check, and the reason is specific rather than a decision to skip it

The question was framed as "a test that every artifact-required identity has a compiler-side source". Reading the three fixed sites in full shows that test cannot be written, because **the three defects share a shape but not a signature**.

**Fact — `crates/tiler-artifact/src/program/keys.rs:147-176`.** The `opaque_identity!` macro gives every identity a `pub fn from_bytes(impl AsRef<[u8]>)` validating only non-emptiness and a length bound, over a doc comment stating "the bytes are treated as opaque: this crate compares and encodes them, and never re-derives them locally." So a type-level provenance constraint is **not available**, and not for a fixable reason: `tiler-artifact` is deliberately ignorant of how these bytes are minted, which is what keeps it consumer-agnostic. Making the type prove provenance would require the artifact crate to know its producer, inverting the dependency direction the architecture requires.

**Fact — `crates/tiler-compiler/src/feasibility.rs:88-132`.** The `FeasibilityRuleSetRef` fix did not replace a constant with a derivation. It replaced `PROTOTYPE_FEASIBILITY_RULE_VERSION` with `GOVERNED_FEASIBILITY_RULE_SET_KEY` and `GOVERNED_FEASIBILITY_RULE_SET_REVISION`, which are **still constants**, and deliberately so: "a `const` rather than a per-target derivation, because the rules are this module's code and do not vary by target: exposing a `fn(target) -> rules` would imply a variation that cannot exist and would invite a second definition of one identity."

**Inference — a scanner keyed on "an artifact-required identity populated from a constant" would fire on the corrected code and pass the broken code's successor.** It would flag `GOVERNED_FEASIBILITY_RULE_SET`, which is right, and it would have no complaint about any future placeholder that happened to be spelled as a function call. That is worse than no check, because a check that fires on correct code trains its readers to suppress it.

**Inference — what actually distinguishes the fixed state from the broken one is prose.** `PROTOTYPE_FEASIBILITY_RULE_VERSION` had no stated authority; `GOVERNED_FEASIBILITY_RULE_SET_REVISION` carries twelve lines saying exactly what mints a new key, what bumps the revision, and what deliberately does neither — "not bumped when a target profile's declared bounds or honourability change: those are the profile's claims, and the profile's canonical descriptor already distinguishes them." A named minting authority with a stated bump discipline is the property that closes the defect, and no gate can check whether a doc comment states one.

**This is the opposite of the `crate::identity` length-framing case, and the contrast is the point.** Length framing had a syntactic signature — `.len() as u64`, `fn push_len(` — so when prose failed to hold the convention, a scanner could. Identity provenance has no such signature. Reaching for the same instrument because it worked once would be pattern-matching on the remedy instead of on the defect.

### What does hold the class, at its true strength

**Fact — both `prototypes/serial-sum-compile` and `prototypes/serial-sum-run` are root workspace members** (`Cargo.toml:11-12`), so the gate compiles the assembler under strict Clippy. **Measurement, 2026-07-25:** the gate rejected `prototypes/serial-sum-run/src/main.rs:1458` with `struct SelectedProvider has no field named capability_api_version` when two independently-green branches were merged. The seam-shape half of the class is genuinely gated, and it demonstrably fires.

**Inference — the assembler is the instrument that found all three.** They were invisible "for as long as the artifact layer had only synthetic fixtures to satisfy", and a real producer cannot fill a field it has no source for. That is a structural check, not a mechanical one, and it now runs on every gate.

**Stated boundary.** The assembler covers the fields on the path it exercises. A seam identity introduced for an unexercised path is unchecked, and a placeholder that compiles is unchecked. The class is narrowed, not closed, and this ticket does not claim otherwise.

**Trigger for reconsideration.** A fourth instance found *off* the assembler path would falsify the structural argument, and the correct response then is a pinned-admission table in `scripts/check_workspace.py` mirroring `pin-the-admitted-unsafe-sites-in-the-workspace-gate` — every admitted placeholder listed with its reason, so a fifth is a diff someone must look at. That is the right instrument for "review is the enforcement and the predicate is mechanical", and it is the wrong one today because the predicate is not yet mechanical.

## The positive evidence, re-taken

The ticket asked for evidence from "a consumer written without knowledge of the phase ladder". Two arrived, and they are worth separating because one is much stronger than the other.

**Weaker — `implement-shapeenv-index-bindings` used the ladder beyond its written scope and it held.** The accepted corpus states the host-evaluability rule for *semantic* extents and nowhere for index domains; the index work inferred that an index-domain extent is upstream of launch geometry, so the same rule binds it, making `LiveDevicePreflight` the last admissible phase. A consumer extending a seam to a case its author never wrote down, without the seam needing a change, is better evidence than the original pass. It is still not clean: it was written in the same programme, by a worker briefed by the same coordinator.

**Stronger, and from a direction nobody chose — `widen-the-apple-numerical-probe-to-a-second-dtype` refuted a universal.** The section below listed "a second dtype through emission" as something no axis had yet exercised. It has now been, and it did not confirm the seam — it broke an assumption the design was carrying: `f16` arithmetic **preserves** the subnormals `f32` flushes, on the same GPU, in the same math modes, from modules declaring `air.compile.denorms_disable` identically. The argument that a module-level declaration makes the flush dtype-independent was an Inference, correctly labelled, and is now measured false.

**This is the method working in the mode that actually pays.** A widening axis that confirms a seam yields weak evidence, because the author's own expectations are doing most of the work. A widening axis that *refutes* something yields strong evidence regardless of who wrote it, because the measurement does not care what the designer expected. The first pass produced confirmations from authors who knew the seams; this one produced a refutation from a probe that could not have been talked into it.

**And it names its own limit.** It is a Measurement on one bounded row refuting a universal — the one direction a single counterexample settles. It establishes that the flush depends on the dtype and establishes nothing about which dtypes flush. `bfloat` is the discriminating case, since its subnormals are not `f32` normals, and it is being measured now.

## Outcome

**Closed on both conditions.** The identity-completeness question is decided against a mechanical check with the reason stated above and a trigger recorded; the positive seam evidence has been re-taken and one instance of it is a refutation rather than a confirmation.

Two items in "What the pass does not show" are now spent — "a second dtype through emission" and "a symbolic extent through a guard" — and two are not: a second target family through feasibility, and the public boundary through reuse. The second of those is review-gated, not engineering-gated, and `record-that-the-frontend-axis-is-review-gated` carries it.
