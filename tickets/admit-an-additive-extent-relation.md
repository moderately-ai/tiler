---
id: admit-an-additive-extent-relation
title: Admit an additive extent relation so a concatenated extent is checkable
status: review
priority: p1
dependencies: [admit-the-sequence-extension-concatenate-family]
related: [design-autoregressive-state-and-kv-cache, scope-the-sequence-extending-tensor-family, promote-the-symbolic-index-profile-to-a-public-boundary]
scopes: [implementation/ir, contracts/foundation, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, extents, kv-cache, language-model]
claimed_from: todo
assignee: agent-additive
lease_expires_at: 1785704070
---
## User-visible outcome

`S == C + T` becomes statable, so a decode step that binds a cache extent inconsistent with its context length **refuses** instead of verifying and returning a plausible tensor.

## Why this is not deferrable any longer

[The sequence-extending family record](../docs/research/shapes/sequence-extending-tensor-family.md) found the gap and deliberately filed no ticket for it, on the ground that doing so would duplicate a constraint handed to the contract work that will need it. That judgement was correct while nothing needed it. [Rung L5](../docs/research/runtime/autoregressive-state-and-kv-cache.md) is that consumer, and it makes the gap load-bearing rather than latent: the stale-state case — binding an allocation whose valid range is `[0, 13)` while binding `C = 14` — is refused by this relation and by nothing else in the stack. The artifact layer sees a well-formed extent, the bytes are inside the allocation, and the semantic layer cannot relate `S` to `C` and `T` at all.

**Fact, at commit `03a10ae`.** `ExtentRelation` admits `Equal`, `Divisible`, `NonNegativeDifference`, `Interval`, and `Factorization` over an `ExtentTerm` that is a symbol or a constant and is, in its own words, deliberately not an arbitrary expression tree. `NonNegativeDifference` is the nearest additive-looking relation and constrains a difference's sign rather than defining a sum.

## Required design and behaviour

- Decide, with the elimination stated, whether the sum becomes a new `ExtentRelation` variant, a derived form of `SourcedExtent`, or is discharged some third way. `SourcedExtent` is static-or-symbol and every symbol needs exactly one root binding, so a derived form is a change to that invariant and not a widening of it.
- Whatever is chosen must also state `C + T <= capacity`, the three-term relation a windowed append would need, or say explicitly that it does not and why.
- Keep the fragment bounded. The reason it is not an expression tree is that a prover over one is a different component; a sum of two terms is not that, and the design must say where the new boundary is.
- The relation participates in canonical identity wherever an extent relation already does.

## Closes when

A decode-shaped program binding `C`, `T`, and `S` inconsistently is refused with a typed diagnostic naming all three, that refusal has a test which fails without the change, and the consistent binding still verifies.

## Implementation outcome — public draft for Tom

**Fact — the exact draft surface.** `ExtentRelation` gains
`AdditiveEquality { sum, left, right }` and the
`ExtentRelation::additive_equality` constructor. The constructor canonicalizes
the two addends because mathematical addition is commutative. The struct-like
variant is `#[non_exhaustive]`, so an external caller may inspect it with a
forward-compatible pattern but cannot construct it directly and must use the
canonical helper. A compile-fail boundary test observes `E0639` for direct
external construction, while its paired compile-pass test inspects all three
fields with the required `..` pattern. A concrete
mismatch returns the new typed
`ConstraintConflict::AdditiveEqualityMismatch { relation, sum, addends }`, whose
relation renders all three terms and whose numeric fields report both observed
sides. A partially observed relation whose known addend exceeds its known sum
instead returns `ConstraintConflict::AddendExceedsSum { relation, sum, addend,
remaining }`, because the remaining term would have to be negative; it does not
mislabel those two observed values as a fully observed mismatch.
`FragmentViolation::UnderdeterminedAdditiveEquality` is the fail-closed boundary
for a symbolic set whose canonical model does not exhibit a solution. These new
public variants and constructor are a tested draft, not self-accepted.

