---
id: correct-the-runtime-route-requirement-relation-prose
title: Correct the floor and capacity-comparison relation prose inside tiler-runtime
status: todo
priority: p2
dependencies: []
related: [correct-the-residual-floor-relation-prose-outside-the-artifact-scopes, correct-the-subgroup-threads-route-dimension-meaning, rename-the-route-resource-floor-vocabulary-for-its-corrected-relation]
scopes: [implementation/runtime, contracts/decisions, research/extensions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, runtime, naming]
---
The relation-vocabulary sweep that produced [`correct-the-residual-floor-relation-prose-outside-the-artifact-scopes`](correct-the-residual-floor-relation-prose-outside-the-artifact-scopes.md) reached only that ticket's three documentation scopes. Two sentences in `tiler-runtime`'s own documentation still describe the live-device route-requirement relation as a floor, and a third framing — "a capacity comparison" — is now imprecise on both sides of the code/docs boundary. The type-name sweep `rename-the-route-resource-floor-vocabulary-for-its-corrected-relation` ran (`ResourceFloor`, `route_floor`, `PROBE_FLOOR`, …) could not have caught these: they name the relation in prose, not the type.

**Fact — two runtime doc comments still assert a floor.** `crates/tiler-runtime/src/load/route.rs:351` documents `LiveDeviceObservation::Quantity` as "Valid only for a quantitative floor. Answering it for a backend feature row is refused rather than coerced." `crates/tiler-runtime/src/adapter.rs:39` reads "would make `Unrecognized`, a wrong-shaped answer, and an unmet floor one outcome instead of three." Neither is true of the landed vocabulary: `crates/tiler-artifact/src/program/requirement.rs` carries a `required` quantity and `is_satisfied_by` reads `RouteResourceDimension::SubgroupThreads => self.required == observed`, so the row is an exact requirement and the refusal `LoadRejection::UnsatisfiedRouteRequirement` names is the row's own relation failing rather than a bound being undershot. Reproduce with `grep -rn "floor" crates/tiler-runtime/src/`.

**Fact — "a capacity comparison" is the same imprecision under a different word, and it spans four sites in two scopes.** `crates/tiler-runtime/src/load/route.rs:341-342` ("an adapter cannot reverse a capacity comparison on its way to an answer"), `crates/tiler-runtime/src/adapter.rs:38`, `docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md:69`, and `docs/research/extensions/backend-provider-composition.md:70` all use it. The sentence's point — that the *loader* performs the comparison and an adapter only reports — is correct and must survive; what is wrong is "capacity", which implies an ordering in which more is better. Under an equality relation a wider device fails exactly as a narrower one does, which is the whole content of the correction `correct-the-subgroup-threads-route-dimension-meaning` landed at `77c36d5`.

**Why this was filed rather than absorbed.** The floor-prose ticket held `contracts/decisions`, `research/scheduling`, and `research/extensions`. `crates/tiler-runtime/**` is `implementation/runtime`, which it did not hold, and correcting only the two documentation halves of the "capacity comparison" phrasing would have forked them from the code comment that states the same thing — the failure the paired-sentence discipline exists to prevent. So that ticket corrected only the floor assertions its scopes could reach in full and left this one whole.

## What closes this

No sentence in `crates/tiler-runtime/` describes a route resource requirement as a floor or as a capacity comparison; the four "capacity comparison" sites move together so code and documentation still say the same thing; `LiveDeviceObservation::Quantity`'s doc states what it actually is (a measured quantity for a quantitative row, valid only for `RouteRequirement::Resource`); the three-way split of `Unowned`, `Misanswered`, and `Unsatisfied` is preserved with its reason intact; and the two documentation edits carry a dated marker naming this ticket, in the shape the floor-prose ticket used. No behaviour changes and no public item is renamed — this is doc-comment text and prose only, so `cargo test -p tiler-runtime --doc` and a per-package Clippy run are the checks that can fail.
