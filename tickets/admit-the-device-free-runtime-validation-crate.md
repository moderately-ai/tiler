---
id: admit-the-device-free-runtime-validation-crate
title: Admit the device-free runtime validation crate
status: done
priority: p0
dependencies: []
related: [prototype-runtime-artifact-validation, record-an-adr-for-the-metal-aot-crate-admission]
scopes: [contracts/foundation, contracts/decisions, implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, workspace, runtime, needs-tom]
---
`prototype-runtime-artifact-validation` says "if the owning production crate is absent, this ticket owns its atomic workspace admission and lockfile update." Attempting that admission established that it cannot be done from `implementation/workspace` alone, and that the accepted contract currently withholds the crate. This ticket owns the part that is not a workspace edit.

## Fact — the path is reserved and the owner is deliberately temporary

`ticketsplease.toml`'s `[scopes]` maps `"implementation/runtime" = ["crates/tiler-runtime/**", "prototypes/serial-sum-run/**"]`, and `[scope_crates]` maps `"implementation/runtime" = "tiler-prototype-run"` under the comment "The runtime mapping is a temporary owner while its production crate is absent … Each crate-admission ticket must atomically add the real workspace package and add or move its mapping here." So the intent to admit a `tiler-runtime` is recorded in the work graph.

## Fact — no contract names the crate, and the accepted profile withholds it

`grep -rn "tiler-runtime" docs tickets spikes crates Cargo.toml prototypes` returns no hit naming a crate; the string exists only in `ticketsplease.toml`'s glob. `docs/architecture.md:352-354` states of the accepted prototype packaging profile: "This is an unstable prototype packaging profile, not the final published crate set. It deliberately omits frontend, proc-macro, Candle, generalized cache, and reusable Metal-runtime crates until the proof reaches those boundaries." The accepted six-library block at `docs/architecture.md:330-342` has no runtime row, and `scripts/check_workspace.py` pins that exact member set.

## Fact — ADR 0077 pre-emptively refuses to be the precedent

`docs/decisions/0077-admit-tiler-metal-aot-as-a-dependency-free-driver.md:80` reads: "No frontend, proc-macro, Candle, generalized cache, or reusable Metal-runtime crate is created for the first proof. **Inference.** `tiler-metal-aot` does not breach this clause and is not an exception to it: it is a build-time compiler driver that never touches a live device, an `MTLDevice`, or a pipeline state, so it is not the reusable Metal-*runtime* crate that clause withholds. A reader must not cite this admission as precedent for admitting one."

## The genuine question, and why it is Tom's

The crate `prototype-runtime-artifact-validation` describes is **device-free** — decoding, integrity and ABI validation, checked expression evaluation, compatibility classification, no `MTLDevice` and no pipeline state. It would depend on `tiler-artifact` and `tiler-ir` and nothing else. By ADR 0077's own stated test for the withheld clause it is not a reusable Metal-*runtime* crate at all, which argues it is admissible today. Against that: `docs/architecture.md`'s "until the proof reaches those boundaries" is not obviously reached — `route-the-runtime-proof-through-the-artifact-envelope` records that the landed `serial-sum-run` proof bypasses the artifact envelope entirely, and `prototype-metal-runtime-execution` is blocked on the `unsafe`/Metal-binding decision — so the evidence a crate admission is supposed to rest on has not been produced by a runtime proof yet.

That is a real architectural choice with valid priorities on both sides, not a detail to settle by implementation.

## Why it could not be landed under `implementation/workspace`

The admission is only coherent if three things move together: the workspace member set and `scripts/check_workspace.py` pins (`implementation/workspace`), the accepted packaging profile in `docs/architecture.md` (`contracts/foundation`), and a superseding decision record (`contracts/decisions`). A worker holding only the first would leave the mechanically checked contract disagreeing with the accepted architecture text — which is precisely the state ADR 0077 was written to end, and which its Consequences describe: "The decision record stops disagreeing with the workspace. Before this, the six-crate profile was written down only in the architecture contract, which said so about itself in the same paragraph." Repeating that pattern knowingly is worse than the first time, because the first time was structural and this would be chosen.

## What closes this

Tom decides whether the device-free validation crate is admitted now or waits for the runtime proof. If admitted: add `crates/tiler-runtime` to `Cargo.toml` members and `[workspace.dependencies]`, add its rows to `scripts/check_workspace.py`'s `EXPECTED_MEMBERS`, `PACKAGE_DESCRIPTIONS`, `PACKAGE_DIRS`, and `EXPECTED_DEPENDENCIES`, move `[scope_crates]`'s `implementation/runtime` mapping off `tiler-prototype-run` onto it, restate the packaging block in `docs/architecture.md`, and record the decision — including whether "device-free" is the operative line that distinguishes it from the withheld Metal-runtime crate. If deferred: say so on `prototype-runtime-artifact-validation` and give that ticket a trigger for reconsideration, because it is currently a p0 whose stated deliverable is unreachable.