**Inference — the representation elimination has one survivor.** Changing
`SourcedExtent` is eliminated because it would make a sourced extent both a root
and a derived expression, violating its accepted static-or-one-symbol totality
and the rule that every symbol has exactly one root binding. A third discharge
outside `ShapeEnv` is eliminated because it would duplicate constraint identity,
validation, and diagnostics at whichever consumer happened to check it. A new
fixed-arity `ExtentRelation` variant preserves those authorities, keeps all
leaves as `ExtentTerm`, and is the smallest form that states the required
relationship without introducing a general expression prover.

**Fact — the bounded solver rule.** With zero or one undetermined term the
relation is checked or solved exactly in mathematical `u128` working arithmetic,
then narrowed against the `u64` extent domain through the existing domain
checks. With two or three free term positions, the existing
interval/congruence/comparison solver first constructs its canonical
lower-bound model. The additive relation is admitted only when that exact model
satisfies it; otherwise the set returns `UnsupportedRelation`. Thus every
accepted set still has an exhibited model, while unconstrained runtime-bound
`S == C + T` is retained by its all-zero model instead of being under-decided.
Evaluating that retained relation against invocation bindings remains work for
the later runtime-preflight consumer; this ticket adds no runtime API.

**Fact — capacity and longer sums.** `C + T <= capacity` is expressible as
`S == C + T` plus the already implemented `capacity - S >= 0`. A direct
three-addend equality is deliberately unsupported. Chaining through a fresh
intermediate symbol would give that symbol no legitimate root source, so a
future windowed three-term append needs its own bounded relation or the accepted
general `ShapeExpr` work; it must not weaken the one-root-binding invariant.

**Fact — canonical identity is append-only.** The one relation encoder writes a
fresh exhaustive tag `0x06` followed by the sum and both canonicalized addends.
The accepted public `const fn` constructors for `SemanticInputConstraint` and
`VariantGuard` remain `const`. Their consumed-wrapper helpers are private, and
`ShapeEnvBuilder::require` and `ShapeEnvBuilder::guard` invoke them at the
authoritative ingestion boundary before declaration checking, storage, sorting,
deduplication, or encoding. Internal regressions insert direct reversed and
helper spellings for both constraints and guards, observing one stored wrapper
in each case and the same constraint identity; downstream direct construction
is structurally unavailable.
Tags `0x01..=0x05` and every pre-existing relation byte remain unchanged, so
`tiler.shape-env.v3` does not move: this admits bytes for a previously
unrepresentable subject and re-encodes no old subject. No other relation encoder
exists under `crates/`; the exhaustive `match` in `constraint.rs` is the complete
population. The identity test proves the additive constraint differs from an
otherwise identical unconstrained environment and that reversing its addends
does not mint a second identity. No pinned shape-environment digest exists.

**Measurement — bounded test evidence on the dispatched macOS checkout.** The
decode fixture binds an allocation-valid extent `S = 13` beside `C = 14` and
`T = 1`; construction returns `AdditiveEqualityMismatch` with observed sides
`13` and `15`, and its rendered diagnostic contains `S`, `C`, and `T`. The
neighbor `S = 15` verifies. Deliberately perturbing that neighbor to `S = 13`
made the exact targeted test fail with the structured mismatch before the
fixture was restored. Targeted `tiler-ir` nextest and doc-tests cover this host
and implementation only; they do not prove a future runtime binder performs the
required preflight evaluation.

**Serialized navigation correction — completed at integration.** The BF16
profile ticket was integrated first, releasing its exclusive
`contracts/navigation` claim. This ticket then added that scope and corrected
the `Sequence extension: Concatenate along one axis` roadmap row: it now records
the additive implementation as an independently reviewed public draft awaiting
Tom, and also corrects the concatenate compiler seating that had already landed.

## Integration review — 2026-08-03

Independent correctness/API review found no blocker at implementation commit
`86bcfac48d813c5c089887bdf63eb1bbbf267cbf`. It verified canonical ingestion
before storage and identity, preservation of the accepted `const` constructors,
typed fail-closed diagnostics, append-only tag `0x06`, externally enforced
canonical construction, and the explicit public-draft disclosure. Its one
low-severity glossary finding was corrected and re-reviewed clean at
`479d0325bc157d5068c93350c9c7cf861982c6e1`; merge commit
`7e19a616fa995fc070d7ad55b73c5799709e5e12` contains both. The ticket remains
`review` because accepting the consequential public variant and constructor is
Tom's boundary; integration of the tested draft is not acceptance.
