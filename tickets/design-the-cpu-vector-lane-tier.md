---
id: design-the-cpu-vector-lane-tier
title: Design the CPU vector-lane tier and its backend consumption
status: in-progress
priority: p2
dependencies: []
related: [prototype-a-bounded-scalar-cpu-backend-vertical, exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio, design-the-subgroup-execution-tier, drive-an-external-physical-implementation-provider-through-compilation]
scopes: [research/scheduling, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [research, design, scheduling, execution-hierarchy, cpu, simd]
claimed_from: todo
assignee: worker-cpu-lane
lease_expires_at: 1785596471
---
## User-visible outcome

A CPU backend can consume schedules that express fixed-width SIMD (AVX/NEON) and width-agnostic (SVE-class) lane parallelism through the same neutral vocabulary and optimizer surfaces the GPU tiers use — so the second target family Tom prioritized on 2026-08-01 grows past the scalar vertical without a parallel scheduling stack.

## Why now

**Fact.** Tom's target-device priorities are Metal on macOS and CPU. The [bounded scalar CPU vertical](../spikes/target-profiles/scalar-cpu-vertical/README.md) proved the contracts end to end — its own governed profile, backend family, representation, payload, one-way commit, bit-exact agreement including exceptional values — and deliberately installed no physical provider and used no parallelism. Nothing represents a CPU vector lane in the implemented schedule vocabulary.

**Fact.** The adopted scheduled-region model designs the tier: `FixedVectorLane` and `ScalableVectorLane` bindings, explicitly distinct from `Subgroup` lanes ("Subgroup lanes and per-thread vector lanes are different bindings"), with legal width and alignment as profile facts. This ticket carries that design into a decidable shape the same way the workgroup tier was carried: representation and verifier obligations first, target realization as atomic facts, backend lowering downstream.

## Questions this must decide, each with its elimination stated

- What the lane tier's verifier obligations are: tail handling (extent not a multiple of width) is the lane analogue of the workgroup tail, and masked-versus-scalar-epilogue is a numerical question when the operation's contract forbids reassociation — decide what a strict-order fold may vectorize at all, citing the reduction-order contract rather than assuming lane order is free.
- How width enters the schedule: fixed width as a literal the profile must match exactly, versus width-symbolic schedules resolved at feasibility — and whether the two binding kinds share one answer.
- Whether a CPU worker-thread scope (the workgroup analogue) enters in the same design or stays explicitly out — the scalar vertical ran one thread; a threaded CPU backend needs the scope-kind distinction the model reserves, and conflating it with this tier would repeat the synonym the model refuses.
- What the CPU target profile declares: per-ISA atomic realization facts (an AVX2 row is not an SVE row), their provenance (compile-target attribution versus runtime CPUID observation — different availability phases, per the landed phase vocabulary), and what the feasibility surface can answer without executing backend code.
- Where lowering lands: the existing structured-kernel vocabulary emits scalar MSL today; state what a CPU emission target consumes (the KIR as-is with a lane-annotated schedule, or new constructs) and defer any new construct to its own implementation ticket.

## Non-goals

Implementation of bindings, emission, or a threaded runtime; any claim about auto-vectorization by the host compiler (a schedule that *relies* on LLVM vectorizing is not a schedule that states lanes); benchmark claims.

## Closes when

Each question is answered with its elimination or deferred with a trigger, the surviving design sits beside the workgroup and subgroup records with worked examples (one strict-order fold, one reassociation-permitted fold, both at a tail-bearing extent), the public drafts are enumerated for Tom, and the outcome is an accepted design, a recorded deferral, or a bounded experiment.
