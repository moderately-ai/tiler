# Tiler repository agent guidance

This file is the global working contract for agents operating in this
repository. More specific guidance in a descendant `AGENTS.md` may refine it
for that subtree.

## Project scope and posture

Tiler is an experimental, consumer-agnostic Rust toolkit for optimizing
declarative tensor computations and producing efficient parallel compute
kernels. `candle-einops`, Candle, and Metal are initial frontend, runtime, and
backend use cases; do not let their APIs become the compiler's semantic model.

The useful analogy is “DataFusion for tensor compute”: a frontend constructs a
public logical tensor program, target-independent optimization derives legal
alternatives, and physical planning chooses a target-aware implementation. The
analogy is not identity. GPU scheduling, synchronization, memory visibility,
resource limits, and numerical behavior can be physical correctness constraints
instead of mere costs.

The project is currently research- and architecture-first. Do not scaffold
crates, stabilize APIs, or begin production kernel implementation unless Tom
explicitly moves the project into that phase. Bounded executable spikes are
encouraged when they answer a named feasibility or correctness question.

## Authority of existing material

- Accepted ADRs are current decisions. Preserve them unless new evidence
  justifies an explicit superseding decision.
- Proposed ADRs and proposed design documents are coherent hypotheses, not
  commitments and not evidence that Tom personally approved every detail.
- Tickets, research notes, source probes, and accepted ADRs should make their
  evidence and status legible; do not silently convert a proposal into fact.
- When evidence resolves a durable choice, update the relevant contract and
  add or accept an ADR. When it does not, record the measurement boundary and
  keep the question explicit.

Start broad design work with `docs/README.md`, then follow its reading order and
the accepted ADR index in `docs/decisions/README.md`.

## How to collaborate with Tom

Work autonomously on questions with a correctness-derived or clearly dominant
answer. Default to the long-term compatible, correct, and performant design
even when it requires more research or work. Do not ask Tom to choose routine
implementation details, settle facts that can be researched, or approve an
obvious correctness requirement.

Ask only when a genuine product or architectural choice remains after research
and the alternatives encode different valid priorities. Questions must be:

- atomic—one decision at a time;
- concise, with only the background needed to decide;
- concrete, preferably using a small tensor-program example;
- explicit about what each option enables and prevents;
- explicit about point and counterpoint, not only a recommendation; and
- accompanied by a recommendation and the evidence behind it.

**Before presenting options, eliminate the ones that do not survive.** Test each
candidate against correctness, performance, and long-term maintainability and
discard every one that fails; if one survives, there is no question and asking
wastes Tom's time on a decision already made by the constraints. Run that
elimination explicitly rather than assuming it — twice this session an option
was offered that a single check would have removed, once because a decoded
artifact binding buffers by slot position could not verify the position meant
what it assumed, and once because an adapter could not learn which numerical
realization its fallback had delivered. Both would have returned a silently
wrong result, which is not a trade-off but a defect.

Be especially suspicious of the cheaper option. A shortcut presented as a
legitimate alternative is the common shape of this error: it looks like a
trade-off because it costs less, and the cost it saves is the part that makes
the answer correct. State the derivation that eliminated a candidate, so a
reader can refute the elimination rather than only the conclusion.

Pause for Tom's answer after asking such a question. Do not bury several
decisions in a long design dump. If Tom asks for more detail, walk through the
example step by step and distinguish node semantics, graph structure, logical
properties, physical properties, and chosen implementation.

Tom must review key public crate, module, trait, type, and call-site boundaries
before they are accepted or merged. A tested implementation may serve as a
concrete draft, but it is not implicit approval of its public interface.
Present consequential alternatives one atomic decision at a time using the
question format above.

In all research and design writing, label these separately:

- **Fact:** supported by primary documentation, inspected source, or a direct
  measurement.
- **Inference:** a conclusion derived from stated facts.
- **Proposal:** a design that remains to be accepted or tested.
- **Measurement:** an observation tied to an exact environment and procedure.

## Architectural guardrails

Preserve these established boundaries unless a ticket is explicitly evaluating
their replacement:

- The public semantic graph describes what tensor operations mean, not how a
  device executes them.
- Model programs as typed operations and values, with ordered named graph
  outputs and support for multi-result operations. Do not assume one SQL-like
  root or one output tensor.
- Prefer explicit atomic semantic operation families and strongly typed
  attributes, bindings, identifiers, constraints, and errors. Code organization
  may share implementations without collapsing semantic distinctions.
