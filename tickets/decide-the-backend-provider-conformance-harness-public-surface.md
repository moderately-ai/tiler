---
id: decide-the-backend-provider-conformance-harness-public-surface
title: Decide the backend-provider conformance harness public surface
status: awaiting-decision
priority: p1
dependencies: [exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio]
related: [publish-the-backend-provider-conformance-suite, audit-backend-authoring-against-all-thirteen-responsibilities, specify-the-consumer-neutral-backend-provider-composition-contract, make-explain-dispositions-assertable-by-a-conformance-suite]
scopes: [implementation/conformance, contracts/foundation, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: [.ticketsplease/decision-queue.md]
tags: [decision, needs-tom, public-boundary, backend-providers, conformance]
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

1. **Partial-facade reopening trigger, sufficient on its own.** A bounded extraction demonstrates two independently authored backend fixtures using one device-free structural subject and one execution subject without optional responsibility fields, a whole-backend provider trait, parsing diagnostics, or callbacks that can manufacture success. It also proves typed host unavailability, caller-owned execution policy, `tiler-reference` as the sole mathematical oracle, and adapter-owned terminal resource lifetime. Reopen immediately for the exact supported subset; do **not** wait for either provenance carrier. Rows the subject cannot establish remain typed unsupported output, never absent/defaulted success.
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

The strongest counterargument is that an explicitly partial facade need not wait for complete provenance: production seams are already public, the portfolio has executed Metal plus CPU, and a typed unsupported population could make its narrower claim honest today. The carrier point is correct and no longer supports deferral. What still does is independent: current source has no common non-self-certifying fixture/output subject, so the only extraction would group independently selected responsibilities, publish Metal-specific machinery, or trust caller-supplied success. Evidence reversing the recommendation is one second independently authored fixture that shares exact structural and execution subjects with the portfolio without those defects. That evidence alone reopens D1; the two carrier landings decide only whether the reopened surface can additionally claim rows 4/6/11 complete.

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
