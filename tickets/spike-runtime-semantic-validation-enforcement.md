---
id: spike-runtime-semantic-validation-enforcement
title: Spike runtime semantic validation enforcement
status: done
priority: p1
dependencies: []
related: [numerical-policy-contract, define-initial-affine-quantization-semantics]
scopes: [research/runtime, contracts/artifacts, contracts/foundation]
shared_scopes: []
paths: []
tags: [tiler-research, runtime, spike]
---
Prototype and measure proof-elided, host/pre-scan, and transactional device enforcement for tensor-value SemanticPreconditions. Specify witness dependencies, deterministic error records, completion observation, private-result publication, no-fallback boundaries, and cost inputs. Keep runtime-profile capability separate from semantic identity.

## Outcome

Delivered the [semantic-enforcement research](../docs/research/runtime/semantic-validation-enforcement.md)
and [executable models](../spikes/runtime/README.md). The work defines witness
identity and commit boundaries; backend-specific GPU enforcement remains future implementation.

## Evidence correction (2026-07-21)

The [runtime experiment repair](repair-shape-and-runtime-experiment-integrity.md)
and [current report](../docs/research/runtime/semantic-validation-enforcement.md)
bind the benchmark claims to the retained samples and recorded `arm64` macOS
27 host metadata. Earlier unretained M4 Max and logical-CPU attributions are not
part of the evidence.
