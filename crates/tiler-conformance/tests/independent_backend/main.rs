// `variant_count` sizes `Subject::ALL`, so a subject added to the vocabulary
// and not to the run below is an array-length error at the declaration rather
// than a census that silently stops covering it.
#![feature(variant_count)]
//! A second, independently authored backend over the portfolio's own subjects.
//!
//! # What this suite is evidence for
//!
//! `decide-the-backend-provider-conformance-harness-public-surface` was
//! accepted on 2026-08-18 as an exact typed deferral, and its first reopening
//! trigger is sufficient on its own: *a bounded extraction demonstrates two
//! independently authored backend fixtures using one device-free structural
//! subject and one execution subject*, and additionally *proves typed host
//! unavailability, caller-owned execution policy, `tiler-reference` as the sole
//! mathematical oracle, and adapter-owned terminal resource lifetime*.
//!
//! What that decision found missing was not a fixture. It was evidence that a
//! subject exists which is **not tailored to one backend's private shape** —
//! the retained fixtures each exercise a different seam, so nothing shows the
//! same subject surviving a second author. This suite is the second author.
//! It states the two subjects, exercises both against a backend that shares no
//! code, no payload format, no execution model, and no numerical claim with
//! any backend already in the tree, and — because a subject only two agreeing
//! fixtures ever passed would prove only that they agree — watches each subject
//! refuse a backend that tries to certify itself.
//!
//! ## The two subjects, stated before they are exercised
//!
//! **The device-free structural subject** is that *a producer cannot state a
//! fact the plan already decided.* Not "the artifact validates", which any
//! backend could satisfy by being careful: the claim is that the seam gives a
//! producer no place to put such a fact. `assemble_plan_artifact` derives the
//! target-profile reference and descriptor digest, the feasibility rule set and
//! revision, the compilation environment, every selected capability provider,
//! every deferred prepared-entry predicate, and each entry's `BackendEntryKey`,
//! and it takes no parameter through which a backend could offer any of them.
//! What a backend does state is exactly what no plan decides: which payloads
//! the entries resolve through, each binding's transport category, whether a
//! zero-thread launch skips its dispatch, and what must hold at launch.
//!
//! **The execution subject** is that *a routed result's verdict comes from
//! `tiler-reference` and never from the adapter.* An adapter reports and the
//! loader compares; the arithmetic is compared against an oracle that never saw
//! the backend. An adapter's `Ok` is not evidence of anything, which is why a
//! `Certify` adapter below reaches terminal success and still fails.
//!
//! Neither subject needs an optional responsibility field, a whole-backend
//! provider trait, a parsed diagnostic, or a callback that can manufacture
//! success — the four defects the decision's candidate D1 was eliminated for.
//!
//! ## What makes this independently authored rather than a rename
//!
//! Renaming `crates/tiler-build/tests/custom_backend` would satisfy the words
//! and defeat the purpose, so the differences are named and each is checkable:
//!
//! | Choice the seam left open | `tests/custom_backend` | this backend |
//! | --- | --- | --- |
//! | payload model | symbol, transport list, and work-item count per entry | a single-assignment node table with a declarative store plan |
//! | control flow | none carried; the image describes entries, not bodies | predication is one optional guard ordinal on the store plan |
//! | evaluation | none; the image is never executed | demand-driven, one forward pass, dead nodes never evaluated |
//! | framing | big-endian | little-endian |
//! | entry symbol | derived per family from the target triple | positional, carrying no identity, and reaching no digest crate |
//! | execution host | none | a worker thread the adapter owns, acquired before the commit |
//! | numerics | scalar host, subnormals preserved | the same, and *therefore* this workload carries subnormals |
//! | delivery positions | two families under one profile | one, and the profile declares one triple |
//!
//! Two of those rows are convergence rather than difference, and saying so is
//! the point. Both backends declare `SubnormalMode::Preserve` exact and refuse
//! both flushing modes, because that is what an honest host-arithmetic
//! interpreter under `STRICT_F32` can say — a second author reaching the same
//! declaration is evidence about the profile vocabulary, not about copying.
//! Both also declare a non-identity transport mapping; see
//! [`nodefold::transport_of`] for why that hazard is real and why finding it
//! twice is the stronger reading.
//!
//! ## Why every file here is named something no other file in the tree is
//!
//! `make citations` resolves a pinned citation by unique path suffix, so a new
//! file whose basename an existing file already carries silently turns every
//! citation written against that basename ambiguous — and the checker then
//! stops checking them. This suite's first `oracle.rs` did exactly that to
//! `crates/tiler-reference/src/oracle.rs`, and ten live citations across four
//! research documents failed the gate on it. The four files a directory-based
//! integration test may name freely are therefore named for this backend and
//! checked against `git ls-files` for uniqueness; `main.rs` is Cargo's and is
//! already a suffix twenty-seven tracked files carry.
//!
//! ## What this suite does not claim
//!
//! It is not a conformance harness and exports nothing: it is an integration
//! test, so it compiles against public surfaces alone and cannot reach a
//! `pub(crate)` item of any crate — including this one, whose library has no
//! public surface at all. It does not decide the facade the deferred decision
//! holds; it supplies the evidence that decision named as sufficient to reopen
//! it. It measures no host and states no performance claim. Its arithmetic
//! coverage is twelve `f32` patterns of one pointwise program, which is a
//! bounded refutation attempt and not certification of anything.

