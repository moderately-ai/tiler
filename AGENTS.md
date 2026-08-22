# Tiler agent guide

Canonical repository-wide guidance. Descendant `AGENTS.md` files may add compatible local detail. Adapt processes when needed, but preserve their rationale and record meaningful deviations.

Priorities: **correctness, long-term maintainability, then performance**.

## Read critical context first

Missing context commonly causes bugs, invalid reviews, and wrong design conclusions. Before consequential work, agents, reviewers, and users directing or accepting it should read in full:

- applicable `AGENTS.md` files;
- the complete ticket;
- every file being edited;
- governing accepted ADRs and contracts;
- relevant construction and consumption sites; and
- correctness-bearing tests, fixtures, manifests, configuration, identities, and generated artifacts.

Use grep, symbol search, `git log -S`, summaries, and excerpts to **locate** evidence, not replace it. They miss multiline constructs, re-exports, generated relationships, and surrounding assumptions. A failed search does not prove absence. State reproducible checks, read branch-owned state from its branch, and review the full diff against its exact base plus required surrounding context.

### A ticket's stated Facts are stale until re-read at your own base

Tickets are written against a tree that has since moved. Assume every Fact, count, and line number in one is wrong until you have re-read its source in full at the commit you are working from. On 2026-08-07 every ticket audited carried at least one false Fact; one had every line citation stale, and several would have led a worker to replace a false claim with a different false claim. Drift is not one-directional: on that ticket four citations moved forward by +71 to +371 lines and a fifth moved **backwards** by 171, so a reader who assumes citations only slide downward will still land in the wrong place.

So a worker's **first** deliverable, before any edit, is a per-Fact verdict — verified, false, or imprecise — each with the file read and the evidence. Repair the ticket and report the repair; never work around a false Fact silently, and never restate one in new words. If repairing it changes what the ticket is for, stop and say so.

**Cite by searchable anchor, not by line number.** A line number rots silently and sends a reader into unrelated code; a quoted distinctive phrase or a symbol name fails loudly and can be re-located. Where a line number genuinely helps, pair it with an anchor and treat the anchor as authoritative.

**An anchor copied from the rendered view fails as absence, which is the dangerous direction.** A rotted line number drops you in unrelated code and you notice; an anchor that finds nothing reads as *the text was removed*, and a worker may then "restore" a claim that was there all along. Quote from the source, because an inline link, an emphasis marker, or a line break renders invisibly and still sits in the bytes grep reads. All three were verified here on 2026-08-08, each scoped to the file the citation names. An inline link splits a sentence in `docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md`, so `grep -c` for the rendered `The drafted body in the source record carries the corrected spelling too` returns 0 while `carries the corrected spelling too` returns 1. `crates/tiler-ir/src/semantic/gather.rs` writes `*labelled draft*`, so `labelled draft public boundary under ADR 0075` returns 0 though the line plainly exists. The same file's wrapped `//!` comment breaks `public boundary under ADR 0075 until Tom accepts its exact included` across two lines, where the single-line `until Tom accepts its` returns 1 — markdown prose here is unwrapped by convention, so the line-break cause reaches code comments rather than documents. A fourth cause was verified on 2026-08-19: sentence-initial capitalization, where the source opens a sentence `The` and the citation quotes `the` — two anchors failed that way in one ticket.

So prefer the shortest distinctive fragment containing no inline link, emphasis marker, line break, or sentence-initial capital — usually one clause — over a full sentence lifted from the rendered view. Where a full sentence is the right anchor, run its grep against the file the citation names **before handing it to anyone**, which is the obligation the coordinator section already places on any supplied command. This is the case "A failed search does not prove absence" above exists for: when an anchor for text you expect finds nothing, suspect the anchor first and open the file. Beware the inverse too — a correction note that quotes retired wording verbatim makes a withdrawn sentence searchable again, so a hit is evidence the string is present, not that the claim still stands. The count therefore **cannot shrink across a successful repair**, so expecting it to is a false progress signal — one was handed to a worker on 2026-08-19, who would have concluded the repair lanes had failed had they trusted it.