- Keep semantic/logical IR, symbolic access relations, fusion alternatives,
  physical schedules, structured kernel IR, artifact programs, and runtime
  state distinct. Do not build a universal IR or densify physical choices into
  the logical graph.
- Hardware axes and resource dimensions belong in typed target profiles,
  physical properties, schedule alternatives, feasibility predicates, and cost
  models. A graph does not become a hypergraph merely because planning is
  multidimensional.
- Keep hard feasibility separate from estimated cost. Reject an infeasible plan
  with an explainable reason; never hide it behind an infinite or arbitrary
  cost.
- Treat placement, memory domains, transfers, synchronization, and resource
  lifetimes as explicit physical contracts. They are not implicit node
  annotations or generic byte copies.
- Keep compiler core independent of Candle, Metal runtime objects, einops
  syntax, and other consumer-specific types.
- Extension mechanisms must preserve validation, reference semantics,
  feasibility, explainability, and versioned identity. “Extensible” does not
  mean unknown behavior is optimizable.
- Preserve the accepted inline Rust developer experience for macro frontends:
  no required consumer `build.rs`, duplicated registry, source scan, Cargo
  subcommand, prepare step, or runtime source JIT. Each invocation is a
  self-contained AOT and embedding unit; broader fusion requires a larger
  explicit inline region rather than inspection of surrounding Rust.
- “Optimal” means the lowest-cost valid plan under the numerical contract and
  target profile. It does not mean the largest fused kernel; a multi-kernel
  program or deliberate materialization may be correct and faster.

Future compatibility should come from explicit seams and invariants, not from
prematurely implementing an unbounded abstraction. When a mature system will
need more than the first supported subset:

- enumerate the broader semantic space far enough to expose identity,
  validation, ABI, and lowering consequences;
- reserve strongly typed extension points where the dependency direction is
  understood;
- make unsupported cases reject explicitly rather than silently approximating
  them; and
- implement the smallest specialized component that proves the architecture,
  while recording what would be required to broaden it.

Do not confuse a type-system reservation, an architectural seam, implemented
support, and a tested guarantee. They are four different maturity claims.

## Correctness priorities

Bias toward failing closed with typed, explainable errors. Never return an
incorrect tensor merely to preserve a fast path.

Give special scrutiny to:

- numerical contracts, dtype conversions, observable materialization rounding,
  reduction order, exceptional values, and quantized compound values;
- complete cache and artifact identity, validation on every cache hit,
  immutable entries, atomic publication, and crash/race behavior;
- platform family, SDK, deployment minimum, compiler flags, toolchain
  provenance, and runtime compatibility stages;
- preflight before routing commit, fallback only before program work, and no
  fallback after allocation, partial encoding, submission, or semantic
  validation failure;
- exact command-buffer terminal success before host validation readback;
- device/context-scoped runtime cache identity and retention of asynchronous
  resources through their final device use; and
- explain output for accepted and rejected rewrites, candidates, guards,
  capabilities, and assumptions.

Empirical testing can find counterexamples and qualify a bounded profile. It
does not prove an unmeasured universal numerical, compatibility, durability, or
performance claim. Preserve `SoundProof`, exhaustive finite evidence,
empirical evidence, normative guarantees, and `Unknown` as different classes.

## Research standards

- Prefer primary specifications, papers, official documentation, and concrete
  source code. Use secondary material only to locate or contextualize primary
  evidence.
- Inspect the exact local dependency revision when making a source claim and
  record the commit or version.
- A failed search is evidence that the search was wrong, not that the thing is
  absent, until the file has been read. Multi-line attributes, wrapped
  signatures, and re-exported names defeat substring matching; `git log -S`
  inherits the same weakness; and a bounded window (`head -N`, a `sed` range, a
  truncated diff) can split the construct being searched for. When a search
  result contradicts a documented claim, open the file before concluding the
  document is wrong.
- Two types with the same shape are not the same concept. Do not conclude what a
  value means from its field types, its name, or its resemblance to another
  type; read where it is constructed. A `{key, u32}` matched against another
  `{key, u32}` produced two confident wrong conclusions in one session, because
  one carried a target profile key beside a rule version and the other a
  rule-set key beside a revision. The construction site is the evidence; the
  declaration is not.
- When asserting absence, state the exact check so a reader can reproduce or
  refute it in one line. Treat a correction that cannot be reproduced that way
  as unverified, including one arriving from a reviewer.
