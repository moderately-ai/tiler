//! Executable model for residual semantic-precondition enforcement.
//!
//! This is a CPU model of control, visibility, and accounting boundaries. It is
//! not a GPU performance model. Run with:
//!
//! `rustc --edition 2021 --test spikes/runtime/semantic_validation_enforcement.rs -o /tmp/tiler-validation-tests && /tmp/tiler-validation-tests`
//! `rustc -O --edition 2021 spikes/runtime/semantic_validation_enforcement.rs -o /tmp/tiler-validation-bench && /tmp/tiler-validation-bench`

#![allow(dead_code)]

use std::hint::black_box;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const NO_ERROR: u64 = u64::MAX;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WitnessId(u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SubjectId(u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ViewId(u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ValueVersion(u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ObligationOrdinal(u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WitnessDependency {
    witness: WitnessId,
    subject: SubjectId,
    view: ViewId,
    version: ValueVersion,
    obligation: ObligationOrdinal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ProvenWitness {
    dependency: WitnessDependency,
}

impl ProvenWitness {
    // In a library this constructor belongs to the verifier, not the caller API.
    fn verifier_established(dependency: WitnessDependency) -> Self {
        Self { dependency }
    }

    fn discharges(self, required: WitnessDependency) -> bool {
        self.dependency == required
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum StableErrorCode {
    Nan = 1,
    Infinite = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SemanticError {
    logical_linear_index: u64,
    code: StableErrorCode,
    obligation: ObligationOrdinal,
}

impl SemanticError {
    fn priority_key(self) -> u64 {
        assert!(self.logical_linear_index < (1_u64 << 48));
        (self.logical_linear_index << 16) | ((self.code as u64) << 8) | self.obligation.0 as u64
    }

    fn from_priority_key(key: u64) -> Option<Self> {
        if key == NO_ERROR {
            return None;
        }
        let code = match ((key >> 8) & 0xff) as u8 {
            1 => StableErrorCode::Nan,
            2 => StableErrorCode::Infinite,
            other => panic!("unknown stable error code {other}"),
        };
        Some(Self {
            logical_linear_index: key >> 16,
            code,
            obligation: ObligationOrdinal((key & 0xff) as u8),
        })
    }
}

fn violation(value: f32, index: usize, obligation: ObligationOrdinal) -> Option<SemanticError> {
    let code = if value.is_nan() {
        StableErrorCode::Nan
    } else if value.is_infinite() {
        StableErrorCode::Infinite
    } else {
        return None;
    };
    Some(SemanticError {
        logical_linear_index: index as u64,
        code,
        obligation,
    })
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Metrics {
    validation_elements: u64,
    compute_elements: u64,
    input_bytes_read: u64,
    private_bytes_written: u64,
    published_bytes: u64,
    publication_copy_bytes: u64,
    discarded_private_bytes: u64,
    dispatches: u32,
    completion_observations: u32,
    error_record_bytes: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommitState {
    Unrouted,
    RoutingCommitted,
    EnforcementCommitted,
    CompletionObserved,
    PublicationCommitted,
    Failed,
}

#[derive(Debug, Eq, PartialEq)]
enum ExecutionError {
    Semantic {
        error: SemanticError,
        metrics: Metrics,
    },
    CompletionFailure,
    InvalidProof,
    FallbackAfterEnforcement,
}

#[derive(Debug)]
struct Execution {
    state: CommitState,
    fallback_attempts: u32,
}

impl Execution {
    fn routed() -> Self {
        Self {
            state: CommitState::RoutingCommitted,
            fallback_attempts: 0,
        }
    }

    fn begin_enforcement(&mut self) {
        assert_eq!(self.state, CommitState::RoutingCommitted);
        self.state = CommitState::EnforcementCommitted;
    }

    fn observe_completion(&mut self, terminal_success: bool) -> Result<(), ExecutionError> {
        assert_eq!(self.state, CommitState::EnforcementCommitted);
        if !terminal_success {
            self.state = CommitState::Failed;
            return Err(ExecutionError::CompletionFailure);
        }
        self.state = CommitState::CompletionObserved;
        Ok(())
    }

    fn publish(&mut self) {
        assert_eq!(self.state, CommitState::CompletionObserved);
        self.state = CommitState::PublicationCommitted;
    }

    fn publish_proven(&mut self) {
        assert_eq!(self.state, CommitState::RoutingCommitted);
        self.state = CommitState::PublicationCommitted;
    }

    fn fail_semantic(&mut self, error: SemanticError, metrics: Metrics) -> ExecutionError {
        assert!(matches!(
            self.state,
            CommitState::EnforcementCommitted | CommitState::CompletionObserved
        ));
        self.state = CommitState::Failed;
        ExecutionError::Semantic { error, metrics }
    }

    fn try_fallback(&mut self) -> Result<(), ExecutionError> {
        self.fallback_attempts += 1;
        if matches!(
            self.state,
            CommitState::EnforcementCommitted
                | CommitState::CompletionObserved
                | CommitState::PublicationCommitted
                | CommitState::Failed
        ) {
            return Err(ExecutionError::FallbackAfterEnforcement);
        }
        Ok(())
    }
}

#[derive(Debug)]
struct RunResult {
    public_output: Option<Vec<f32>>,
    metrics: Metrics,
    state: CommitState,
}

fn compute(value: f32) -> f32 {
    value.mul_add(2.0, 1.0)
}

fn proof_elided(
    input: &[f32],
    proof: ProvenWitness,
    required: WitnessDependency,
) -> Result<RunResult, ExecutionError> {
    if !proof.discharges(required) {
        return Err(ExecutionError::InvalidProof);
    }
    let mut execution = Execution::routed();
    // No runtime enforcement exists: the proof is a compiler-owned fact.
    let output: Vec<_> = input.iter().copied().map(compute).collect();
    execution.publish_proven();
    Ok(RunResult {
        public_output: Some(output),
        metrics: Metrics {
            compute_elements: input.len() as u64,
            input_bytes_read: std::mem::size_of_val(input) as u64,
            published_bytes: std::mem::size_of_val(input) as u64,
            dispatches: 1,
            ..Metrics::default()
        },
        state: execution.state,
    })
}

fn host_scan(input: &[f32], obligation: ObligationOrdinal) -> Result<RunResult, ExecutionError> {
    let mut execution = Execution::routed();
    execution.begin_enforcement();
    let mut metrics = Metrics::default();
    for (index, value) in input.iter().copied().enumerate() {
        metrics.validation_elements += 1;
        metrics.input_bytes_read += std::mem::size_of::<f32>() as u64;
        if let Some(error) = violation(value, index, obligation) {
            return Err(execution.fail_semantic(error, metrics));
        }
    }
    execution.state = CommitState::CompletionObserved;
    let output: Vec<_> = input.iter().copied().map(compute).collect();
    metrics.compute_elements = input.len() as u64;
    metrics.input_bytes_read += std::mem::size_of_val(input) as u64;
    metrics.published_bytes = std::mem::size_of_val(input) as u64;
    metrics.dispatches = 1;
    execution.publish();
    Ok(RunResult {
        public_output: Some(output),
        metrics,
        state: execution.state,
    })
}

fn parallel_error_scan(
    input: &[f32],
    chunks: usize,
    obligation: ObligationOrdinal,
) -> Option<SemanticError> {
    let best = AtomicU64::new(NO_ERROR);
    let chunk_len = input.len().div_ceil(chunks.max(1));
    thread::scope(|scope| {
        for (chunk_number, chunk) in input.chunks(chunk_len.max(1)).enumerate() {
            let best = &best;
            scope.spawn(move || {
                let start = chunk_number * chunk_len;
                for (offset, value) in chunk.iter().copied().enumerate() {
                    if let Some(error) = violation(value, start + offset, obligation) {
                        best.fetch_min(error.priority_key(), Ordering::Relaxed);
                    }
                }
            });
        }
    });
    SemanticError::from_priority_key(best.load(Ordering::Acquire))
}

fn device_pre_scan(
    input: &[f32],
    chunks: usize,
    obligation: ObligationOrdinal,
    terminal_success: bool,
) -> Result<RunResult, ExecutionError> {
    let mut execution = Execution::routed();
    execution.begin_enforcement();
    let error = parallel_error_scan(input, chunks, obligation);
    let mut metrics = Metrics {
        validation_elements: input.len() as u64,
        input_bytes_read: std::mem::size_of_val(input) as u64,
        dispatches: 1,
        completion_observations: 1,
        error_record_bytes: std::mem::size_of::<u64>() as u32,
        ..Metrics::default()
    };
    execution.observe_completion(terminal_success)?;
    if let Some(error) = error {
        return Err(execution.fail_semantic(error, metrics));
    }
    let output: Vec<_> = input.iter().copied().map(compute).collect();
    metrics.compute_elements = input.len() as u64;
    metrics.input_bytes_read += std::mem::size_of_val(input) as u64;
    metrics.published_bytes = std::mem::size_of_val(input) as u64;
    metrics.dispatches += 1;
    execution.publish();
    Ok(RunResult {
        public_output: Some(output),
        metrics,
        state: execution.state,
    })
}

fn transactional_device(
    input: &[f32],
    chunks: usize,
    obligation: ObligationOrdinal,
    terminal_success: bool,
) -> Result<RunResult, ExecutionError> {
    let mut execution = Execution::routed();
    execution.begin_enforcement();
    let best = AtomicU64::new(NO_ERROR);
    let mut private = vec![0.0_f32; input.len()];
    let chunk_len = input.len().div_ceil(chunks.max(1));
    thread::scope(|scope| {
        for (chunk_number, (source, destination)) in input
            .chunks(chunk_len.max(1))
            .zip(private.chunks_mut(chunk_len.max(1)))
            .enumerate()
        {
            let best = &best;
            scope.spawn(move || {
                let start = chunk_number * chunk_len;
                for (offset, (value, output)) in source
                    .iter()
                    .copied()
                    .zip(destination.iter_mut())
                    .enumerate()
                {
                    if let Some(error) = violation(value, start + offset, obligation) {
                        best.fetch_min(error.priority_key(), Ordering::Relaxed);
                    }
                    *output = compute(value);
                }
            });
        }
    });
    let bytes = std::mem::size_of_val(input) as u64;
    let mut metrics = Metrics {
        validation_elements: input.len() as u64,
        compute_elements: input.len() as u64,
        input_bytes_read: bytes,
        private_bytes_written: bytes,
        dispatches: 1,
        completion_observations: 1,
        error_record_bytes: std::mem::size_of::<u64>() as u32,
        ..Metrics::default()
    };
    execution.observe_completion(terminal_success)?;
    if let Some(error) = SemanticError::from_priority_key(best.load(Ordering::Acquire)) {
        metrics.discarded_private_bytes = bytes;
        drop(private);
        return Err(execution.fail_semantic(error, metrics));
    }
    // Ownership promotion is the modeled zero-copy publication mode.
    metrics.published_bytes = bytes;
    execution.publish();
    Ok(RunResult {
        public_output: Some(private),
        metrics,
        state: execution.state,
    })
}

fn dependency() -> WitnessDependency {
    WitnessDependency {
        witness: WitnessId(7),
        subject: SubjectId(11),
        view: ViewId(13),
        version: ValueVersion(17),
        obligation: ObligationOrdinal(0),
    }
}

fn median_duration(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[samples.len() / 2]
}

fn time_strategy(
    mut strategy: impl FnMut() -> Result<RunResult, ExecutionError>,
    iterations: usize,
) -> Duration {
    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let result = black_box(strategy()).expect("valid benchmark input");
        black_box(result.public_output.unwrap());
        samples.push(start.elapsed());
    }
    median_duration(samples)
}

fn main() {
    println!("elements,strategy,median_us,validation_elements,input_bytes,private_bytes,dispatches,observations");
    for &elements in &[65_536_usize, 1_048_576, 8_388_608] {
        let input = vec![0.25_f32; elements];
        let iterations = if elements < 1_000_000 { nine() } else { 5 };
        let required = dependency();
        let proof = ProvenWitness::verifier_established(required);
        let strategies: [(&str, Box<dyn Fn() -> Result<RunResult, ExecutionError>>); 4] = [
            ("proof", Box::new(|| proof_elided(&input, proof, required))),
            ("host", Box::new(|| host_scan(&input, ObligationOrdinal(0)))),
            (
                "pre_scan",
                Box::new(|| device_pre_scan(&input, 8, ObligationOrdinal(0), true)),
            ),
            (
                "transactional",
                Box::new(|| transactional_device(&input, 8, ObligationOrdinal(0), true)),
            ),
        ];
        for (name, strategy) in strategies {
            let sample = strategy().unwrap();
            let elapsed = time_strategy(strategy, iterations);
            println!(
                "{elements},{name},{},{},{},{},{},{}",
                elapsed.as_nanos() / 1_000,
                sample.metrics.validation_elements,
                sample.metrics.input_bytes_read,
                sample.metrics.private_bytes_written,
                sample.metrics.dispatches,
                sample.metrics.completion_observations,
            );
        }
    }
}

const fn nine() -> usize {
    9
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_paths_have_the_same_result() {
        let input = [0.0, -2.5, 9.0, 4.25];
        let required = dependency();
        let proof = ProvenWitness::verifier_established(required);
        let expected = proof_elided(&input, proof, required).unwrap().public_output;
        assert_eq!(
            host_scan(&input, ObligationOrdinal(0))
                .unwrap()
                .public_output,
            expected
        );
        assert_eq!(
            device_pre_scan(&input, 3, ObligationOrdinal(0), true)
                .unwrap()
                .public_output,
            expected
        );
        assert_eq!(
            transactional_device(&input, 3, ObligationOrdinal(0), true)
                .unwrap()
                .public_output,
            expected
        );
    }

    #[test]
    fn parallel_errors_choose_canonical_logical_priority() {
        let input = [0.0, f32::INFINITY, f32::NAN, f32::NEG_INFINITY];
        let expected = SemanticError {
            logical_linear_index: 1,
            code: StableErrorCode::Infinite,
            obligation: ObligationOrdinal(0),
        };
        for chunks in 1..=8 {
            for _ in 0..20 {
                assert_eq!(
                    parallel_error_scan(&input, chunks, ObligationOrdinal(0)),
                    Some(expected)
                );
            }
        }
    }

    #[test]
    fn invalid_host_and_pre_scan_never_compute_or_publish() {
        let input = [1.0, f32::NAN, 3.0];
        assert!(matches!(
            host_scan(&input, ObligationOrdinal(0)),
            Err(ExecutionError::Semantic { .. })
        ));
        assert!(matches!(
            device_pre_scan(&input, 2, ObligationOrdinal(0), true),
            Err(ExecutionError::Semantic { .. })
        ));
    }

    #[test]
    fn transactional_failure_discards_private_result() {
        let input = [1.0, f32::NAN, 3.0];
        let Err(ExecutionError::Semantic { metrics, .. }) =
            transactional_device(&input, 2, ObligationOrdinal(0), true)
        else {
            panic!("expected semantic validation failure");
        };
        assert_eq!(metrics.discarded_private_bytes, 12);
        assert_eq!(metrics.published_bytes, 0);
    }

    #[test]
    fn terminal_execution_failure_precedes_error_record_interpretation() {
        let input = [f32::NAN];
        assert!(matches!(
            device_pre_scan(&input, 1, ObligationOrdinal(0), false),
            Err(ExecutionError::CompletionFailure)
        ));
        assert!(matches!(
            transactional_device(&input, 1, ObligationOrdinal(0), false),
            Err(ExecutionError::CompletionFailure)
        ));
    }

    #[test]
    fn fallback_is_closed_after_enforcement_commit() {
        let mut execution = Execution::routed();
        execution.begin_enforcement();
        assert_eq!(
            execution.try_fallback(),
            Err(ExecutionError::FallbackAfterEnforcement)
        );
        assert_eq!(execution.fallback_attempts, 1);
    }

    #[test]
    fn witness_reuse_requires_exact_dependency_provenance() {
        let required = dependency();
        let proof = ProvenWitness::verifier_established(required);
        let mut changed_version = required;
        changed_version.version = ValueVersion(18);
        let mut changed_view = required;
        changed_view.view = ViewId(14);
        assert!(proof.discharges(required));
        assert!(!proof.discharges(changed_version));
        assert!(!proof.discharges(changed_view));
    }

    #[test]
    fn accounting_exposes_extra_pass_and_private_storage() {
        let input = [1.0, 2.0, 3.0, 4.0];
        let host = host_scan(&input, ObligationOrdinal(0)).unwrap().metrics;
        let pre = device_pre_scan(&input, 2, ObligationOrdinal(0), true)
            .unwrap()
            .metrics;
        let txn = transactional_device(&input, 2, ObligationOrdinal(0), true)
            .unwrap()
            .metrics;
        assert_eq!(host.validation_elements, 4);
        assert_eq!(pre.dispatches, 2);
        assert_eq!(pre.completion_observations, 1);
        assert_eq!(txn.dispatches, 1);
        assert_eq!(txn.private_bytes_written, 16);
        assert_eq!(txn.publication_copy_bytes, 0);
    }
}