**The same false-absence trap has a second cause: a module split.** A citation naming a file path fails the same dangerous way when the item moved to a submodule, because **the named file usually still exists** — so a grep against it returns 0 and reads as *the item was removed* rather than *the module was split*. Three separate lanes hit this on 2026-08-22. Two citations named `crates/tiler-compiler/src/request.rs` and `crates/tiler-compiler/src/target.rs`; both files are still there, while `MAX_TARGET_PROFILES_PER_REQUEST` had moved to `target/request.rs` and is re-exported from `target.rs`, and `physical_plan_combinations` had moved to `request/budget.rs`. A branch audit independently concluded that 91 lines of `request.rs` were absent from `main` when they were sitting in `request/tests.rs`, and only caught it by rebuilding the comparison tree-wide instead of per-path. So after any split, **search the tree, not the named path** — a re-export means the old path still compiles and still reads as authoritative while no longer holding the item. This is why "landed content looks absent" is a distinct hazard from a rotted line number: the reader's next move is to restore something that was never lost.

This obligation is not discharged by a mechanical check. A checker that resolves citations can itself stop matching, and none of them reads for meaning — a citation can resolve perfectly and still support a claim the code no longer makes. The reading is the control; a checker only catches the cheapest subset.

For broad design work, start with `docs/README.md`; accepted decisions are indexed in `docs/decisions/README.md`.

## Project direction and authority

Tiler is an experimental, consumer-agnostic Rust toolkit for optimizing declarative tensor programs and producing parallel kernels. Candle, `candle-einops`, and Metal are initial consumers, not the compiler's semantic model.

Think “DataFusion for tensor compute”: frontends build logical programs, target-independent optimization derives legal alternatives, and physical planning chooses target-aware implementations. GPU scheduling, synchronization, memory visibility, resource limits, and numerics may constrain correctness, not only cost.

Keep work research- and architecture-first until Tom opens implementation. Prefer bounded spikes that answer named questions; premature crates and APIs harden unsupported assumptions.

Use evidence in this order:

1. accepted ADRs and merged contracts;
2. inspected source and reproducible measurements;
3. proposed ADRs and design documents;
4. ticket claims and worker summaries.

Treat the lower two as unverified. Supersede accepted decisions only with new evidence, explicitly preserving prior rationale.

## Architectural guidance

- Keep the public graph about **what** operations mean, not **how** hardware runs them. Prefer typed operations and values, ordered named outputs, and multi-result support.
- Keep logical IR, access relations, fusion alternatives, physical schedules, kernel IR, artifact programs, and runtime state distinct. Their different invariants enable validation, optimization, and explanation.
- Prefer explicit operation families and typed attributes, bindings, identifiers, constraints, and errors. Shared code should not erase semantics.
- Model hardware through typed target profiles, properties, schedule alternatives, feasibility predicates, and cost models. Separate hard feasibility from estimated cost so impossible plans fail clearly.
- Represent placement, memory domains, transfers, synchronization, and lifetimes explicitly; implicit copies hide ownership and ordering.
- Interpret “optimal” as the lowest-cost **valid** plan under the numerical contract and target profile. Maximum fusion is not inherently best.
- Keep the compiler core independent of Candle, Metal runtime objects, einops syntax, and other consumer types.
- Admit extensions only when validation, reference semantics, feasibility, explainability, and versioned identity remain defined.
- Preserve self-contained inline Rust AOT embedding: no required consumer `build.rs`, duplicated registry, source scan, Cargo subcommand, prepare step, or runtime JIT. Broader fusion should use a larger explicit region.
- Prefer typed, explainable failure over a silently wrong fast path.

Give extra scrutiny to numerics and dtype conversion; cache/artifact identity and publication; platform and toolchain compatibility; fallback timing; command-buffer completion; device-scoped caches and resource lifetimes; and explanations for accepted and rejected plans.

