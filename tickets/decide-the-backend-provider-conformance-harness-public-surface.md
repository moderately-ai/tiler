---
id: decide-the-backend-provider-conformance-harness-public-surface
title: Decide the backend-provider conformance harness public surface
status: in-progress
priority: p1
dependencies: [exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio]
related: [publish-the-backend-provider-conformance-suite, audit-backend-authoring-against-all-thirteen-responsibilities, specify-the-consumer-neutral-backend-provider-composition-contract, make-explain-dispositions-assertable-by-a-conformance-suite]
scopes: [implementation/conformance, contracts/foundation, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: [.ticketsplease/decision-queue.md]
tags: [decision, needs-tom, public-boundary, backend-providers, conformance]
claimed_from: todo
assignee: worker-packet
lease_expires_at: 1787455299
---
## User-visible outcome

Third-party backend authors have one accepted reusable conformance-harness facade, or one explicit typed deferral with a reconsideration trigger, before Tiler publishes a suite that claims bounded provider correctness.

## Exact-current discovery — 2026-08-17, re-audited at `b085f9dcd95c77ecdf42e93d3e083f02a584a4a8`

1. **Fact — verified.** ADR 0106 admits `tiler-conformance` as the cross-layer evidence member and its complete crate header says `There is none` under `# Public surface`: every module is test-only and every item remains crate-private. The crate is the mechanically correct owner, but its admission accepted no reusable facade.
2. **Fact — verified.** `publish-the-backend-provider-conformance-suite` expressly requires reusable public types and calls for third-party authors. Implementing it under the current crate boundary would either export an unaccepted namespace or create a second owner elsewhere. ADR 0075 reserves that choice to Tom.
3. **Fact — verified.** ADR 0090 deliberately has no runtime-adapter registry. A consumer calls `route_with_adapter` with the adapter it selected, so “missing runtime adapters” is not a constructible discovery failure. Wrong, refusing, or incompatible explicitly supplied adapters are constructible conformance subjects.
4. **Fact — verified.** `ProviderOffer` says an empty offer is legitimate. The packet must preserve the difference between silence, a typed decline, malformed provider output, and absence of any feasible global plan.
5. **Fact — repaired current population.** The thirteen-row backend-composition matrix is historical audit structure, not thirteen current public seams. The current responsibility census is **eleven externally participated rows** — semantic authority; index/access lowering and its realization law; physical implementation; target profile; the ordinary Cargo emitter edge; build orchestration; backend family plus representation; payload provenance; compiler-plan/entry mapping; the explicitly supplied runtime adapter; and its minted live context — plus **two explicit exclusions**: retired scalar lowering under ADR 0105 and compiler-owned opaque-call declaration. These are eleven responsibilities, not eleven uniform registries or installable traits. A green suite may not imply it exercised a row it deliberately cannot expose.
6. **Fact — verified dependency boundary.** Complete artifact-level selected-physical provenance and compile-profile selection provenance are not yet carried; their exact implementation tickets remain blocked on earlier public decisions. The suite may decide its facade now, but the implementation cannot claim those rows until the carriers land.
7. **Fact — verified explain boundary.** Public explain products expose human rendering, not a structured disposition iterator, and their contract forbids parsing that rendering. The exact facade must choose structured explain assertion or an explicit documented exclusion; `make-explain-dispositions-assertable-by-a-conformance-suite` cannot depend on the finished suite without creating the old backward edge.

## Required decision packet

- Re-audit the complete current `tiler-conformance` module/test population, ADR 0106, the provider-composition responsibility matrix and corrections, every public provider/compiler/build/artifact/runtime seam the suite would exercise, and every out-of-crate fixture or spike proposed as evidence.
- Enumerate the nondominated ownership/facade candidates, including retaining the private gate-only crate, publishing a minimal harness from `tiler-conformance`, splitting reusable structural checks from device-reaching executions, further bounded research, and typed deferral where each is genuinely applicable. Do not manufacture a new crate or put provider conformance in a production layer merely to avoid the existing no-public-surface decision.
- Fix exact modules, types, constructors, result/refusal vocabulary, caller-owned fixtures, host-unavailable reporting, async-lifetime boundary, deterministic population counts, and which checks are structural versus provider-supplied semantic evidence. Keep `tiler-reference` as the sole oracle and exclude benchmarking, certification of arbitrary mathematics, performance, dynamic plugins, and silent hardware skips.
- Decide explain coverage atomically: a structured non-rendered assertion surface, or a documented suite exclusion that prevents any full-disposition claim. State the downstream effect on `make-explain-dispositions-assertable-by-a-conformance-suite`.
- Separate the facade decision from implementation prerequisites. The physical-provenance and compile-selection carriers may remain blocked while this packet fixes the public shape, but the suite implementation must depend on them and may not default, infer, or omit their rows.
- State correctness, fail-closed strictness, public compatibility, host runtime/memory, identity/schema, unsupported-population, strongest-counterargument, reversal evidence, and independent subject perturbations for every survivor. Pass independent strongest-reasoning review before queueing one Tom question.

## Exact-current evidence packet

### Owner and current population

**Fact.** [`tiler-conformance`'s complete crate root](../crates/tiler-conformance/src/lib.rs) has no public surface. Its 22 source files currently contribute 84 tests: 81 device-free tests and 3 tests in the macOS-gated `device_buffer`, `dispatch`, and `envelope/apple` modules. The existing portability census derives that population from source and reports it at runtime; `cargo test -p tiler-conformance --lib portability:: -- --nocapture` printed exactly those counts at this packet base. This census is evidence about Tiler's private gate, not a reusable provider contract.

**Fact.** The private modules are fixed Tiler-owned verticals. They cover the BF16 bridge boundary, serial reductions, loop-carried reductions, envelope publication, applicability, preflight classification, measurements, retained records, dispatch and resource retention. They use crate-private helpers, panics/assertions and diagnostic strings. Promoting those modules would publish Metal-specific fixtures and accidental test machinery rather than a consumer-neutral facade.

**Fact.** Existing out-of-crate evidence is deliberately distributed across the owning production seams: [`external_physical_provider.rs`](../crates/tiler-compiler/tests/external_physical_provider.rs) drives the public physical provider; [`custom_backend`](../crates/tiler-build/tests/custom_backend/main.rs) drives neutral build orchestration; the runtime adapter fixtures drive the explicitly supplied adapter; and the retained [three-family portfolio spike](../spikes/runtime/backend-provider-portfolio/README.md) composes them. No common fixture type, output-buffer vocabulary, or backend-neutral execution owner exists among them.

**Fact.** [`RuntimeAdapter`](../crates/tiler-runtime/src/adapter.rs) already owns asynchronous lifetime: its implementation must retain device resources through terminal device use, and `route_with_adapter` returns only after the adapter's `dispatch` method reports terminal success. A conformance facade must not create a second completion token or pretend it can inspect an adapter's private resources. It can only exercise the public route and compare externally observable results supplied through a separately justified fixture boundary.

**Fact.** The current private `Measured<T>` uses `Unavailable(String)` and an ambient `TILER_REQUIRE_METAL_CONFORMANCE` switch. That is suitable for the repository gate and is not an acceptable public contract: a reusable surface would need a typed unavailable outcome, explicit caller policy, and no skip-to-pass conversion.

### Responsibility census and evidence ownership

| Row | Current responsibility | What a future suite could establish | Hold or exclusion |
| --- | --- | --- | --- |
| 1 | semantic authority | structural registration/freeze, identity and duplicate/ambiguity refusals | provider-supplied semantic law still requires independent reference cases |
| 2 | index/access lowering plus realization law | structural installation, request binding and verifier refusal | mathematical equivalence is provider evidence checked against `tiler-reference` |
| 4 | physical implementation | empty offer, decline, malformed proposal, verification, offered/selected separation | artifact-selected occurrence provenance waits for its carrier |
| 6 | target profile | exact key/descriptor comparison and malformed/missing fact refusal | compilation-selection attribution waits for its carrier |
| 7 | backend emitter | deterministic bytes for the same checked input | ordinary Cargo edge; no registry or discovery check exists |
| 8 | build orchestration | checked producer declarations and artifact construction refusals | use accepted closure seams, not a conformance-owned orchestrator trait |
| 9 | backend family + representation | pair comparison and cross-family refusal | neither member may default from the other |
| 10 | payload provenance | from-bytes validation and identity movement | backend owns payload semantics |
| 11 | plan/entry mapping | derivation, association and forgery refusal | complete physical-selection carrier must land first |
| 12 | runtime adapter | explicit wrong/refusing/incompatible adapter behavior and one-way commit | no registry and no “missing adapter” case |
| 13 | live context | only the selected adapter/route mints it; stated environment cannot substitute | exercised with row 12, never directly constructed |
| 3 | scalar lowering | none | retired; counted exclusion |
| 5 | opaque calls | none | compiler-owned; counted exclusion |

The typed census for a future implementation must derive eleven and two from closed owner enums (using `core::mem::variant_count` in the implementation's tests), print both populations, and fail on a moved row. A hand-maintained `13`, a glob, or eleven success booleans is not sufficient.

### Provenance and explain holds

**Fact.** [`package-selected-physical-implementation-provenance-in-artifact-identity`](package-selected-physical-implementation-provenance-in-artifact-identity.md) must carry occurrence-bound physical selection before rows 4 and 11 can be reported complete. [`carry-required-compilation-selection-identity-on-compile-profile-contexts`](carry-required-compilation-selection-identity-on-compile-profile-contexts.md) must carry required selection identity before row 6 can be reported complete. A reusable facade may expose a named partial responsibility subset before either lands, provided its type/report cannot be read as complete and prints the held rows as unsupported. These carriers gate only the corresponding complete rows; they do not gate reconsideration or publication of an independently sound partial facade. `Complete` may neither omit, infer nor substitute any held row.

**Fact.** Public `ExplainReport` and `VerifiedCompilationExplain` expose human rendering (and the latter's candidate count), while `ExplainDisposition` and record iteration remain compiler-private. Rendering explicitly is not a parse target. This packet therefore excludes explain-disposition coverage from every current public-facade candidate. If a future packet wants to claim it, [`make-explain-dispositions-assertable-by-a-conformance-suite`](make-explain-dispositions-assertable-by-a-conformance-suite.md) must first obtain an independently accepted structured accessor. Until then no report may say “all provider dispositions” or treat a rendered substring as evidence.

## Pareto gate

### Candidates considered

| Candidate | Correctness and strictness | Public compatibility and maintenance | Host runtime and memory | Disposition |
| --- | --- | --- | --- | --- |
| A. Preserve the private gate with no recorded trigger | sound and honest, but leaves the publication request liable to be reopened from stale proposal prose | zero API, but no durable answer about why extraction is blocked | current 81/3 split only | dominated by E, which preserves the same code and records exact reopening evidence |
| B. Publish only result/report vocabulary | cannot establish any property; a caller could manufacture `Passed` or translate an arbitrary string | small API but falsely looks like a harness and becomes compatibility debt | trivial | eliminated: self-certification is not conformance evidence |
| C. Publish one whole-backend `run` facade | requires one bundle with absent/defaulted rows or forces partial providers to claim responsibilities they do not own; it also conflates device-free and host/device outcomes | smallest call count, but contradicts ADR 0090's per-responsibility composition | necessarily reaches every selected device path and retains its resources | eliminated: missing authority and silent/defaulted coverage are constructible |
| D1. Publish an explicitly partial split structural/device facade now | the partial label can truthfully exclude rows 4/6/11 and explain, so neither carrier nor structured explain is a prerequisite; soundness still requires a neutral non-self-certifying subject and exact supported-row type | best eventual separation and smaller initial coverage, but no current source authority fixes the fixture/output types; choosing signatures now would invent a second orchestrator or let callbacks self-certify | structural half can stay device-free; execution half pays only selected route/toolchain/device costs | eliminated at this base solely by the missing neutral subject/second independent fixture, not by the carriers |
| D2. Publish a split facade claiming all eleven rows now | cannot bind complete selected physical/plan-entry or compile-selection provenance and cannot assert structured explain; omission, inference or default would be unsound | eventual complete surface, but prematurely couples unresolved owner APIs | same split cost as D1 plus every row | eliminated by the D1 subject gap and the row-specific carrier holds; reconsider coverage incrementally rather than waiting to publish all rows together |
| E. Typed public-surface deferral | preserves every current fail-closed check and makes all unsupported claims explicit; no caller can forge a report because none is exported | no compatibility commitment; durable independent reopening and coverage-expansion triggers prevent repeated speculative API design | exactly the existing private gate; zero production/runtime allocation and zero new host work | sole current survivor |

### Why D is not ready rather than merely less convenient

The retained fixtures disagree on the shape a reusable subject would have to own. Semantic/index checks consume registries and laws; physical checks consume a compiler-owned `ImplementationContext`; build checks use closure seams and backend-specific payload validation; runtime checks consume an adapter whose associated types and private resources deliberately stay backend-owned; numerical checks need backend-supplied cases evaluated by `tiler-reference`. A generic callback returning `Result<(), _>` would certify itself. A single struct carrying all of them would recreate the provider bundle ADR 0090 rejected. Optional fields would turn an omitted responsibility into a default. Publishing the current Metal `Measured<T>`, preflight vocabulary, buffers, or publication proof would instead make Tiler's one backend fixture the cross-backend contract.

The split architecture remains the reopening direction because its evidence classes are real: device-free structural derivation can run everywhere, while execution evidence may be unavailable and must retain resources through terminal use. What is missing for **even D1's explicitly partial surface** is a source-derived, non-self-certifying subject common to two independently authored backend fixtures. The three-family spike is one composed vertical and one authoring family of fixtures, not that second derivation. Carrier landings cannot supply this subject: they make additional provenance readable after a subject exists, while the present gap is what value the harness accepts and how it obtains evidence without trusting a success callback.

## Recommendation held for Tom: exact typed deferral

Accept **no public Rust spelling** from `tiler-conformance` now. Keep every current module private and test-only; add no `pub mod`, re-export, trait, report, result, fixture, builder, constructor, error, completion token or environment-policy type. Do not publish the private `Measured<T>`, `MeasurementBoundary`, preflight classifications, Metal buffer helpers, publication records, or any spike type. Do not add a provider/adaptor registry or a conformance-owned whole-backend bundle.

This is a typed decision outcome, not “later” without a condition. Its triggers are independent rather than an all-or-nothing conjunction:

1. **Partial-facade reopening trigger, sufficient on its own.** A bounded extraction demonstrates two independently authored backend fixtures using one device-free structural subject and one execution subject without optional responsibility fields, a whole-backend provider trait, parsing diagnostics, or callbacks that can manufacture success. It also proves typed host unavailability, caller-owned execution policy, `tiler-reference` as the sole mathematical oracle, and adapter-owned terminal resource lifetime. Reopen immediately for the exact supported subset; do **not** wait for either provenance carrier. Rows the subject cannot establish remain typed unsupported output, never absent/defaulted success. **Corrected 2026-08-22 — read this route with the correction at the end of this record.** It is one sufficient route and never a necessary precondition; the operative reopening condition is stated once there.
2. **Physical coverage-expansion trigger.** When the artifact physical-selection carrier lands, rows 4 and 11 become eligible for a complete claim by a facade whose neutral subject is already accepted. Landing alone does not define that subject and therefore does not by itself justify a public facade.
3. **Compile-profile coverage-expansion trigger.** When the compilation-selection carrier lands, row 6 becomes eligible for a complete claim under the same rule. It neither blocks nor completes a facade that truthfully excludes row 6.
4. **Explain coverage-expansion trigger.** Explain may be explicitly excluded from a partial or complete responsibility report today. An independently accepted structured, non-rendered accessor permits adding explain-disposition coverage later; it is not a prerequisite to publishing a facade that types the exclusion and makes no full-disposition claim.

Trigger 1 is the current public-boundary blocker and is sufficient to reopen this decision. Triggers 2–4 expand what an already-sound subject can claim and should each cause a coverage review, not hold unrelated partial publication. The expected direction remains a split structural/execution surface with shared read-only report views, but **no module or type name is accepted by this deferral**.

### Exact consequence if accepted

- **Identity/schema:** no bytes, domains, versions, pins, artifact identities, cache subjects or explain identities move.
- **Public surface:** remains empty. External authors continue to exercise each accepted production seam through their own integration tests and retained spikes; Tiler publishes no reusable conformance claim.
- **Runtime/memory:** no production path changes. The private portability census remains 81 device-free plus 3 macOS-gated tests at this base; device runs keep their existing resource lifetime.
- **Unsupported population:** third-party reusable reports; certification; arbitrary mathematical correctness; benchmarks/performance; dynamic plugins; adapter discovery; missing-adapter tests; explain-disposition coverage; complete rows 4/6/11 before their carriers; generic device/output buffers; non-Metal availability policy; and any pass synthesized from unavailable hardware.
- **Graph:** the existing publication ticket remains blocked because its stated outcome claims the complete compilation→artifact→route→execution suite. Its provenance and selection dependencies are holds for that complete outcome only. If Trigger 1 fires first, a separately scoped partial-publication carrier (or a truthfully narrowed implementation ticket) must depend on this public-boundary decision and the neutral-subject evidence, not inherit the complete suite's carrier dependencies. The explain ticket remains blocked while no facade is accepted; an accepted partial facade that explicitly excludes explain can close it without an implementation edge, while including explain still requires the structured-accessor path. Neither answer creates a backward edge. The independently reviewed typed-deferral packet is queue item 14, held behind the already ordered LiveRow, artifact-provenance, coincident-RMS, gather, and Metal-facade questions; adding that row records order and does not present another question while LiveRow remains active.

### Strongest counterargument and reversal evidence

The strongest counterargument is that an explicitly partial facade need not wait for complete provenance: production seams are already public, the portfolio has executed Metal plus CPU, and a typed unsupported population could make its narrower claim honest today. The carrier point is correct and no longer supports deferral. What still does is independent: current source has no common non-self-certifying fixture/output subject, so the only extraction would group independently selected responsibilities, publish Metal-specific machinery, or trust caller-supplied success. Evidence reversing the recommendation is one second independently authored fixture that shares exact structural and execution subjects with the portfolio without those defects. That evidence alone reopens D1; the two carrier landings decide only whether the reopened surface can additionally claim rows 4/6/11 complete. **Corrected 2026-08-22 — this sentence is the operative reopening condition; see the correction at the end of this record.**

### Required independent subject perturbations at reopening

Do not perturb assertions. With assertions unchanged, independently:

- remove one responsibility from the typed census and require the report to name the missing row; move one excluded row into the active enum and require the `variant_count` pin to fail;
- return a legitimate empty `ProviderOffer`, a named decline, a malformed proposal, and a well-formed proposal that leaves no feasible global plan, requiring four distinct outcomes;
- duplicate and revise provider identity independently; remove and reorder installed providers independently; require deterministic freeze and offered/selected separation;
- forge scheduled-region structure, request binding, resources, selected occurrence association, compilation selection, payload provenance, entry mapping, backend family and representation independently;
- remove the explicitly supplied adapter, which must be a compile-time missing argument rather than a runtime discovery outcome; then supply wrong, refusing and incompatible adapters independently;
- report host/toolchain/device unavailable and require a typed unavailable outcome that cannot compare equal to pass; set explicit require-device policy and require failure without reading an ambient environment variable;
- return from dispatch before terminal completion or drop a harness-owned lifetime witness while work remains, requiring the execution case to fail without introducing a second runtime completion authority;
- perturb one semantic input/value and one allowed numerical grouping independently, requiring `tiler-reference` comparison to fail while structural checks remain green;
- perturb selected physical provenance and compilation selection independently after their carriers land, requiring artifact/profile identity checks to fail; and
- perturb a rendered explain string while leaving the structured subject unchanged, requiring conformance behavior not to move. If structured explain remains excluded, any “complete disposition coverage” label must fail construction.

## Stop boundary

This ticket authorizes research and ticket/document corrections only. It authorizes no public export, new crate, module move, production implementation, provider default, adapter discovery, or conformance claim. Do not add it to the Tom decision queue until the exact current packet is Pareto-complete and independently reviewed.

## Independent review — 2026-08-17

The repaired packet at `ed1d557170ff8a2afb0fac11a39765dfc5b83a00` received an independent exact-commit review over `b085f9dcd95c77ecdf42e93d3e083f02a584a4a8`. The review found no remaining correctness or graph defect after candidate D was split into an explicitly partial D1 and an all-eleven-row D2. It independently confirmed that the provenance carriers and structured explain gate only their corresponding complete coverage claims, while the current blocker for even D1 is the absence of a neutral, non-self-certifying subject shared by two independently authored fixtures. The portability census remained 22 source files, 81 device-free tests, and 3 macOS-gated tests; `tkt lint --format json`, `make citations`, `git diff --check`, and exact-base `tkt guard` were green. The packet remains held from presentation while the earlier LiveRow decision owns Tom's single active question.

## Closes when

Tom accepts one exact current-source facade or an explicit typed deferral with a trigger; the suite, explain, provenance, and selection dependencies then reflect that answer without a cycle or an implicit coverage gap.


## Accepted decision — 2026-08-18

Tom accepted **the exact typed deferral** at reviewed packet `ed1d557170ff8a2afb0fac11a39765dfc5b83a00`, in the live coordination session with the orchestrator, relayed first-hand by the coordinator, by replying `agreed, next decision` to the accept-or-reopen question presented in explain-then-recommend form.

No public Rust spelling leaves `tiler-conformance` now: no `pub mod`, re-export, trait, report, result, fixture, builder, constructor, error, completion token, or environment-policy type; no provider/adaptor registry; no conformance-owned whole-backend bundle. The private test-only gate continues unchanged. The recorded reopening trigger is one second independently authored backend fixture sharing the same neutral, non-self-certifying structural and execution subjects as the portfolio; that evidence alone reopens the partial-facade question, and the two held carriers then expand only their named rows. The carriers `publish-the-backend-provider-conformance-suite` and `make-explain-dispositions-assertable-by-a-conformance-suite` move to `deferred` with that trigger rather than becoming dispatchable, because this decision's completion means "do not build now", not "build".


## Correction — 2026-08-22: this record stated its reopening condition in two places, and one of them is not a precondition

Filed under [`reconcile-the-conformance-decisions-two-statements-of-its-own-trigger`](reconcile-the-conformance-decisions-two-statements-of-its-own-trigger.md) after a delivered fixture satisfied one wording and not the other. Both wordings are preserved above and quoted again here, so this record's counts do not shrink across the repair.

**The two wordings.** Numbered Trigger 1 reads *"A bounded extraction demonstrates two independently authored backend fixtures using one device-free structural subject and one execution subject"*. The reversal-evidence paragraph reads *"Evidence reversing the recommendation is one second independently authored fixture that shares exact structural and execution subjects with the portfolio without those defects. That evidence alone reopens D1."*

**They were read as competing statements of one trigger. They are not.** Both are stated as **sufficient** conditions, and this record nowhere states a necessary one. Trigger 1's own lead is *"Partial-facade reopening trigger, sufficient on its own"*; the closing paragraph of the recommendation says Trigger 1 *"is sufficient to reopen this decision"* and that the *"triggers are independent rather than an all-or-nothing conjunction"*. A weaker sufficient condition does not contradict a stronger one — it subsumes it. So the question "which wording governs" was malformed: satisfying either reopens the decision, and satisfying neither does not.

**What this record names as the blocker.** The Pareto table's disposition for candidate D1 is *"eliminated at this base solely by the missing neutral subject/second independent fixture, not by the carriers"*. The blocker is the missing **subject and second fixture**, not a missing extraction. The `## Why D is not ready` section agrees: what is missing is *"a source-derived, non-self-certifying subject common to two independently authored backend fixtures"*. A bounded extraction was the proposed vehicle for demonstrating that subject, never an independent requirement — and requiring it before reopening would be circular, because building the shared expression is the reopened decision's own work.

**The acceptance provenance, which settles it.** The `## Accepted decision — 2026-08-18` section above records who (Tom), date (2026-08-18), venue (the live coordination session with the orchestrator), relay (first-hand by the coordinator), the exact accepted packet (`ed1d557170ff8a2afb0fac11a39765dfc5b83a00`), and Tom's words (`agreed, next decision`) to an accept-or-reopen question in explain-then-recommend form. That section states the trigger once: *"The recorded reopening trigger is one second independently authored backend fixture sharing the same neutral, non-self-certifying structural and execution subjects as the portfolio; that evidence alone reopens the partial-facade question, and the two held carriers then expand only their named rows."* Its clause about the two held carriers is drawn from numbered triggers 2 and 3, so it was written with the numbered list in view rather than in ignorance of it — it records a reading, not an omission.

`.ticketsplease/decision-queue.md` item 14, the artifact actually presented, carries the same split intact and with the same modality: its Recommendation says *"One bounded two-fixture extraction with typed host unavailability, caller-owned policy"* … *"is sufficient to reopen a partial facade independently"*, and its Strongest counterpoint says *"A second independently authored fixture using the same non-self-certifying structural and execution subjects reverses the recommendation immediately"*. Neither claims necessity. Tom's assent ratified a packet containing both, which is consistent with both being sufficient and inconsistent with only one governing.

**The operative reopening condition, stated once.** One second independently authored backend fixture that shares the portfolio's neutral, non-self-certifying structural and execution subjects, without grouping independently selected responsibilities, publishing Metal-specific machinery, or trusting caller-supplied success, reopens candidate D1. The comparator is **the portfolio**, which carries both subjects itself: `spikes/runtime/backend-provider-portfolio` assembles through `assemble_plan_artifact` and routes through `route_with_adapter`, comparing twelve output bit patterns against `tiler-reference`. Reopening presents the partial-facade question again; it authorizes no public export, and the accepted deferral's stop boundary is unchanged.

**Correction to a ground recorded against the trigger.** The 2026-08-22 evaluation on the carrier ticket rested partly on `crates/tiler-build/tests/custom_backend` being unable to carry the execution subject, because `crates/tiler-build/Cargo.toml` declares neither `tiler-runtime` nor `tiler-reference` (six tiler dependencies: artifact, cache, compiler, ir, metal, metal-aot — re-verified at `77cd0104`). The manifest fact is true and the inference from it does not reach this trigger: no wording of the condition names `custom_backend` or requires both fixtures to sit inside `make full`. The comparator every wording names is the portfolio, and this record itself counts that retained spike as the first fixture — so a reading that disqualifies spike evidence would leave zero fixtures and make the condition unsatisfiable by construction.


## Re-derived packet — 2026-08-22, at base `b6248f91b22d1c2c18c8d9eb07dc9058dc1c342e`

The 2026-08-22 correction above reopened this decision. This section re-derives it at the reopening base. The 2026-08-17 packet is preserved above in full — including the claims this section falsifies, so no grep count shrinks across the repair — and where the two disagree, **this section is current and the older statement is dated by it**.

### Per-Fact verdict on the 2026-08-17 discovery, re-read at this base

| # | Claim | Verdict at `b6248f91` | Evidence |
| --- | --- | --- | --- |
| 1 | ADR 0106 admits the crate; its header says `There is none` under `# Public surface` | **Verified** | `grep -c 'There is none' crates/tiler-conformance/src/lib.rs` returns 1. ADR 0106's own 2026-08-07 supersession note keeps `item 5's no-public-surface bullet stands`, and its Consequences say `unchanged from item 5: no public surface and no support-matrix authority` |
| 2 | The complete-suite ticket requires reusable public types | **Verified, one status detail stale** | `publish-the-backend-provider-conformance-suite` is `status: deferred` (moved by the 2026-08-18 acceptance), not merely open. The requirement itself is unchanged |
| 3 | ADR 0090 has no runtime-adapter registry; missing adapters are not constructible | **Verified** | ADR 0090 item 4 block: `no runtime-adapter registration, and this record proposes none`; and `Row 12, the runtime adapter, has no registry either`. The fixture's adapter module restates it: `Nothing here registers anything` |
| 4 | An empty `ProviderOffer` is legitimate | **Verified** | `An empty offer is legitimate` in `crates/tiler-compiler/src/frontier.rs`; `An empty [`ProviderOffer`] is a legitimate local result` in `crates/tiler-compiler/src/physical_provider.rs` |
| 5 | Eleven externally participated rows plus two counted exclusions | **Verified as the accepted census; its source table has since drifted** | `grep -c '^| [0-9]' docs/research/extensions/backend-provider-composition.md` returns 31 across three tables; the responsibility matrix itself is thirteen rows, and 13 − 2 = 11 holds. But that table still prints `nothing installs one` for row 4 and `no indirection at all — statically Metal` for row 8, both of which ADR 0090's own dated corrections retire. The count is right and the table a reader would check it against is stale — filed below |
| 6 | Selected-physical and compile-profile selection provenance **are not yet carried**, and their tickets **remain blocked** | **FALSE — both landed** | `package-selected-physical-implementation-provenance-in-artifact-identity` and `carry-required-compilation-selection-identity-on-compile-profile-contexts` are both `status: done`. Verified in source, not from status: `pub fn selected_physical_implementations` exists on both the in-memory and decoded views (`crates/tiler-artifact/src/program/model.rs`, `crates/tiler-artifact/src/program/codec/view.rs`), with encode, decode, and budget paths behind it; the compile-selection carrier landed a full identity cascade including the new owned domain `tiler.metal-aot.compilation-selection.v1` and `FACT_SOURCE_PROVENANCE_SCHEMA_VERSION` 3 → 4 |
| 7 | `ExplainDisposition` and record iteration stay compiler-private; public explain exposes rendering | **Verified** | `pub(crate) enum ExplainDisposition` in `crates/tiler-compiler/src/explain.rs`; `pub struct ExplainReport` in `crates/tiler-compiler/src/session.rs`; `pub struct VerifiedCompilationExplain` in `crates/tiler-compiler/src/explain.rs`. No public re-export |

Evidence-packet Facts re-read at the same base: the portability census is **unchanged** — `portability census: 22 source file(s); 81 device-free test(s) and 3 in the macOS-gated module(s) ["device_buffer.rs", "dispatch.rs", "envelope/apple.rs"]`. The out-of-crate fixtures `crates/tiler-compiler/tests/external_physical_provider.rs` and `crates/tiler-build/tests/custom_backend/main.rs` both exist. `Measured<T>`, its `Unavailable(String)`, and `REQUIRE_MEASUREMENT: &str = "TILER_REQUIRE_METAL_CONFORMANCE"` are all still in `crates/tiler-conformance/src/measurement.rs`.

**Stale command citation, repaired.** The packet cites `cargo test -p tiler-conformance --lib portability:: -- --nocapture`. That command no longer runs in this environment — the harness refuses `cargo test` except for doctests. The equivalent is `cargo nextest run -p tiler-conformance --lib -E 'test(portability)' --no-capture`, which printed the census line quoted above and reported `1 test run: 1 passed, 83 skipped`, i.e. 84 library tests, matching 81 + 3.

### The reopening was correct, and it does not depend on the reconciliation

Re-derived independently rather than accepted from the coordinator. Every anchor below was grepped against this ticket file before being written down; each returns 2, because the 2026-08-22 correction quotes the retired wording alongside the original, which is the convention that keeps counts from shrinking across a repair.

Numbered Trigger 1's lead is `sufficient on its own`; the recommendation closes `is sufficient to reopen this decision`; and the record states `triggers are independent rather than an all-or-nothing conjunction`. **No sentence in this record states a necessary condition.** Two sufficient conditions of different strengths do not compete — the weaker subsumes the stronger — so the reconciliation's verdict holds.

**And it is not load-bearing here, because the delivered fixture satisfies both wordings.** The one-fixture wording names the portfolio as comparator; `spikes/runtime/backend-provider-portfolio` assembles through `assemble_plan_artifact`, routes through `route_with_adapter`, and compares twelve output bit patterns against `tiler-reference` (its README states each step). `crates/tiler-conformance/tests/independent_backend/` does exactly those three things over twelve `f32` patterns. The two-fixture wording is satisfied by the pair. Trigger 1's four additional clauses are each met and each perturbed: typed host unavailability (`HostUnavailable`; `ExecutionOutcome` deliberately carries no `PartialEq` and no `Default`, and `completed()` answers `None`), caller-owned execution policy (`HostPolicy` applied at the call site by `apply_policy`, and no environment variable is read in the file), `tiler-reference` as sole oracle (`workload.rs` writes down no expected value; the expectation is whatever `ReferenceEvaluator` returns), and adapter-owned terminal lifetime (`TerminalUse` is constructible only inside `worker_loop`, and `dispatch` refuses unless it witnesses one per submission).

**Verified running at this base:** `cargo nextest run -p tiler-conformance --test independent_backend --no-capture` → `12 tests run: 12 passed, 0 skipped`.

### What the current public surface already permits — read first, per the readiness gate

This is the finding that reshapes the decision, and it is measured rather than argued.

**Fact.** The fixture merge `829bd1f0` changed **no production crate source at all**. `git show --stat 829bd1f0` lists seven files: the five test files, its own ticket, and 24 added lines of *header prose* in `crates/tiler-conformance/src/lib.rs`. No crate under `crates/` acquired an item, a re-export, or a visibility change to admit it.

**Fact.** `grep -rn 'tiler_conformance' crates/tiler-conformance/tests/` returns **0**. The fixture reaches nothing from the library it sits beside — it cannot, because that library exports nothing, and an integration test compiles against public surfaces alone. The crate header states the design reason: a module here `could reach every pub(crate) item in this crate — which would prove nothing about the boundary an external author actually faces`.

**So a third-party backend author can already author complete conformance evidence against the accepted seams, with no facade whatsoever.** nodefold declares its own target profile, translates verified kernels into its own representation, assembles through `assemble_plan_artifact`, validates its own payload bytes, supplies its own `RuntimeAdapter`, routes through `route_with_adapter`, and is judged by `tiler-reference` — 3,758 lines, entirely on public API. The question this ticket was filed to answer, *can they*, is answered yes by the existing surface.

**And rows 4, 6, and 11 are now publicly readable too**, which the 2026-08-17 packet's Fact 6 assumed they were not. `selected_physical_implementations()` is a `pub fn` on `DecodedVariant`, so an author can assert selected-physical provenance from decoded bytes today without any new surface.

The live question is therefore not capability but **reuse**: should Tiler publish machinery so the next author does not re-derive what nodefold derived? That is a different question from the one the 2026-08-17 packet answered, and the option set below is derived against it.

### What a facade would actually contain, itemized from the demonstrated subject

The gate forbids scoring an option without naming its substance, so here is every candidate export, taken from the one fixture that has now demonstrated the subject.

| Candidate export | Where it is in the fixture | What publishing it would be worth |
| --- | --- | --- |
| `HostPolicy`, `HostRequest`, `HostUnavailable`, `ExecutionOutcome`, `apply_policy` | `nodefold_adapter.rs` | Genuinely neutral in shape, but its payload is *this backend's* resource — a worker thread and a stack size. A neutral version degenerates to a policy enum plus a `String` reason, roughly two types |
| `Disagreement`, `agrees_with_reference` | `nodefold_adapter.rs` | An element-wise `&[u32]` comparison, about twenty lines. Neutral, and trivial |
| Reference evaluation of the same semantic graph | `workload::reference_bits` | Genuinely reusable and genuinely non-self-certifying — the caller cannot manufacture the oracle. But it wraps `ReferenceEvaluator`, so if it belongs anywhere it belongs in `tiler-reference`, not in a conformance facade |
| A typed supported/unsupported responsibility-row report | `Subject` / `SUBJECT_COVERAGE` in `main.rs` | **This is the crux, and it cannot work.** Tiler cannot run a third party's backend, so a published report can only record what the caller tells it. That is precisely `callbacks that can manufacture success`, the defect Trigger 1 requires the extraction to avoid — and it is unavoidable for any published report type. It is also why the fixture's own census is per-fixture by construction: it reads `include_str!("main.rs")` back and checks that each named case still exists |
| The structural assertions themselves | `main.rs` | Nothing to factor out. They are `assert_eq!` against public accessors — `variant.target_profile()`, `feasibility_rules()`, `deferred_predicates()`, `entry.backend_entry_key()` — and the evidence comes from the seam refusing, not from any harness type |
| Whole-backend routing | — | Already public and already exactly this: `route_with_adapter` takes the decoded program, the caller's adapter, the expected identity, and the ABI facts, and returns the output bits |

**The pattern is uniform: every reusable piece is either already public, or is vocabulary that can only record a caller's claim.** The evidence in this fixture is produced by the production seams refusing — `assemble_plan_artifact` deriving an entry key a producer cannot supply, the decoder refusing a forged one as `UnmappedBackendEntry`, the loader refusing a preferred representation, `tiler-reference` refusing a certifying adapter — and none of that is a harness. That is the same ground on which candidate B was eliminated in 2026-08-17: `self-certification is not conformance evidence`.

### Pareto gate, re-run at this base

Every 2026-08-17 elimination that rested on the missing subject or the held carriers is now void and is re-derived rather than carried forward.

| Candidate | Correctness and fail-closed strictness | Public compatibility and maintenance | Host runtime and memory | Disposition at this base |
| --- | --- | --- | --- | --- |
| **A. Re-defer with a narrowed trigger** | Sound; publishes nothing | No commitment, but re-litigates a question whose evidence has now arrived and been assessed | Unchanged | **Eliminated.** The record already rules that a hold without a recorded finding `leaves the publication request liable to be reopened from stale proposal prose`. The finding now exists; deferring discards it |
| **B. Publish result/report vocabulary** | **Fails.** A published report can only record caller claims; Tiler cannot execute a third party's backend | Small API that reads as a harness and is not one | Trivial | **Eliminated on the 2026-08-17 ground, now with evidence rather than by prediction** — the demonstrated subject shows the evidence is produced by the seams, so a report type adds no evidence and one more forgeable surface |
| **C. Whole-backend `run` facade** | Conflates device-free and host outcomes; bundles independently selected responsibilities | Contradicts ADR 0090's per-responsibility composition | Reaches every selected device path | **Eliminated, and additionally now redundant**: `route_with_adapter` already is the public whole-route call, and it is what the fixture uses |
| **D1. Explicitly partial split structural/execution facade** | Its 2026-08-17 blocker is gone — the subject exists and is demonstrated. But the itemization above shows the split has no substance to own: the structural half is `assert_eq!` against public accessors, and the execution half is `route_with_adapter` plus a twenty-line comparison | Would publish two types whose payload is one backend's resource, from a crate documented as the top of the evidence graph that `nothing depends on and nothing may` | Device-free half stays device-free | **Eliminated on new grounds** — not for a missing subject, but because the demonstrated subject shows the seams already are the harness |
| **D2. Split facade claiming all eleven rows** | Both 2026-08-17 grounds are void: the subject exists and both carriers landed, so rows 4, 6 and 11 are now claimable, and explain can be truthfully excluded under Trigger 4 | Inherits every D1 defect across eleven rows instead of a subset | Same split cost plus every row | **Eliminated by D1's ground, which it inherits at greater width** |
| **E. Typed deferral (the 2026-08-18 accepted outcome)** | Sound, but its stated sole surviving blocker — `solely by the missing neutral subject/second independent fixture` — no longer exists | Re-deferring after the trigger fired records nothing | Unchanged | **Eliminated: its own reopening condition fired and was assessed** |
| **F. Further bounded research — a third fixture** | Sound and cheap | Would test the same conclusion in a third shape | Unchanged | **Eliminated as dominated, with the reason stated rather than assumed.** The obvious gap — nodefold carries **zero** deferred prepared-entry predicates because its workgroup capacity is a compile-time profile fact — is already covered by the *other* member of the pair: the portfolio's CPU leg routes a Metal-assessed plan whose variant does carry deferred predicates and answers them through `prepare`. Both branches of that routing path are therefore already exercised across the pair |
| **G. Decide it: no public conformance facade; the accepted seams are the harness** | Preserves every current fail-closed check; publishes nothing forgeable; makes the finding durable and states its own reversal condition | Zero API commitment, and it retires a standing trigger instead of leaving one to be re-litigated | Exactly the existing private gate | **Sole survivor** |

**One option dominates, so no choice is manufactured.** G is top-tier on correctness and strictness and is worse than no other candidate on any key dimension. Identity, schema, bytes, domains, pins, cache subjects, and explain identities are untouched; the public surface stays empty; the private gate is unchanged at 22 source files, 81 device-free and 3 macOS-gated tests, plus the fixture's 12.

### The recommendation, stated exactly

**Accept that Tiler publishes no backend-provider conformance facade, and close the question rather than deferring it again.** The accepted production seams — `TargetProfileBuilder`, `assemble_plan_artifact`, `DecodedProgram`, `RuntimeAdapter`, `route_with_adapter`, and `tiler-reference` — already constitute the harness, and `crates/tiler-conformance/tests/independent_backend/` is the retained, executable, worked demonstration that a third-party author reaches all of them with no Tiler-owned facade and no change to any production crate.

The unsupported population is unchanged and stays typed as unsupported rather than absent: third-party reusable reports; certification; arbitrary mathematical correctness; benchmarks and performance; dynamic plugins; adapter discovery; missing-adapter tests; explain-disposition coverage; generic device or output buffers; non-Metal availability policy; and any pass synthesized from unavailable hardware.

**Reversal condition, replacing the retired trigger.** A backend whose conformance evidence *cannot* be expressed against the public seams — concretely, one that must reach a `pub(crate)` item of any Tiler crate to state its structural or execution subject — reopens this decision. That is a failure a future fixture would hit as a compile error, so it is loud rather than silent. Trigger 4 is also retired as moot: with no facade, no report exists that could make a full-disposition claim, and `ExplainDisposition` remains `pub(crate)` with no public re-export.

**Strongest counterargument.** A published facade's value might be contractual rather than technical: an author who wants to say *my backend is Tiler-conformant* needs a Tiler-owned definition of that claim. The answer is that this is **certification**, which sits in the unsupported population above and which no candidate here proposes to enter. Evidence that would reverse the recommendation: a named consumer who needs the claim rather than the capability, at which point the question is certification policy, not a Rust surface.

**Second counterargument, and the honest bound on this finding.** It rests on one fixture pair, n = 2. The itemization is what carries it rather than the count — the argument is that a report type cannot obtain evidence Tiler cannot produce, which is structural and does not improve with more fixtures. Still, a third backend in a materially different shape could surface shared machinery both of these hid, and the reversal condition above is written to catch exactly that case.

### Graph consequences if accepted

- `publish-the-partial-backend-conformance-facade` — **close**, not build. Its premise is that a partial facade should be published; G answers that no facade should be. Its Non-goals paragraph also carries a claim this base falsifies, repaired separately below.
- `publish-the-backend-provider-conformance-suite` — the complete-suite outcome no longer needs a public surface, and three of its five dependencies are now `done`. Either narrow it to a private cross-layer gate ticket in `tiler-conformance` (where the fixture already is a working instance) or close it. Recommend narrowing, so the row-coverage work has a home.
- `make-explain-dispositions-assertable-by-a-conformance-suite` (`p2`, `deferred`) — **close.** The accepted deferral already said an accepted facade that excludes explain can close it without an implementation edge; no facade at all closes it on the same ground, with no backward edge created.
- Two documentation repairs are filed below rather than left implicit.

### Follow-ups this packet does not do

1. **The responsibility matrix a reader would check the eleven-plus-two census against is stale.** `docs/research/extensions/backend-provider-composition.md` still prints `nothing installs one` for row 4 and `no indirection at all — statically Metal` for row 8, both retired by ADR 0090's own dated corrections (row 4 has `InstalledPhysicalProviders`; row 8 is the promoted `assemble_plan_artifact` closure, accepted 2026-08-05). The census count is unaffected; the table is misleading.
2. **The decision-queue item 14 presentation hold is stale.** It reads `item 6 remains Tom's single active question`. Item 6 resolved 2026-08-18 and items 24 and 25 resolved 2026-08-19. Stated precisely rather than rounded up: **no queue item is awaiting an answer from Tom.** Items 9 and 11 are neither accepted nor resolved, but both are *held before presentation* by their own release triggers rather than sitting with Tom (`do not accept the row yet`; `do not present a carrier yet`), and every other item 1–25 is accepted or resolved. So the presentation slot is free. **This packet must not be queued yet regardless** — this ticket's own Stop boundary requires the packet to be Pareto-complete *and independently reviewed*, and it has not been reviewed. Independent strongest-reasoning review is the next step, not presentation.
3. **A `tiler-reference` evaluation convenience** is the one genuinely reusable, genuinely non-self-certifying helper the fixture contains (`workload::reference_bits`). If it is worth publishing it belongs in `tiler-reference` and is a small, non-Tom decision — deliberately not folded into this public-boundary answer.

### Checks run at this base

`cargo nextest run -p tiler-conformance --lib -E 'test(portability)' --no-capture` (census line quoted above, green); `cargo nextest run -p tiler-conformance --test independent_backend --no-capture` (12 of 12 green); plus `tkt lint`, `make citations`, `git diff --check`, and exact-base `tkt guard` recorded on the branch.

## Independent review — 2026-08-22, verdict **RETURN**; re-derive before presenting

Reviewed at the exact packet commit `5b08faae` in a detached worktree. **The reviewer revised its own verdict mid-review**, from *accept with repairs* to *return*, when a counterexample probe surfaced three further public types it had not found. That is the right shape for an independent derivation and the revision is why this is a return.

**What was refuted: the packet's crux.** Its load-bearing step is that *"Tiler cannot run a third party's backend, so a published report can only record what the caller tells it… unavoidable for any published report type"*, and its own table calls that row *"the crux, and it cannot work."* **This repository has confronted that exact problem five times and answered it the opposite way, in public API.** Verified at source by the coordinator at `ba3e9da3`:

- `ConvergenceEvidence::CallerAsserted` — `crates/tiler-ir/src/schedule/synchronization.rs:387`, `pub`, identity tag `0x02`, documented *"Always refused. It exists so that 'the caller said so' is a statement the model can make and the verifier can reject by name, instead of a possibility the type system silently forecloses and no test can drive."*
- `ToolchainEvidence::ReportedVersions` — the closest analogue: an evidence class whose entire content is self-reported version strings from a toolchain Tiler cannot verify, published safely by bounding what may be concluded (`reuse_scope()` → `SameHost`).
- `ValueDomainProvenance::CallerDeclaredUnvalidated`, `ConformanceEvidenceClass`, `TargetFactAuthority` — all `pub`, all the same shape.

**The correct rule is narrower than the packet's:** a report is self-certifying only where Tiler can neither re-derive **nor label-and-refuse** what it asserts. A labelled, non-privileged, refusable class is not forgeable — naming your own claim is what disqualifies it.

**Consequence for the option set, which is why this returns rather than being repaired in place.** Candidate B's elimination rested on impossibility; with the crux false it rests only on marginal value. So **B-restricted** — a `ConvergenceEvidence`-shaped conformance vocabulary where a caller's assertion is nameable and non-privileged — is a live, never-enumerated candidate, as is **H**, a documented authoring reference. With two unexamined frontier points, *"one option dominates"* is not established. AGENTS.md governs directly: *"If further reading changes the purpose or option set, repair the ticket and repeat the gate before presenting it."*

**What survived the review intact**, and should not be re-derived: both decisive commands reproduce; both nextest checks reproduce exactly (12/12 green, census 22/81/3); Fact 6's falsification is confirmed in source; and eliminations A, C, D1, D2, and F all survive. **The recommendation is still expected to win** on its itemization leg, which held — but it must win on a ground that holds, and say so against this pattern rather than in ignorance of it.

**Also folded into the same pass:** the "accepted seams" mislabel, the 3,758 → 3,348 line count, the reversal condition's capability/reuse mismatch, and the missing ADR body or carrier.

**Scheduling.** Re-derivation needs `contracts/decisions`, held by the live oracle-identity lane. **Release trigger: that lane merges or stops at a gated boundary.** Do not present to Tom before the re-derivation lands.

## Coordinator check of the re-derived packet's crux, 2026-08-22 at `2c312826` — the conclusion holds, one stated evidence line does not

Checked before this packet reaches Tom, because a decision packet is the worst place for an imprecise evidence line to survive.

**Holds — the fixture reaches the accepted seams with no facade.** `grep -rn 'tiler_conformance' crates/tiler-conformance/tests/` returns **0**. The independent fixture imports directly from the production crates instead: `tiler_artifact::program`, `tiler_build::{BackendEntryDeclaration, PlanDeterminismDeclaration, assemble_plan_artifact}`, `tiler_compiler::session`, `tiler_compiler::target`, `tiler_ir::kernel`, and `tiler_ir::program::StageRef`. So an external author demonstrably reaches the seams today without any published conformance surface, which is the packet's load-bearing point and it is sound.

**Imprecise — "the fixture merge `829bd1f0` changed no production crate source".** It did: `git diff 829bd1f0^1 829bd1f0 -- crates/tiler-conformance/src/lib.rs` shows **24 lines added**. They are entirely `//!` documentation — the section `# One integration test, and why it is not a module here` — carrying no public item, no code, no `metal` edge, and no macOS predicate. So the **conclusion is untouched** and the accurate claim is *narrower and stronger*: the merge added **no public item to any production crate**, which is what the argument actually needs. State it that way. As written, the sentence is refutable by a one-command diff, and a reviewer who ran that diff would reasonably distrust the rest of the packet for it.

Neither finding changes the packet's recommendation. Whoever re-derives it should carry the corrected wording rather than restating the original, and should not treat this note as having done the re-derivation.
