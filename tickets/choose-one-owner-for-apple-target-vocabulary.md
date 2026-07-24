---
id: choose-one-owner-for-apple-target-vocabulary
title: Choose one owner for the shared Apple target vocabulary
status: in-progress
priority: p2
dependencies: []
related: [prototype-metal-kir-lowering, prototype-apple-aot-driver, compile-golden-msl-through-the-aot-driver-in-the-gate]
scopes: [implementation/metal, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, api-hardening]
claimed_from: todo
assignee: agent-choose-one-owner-for-apple-target-vocabulary
lease_expires_at: 1784917695
---
`tiler-metal` and `tiler-metal-aot` now each define their own MSL language
version, Apple platform family, and deployment minimum:

- `tiler_metal_aot::input::{MslVersion, AppleSdk/ApplePlatform, DeploymentMinimum}`
- `tiler_metal::target::{MslLanguageVersion, MetalPlatform, MetalDeploymentMinimum}`

These describe the same facts about the same targets. Two independent
vocabularies for one domain is how a "3.1" in one crate and a "3.1" in the other
eventually disagree — most likely when one gains a version or platform the other
does not, and a caller translates between them by hand.

The duplication was **forced, not careless**. `tiler-metal-aot` is deliberately
dependency-free (it shells out to `xcrun` and must not drag in the lowering
stack), so it cannot depend on `tiler-metal`. And an MSL language version is not
target-neutral enough to belong in `tiler-artifact` alongside genuinely
backend-agnostic artifact vocabulary. So neither existing crate is an obviously
correct owner, which is why this needs a decision rather than a refactor.

Options to weigh, none obviously right:

- **`tiler-metal` owns it and `tiler-metal-aot` depends on it** — natural
  direction (source emission knows the language), but it breaks the driver's
  dependency-free property, which exists so the driver stays usable and auditable
  in isolation.
- **A small shared crate** owned by neither — clean, but admits a new workspace
  member for three enums, and `AGENTS.md` warns against scaffolding crates ahead
  of need.
- **Keep both and add a checked correspondence** — a test asserting the two
  vocabularies stay in step. Cheapest, keeps both crates' properties, but it is a
  guard rather than a fix and grows with every added variant.
- **Accept the duplication explicitly** and record why, so the next reader does
  not "fix" it into a worse shape.

Whichever is chosen, record the reasoning where a future reader meets the
duplication, not only in this ticket. If the decision is to keep both, that
outcome is legitimate — an unrecorded accidental duplication is the failure, not
duplication itself.