- Keep facts about a tested host/toolchain separate from portable guarantees.
- Turn important unknowns into bounded experiments with explicit inputs,
  outputs, metrics, unsupported cases, and stop conditions.
- A failed or unavailable measurement is useful evidence when the limitation is
  precise. Do not fill the gap with an assumption.
- Challenge prior design text when evidence conflicts with it, but preserve the
  original rationale and supersede durable decisions explicitly.

Research recommendations should end in one of four concrete outcomes: a
correctness-derived contract update, an accepted architectural decision, a
bounded experiment, or an explicitly deferred question with a trigger for
reconsideration. Avoid accumulating open-ended notes that do not say what
evidence or decision would close them.

Use subagents for independent, bounded research tracks when parallel evidence
collection reduces uncertainty. Give each agent a non-overlapping ticket scope
and exact base commit. Ask agents to report conclusions, measurement boundaries,
tests, and commit hashes. For synthesis, read the artifacts they surface rather
than duplicating their entire research process.

### Coordinating parallel workers

These are failure modes observed in practice, not hypotheticals.

- **State the board from the board.** Counts of running workers, merged
  branches, ticket statuses, and whether a commit reached a remote are all
  cheap to check and were all reported wrongly from memory. Check before
  asserting.
- **Push before dispatching.** A worker's worktree derives from the remote, so
  local-only commits make every base you hand out unreachable. Push, confirm
  `git rev-list --left-right --count origin/main...main` is `0 0`, then dispatch.
- **Chain the gate to what follows it, not merely before it.** Running the gate
  and then pushing in one shell line joined by `;` pushes whichever way the gate
  went; `&&` is what makes the rule enforceable rather than aspirational. A red
  commit reached `origin/main` this way after the gate had correctly reported the
  failure, so "never commit on a red gate" needs its mechanism stated beside it.
- **Gate the exact commit you hand out.** A base is a starting point several
  workers build on, so a red one multiplies. Running the gate on a later state
  does not cover it: a ticket-status edit made only to unblock a claim once left
  a pushed base failing documentation validation, and the worker that inherited
  it had to prove no intermediate commit could be green. Gate, then push, then
  dispatch, in that order.
- **A brief's assertions are claims, held to the same standard as any other.** A
  dispatch that states a fact saves a worker the lookup and costs it the
  verification, so a wrong one propagates with your authority behind it. Cite
  where each claim came from and say which are unverified. Three briefs this
  session asserted something false about the code they described, and each was
  caught by the worker rather than the author.
- **Refill the scope a landing frees, in the same turn.** Reporting a merge and
  waiting leaves dependency-satisfied work unclaimed for no reason.
- **Two workers from one base can both be right and still not compose.** A
  pinned identity digest rebaselined independently on two branches yields a
  merged tree matching neither, and each branch's own tests pass. Recompute such
  a value on the merged tree rather than taking either side, and never resolve a
  conflict in a golden or pinned value by picking a branch.
- **Integrate on the diff, not the report.** A worker's summary is a claim about
  its work. The gate is real evidence and a correctness *argument* is not
  checked by it.
- **Do not delegate what is smaller than its brief.** Writing a dispatch for a
  one-line status change costs more than the change and adds a merge.
- **A question answerable by reading is research, not a decision to escalate.**
  Escalating one stalls the work and moves your job to someone else. Routine
  operations — pushing your own branch, closing a ticket whose remainder is
  tracked — are not reserved boundaries.

## Experiments, prototypes, and evidence

Preserve reproducible experiments, prototypes, fixtures, and referenced
measurements in the appropriate dedicated directory under `spikes/`. Research
documents should link to the checked-in harness or fixture supporting a claim.

Do not delete an experiment directory merely because a run completed. Keep the
reusable source, inputs, harness, and any result fixture cited by documentation.
Add a narrow `.gitignore` in the experiment area for regenerable local data such
as interpreter caches, compiler outputs, and scratch work. Do not ignore
referenced evidence or result fixtures needed to reproduce a conclusion.

Temporary operating-system directories are acceptable for isolated runs only
when the checked-in harness reconstructs them. Cleanup must target regenerable
run products, never the preserved experiment. Prefer keeping compact raw data
when it materially supports a measurement; otherwise record enough exact
environment, commands, and summarized results to reproduce it.

## Documentation as a coherent contract

