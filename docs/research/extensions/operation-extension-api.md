---
schema: "tiler-doc/v1"
id: "tiler.research.extensions.operation-extension-api"
kind: "research"
title: "Experimental operation API sketch"
topics: ["extensions", "operations", "rust"]
catalog_group: "foundation-semantics-extensions"
research_status: "complete"
disposition: "partially-adopted"
implementation_status: "partial"
evidence_classes: ["primary-source-synthesis", "executable-model"]
informs: ["tiler.contract.operation-extensions"]
adopted_by: ["ADR-0044"]
ticket: "operation-extension-surface"
---

# Experimental operation API sketch

This sketch records the shape validated by the compile-checking spike. Names
and allocation details remain experimental.

## Construction flow

```rust,ignore
let mut registry = RegistryBuilder::new();
registry.register_semantics(my_operation)?;
registry.register_capability(my_decomposition)?;
registry.register_capability(my_metal_lowering)?;

let registry = registry.freeze()?;
let request = CompilationRequest::new(graph, registry, targets, options)?;
```

The caller supplies providers explicitly. Optional adapters may call the same
builder API, but there is no hidden global registry.

## Semantic root

```rust,ignore
pub trait SemanticOperation: Send + Sync + 'static {
    fn key(&self) -> &OpKey;
    fn schema(&self) -> &OperationSchema;
    fn semantics(&self) -> &SemanticContract;

    fn infer_and_validate(
        &self,
        context: &InferenceContext<'_>,
        operation: &UnverifiedOperationView<'_>,
    ) -> Result<InferredResults, Diagnostic>;
}
```

The production design should prefer immutable descriptor data over callbacks
where the answer is static. `infer_and_validate` remains a callback because
shape and dtype relationships depend on operands and constraints. The host
revalidates every returned result before admitting the operation.

`SemanticContract` identifies normative meaning and required conformance data.
It is not the provider revision and does not claim the callback is its own
formal specification.

## Bounded result inference refinement

The initial sketch returned an owned `InferredResults`. The implemented
prototype instead gives an inferencer a host-owned, fallible result writer.
This keeps arbitrary multi-result operations while making result-count and
canonical-byte limits enforceable before the host retains an unbounded result
collection.

**Fact:** At Burn revision
[`e5467f02`](https://github.com/tracel-ai/burn/blob/e5467f02c3cf88eb5d709f190c170005ce26038d/crates/burn-ir/src/operation.rs),
`CustomOpIr` declares a `Vec<TensorIr>` of outputs. This establishes a useful
multi-result precedent, but the provider constructs the collection and the API
does not impose Tiler's host-owned admission budget.

**Fact:** At Candle revision
[`31f35b14`](https://github.com/huggingface/candle/blob/31f35b147389700ed2a178ee66a91c3cc25cc80d/candle-core/src/custom_op.rs),
`CustomOp1`, `CustomOp2`, and `CustomOp3` each return one `(Storage, Shape)`.
At ug revision
[`8c6dd50d`](https://github.com/LaurentMazare/ug/blob/8c6dd50d6e96a22db70e1462c0e49d0cda8294f7/ug-core/src/lazy_buffer.rs),
a custom operation likewise produces one `LazyBuffer`. Those interfaces are
too narrow for Tiler's accepted multi-result semantic graph.

**Fact:** MLIR's `InferTypeOpInterface` at revision
[`98fe06cb`](https://github.com/llvm/llvm-project/blob/98fe06cb6487cf23d5a56bd5ff0c6b4e378d1be3/mlir/include/mlir/Interfaces/InferTypeOpInterface.td)
uses a caller-owned `SmallVectorImpl<Type>&` for inferred return types and then
checks compatibility with declared return types. It demonstrates caller-owned
result collection and host validation, though it does not provide Tiler's
fallible per-result quota boundary.

**Inference:** An unrestricted owned vector makes multi-result inference easy
but gives extension code control over collection growth. A single-result API
avoids that issue only by rejecting required semantics. A host-owned fallible
writer preserves both requirements and leaves policy with the compiler host.

**Proposal implemented by the current prototype:** `OperationInferencer`
receives a borrowed `OperationInferenceRequest` and an
`OperationInferenceOutputs`. Every `try_push` validates and charges count and
canonical bytes before retention. The writer is sticky after its first error,
and only the host can finalize it. Callback success does not commit anything:
the host still verifies arity and every inferred fact before graph mutation.

## Separate capabilities

```rust,ignore
pub trait ReferenceEvaluator: Send + Sync + 'static { /* typed evaluation */ }
pub trait DecompositionProvider: Send + Sync + 'static { /* proposed graph */ }
pub trait RewriteProvider: Send + Sync + 'static { /* transactional proposal */ }
pub trait AccessLoweringProvider: Send + Sync + 'static { /* domain/access */ }
pub trait PhysicalImplementationProvider: Send + Sync + 'static { /* typed boundary */ }
pub trait KernelLoweringProvider: Send + Sync + 'static { /* structured KIR */ }
pub trait CostEvidenceProvider: Send + Sync + 'static { /* estimate only */ }
```

Each registration has:

```rust,ignore
ProviderKey { namespace, name, capability_api_version }
ProviderRevision(canonical_bytes)
compatible_operation: OpKey
declared_preconditions
capability_object
```

An opaque physical implementation is still typed: it must expose operand and
result ABI, effects, aliasing, placement, target requirements, numerical
contract, resource envelope, and failure stage. It is not an unrestricted
backend callback hidden inside semantic IR.

## Registry lookup and selection

The registry indexes semantic authority by `OpKey` and capability candidates by
`(OpKey, CapabilityKind, ProviderKey)`. Freeze rejects collisions and produces
canonical ordering. Planning selects providers through explicit compatibility,
feasibility, and cost logic; registration order never acts as precedence.

An operation can therefore be:

```text
semantically valid + reference evaluable + not lowerable
semantically valid + exactly decomposable + fusible after decomposition
semantically valid + direct access lowering + fusible
semantically valid + typed opaque physical implementation + fusion boundary
semantically valid + no executable capability + diagnosed unsupported
```

## Host contexts

Provider callbacks receive narrow borrowed or opaque contexts instead of the
compiler session or mutable graph. Contexts expose only phase-relevant queries,
budgets, builders, and diagnostic sinks. Proposed graphs/access models/kernel
fragments return to Tiler and enter the normal verifier before commit.

This prevents extension code from bypassing graph identity, numerical policy,
target feasibility, or transaction boundaries through a convenience API.

## Evolution rules

- Add a new optional capability without changing existing operation traits.
- Change semantic meaning with a new `OpKey.semantic_version`.
- Change output-affecting provider behavior with a new provider revision.
- Change the Rust calling contract with a new capability API version.
- Never infer compatibility from a Rust type name, crate version, or function
  address.
- Keep built-ins on the same registration path so the public boundary is
  continuously exercised.