Future compatibility usually comes from explicit seams, not universal abstractions. Map enough of the wider semantic space to expose identity, validation, ABI, and lowering consequences; reject unsupported cases; then build the smallest component that tests the architecture.

Keep maturity and evidence claims distinct: reserved type, architectural seam, implemented support, tested guarantee; and `SoundProof`, exhaustive finite evidence, empirical evidence, normative guarantee, `Unknown`. Measurements bound claims but do not prove unmeasured universals.

## Decisions and evidence

Work autonomously when correctness and repository priorities leave one dominant answer. This is pre-production software without external consumers, so complete replacements should remove superseded internal paths rather than preserve unneeded compatibility.

Treat questions answerable by reading or measurement as research. Before escalating, compare options on correctness, maintainability, and performance. A cheaper path that can silently return wrong results is a defect, not a trade-off.

When several valid priorities remain, ask one concrete question at a time. State what each option enables and prevents, give the strongest counterpoint, and recommend one. Small tensor examples expose differences efficiently.

### Decision-packet readiness gate

Do not present a consequential decision merely because one plausible answer has been found. Before a decision reaches Tom, apply this gate; Tom accepted it on 2026-08-12 through [`require-pareto-complete-decision-packets-before-tom-review`](tickets/require-pareto-complete-decision-packets-before-tom-review.md).

1. Re-audit every ticket Fact at the exact current base, then read the relevant construction, validation, consumption, refusal, identity, schema, and dependency paths. A local API shape is not decision-ready while its consumer, owner, authority, or prerequisite remains unresolved.
2. Enumerate every materially distinct option, including the status quo, a narrower fail-closed slice, the complete replacement, further bounded research, and deferral when each is genuinely applicable. Do not pad the list with cosmetic variants or knowingly dominated choices.
3. Eliminate before ranking any option that can silently return a wrong result, invent or default missing authority, fall back across an unstated policy, conflate identities, omit validation, or claim a complete outcome while depending on unresolved work. Split missing prerequisites and healing work into the ticket graph instead of treating them as implementation detail.
4. Compare the survivors on all key dimensions: correctness; fail-closed contract strictness; long-term maintainability and compatibility; and Tiler host runtime and memory. Keep kernel performance separate unless the decision is specifically about kernels. State identity, schema, public-surface, and unsupported-population consequences alongside the comparison.
5. Present only the nondominated frontier. Every presented candidate must be top-tier on correctness and strictness, and no presented candidate may be worse than another on every key dimension. When one option dominates, recommend or take that option rather than manufacturing a choice. When a real trade-off survives, ask one concrete question between the frontier candidates.
6. For every survivor, state its strongest counterargument, the evidence that could reverse it, the negative controls or subject perturbations that would test it, and the follow-up tickets/dependencies required to leave no work implicit. Use an independent derivation for public-boundary, identity, schema, numerical, or cross-layer decisions where being wrong could silently admit or misidentify a program.

A matrix is a summary, not the analysis. Do not score an option green because a prerequisite was assumed, an unsupported population was omitted, or a failure was renamed. If further reading changes the purpose or option set, repair the ticket and repeat the gate before presenting it.

Tom retains decisions about:

- consequential public crate, module, trait, type, or call-site boundaries;
- genuine product or architecture expansion, and destructive or irreversible actions; and
- movement between research and implementation.

Adding scopes required by authorized work is scheduling metadata; add and explain them. A tested public boundary remains a labelled draft until Tom accepts its exact included and excluded surface.

Label research claims as **Fact**, **Inference**, **Proposal**, or **Measurement** so readers know their authority.

## Research

Prefer primary specifications, papers, official documentation, and exact source revisions. Use secondary sources for discovery. Record dependency versions or commits.

For inaccessible sources, record the exact reference, attempts, and decision it would inform. A visible evidence gap is better than an invented citation.

