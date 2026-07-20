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

Pause for Tom's answer after asking such a question. Do not bury several
decisions in a long design dump. If Tom asks for more detail, walk through the
example step by step and distinguish node semantics, graph structure, logical
properties, physical properties, and chosen implementation.

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
- Keep facts about a tested host/toolchain separate from portable guarantees.
- Turn important unknowns into bounded experiments with explicit inputs,
  outputs, metrics, unsupported cases, and stop conditions.
- A failed or unavailable measurement is useful evidence when the limitation is
  precise. Do not fill the gap with an assumption.
- Challenge prior design text when evidence conflicts with it, but preserve the
  original rationale and supersede durable decisions explicitly.

Use subagents for independent, bounded research tracks when parallel evidence
collection reduces uncertainty. Give each agent a non-overlapping ticket scope
and exact base commit. Ask agents to report conclusions, measurement boundaries,
tests, and commit hashes. For synthesis, read the artifacts they surface rather
than duplicating their entire research process.

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

## Ticketsplease and parallel work

This repository uses ticketsplease (`tkt`) as the work graph. Follow its skill
instructions whenever selecting, creating, claiming, dispatching, completing,
or rolling up research work.

- Inspect `git status` before editing; uncommitted files may be Tom's work or
  another agent's claim.
- Use `tkt ready`, `tkt tracks`, or `tkt next` to select dependency-satisfied,
  conflict-aware work.
- Create or enter the dedicated worktree and branch before claiming a ticket so
  the claim does not dirty another agent's worktree.
- Keep one ticket per branch when practical and stay within declared scopes.
- Add a scope before touching a mapped contract area; `paths` do not substitute
  for scopes in scheduling.
- Run the ticket's experiment/tests, `tkt lint`, `git diff --check`, and
  `tkt guard` against the ticket's true branch base before integration.
- Treat guard success as a scope check, not a semantic or test guarantee.
- Mark a ticket `done` only when its stated outcome is actually supported.
  Split a remaining feasibility gate into a follow-up ticket instead of hiding
  it or overstating completion.
- Preserve other agents' and Tom's dirty changes. Stage and commit exact paths;
  never sweep unrelated modifications into a commit.

## Repository and toolchain operations

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
