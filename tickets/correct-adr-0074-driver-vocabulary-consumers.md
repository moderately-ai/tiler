---
id: correct-adr-0074-driver-vocabulary-consumers
title: Amend ADR 0074 for the driver vocabulary's out-of-crate consumers
status: todo
priority: p2
dependencies: []
related: [harden-public-enums-non-exhaustive, choose-one-owner-for-apple-target-vocabulary, record-an-adr-for-the-metal-aot-crate-admission]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions, api-hardening]
---
Two factual claims inside accepted ADR 0074's convention 5 were falsified by a merge that landed after the convention was amended, and neither has been corrected. The normative rule is fine; the record's account of where it applies is not. Found while writing ADR 0077, whose Consequences had to state the 5b classification for two driver enums that ADR 0074 says have no out-of-crate consumer.

**Fact — the premise sentence is false.** `docs/decisions/0074-use-explicit-public-api-conventions.md` under "### 5" states: "It is also the whole rule for a type that has no consumer outside its defining crate at all, which is why the `tiler-metal-aot` half of `harden-public-enums-non-exhaustive` is untouched by the clauses below." `crates/tiler-metal/src/target_correspondence.rs` maps `tiler_metal_aot::input::{MslVersion, ApplePlatform}` onto `tiler_metal::target::{MslLanguageVersion, MetalPlatform}` totally, from outside the crate that defines them, and `crates/tiler-metal/src/golden_compilation.rs::resolved_toolchain` recognizes `tiler_metal_aot::diagnostic::DriverError` out of crate. So `tiler-metal-aot` types do have out-of-crate consumers, and the `tiler-metal-aot` half of that ticket is emphatically not untouched by clauses 5b and 5c.

**Fact — the site enumeration is incomplete.** The same section states "**Fact — three such sites exist**" and lists `FusionNumericalProof::canonical_explain_evidence_bytes`, `fusion_legality::effect_tag`, and `tiler_metal::emit::realization_requirements`. `crates/tiler-metal/src/target_correspondence.rs` is a fourth 5b site by the record's own definition — every arm must produce the counterpart its variant determines and no wildcard value is derivable — and its own module documentation cites ADR 0074 convention 5b explicitly.

**Fact — the ordering, so the correction is attributable rather than a reproach.** `f6da4c4` amended conventions 4 and 5 on 2026-07-24 at 12:28. `45d9827` added the checked correspondence at 13:53 the same day. The ADR was accurate when written and stopped being accurate 85 minutes later. `fbe0b4f` and `8194c94` touched the ADR afterwards for the measurement probe and a naming resolution, and neither revisited these two claims.

**Fact — the operational risk is already contained; the authority conflict is not.** `tickets/harden-public-enums-non-exhaustive.md` carries a "**Revised again 2026-07-24 after `choose-one-owner-for-apple-target-vocabulary`**" block that removes `MslVersion` from its mark list, records `ApplePlatform` as never having been on it, and adds `DriverError` as a 5c type — stating in terms that "The premise that `tiler-metal-aot`'s types 'have no consumer outside `tiler-metal-aot` at all' did not survive checking". So a worker who reads the ticket gets the right answer and a worker who reads the accepted ADR gets the wrong one. That is the duplicated-authority failure `AGENTS.md`'s documentation contract names, and the ticket cannot fix it because the stale text is in the ADR.

**What closes this.** ADR 0074 already carries an "Amendments" section and was amended once before by `resolve-non-exhaustive-recognizer-hole`, so the mechanism exists and this is a factual correction inside it rather than a change to the normative rule — no clause of convention 5 changes meaning. Correct the premise sentence so it no longer claims `tiler-metal-aot` has no out-of-crate consumers, extend the 5b site enumeration to include `crates/tiler-metal/src/target_correspondence.rs`, and record the correction in "Amendments" with the two commits above so a reader who already applied the earlier text can see what moved. Check the rest of convention 5 for any further claim that depends on the false premise before concluding two sites is the whole of it — read the section in full rather than grepping for the crate name, since the premise is stated once and relied on implicitly.

Do not change `decision_status`, and do not restate the classification reasoning that already lives on `crates/tiler-metal-aot/src/input.rs`, `crates/tiler-metal/src/target.rs`, and `crates/tiler-metal/src/target_correspondence.rs`.