Never circumvent an access control to retrieve a source. A CAPTCHA, bot wall, interstitial, login form, or paywall is the publisher refusing automated access, so do not drive a browser through it, retry under another user agent, or seek an evasion mirror — this holds even where we could read the work by other means. Plain fetches of public URLs stay fine. On hitting a barrier, stop on that source and hand it to Tom, who retrieves documents and resolves DOIs himself: give the full citation, the DOI or library identifier with your confidence in it, what blocked you and where, the claim that stays thinner without it, and what would change if it disagrees. Rank the list by how much conclusions actually depend on each. A circumvented citation is worse than an absent one.

Turn important unknowns into bounded experiments with explicit inputs, outputs, metrics, unsupported cases, and stop conditions. Separate host-specific observations from portable guarantees.

Research should end in a contract update, accepted decision, bounded experiment, or deferred question with a reconsideration trigger.

Keep reproducible spikes, inputs, harnesses, and cited fixtures under `spikes/`, linked from their documents. Run them manually from documented commands so exploratory dependencies do not silently become repository gates. Retain evidence needed to rerun claims; ignore only regenerable local data.

## Ticketed parallel work

`ticketsplease` (`tkt`) is the work graph; follow its skill for command mechanics.

### State and dispatch

Read branch, commit, status, divergence, worktrees, workers, claims, ticket states, and ready work from the repository, not memory. Stage exact paths because uncommitted files may belong to someone else.

Gate the exact base, push it, then confirm:

```sh
git rev-list --left-right --count origin/main...main
# 0 0
```

Prefer `gate && push`; the safe order is **gate → push → dispatch**.

Claim first, commit and push the claim, then create `tkt/<id>` and its worktree from that commit:

```text
/Users/tsanterre/workspace/github.com/moderately-ai/.worktrees/tiler/<ticket>/edit
```

Verify the checkout commit. Keep one ticket per branch where practical. Read mappings from `ticketsplease.toml` and add required scopes before editing.

For overlapping live claims, inspect scope declarations on the owning branch and compare actual non-empty diffs. Empty diffs prove nothing. Use `tkt why` pairwise for hand-built batches.

A concise brief should name the role, exact base and path, edit permissions, scopes, authorities, outcome, non-goals, checks, stop conditions, public boundaries, and required evidence. Cite brief assertions; wrong context propagates quickly.

### Route subagent models by change risk

Choose the worker and reviewer model from the consequences of being wrong, not from a standing experiment or a desire to distribute turns evenly.

- Use the strongest reasoning model available (currently Sol) for accepted ADRs and contracts, public-boundary decisions, identity or schema evolution, numerics, cross-layer correctness, ambiguous authority, broad population audits, and work whose failure could silently admit or misidentify a program.
- Use the balanced model (currently Terra) for tightly bounded implementation, localized tests, mechanical censuses, and prose repairs whose authority, affected population, and failure condition are already explicit. If its first source audit exposes an identity consequence, public-boundary choice, conflicting authority, or materially wider population, stop and reassign or review with the stronger model.
- Select independent review by the same risk test. A bounded Terra change does not automatically need Sol review; identity-, authority-, and boundary-sensitive changes do. Model diversity is useful when it supplies a genuinely independent derivation, not as a permanent bake-off.

Model choice never substitutes for the reading, source-first Fact audit, subject perturbation, exact-base review, or gates required below. Record a non-obvious routing choice in the brief so a later coordinator can reproduce why that model was appropriate.

**The coordinator reads before briefing, and reads again before merging.** A brief is the highest-leverage place to inject a false claim, because every worker receiving it treats it as settled. Three coordinator-authored claims went wrong on 2026-08-07: a rewritten obligation class that the code discharges differently, an enumeration command that miscounted by matching the false positive it was written to exclude, and pin values already superseded when the brief shipped.

Before briefing:

- Re-read the ticket at the base being dispatched from, not the version you remember or last edited.
- Assert nothing you have not read in a file **this session, at this base**. A worker's report, an earlier summary, and your own recollection are all secondhand.
- Give each factual claim a command the worker can rerun, and **run it yourself first** — a supplied command that has never been executed is a claim, not a check.
- Mark anything you could not verify as unverified, and tell the worker to contradict you with evidence rather than defer. Workers that pushed back were right every time it happened.

Before merging: read the full diff, and re-read the source behind any claim the merge depends on. Give particular attention to sites a worker classified as *already correct* — a wrong "no change needed" leaves no diff to review and is the cheapest place for an error to survive.

### Worker, review, and integration

Workers should verify base, branch, claim, scopes, and clean status; read the complete ticket and critical files; then run targeted checks, `tkt lint`, `git diff --check`, and `tkt guard` against the true base.

Treat `tkt guard` as scope evidence only. Before commit it may report no changes; from stale ticket state it may misread branch-local scopes.

If work cannot finish, preserve a gated, coherent boundary and map the remainder. If discovery changes architecture or identity, stop and create the needed tickets and edges. Return the exact commit, correctness argument, commands, failure evidence, measurement boundary, unsupported cases, and scope confirmation. Workers leave merging, integration-tree mutation, ticket closure, and outcome expansion to the coordinator.

Review a clean detached worktree at the exact commit:

```text
<root>/<ticket>/review-<role>-<short-sha>
```

Read the full diff, relevant construction and consumption sites, and failure paths. Report severity, location, and reproduction; a bare “looks good” is not evidence.

Integrate the reported hash, not the summary or branch label. Record `HEAD` before and after and confirm the hash is an ancestor of `main`; unchanged `HEAD` means nothing landed. Recompute generated values, goldens, and identity pins on the merged tree. Keep identity-domain changes coherent across owning version, ledgers, and pins.

Return wrong, abandoned, or off-scope work to its branch rather than salvaging a partial merge. Keep one named integrator in the integration worktree.

### Graph and cleanup

Update tickets with facts, commands, measurements, and hashes. Close only supported outcomes; split bounded remainder into a narrow ticket, then close the revised parent so dependents can proceed.

Use separate tickets for out-of-scope defects, public-boundary redesigns, missing identity or validation authority, bounded research, independent performance work, and deferred capabilities. Deferred tickets belong in `deferred` and should end with `## Trigger check log`; each dated entry records `fired`, `not fired`, or `unevaluable` plus a reproducing command.

Refill capacity after landings. Remove only clean, preserved worktrees with `git worktree remove`, then prune. Avoid force-removing ambiguous state.

Keep Cargo output worktree-local. Gates can use 7–15 GB, so clear local `target/` data and size concurrency from measured free space.

Make new checks fail deliberately before trusting them. Name and count populations so “nothing ran” cannot look green. Size one with `grep -o … | wc -l` and say which unit you report; `grep -c` counts **lines**, not occurrences, and undercounted a population by a quarter on 2026-08-19. **Anchor the pattern too**: an unbounded `fn shape` also matches `fn shape_product` and `fn shape_of`, inflating a population from 26 to 43 on 2026-08-22. Bound it (`fn shape\b`) and say which form you counted.

**Perturb the subject, never the assertion, and show the failure text.** Editing an assertion until it fails proves only that the assertion runs. Break the thing the check exists to guard — reuse a tag, drop a field, widen the ledger row, gate a module behind a platform predicate — and quote what the check said. A report that claims a check can fail without showing its message has not demonstrated it. Where a check guards several independent properties, perturb each one separately; a perturbation that reddens everything cannot show which assertion is load-bearing.

**Size enumerations from the type, not by hand.** `core::mem::variant_count` makes a widened vocabulary a build error at the enumeration rather than a population that silently shrinks while still reporting no collision. A hand-written length, a successor chain, and a wildcard-free match can all be satisfied by an enumeration that has stopped covering its domain. Where the population cannot be typed, assert a floor and print the census.