## Decision — Tom, 2026-07-25

**Approved: admit the device-free runtime validation crate.** It decodes and validates artifacts and binds by program identity; it never touches a live device, an `MTLDevice`, or a pipeline state.

**Why ADR 0077's non-precedent clause does not bar it.** That clause withholds "reusable Metal-runtime crates until the proof reaches those boundaries". By ADR 0077's own test a crate that touches no device object is not one of those, so this is admitted on the clause's own terms rather than in spite of it. Record that reasoning in the admitting ADR — a future reader must be able to see that the non-precedent clause was applied rather than waived.

**This is the last structural piece of the spine.** `prototype-runtime-artifact-validation` (p0) unblocks, and behind it `route-the-runtime-proof-through-the-artifact-envelope` — which removes the bypass. The runtime proof currently loads a `metallib` directly, so the envelope that round-trips today is still not in the execution path. Two descriptions of one compilation exist and only one is load-bearing; this is what collapses them.

The accepted packaging profile must be amended in the same change, not left to disagree with the workspace.

## Outcome

**Landed.** `crates/tiler-runtime` is a workspace member with the closure `[tiler-artifact]`, and the five things ADR 0081 item 4 requires to move together moved together: `Cargo.toml`'s `members` and `[workspace.dependencies]`; `scripts/check_workspace.py`'s `EXPECTED_MEMBERS`, `EXPECTED_WORKSPACE_DEPENDENCIES`, `PACKAGE_DESCRIPTIONS`, `PACKAGE_DIRS`, and `EXPECTED_DEPENDENCIES`; `docs/architecture.md`'s accepted packaging profile and component-ownership table; `ticketsplease.toml`'s `[scope_crates]` owner; and [ADR 0081](../docs/decisions/0081-admit-tiler-runtime-as-a-device-free-artifact-loader.md).

**ADR 0077's non-precedent clause is recorded as applied, not waived.** ADR 0081 item 3 states the clause's own test — "never touches a live device, an `MTLDevice`, or a pipeline state" — shows the crate meets it structurally (item 2 refuses the dependency that would let it fail), and says in terms that the withheld crate is still withheld and that this record is not precedent for one either. `docs/architecture.md`'s "deliberately omits … reusable Metal-*runtime* crates" sentence keeps that clause verbatim and states beside it why the loader is not one, rather than deleting or narrowing it. ADR 0056's status line records the same thing from the other side.

**Two acceptance edits that ADR 0077 reserved were still outstanding and are landed here.** `accept-adr-0077-metal-aot-crate-admission` is `done` and the record's `decision_status` is `accepted`, but its body still opened "**Status:** proposed. Tom accepts; nothing here is operative until he does", and its Implementation boundary still said "One edit is deliberately withheld until acceptance … Whoever lands the acceptance adds the in-body marker" — the marker on ADR 0056's Decision paragraph had never been added. This is the asymmetry `AGENTS.md` names: a disclosure required while a decision is proposed becomes wrong once it is accepted, and the check enforcing it stops applying at exactly that moment. Fixed rather than worked around, because this ticket cites ADR 0077's clause as operative and could not do so against a record whose body said it was not. ADR 0077's status line now reads accepted, its Implementation boundary records the edit as made, and ADR 0056's Decision carries the `**Retired:**` marker beside the AOT-invocation sentence.

**Two stale sentences in the packaging profile were corrected in the block being rewritten.** `tiler-prototype-compile`'s edge list omitted `tiler-metal-aot`, which `EXPECTED_DEPENDENCIES` has pinned since `prototype-apple-aot-driver`; and `tiler-prototype-run` was described as `-> [tiler-artifact] + planned platform Metal bindings` with the note "the runner's Metal bindings remain part of the accepted profile rather than a landed edge: `tiler-prototype-run` is still a stub". The runner is not a stub — it dispatches on a real device — and the `metal` edge is pinned. Both blocks now match the pinned reality.

**What landed in the crate under this ticket, and what did not.** `crates/tiler-runtime/src/load.rs` carries the decode stage: `DecodedProgram::decode`, the identity/feature/routing/payload/section accessors, and `LoadRejection`, which carries `ArtifactCodecFailure` whole rather than restating the codec's five classes. Three unit cases pin that foreign bytes classify as malformed rather than damaged, that empty input is refused, and that the classified rejection keeps the codec's own failure reachable as its `source`. The compatibility, binding, object-resolution, and routing-commit stages are `prototype-runtime-artifact-validation`'s and land next; ADR 0081's `implementation_status` is `partial` and names exactly which of them the envelope's public read surface does not permit at all.

**Measurement.** `uv run --locked python scripts/check_workspace.py` passes; `cargo nextest run -p tiler-runtime` runs 3 tests, 3 passed; `uv run --locked python scripts/docs.py render` regenerated the ADR chronology and topic catalogs with ADR 0081. The complete `scripts/check_repository.py` result is recorded on the branch's final commit rather than here.