Treat the documentation corpus as one system. A decision may affect the IR,
optimizer, artifact identity, runtime, testing, roadmap, and open-question
index simultaneously. Before declaring it recorded:

- search for conflicting terminology, stale status language, and duplicated
  authorities;
- update every normative contract whose behavior changes;
- keep accepted decisions, proposals, measurements, and future work visibly
  distinct;
- ensure identifiers, schemas, examples, and dependency directions agree
  across documents; and
- remove an open question only after its answer is represented in the durable
  contract or an accepted ADR.

**A decision recorded is not a decision applied.** Writing an outcome on a
ticket, or a decision section in a record, changes nothing a reader or a check
consults. Accepting an ADR means moving its `decision_status`, regenerating the
catalogs that view it, correcting every contract sentence whose truth depended
on the old status, and releasing whatever the work graph gated on it. Note the
asymmetry: a disclosure required while a decision is proposed becomes *wrong*
once it is accepted, and the check enforcing it stops applying at exactly that
moment — so that staleness is invisible to the gate and has to be found by
reading.

**A doc comment is a claim, and it is load-bearing.** Text describing what a
type exposes is read by the next worker as fact, and an overstated one makes
unreachable work look reachable. Describe what the code does now rather than
what it is intended to do; when a comment and the source disagree, the source
wins and the comment is the defect.

Examples are part of the design work. Prefer a small end-to-end tensor program
that shows inputs, typed operations, multiple values or outputs when relevant,
logical properties, candidate physical plans, rejected alternatives, and the
observable result. Do not let an example quietly introduce semantics that the
normative text has not defined.

Before completing documentation work, run:

```sh
uv run --locked python scripts/docs.py render
uv run --locked python scripts/check_repository.py
```

Generated catalog blocks are checked-in views over frontmatter. Edit source
metadata, not generated list items, and rerun the renderer. The complete gate
owns documentation validation, Python discovery and execution, Ruff,
ShellCheck and shell syntax, ticketsplease lint, and the Rust gate; do not
substitute a hand-picked subset of those commands.

## Ticketsplease and parallel work

This repository uses ticketsplease (`tkt`) as the work graph. Follow its skill
instructions whenever selecting, creating, claiming, dispatching, completing,
or rolling up research work.

- Inspect `git status` before editing; uncommitted files may be Tom's work or
  another agent's claim.
- Use `tkt ready`, `tkt tracks`, or `tkt next` to select dependency-satisfied,
  conflict-aware work.
- Atomically claim the ticket first so another worker cannot win the same work,
  then immediately create or enter its dedicated worktree and `tkt/<id>` branch
  from the exact base commit the dispatch names. Do not edit scoped content
  between those steps. **Verify that commit is checked out before anything
  else**, with `git log --oneline -1`: a worktree may be created from a stale
  `origin/main` and land hundreds of commits behind, in which case the ticket
  files the work depends on are simply absent. Treat a base you cannot resolve,
  or one that proves to be an ancestor of the named commit, as a dispatch error
  to report and correct rather than a starting point to work from.
- Keep one ticket per branch when practical and stay within declared scopes.
- Add a scope before touching a mapped contract area; `paths` do not substitute
  for scopes in scheduling.
- Run the ticket's experiment/tests, `tkt lint`, `git diff --check`, and
  `tkt guard` against the ticket's true branch base before integration.
- Treat guard success as a scope check, not a semantic or test guarantee.
- Mark a ticket `done` only when its stated outcome is actually supported.
  Split a remaining feasibility gate into a follow-up ticket instead of hiding
  it or overstating completion.
- Once a remainder is split into its own live ticket, the parent's stated
  outcome *is* supported and the parent closes. Leaving it in `review` while its
  child waits on it creates a deadlock the graph cannot resolve: `review` does
  not satisfy dependents, so the child is unclaimable and the parent has nothing
  left to do. Splitting is what lets the parent close, not a reason to hold it
  open.
- Preserve other agents' and Tom's dirty changes. Stage and commit exact paths;
  never sweep unrelated modifications into a commit.

### Isolated worktree convention

Coordinator-created worktrees live outside the repository under:

```text
/Users/tsanterre/workspace/github.com/moderately-ai/.worktrees/tiler
```

Use these layouts and ownership rules:

- A ticket has one writable editor worktree at `<root>/<ticket>/edit`, for
  example
  `/Users/tsanterre/workspace/github.com/moderately-ai/.worktrees/tiler/prototype-canonical-index-region-slice/edit`.
  It checks out `tkt/<ticket>` from the ticket's recorded exact base commit.