mod nodefold;
mod nodefold_adapter;
mod nodefold_graph;
mod workload;

use std::collections::BTreeMap;

use tiler_artifact::program::{
    ArithmeticType, RecordedArtifactProgramIdentity, TargetProfileDescriptorDigest,
    TargetProfileKey, TargetProfileRef, decode_artifact,
};
use tiler_compiler::session::{
    Compilation, CompileRequest, NumericalContract, PlanAlternative, compile,
};
use tiler_compiler::target::{DTypeDispatchability, TargetRequest};
use tiler_ir::semantic::SemanticProgram;
use tiler_runtime::load::DTypeDispatch;

use nodefold_adapter::{
    Behaviour, Binding, Evaluation, ExecutionOutcome, HostPolicy, HostRequest, Lifetime, RouteEnd,
    agrees_with_reference, apply_policy, route,
};
use nodefold::{EntryPerturbation, NodefoldRefusal, Produced};
use nodefold_graph::{GRAPH_DOMAIN, GRAPH_SCHEMA, GraphRefusal};

/// The two subjects this suite exists to exercise.
///
/// Sized from the type rather than by hand: a third subject added here without
/// a row in [`SUBJECT_COVERAGE`] fails to compile at the array length, where a
/// hand-written count would have let the census quietly stop covering it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Subject {
    /// A producer cannot state a fact the plan already decided.
    Structural,
    /// A routed result's verdict comes from the oracle and never the adapter.
    Execution,
}

impl Subject {
    const ALL: [Self; std::mem::variant_count::<Self>()] = [Self::Structural, Self::Execution];
}

/// Which case exercises each subject soundly, and which perturbs it.
///
/// Read by [`both_subjects_are_exercised_and_each_is_perturbed`], which prints
/// the census so a reader can watch it move.
const SUBJECT_COVERAGE: [(Subject, &str, &str); 2] = [
    (
        Subject::Structural,
        "the_assembled_artifact_carries_facts_this_backend_never_supplied",
        "a_backend_minted_entry_key_is_refused",
    ),
    (
        Subject::Execution,
        "the_routed_result_agrees_with_the_reference_oracle",
        "a_self_certifying_adapter_reaches_terminal_success_and_still_fails",
    ),
];

/// Compiles this fixture's program against this backend's own declared profile.
fn compiled(program: &SemanticProgram) -> Compilation {
    let profile = nodefold::nodefold_profile().expect("the nodefold profile declares");
    compile(CompileRequest::new(
        program,
        NumericalContract::STRICT_F32,
        TargetRequest::new([profile]).expect("a singleton target request"),
    ))
    .expect("the program compiles against the nodefold profile")
    .into_targets()
    .pop()
    .expect("one target outcome")
    .into_parts()
    .1
    .expect("the nodefold target compiles")
}