**Verify that a check reaches its subject at all.** Several here could not. `cargo doc` cannot fail for a `#[cfg(test)]` module, or for an integration test under `tests/`, because rustdoc compiles neither. A grep anchored `^(pub )?` cannot match a `pub(crate)` item. A single-line matcher cannot see an attribute that wraps. A closing condition demanding a grep be empty cannot be met where the convention quotes retired text inside dated corrections. Before trusting a check, state what it would take for it to say *no*, and confirm that case is reachable.

**A census is only as complete as its own search vocabulary, and a closing condition built on one can close a ticket green over sites it never looked at.** This is the same anchoring hazard as above, moved to the place where it does the most damage. Three instances on 2026-08-22, each verified. `repair-the-stale-three-carried-subject-claims` closed on a census greping `three carried subjects` and `three subjects`, while live stale sites in `crates/tiler-artifact` spell it `three reached semantic subjects` — which contains neither, so the census returns **0** on those files and the ticket closed over them. A governed-key count used `tiler::[a-z0-9-]+@[0-9]+`, which excludes `_` and silently dropped five underscore-bearing keys, reporting 50 where the anchored form gives 55. And a commandless-entry census over trigger logs reported a ticket as carrying no command because its verb list omitted `sw_vers`. **Over-anchoring under-counts as reliably as under-anchoring over-counts, and it fails silently in the direction that reads as clean.** So a census must state the spellings it searched for and why that set is complete, and a closing condition resting on one is only as strong as that argument. Where the population can be typed, `core::mem::variant_count` retires the question; where it cannot, prefer a claim about what you read over a count you cannot derive — a number that has drifted twice is better withdrawn than corrected a third time.

**A mechanical check does not discharge a reading obligation.** It is one more artifact that can quietly stop working, and it never reads for meaning. Use checks to make regressions loud; use reading to decide whether the claim was ever true.

## Verify and ship

During development, use targeted formatting, `cargo check`, package nextest, Clippy with warnings denied, and rustdoc for touched packages:

```sh
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

Use `make check` while iterating and `make full` once on the completed batch before pushing `main`. The full gate adds rustdoc, release numerical tests, `tkt lint`, and shellcheck. Warnings fail because deferred warnings become hidden debt.

Keep both workspace test commands because nextest omits doc-tests:

```sh
cargo nextest run --workspace
cargo test --workspace --doc
```

The second preserves ADR 0051 compile-fail evidence.

On this macOS host, a nextest leaky verdict that moves between unrelated tests usually indicates the known pipe-inheritance race; recurrence in one test suggests a real unreaped child.

Design deterministic tests: derive ordering from observed state, count populations, isolate mutable paths, and give re-executed binaries private copies. Investigate intermittent failures rather than rerunning until green.

Gate the exact published commit, chain publication with `&&`, verify zero remote divergence, and freeze `main` during the chained gate-and-push.

A delta may reuse the latest green gate only when it touches none of:

```text
crates/
prototypes/
Cargo.toml
Cargo.lock
.config/
Makefile
rust-toolchain.toml
rustfmt.toml
deps.sh
check-citations.sh
```

Record the carry reasoning and rerun `tkt lint` **and `make citations`**; when uncertain, run `make full`. Naming the citation check here is load-bearing: `tickets/` is not in the list above, so a ticket-only delta carries the gate, and a check named only in the `Makefile` would be skipped by exactly the deltas it exists to police.

For redirected gates, inspect terminal log lines and use commit-unique filenames. Compound shell status and shared logs can misreport results.

## Performance

Establish correctness and measurement validity first. Define workload, target, metric, baseline, warm-up, repetitions, noise controls, and oracle; measure the dominant cost; make the narrowest change; rerun identically; report variance, regressions, environment, and limits.

Keep feasibility separate from cost and measurements bounded to their profile. Run CPU timing and profiling on the idle M3 Pro, not the coordination host during agent waves. Device-specific measurements stay on their claimed hardware.

## Documentation and durable records

Documentation is manually maintained. One mechanical property is checked: `make citations` resolves every local markdown link in an open ticket, a live document, or a retained spike record, so a catalog row or cross-reference that points at nothing fails the gate. `check-citations.sh` states which link shapes it declines to resolve and why. Nothing else is validated — not frontmatter, not supersession, not whether an entry point still lists the right documents, not whether a link that resolves points at the document it claims to, and not the heading anchor after a `#`. Update catalogs and dependent contract language with the decision or metadata they describe.