- Claim the ticket before creating its branch or worktree. The coordinator must
  record the exact base commit and give the worker a task message containing
  the ticket ID, role, branch, absolute worktree path, exact base, allowed
  scope, and whether edits are permitted. A worker must verify those facts and
  a clean status before acting.
- Reviews use a new read-only, detached worktree at the exact commit being
  reviewed. Name it `<root>/<ticket>/review-<role>-<short-sha>`, for example
  `.../prototype-canonical-index-region-slice/review-authority-2a06be1`.
  Reviewers must not inspect or run commands from a live editor worktree, and a
  review is not valid unless its detached worktree starts clean and resolves to
  the requested commit.
- The integration worktree is reserved to one explicitly named integrator at a
  time. Ticket editors and reviewers must not mutate it, and no second actor
  may perform integration, conflict resolution, ticket finalization, or local
  merging there concurrently.
- Keep Cargo outputs worktree-local. Use each worktree's ordinary `target/` or
  another directory unique to that worktree; never share one
  `CARGO_TARGET_DIR` across editor, reviewer, or integration worktrees.

The coordinator owns cleanup. First verify the worktree is clean and that its
commit or branch is preserved as intended. Then run `git worktree remove` with
the exact registered worktree path, followed by `git worktree prune` from a
surviving checkout when stale administrative records may remain. Stop on dirty
or ambiguous state; do not force removal or delete a registered worktree with
`rm`, Finder, or another raw filesystem operation.

## Repository and toolchain operations

### Rust contributor standards

This repository owns its Rust build policy. `AGENTS.md` is the canonical
cross-harness guidance; harness-specific entry points must reference it rather
than duplicate or weaken it.

- `rust-toolchain.toml` pins the exact dated nightly and required components.
  The workspace deliberately declares no stable `rust-version` while accepted
  dependent-array const parameters require nightly. A future stable MSRV needs
  separate conformance evidence and an explicit policy change.
- Keep workspace Rust and Clippy lints inherited by every crate, with the single exception `scripts/check_workspace.py` pins in `UNINHERITED_LINT_MEMBERS`. That table names the one member permitted to diverge and the exact lint table it may declare instead, so a second member dropping inheritance fails the gate. New public APIs require documentation, and warnings fail the repository gate.
- Unsafe code is forbidden except at an individual function or module admitted case by case under [ADR 0079](docs/decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md), which states the four conditions a site must meet: a foreign API leaves no safe route, the `#[allow]` carries a `reason`, an assertion checked against the foreign object's own report bounds what the block touches, and a `SAFETY` comment names the invariant relied on. A crate-level `unsafe_code = "allow"`, a second member dropping lint inheritance, and any relaxation of the workspace `forbid` are all outside that decision and each remains Tom's. Citing ADR 0079 is not sufficient to admit a new site; its four conditions are what generalize. `scripts/check_workspace.py` pins every admitted site as a `(path, item signature, reason)` triple in `ADMITTED_UNSAFE_SITES`, so adding, moving, renaming, removing, or rewording one fails the gate until the pin is updated in the same change.
- Preserve the workspace dev-profile defaults: line-table debug information,
  unpacked split debug information, and optimization level 1 for dependencies.
  If a debugger needs full information, add a temporary or justified
  per-package override rather than inflating the whole workspace.
- Keep release tuning local to an actual shipping package. Do not enable
  workspace-wide LTO for ordinary development; CI or release automation may
  select it through Cargo profile environment variables when measured need
  justifies it.
- Do not vendor third-party Rust repositories as submodules. Pin an actively
  used fork by exact Git revision and keep editable checkouts in the workspace
  hierarchy outside this repository.
- Do not share one `CARGO_TARGET_DIR` across unrelated workspaces. Use a
  compiler cache for cross-workspace reuse, and prefer targeted sweeping over
  destructive cleanup when disk usage grows.
- Nightly-only Cargo settings belong in `.cargo/config.toml` only when the
  pinned toolchain is nightly and the setting is explicitly required. Do not
  introduce ambient user configuration as a repository requirement.

Run the Rust-only sub-gate from the repository root with:

```sh
uv run --locked python scripts/check_rust.py
```