/// Returns the target-profile reference the artifact derives, read from the plan.
fn profile_ref(compilation: &Compilation) -> TargetProfileRef {
    TargetProfileRef {
        key: TargetProfileKey::new(compilation.target_profile_key())
            .expect("the compiled profile key is governed"),
        descriptor: TargetProfileDescriptorDigest::from_bytes(
            compilation.target_profile_descriptor(),
        )
        .expect("the compiled profile descriptor is well formed"),
    }
}

/// Returns what this backend's target family states about dtype dispatch.
fn dtype_dispatch() -> BTreeMap<ArithmeticType, DTypeDispatch> {
    nodefold::DTYPE_ROWS
        .into_iter()
        .map(|(dtype, verdict)| {
            let arithmetic = match dtype {
                tiler_ir::kernel::KernelType::F32 => ArithmeticType::F32,
                other => panic!("this backend declares no dispatchability for {other:?}"),
            };
            let dispatch = match verdict {
                DTypeDispatchability::Dispatchable => DTypeDispatch::Dispatchable,
                DTypeDispatchability::Unsupported => DTypeDispatch::Unsupported,
            };
            (arithmetic, dispatch)
        })
        .collect()
}

fn plan(compilation: &Compilation) -> PlanAlternative<'_> {
    compilation
        .selected()
        .expect("the compilation retained a selected alternative")
}

fn produce(perturbation: EntryPerturbation) -> Result<Produced, NodefoldRefusal> {
    let program = workload::program();
    let compilation = compiled(&program);
    nodefold::assemble(&program, plan(&compilation), perturbation)
}

fn sound() -> Produced {
    produce(EntryPerturbation::Derived).expect("the sound production path completes")
}

fn expected_identity(produced: &Produced) -> RecordedArtifactProgramIdentity {
    RecordedArtifactProgramIdentity::from_bytes(produced.artifact.canonical_identity().as_bytes())
        .expect("the assembled identity is well formed")
}

fn run(behaviour: Behaviour, produced: &Produced) -> RouteEnd {
    let program = workload::program();
    let compilation = compiled(&program);
    route(
        &produced.bytes,
        &expected_identity(produced),
        profile_ref(&compilation),
        dtype_dispatch(),
        &workload::OPERANDS,
        behaviour,
    )
}

// ---------------------------------------------------------------------------
// The device-free structural subject
// ---------------------------------------------------------------------------

/// Every fact below was derived from the plan, and none was offered by this backend.
#[test]
fn the_assembled_artifact_carries_facts_this_backend_never_supplied() {
    let program = workload::program();
    let compilation = compiled(&program);
    let produced = sound();
    let decoded = decode_artifact(&produced.bytes)
        .expect("the assembled envelope decodes");

    assert_eq!(
        decoded.variants().len(),
        1,
        "one compilation packaged one variant",
    );
    let variant = decoded.variants().next().expect("one packaged variant");
    assert_eq!(
        variant.target_profile().key.as_str(),
        nodefold::PROFILE_KEY,
        "the artifact names the profile this backend declared and the plan compiled against",
    );
    assert_eq!(
        variant.target_profile().descriptor.as_bytes(),
        compilation.target_profile_descriptor(),
        "the descriptor digest is the compilation's own, byte for byte",
    );
    assert_eq!(
        variant.feasibility_rules().key.as_str(),
        compilation.feasibility_rule_set_key(),
        "the feasibility rule set is the compilation's",
    );
    assert_eq!(
        variant.feasibility_rules().revision,
        compilation.feasibility_rule_set_revision(),
        "the feasibility revision is the compilation's",
    );
    assert_eq!(
        variant.deferred_predicates().len(),
        0,
        "this backend's workgroup capacity is a compile-time profile fact, so the compiler defers \
         no prepared-entry predicate; a non-zero count here would mean it had learned to",
    );

    let stage_keys: Vec<Vec<u8>> = plan(&compilation)
        .kernels()
        .iter()
        .map(|kernel| kernel.canonical_identity().as_bytes().to_vec())
        .collect();
    let packaged: Vec<Vec<u8>> = variant
        .entries()
        .map(|entry| entry.backend_entry_key().as_bytes().to_vec())
        .collect();
    assert_eq!(
        packaged, stage_keys,
        "each packaged entry is keyed by its own stage kernel's canonical identity, which this \
         backend has no parameter through which to supply",
    );
}

