---
id: record-seam-evidence-from-the-first-widening-pass
title: Seam evidence from the first widening pass, and what it does not yet show
status: todo
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
| `FeasibilityRuleSetRef::key` | a rule *version*; no rule name | `split-profile-and-feasibility-rule-identity` (open) |

Three instances of one shape is not three coincidences. The seam was designed so that `tiler-artifact` states what identity it needs and `tiler-compiler` supplies it, and the compiler was built without ever being asked for the second half of any of them — because until an assembler existed, nothing asked. **This is the redesign-forcing finding the method predicts**, and it was invisible for as long as the artifact layer had only synthetic fixtures to satisfy.

What it does *not* show: whether the pattern continues. Two of three are closed and the third is open, and no check exists that would catch a fourth. Whether that check should exist — a test that every artifact-required identity has a compiler-side source — is worth deciding rather than assuming; it is the difference between having fixed three bugs and having closed the class.

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