**The link population and the pinned-citation population are not the same set.** The difference is `spikes/**`. A spike record is checked for its links and deliberately **not** for its pinned citations: a spike is evidence about the base its own record names, so demanding its line pins resolve at the tip would both be unsatisfiable and turn a `crates/` landing into a red gate through exploratory material — which [the spike catalog](spikes/README.md) already rejected on costed evidence under "Whether a spike still runs". Measured at `77cd0104` when the corpus was added: 68 records, 590 local links all resolving, and 50 pinned citations declined, 7 of which do not resolve and correctly do not fail. The declined count is printed on the census, so the exclusion is a number rather than a silence. Spike *links* into `crates/` are resolved like any other — 12 of them, 11 inline plus one reference definition — so deleting or renaming a file a spike links to does fail the gate, while a line moving inside one never can.

Until 2026-08-22 `spikes/**` was in no population at all, and the gap was found by perturbation rather than by reading: a planted broken link there left the gate green, with output byte-identical to the unperturbed run.

Applying an ADR means aligning status, catalogs, contracts, terminology, and released graph edges. Read affected documents in full before declaring the sweep complete.

When a research ticket cannot edit `docs/decisions/`, preserve a verbatim-landable ADR body and file a carrier ticket; editing during transfer creates a fork.

Record acceptance provenance: who, date, venue, and relay source. Keep relayed changes cheap to reverse until verified. Treat comments and examples as claims about current behavior.

When work advances a support-matrix row, name the row and extent in its Outcome or file the ledger update. Use examples to exercise general machinery, not specialize semantics around one case.

## Toolchain and environment

Tiler develops on macOS and has no CI. Use `./deps.sh` to bootstrap and `./deps.sh --check` for non-mutating inspection.

`rust-toolchain.toml` is the Rust version authority. Spikes inherit it; use explicit dated selectors only for migration probes. `rust-src` keeps diagnostics and `trybuild` goldens reproducible. Stable MSRV support needs separate evidence while accepted features require nightly.

Other tools remain unpinned unless a concrete failure justifies maintenance cost. Keep required Cargo settings repository-local rather than relying on user configuration.

Crates should inherit workspace Rust and Clippy lints; inspect `[lints]` changes because inheritance is not enforced. `prototypes/` builds and tests but is excluded from the crates' style gate. Prefer splitting genuine complexity over wildcard matches that weaken exhaustiveness.

Unsafe code is admitted only at named sites under [ADR 0079](docs/decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md): no safe foreign-API route, a reasoned `#[allow]`, a bounding assertion, and a `SAFETY` explanation. Broad unsafe or lint relaxations remain Tom's decision.

Preserve the measured dev profile in `Cargo.toml`; reopen it only with new measurements. Keep release tuning local to a shipping package or explicit experiment.

Prefer exact-revision dependencies over vendored submodules, keep editable third-party checkouts outside the repository, and avoid shared `CARGO_TARGET_DIR`s. Clone research repositories with `gwc <url>` or its documented fallback so workspace layout stays consistent.

Changing Rust, Xcode, SDK, simulator, GPU, or other host components for a measurement requires Tom's authorization because it changes the evidence environment. Record authorized changes and rerun affected measurements.

## Implementation boundary

Research completion does not authorize production implementation. Before proposing the transition, audit contradictions and missing invariants, separate measurements from proposals, rank unknowns by architectural impact and experiment cost, and identify the smallest useful vertical slice. Tom decides whether to implement, continue research, or narrow scope.