/// Two assemblies of one plan produce the same bytes and the same identity.
#[test]
fn two_assemblies_of_one_plan_are_byte_identical() {
    let first = sound();
    let second = sound();
    assert_eq!(
        first.artifact.canonical_identity().as_bytes(),
        second.artifact.canonical_identity().as_bytes(),
        "one plan and one payload derive one canonical identity",
    );
    assert_eq!(
        first.bytes, second.bytes,
        "and one envelope, byte for byte; a difference here would be a fact leaking in from \
         somewhere other than the plan and the payload",
    );
    // Each `sound()` rebuilds the semantic program and recompiles it, so this
    // compares two independent derivations rather than one cached one. A
    // fixture that held a single built program would only be comparing the
    // encoder against itself.
}

/// A key this backend minted for itself, in place of the plan's, is refused.
///
/// The self-certifying case for the structural subject: the producer states an
/// entry identity rather than transporting the one the stage kernel decided.
#[test]
fn a_backend_minted_entry_key_is_refused() {
    let refusal = match produce(EntryPerturbation::ForgedEntryKey) {
        Err(refusal) => refusal.to_string(),
        Ok(produced) => match decode_artifact(&produced.bytes) {
            Err(rejection) => rejection.to_string(),
            Ok(_) => panic!(
                "a forged entry key assembled and decoded; the packaged entry's identity is then \
                 whatever the producer said it was, and the structural subject does not hold",
            ),
        },
    };
    eprintln!("forged entry key refused: {refusal}");
    assert!(
        refusal.contains("UnmappedBackendEntry"),
        "the refusal must name the packaged entry that reached no mapping, which is what makes \
         the key the plan's rather than the producer's; it said: {refusal}",
    );
}

/// A symbol the emitted graph does not carry reaches the backend and nothing earlier.
///
/// ADR 0090 item 8 in one case: the envelope's framing, digests, schema,
/// canonical order, arena closure, and identity all hold — the artifact is
/// structurally perfect — and only this backend's own `validate_payload` can
/// see that the mapping names nothing it can execute.
#[test]
fn an_unmapped_symbol_is_caught_by_this_backend_and_by_nothing_above_it() {
    let produced = produce(EntryPerturbation::UnmappedSymbol)
        .expect("an unmapped symbol is not an assembly-time defect");
    decode_artifact(&produced.bytes)
        .expect("nor a decode-time one: the envelope is sound and the bytes are opaque to it");
    let end = run(Behaviour::sound(HostPolicy::Require), &produced);
    match end {
        RouteEnd::AdapterRefused(refusal) => {
            let rendered = refusal.to_string();
            eprintln!("unmapped symbol refused by the backend: {rendered}");
            assert!(
                rendered.contains("names no graph entry"),
                "the backend must say the mapping named nothing it carries; it said: {rendered}",
            );
        }
        other => panic!("the unmapped symbol was not refused by the backend: {other:?}"),
    }
}