The Rust sub-gate checks the exact workspace/dependency/target contract,
formatting, all targets, strict Clippy, development tests, optimized numerical
tests, doctests, warning-free rustdoc, immutable Cargo locks, and each governed
spike workspace named below. It accepts only the CI-proven macOS
arm64 and GNU Linux x86-64 profiles, each with a 64-bit little-endian address
space and native 64-bit atomics. Use explicit dated-toolchain selectors in
compiler-migration probes; never replace the repository pin with rolling
`nightly`.

**The Rust sub-gate owns every Cargo invocation the repository gate makes.** It
is the only phase that selects the pinned toolchain explicitly, rejects rather
than merely strips hostile Rust environment controls, validates the Cargo
configuration visible from each workspace, snapshots the governed lockfiles,
and gives each workspace its own `CARGO_TARGET_DIR`. A Cargo command run from
the pytest phase, a shell script, or a research harness has none of those and
is a weaker check wearing the same name, so add it here instead.

**A spike Cargo workspace is compiled by the gate exactly when it retains a
compiler-produced golden artifact — a `trybuild` `.stderr` — captured on the
toolchain `rust-toolchain.toml` pins.** Such a file is a positive claim about
what a compiler emits and it outlives whatever produced it: the claim stays on
disk unchanged when the source beside it is edited, and only a compilation
compares the two. Every other predicate over a spike compares a record to a
file that a source edit silently invalidates. A spike whose evidence is
deliberately tied to a *different* toolchain is not gate-compilable at all —
reproducing it needs a compiler the gate has no authority to install, and
re-recording it on the pin would destroy the claim the spike exists to make;
it is named as an explicit off-pin exclusion instead, and its diagnostics
remain retained evidence verified against their record rather than reproduced.
A spike that retains no such artifact is not compiled: its conclusion is about
whatever code is present, so there is nothing checked in that can go stale
against it. `scripts/check_rust.py` enforces the rule in both directions —
`GATED_SPIKE_WORKSPACES`, `OFF_PIN_SPIKE_WORKSPACES`, and a custody predicate
that fails when a retained diagnostic appears outside both sets, so admitting a
golden fixture without deciding its posture is not reachable. Each gated
workspace must also name every one of its `trybuild` cases in its own run
transcript, because a glob that stops matching reports a passing test having
compiled nothing.

**Do not add a `rust-toolchain.toml` to a spike workspace.** The repository pin
is the sole toolchain authority for everything the gate compiles, and a
directory-local file would silently select another compiler for the evidence.

The canonical complete contributor and CI gate is:

```sh
uv run --locked python scripts/check_repository.py
```

`rust-toolchain.toml`, `.python-version`, `pyproject.toml`, and
`tool-versions.toml` are the sole Rust, Python, uv/development-dependency, and
ticketsplease version authorities respectively. Do not duplicate their values
in scripts or CI configuration.

Bootstrap a fresh development checkout with `./deps.sh`. It installs or
verifies the supported host prerequisites, the pinned Rust toolchain, uv,
Python, pytest, Ruff, ticketsplease, and the locked development environment.
`./deps.sh --check` is the non-mutating diagnostic form. Tiler supports this
bootstrap path on macOS and Debian-family Linux only; Windows and other Linux
distributions are explicitly unsupported rather than maintained as untested
branches.

When cloning any repository for research, use only the workspace-aware helper:

```sh
gwc <repository-url>
```

If a noninteractive shell resolves `gwc` incorrectly, use:

```sh
zsh -ic 'gwc <repository-url>'
```

or invoke:

```text
/Users/tsanterre/workspace/github.com/tomsanbear/scripts/git-workspace-clone.sh
```

Never use raw `git clone`; the helper preserves the workspace hierarchy.

Do not install, download, select, or mutate Rust, Xcode, SDK, simulator, GPU, or
other host toolchain components merely to complete a measurement without Tom's
authorization. Once authorized, record the exact resulting component/build and
rerun any measurement previously blocked by its absence.

Use `apply_patch` for file edits. Preserve user-owned changes and avoid
destructive Git or filesystem operations. Generated caches should normally be
ignored in their experiment area rather than repeatedly deleted.

## Implementation boundary

Research completion does not itself authorize production implementation. Before
scaffolding, run the research-readiness gate: audit contradictions and missing
invariants, distinguish measured feasibility from proposals, rank remaining
unknowns by architecture impact and experimental cost, and propose the smallest
vertical slice. Tom decides whether that gate moves the project into
implementation, requires another research wave, or narrows scope.