/// Every refusal this representation names is reachable from a byte run.
///
/// A decoder whose refusal vocabulary has unreachable members is one whose
/// validation obligations are partly decorative, and this backend's validation
/// is the one thing the artifact layer provably cannot discharge for it. The
/// population is sized with `variant_count` rather than counted by hand, so a
/// refusal added to the vocabulary and left unreached fails here instead of
/// leaving a census that has quietly stopped covering its own domain.
#[test]
fn every_named_graph_refusal_is_reachable_from_bytes() {
    let program = workload::program();
    let compilation = compiled(&program);
    let emitted = nodefold::emitted_object(plan(&compilation)).expect("the plan translates");
    nodefold_graph::decode(&emitted).expect("the emitted graph decodes");

    // The four framing refusals, reached by perturbing the emitted bytes.
    let mut foreign = emitted.clone();
    foreign[0] ^= 0xff;
    let mut versioned = emitted.clone();
    versioned[GRAPH_DOMAIN.len()] = GRAPH_SCHEMA.0.to_le_bytes()[0].wrapping_add(9);
    let mut trailing = emitted.clone();
    trailing.push(0);

    // The first node tag of a hand-built graph, which is the one byte that
    // cannot be reached by perturbing a structure — every other tag value is
    // either valid or unreachable behind an earlier check.
    let mut unknown_tag = nodefold_graph::encode(&minimal(|_| {}));
    unknown_tag[first_node_tag(&minimal(|_| {}))] = 0xff;

    let observed = [
        refusal(&foreign),
        refusal(&versioned),
        refusal(&emitted[..emitted.len() - 1]),
        refusal(&trailing),
        refusal(&unknown_tag),
        refusal(&nodefold_graph::encode(&minimal(|entry| entry.symbol.clear()))),
        refusal(&nodefold_graph::encode(&minimal(|entry| {
            entry.nodes[1] = nodefold_graph::Node::IndexAdd(0, 9);
        }))),
        refusal(&nodefold_graph::encode(&minimal(|entry| {
            entry.nodes[3] = nodefold_graph::Node::F32Multiply(0, 0);
        }))),
        refusal(&nodefold_graph::encode(&minimal(|entry| entry.store.buffer = 9))),
        refusal(&nodefold_graph::encode(&minimal(|entry| {
            entry.nodes[3] = nodefold_graph::Node::Load {
                buffer: 9,
                offset: 0,
            };
        }))),
        refusal(&nodefold_graph::encode(&minimal(|entry| entry.buffers.clear()))),
        refusal(&nodefold_graph::encode(&minimal(|entry| entry.store.buffer = 0))),
    ];

    let mut distinct: Vec<std::mem::Discriminant<GraphRefusal>> = Vec::new();
    for found in &observed {
        let kind = std::mem::discriminant(found);
        if !distinct.contains(&kind) {
            distinct.push(kind);
        }
    }
    eprintln!("graph refusal census: {} reached, {observed:?}", distinct.len());
    assert_eq!(
        distinct.len(),
        std::mem::variant_count::<GraphRefusal>(),
        "this backend names {named} refusal(s) and {reached} distinct one(s) were reached from \
         bytes; an unreachable refusal is a validation obligation that is not being performed",
        named = std::mem::variant_count::<GraphRefusal>(),
        reached = distinct.len(),
    );
}

/// Decodes one byte run that must not decode, and returns what refused it.
fn refusal(bytes: &[u8]) -> GraphRefusal {
    match nodefold_graph::decode(bytes) {
        Err(refusal) => refusal,
        Ok(_) => panic!("a byte run built to be refused decoded instead"),
    }
}

/// The smallest well-formed graph this backend would execute, after one edit.
///
/// Built here rather than emitted so a case can move exactly one field. The
/// unedited value decodes, which the first assertion below relies on: an edit
/// that stopped mattering would otherwise look like a reached refusal.
fn minimal(edit: impl FnOnce(&mut nodefold_graph::GraphEntry)) -> nodefold_graph::Graph {
    let mut entry = nodefold_graph::GraphEntry {
        symbol: "nodefold.minimal".to_owned(),
        canonical_nan: 0x7fc0_0000,
        buffers: vec![
            nodefold_graph::GraphBuffer {
                write: false,
                element_count: 4,
            },
            nodefold_graph::GraphBuffer {
                write: true,
                element_count: 4,
            },
        ],
        nodes: vec![
            nodefold_graph::Node::InvocationIndex,
            nodefold_graph::Node::IndexConstant(4),
            nodefold_graph::Node::IndexLessThan(0, 1),
            nodefold_graph::Node::Load {
                buffer: 0,
                offset: 0,
            },
        ],
        store: nodefold_graph::StorePlan {
            guard: Some(2),
            buffer: 1,
            offset: 0,
            value: 3,
        },
    };
    edit(&mut entry);
    nodefold_graph::Graph {
        entries: vec![entry],
    }
}

/// Byte offset of the first node tag in one encoded single-entry graph.
///
/// Derived from the framing this backend writes rather than searched for, so a
/// framing change moves it here and does not leave the case flipping an
/// unrelated byte that happens to decode.
fn first_node_tag(graph: &nodefold_graph::Graph) -> usize {
    let entry = &graph.entries[0];
    GRAPH_DOMAIN.len()
        + 4                        // schema major and minor
        + 4                        // entry count
        + 4                        // symbol length
        + entry.symbol.len()
        + 4                        // canonical NaN
        + 4                        // buffer count
        + entry.buffers.len() * 9  // one write flag and one element count each
        + 4 // node count
}

// ---------------------------------------------------------------------------
// The execution subject
// ---------------------------------------------------------------------------

/// The routed result agrees with `tiler-reference`, which is the only oracle here.
#[test]
fn the_routed_result_agrees_with_the_reference_oracle() {
    let produced = sound();
    let reference = workload::reference_bits(&workload::program());
    let outcome = apply_policy(
        HostPolicy::Require,
        run(Behaviour::sound(HostPolicy::Require), &produced),
    )
    .expect("this caller required the execution host and this host supplied it");
    let bits = outcome
        .completed()
        .expect("a completed route publishes its bits");
    assert_ne!(
        reference.as_slice(),
        workload::OPERANDS.as_slice(),
        "the oracle must transform its input; a program whose reference output equalled its \
         operands would let a backend that copied its input through compare equal",
    );
    let compared = agrees_with_reference(bits, &reference)
        .unwrap_or_else(|disagreement| panic!("the routed result disagrees: {disagreement}"));
    assert_eq!(
        compared,
        workload::OPERANDS.len(),
        "every operand's output was compared, not a prefix of them",
    );
    eprintln!("nodefold route: {compared} element(s) equal to tiler-reference");
}

/// An adapter that reports terminal success without evaluating still fails.
///
/// The self-certifying case for the execution subject. This adapter takes the
/// route to its commit, receives the storage, reports its terminal use, and
/// never folds anything. Everything the adapter itself could be asked about is
/// green; the verdict comes from the oracle and the oracle refuses it.
#[test]
fn a_self_certifying_adapter_reaches_terminal_success_and_still_fails() {
    let produced = sound();
    let reference = workload::reference_bits(&workload::program());
    let behaviour = Behaviour {
        evaluation: Evaluation::Certify,
        ..Behaviour::sound(HostPolicy::Require)
    };
    let outcome = apply_policy(HostPolicy::Require, run(behaviour, &produced))
        .expect("a self-certifying adapter completes its route, which is the whole problem");
    let bits = outcome
        .completed()
        .expect("it publishes bits like any other completion");
    let disagreement = agrees_with_reference(bits, &reference)
        .expect_err("a suite that passed this fixture would prove only that it agrees with itself");
    eprintln!("self-certifying adapter refused by the oracle: {disagreement}");
    assert_eq!(
        disagreement.index, 0,
        "the first operand already disagrees, so nothing about this depends on which element is \
         compared first",
    );
    assert_eq!(
        disagreement.produced, 0,
        "the certifying adapter published the storage exactly as it was allocated",
    );
    assert_ne!(
        disagreement.required, 0,
        "and the oracle requires something else",
    );
}

/// An adapter that returns from `dispatch` while work is outstanding fails.
#[test]
fn an_adapter_that_returns_before_terminal_use_fails_its_own_witness() {
    let produced = sound();
    let behaviour = Behaviour {
        lifetime: Lifetime::ReturnBeforeTerminalUse,
        ..Behaviour::sound(HostPolicy::Require)
    };
    match run(behaviour, &produced) {
        RouteEnd::Failed(failure) => {
            let rendered = failure.to_string();
            eprintln!("early return refused: {rendered}");
            assert!(
                rendered.contains("terminal use(s) were witnessed"),
                "the failure must name the outstanding work; it said: {rendered}",
            );
        }
        other => panic!("returning before terminal use was not a failure: {other:?}"),
    }
}

/// An adapter that reports a representation it cannot decode is refused by the loader.
///
/// The adapter reports and the loader compares. An adapter that could decide
/// its own eligibility on the way to an answer would make "this host cannot
/// execute these bytes" and "this artifact is for another target" one outcome.
#[test]
fn an_adapter_that_reports_what_it_prefers_is_refused_by_the_loader() {
    let produced = sound();
    let behaviour = Behaviour {
        binding: Binding::Preferred,
        ..Behaviour::sound(HostPolicy::Require)
    };
    match run(behaviour, &produced) {
        RouteEnd::Refused(rejection) => {
            eprintln!("preferred representation refused by the loader: {rejection}");
        }
        other => panic!("a preferred representation was not refused by the loader: {other:?}"),
    }
}

/// An unavailable execution host is a typed outcome with no path to a pass.
#[test]
fn an_unavailable_execution_host_is_typed_and_cannot_pass() {
    let produced = sound();

    // The caller that reports unavailability gets an outcome, and the outcome
    // publishes no bits: `completed` answers `None`, and the oracle comparison
    // takes bits, so there is no expression that reaches it from here.
    let reporting = Behaviour {
        host: HostRequest::unsatisfiable(HostPolicy::Report),
        ..Behaviour::sound(HostPolicy::Report)
    };
    let outcome = apply_policy(HostPolicy::Report, run(reporting, &produced))
        .expect("a reporting caller receives an outcome rather than an error");
    match &outcome {
        ExecutionOutcome::Unavailable(unavailable) => {
            eprintln!("execution host unavailable: {unavailable}");
            assert!(
                unavailable.reason.contains("execution thread"),
                "the outcome must name what the host could not supply; it said: {reason}",
                reason = unavailable.reason,
            );
        }
        ExecutionOutcome::Completed(_) => {
            panic!("this host satisfied a request no host should satisfy")
        }
    }
    assert!(
        outcome.completed().is_none(),
        "an unavailable host publishes no bits, so nothing can compare it against the oracle",
    );

    // The caller that requires the host gets an error. The adapter's report is
    // identical in both runs; only the caller's policy differs, and no ambient
    // environment variable is consulted in either.
    let requiring = Behaviour {
        host: HostRequest::unsatisfiable(HostPolicy::Require),
        ..Behaviour::sound(HostPolicy::Require)
    };
    let error = apply_policy(HostPolicy::Require, run(requiring, &produced))
        .expect_err("a requiring caller does not accept an unavailable host");
    eprintln!("requiring caller: {error}");
    assert!(
        error.contains("required the execution host"),
        "the error must say the policy was the caller's; it said: {error}",
    );
}

/// A transport map that disagrees with the payload is caught only by the oracle.
///
/// Every identity in this artifact is the plan's, so nothing above the backend
/// has anything to compare: the mapping is a statement no plan makes, and the
/// bytes it disagrees with are opaque to every layer that could read them. The
/// route completes, the adapter reports terminal success, and the arithmetic is
/// wrong — which is the clearest statement of why the verdict has to come from
/// an oracle that never saw the backend.
#[test]
fn a_transport_map_that_disagrees_with_the_payload_is_caught_only_by_the_oracle() {
    let produced = produce(EntryPerturbation::IdentityTransports)
        .expect("a disagreeing transport map is not an assembly-time defect");
    decode_artifact(&produced.bytes)
        .expect("nor a decode-time one: every packaged identity is still the plan's");
    let reference = workload::reference_bits(&workload::program());
    let outcome = apply_policy(
        HostPolicy::Require,
        run(Behaviour::sound(HostPolicy::Require), &produced),
    )
    .expect("the route completes; the bindings are placed, just not where the graph reads them");
    let bits = outcome
        .completed()
        .expect("a completed route publishes its bits");
    let disagreement = agrees_with_reference(bits, &reference)
        .expect_err("a suite without an independent oracle would report this route as a pass");
    eprintln!("disagreeing transport map refused by the oracle: {disagreement}");
}

// ---------------------------------------------------------------------------
// The census
// ---------------------------------------------------------------------------

/// Both subjects are exercised soundly and both are perturbed.
#[test]
fn both_subjects_are_exercised_and_each_is_perturbed() {
    assert_eq!(
        SUBJECT_COVERAGE.len(),
        Subject::ALL.len(),
        "every subject this suite names has a coverage row",
    );
    for subject in Subject::ALL {
        let row = SUBJECT_COVERAGE
            .iter()
            .find(|(named, _, _)| *named == subject)
            .unwrap_or_else(|| panic!("{subject:?} has no coverage row"));
        assert!(
            !row.1.is_empty() && !row.2.is_empty(),
            "{subject:?} needs both a sound case and a perturbation",
        );
    }
    eprintln!("subject census: {SUBJECT_COVERAGE:?}");
}
