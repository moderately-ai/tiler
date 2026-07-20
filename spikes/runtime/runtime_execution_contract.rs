//! Dependency-free transition model for the consumer-neutral runtime contract.
//!
//! This is deliberately not a backend implementation. It makes routing
//! authority, commit, exact validation completion, stage order, named outputs,
//! and retention observable so their invariants can be tested.

use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct LiveDeviceKey {
    provider: &'static str,
    runtime_instance: u64,
    device_token: u64,
    context_scope: u64,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PipelineCacheKey {
    device: LiveDeviceKey,
    payload_digest: &'static str,
    entry: &'static str,
    specialization: Vec<u64>,
    descriptor_digest: &'static str,
    runtime_mode: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FailureStage {
    PipelinePreparation,
    RoutingCommit,
    Allocation,
    Encoding(&'static str),
    Completion(u64),
    ValidationReadback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FailureKind {
    FatalPreparation,
    StalePreparedSelection,
    AllocationFailure,
    EncodingFailure,
    DeviceExecutionFailure,
    SemanticValidationError,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RuntimeFailure {
    stage: FailureStage,
    kind: FailureKind,
    routing_committed: bool,
    enforcement_committed: bool,
    last_encoded_stage: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PrepareOutcome {
    Ready,
    CapabilityMiss,
    Fatal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TerminalStatus {
    Completed,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ValidationRecord {
    Valid,
    Invalid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ValidationPlan {
    None,
    DevicePreScan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Candidate {
    id: &'static str,
    stages: Vec<&'static str>,
    scratch: Vec<&'static str>,
    outputs: Vec<&'static str>,
    validation: ValidationPlan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PublishedOutput {
    key: &'static str,
    dependency_receipt: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Published {
    variant: &'static str,
    outputs: Vec<PublishedOutput>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ExecutionOutcome {
    /// The outer consumer still owns the operation and may execute its
    /// semantically equivalent fallback. No routing commit occurred.
    FallbackSelected {
        misses: Vec<&'static str>,
    },
    Published(Published),
}

#[derive(Debug)]
struct ScriptedAdapter {
    preparation: BTreeMap<&'static str, PrepareOutcome>,
    stale_at_commit: bool,
    allocation_failure: Option<&'static str>,
    encoding_failure: Option<&'static str>,
    validation_terminal: TerminalStatus,
    validation_record: ValidationRecord,
    next_receipt: u64,
    log: Vec<String>,
}

impl Default for ScriptedAdapter {
    fn default() -> Self {
        Self {
            preparation: BTreeMap::new(),
            stale_at_commit: false,
            allocation_failure: None,
            encoding_failure: None,
            validation_terminal: TerminalStatus::Completed,
            validation_record: ValidationRecord::Valid,
            next_receipt: 1,
            log: Vec::new(),
        }
    }
}

impl ScriptedAdapter {
    fn prepare(&mut self, candidate: &Candidate) -> PrepareOutcome {
        let outcome = self
            .preparation
            .get(candidate.id)
            .copied()
            .unwrap_or(PrepareOutcome::Ready);
        self.log
            .push(format!("prepare:{}:{outcome:?}", candidate.id));
        outcome
    }

    fn revalidate_for_commit(&mut self, candidate: &Candidate) -> bool {
        self.log.push(format!("revalidate:{}", candidate.id));
        !self.stale_at_commit
    }

    fn routing_commit(&mut self, candidate: &Candidate) {
        self.log.push(format!("routing-commit:{}", candidate.id));
    }

    fn allocate(&mut self, resource: &'static str) -> Result<(), ()> {
        self.log.push(format!("allocate:{resource}"));
        if self.allocation_failure == Some(resource) {
            Err(())
        } else {
            Ok(())
        }
    }

    fn encode(&mut self, stage: &'static str) -> Result<(), ()> {
        self.log.push(format!("encode:{stage}"));
        if self.encoding_failure == Some(stage) {
            Err(())
        } else {
            Ok(())
        }
    }

    fn submit(&mut self, label: &'static str) -> u64 {
        let receipt = self.next_receipt;
        self.next_receipt += 1;
        self.log.push(format!("submit:{label}:{receipt}"));
        receipt
    }

    fn retain_until(&mut self, receipt: u64, resources: &[&'static str]) {
        self.log
            .push(format!("retain:{receipt}:{}", resources.join(",")));
    }

    fn wait_exact(&mut self, receipt: u64) {
        self.log.push(format!("wait-exact:{receipt}"));
    }

    fn inspect_terminal_after_wait(&mut self, receipt: u64) -> TerminalStatus {
        self.log.push(format!(
            "inspect-terminal-after-wait:{receipt}:{:?}",
            self.validation_terminal
        ));
        self.validation_terminal
    }

    fn establish_coherence(&mut self, receipt: u64) {
        self.log.push(format!("cohere:error-record:{receipt}"));
    }

    fn read_validation_record(&mut self, receipt: u64) -> ValidationRecord {
        self.log.push(format!("read:error-record:{receipt}"));
        self.validation_record
    }

    fn publish(&mut self, keys: &[&'static str], receipt: u64) {
        self.log
            .push(format!("publish:{}:{receipt}", keys.join(",")));
    }
}

fn failure(
    stage: FailureStage,
    kind: FailureKind,
    routing_committed: bool,
    enforcement_committed: bool,
    last_encoded_stage: Option<&'static str>,
) -> RuntimeFailure {
    RuntimeFailure {
        stage,
        kind,
        routing_committed,
        enforcement_committed,
        last_encoded_stage,
    }
}

fn execute(
    candidates: &[Candidate],
    adapter: &mut ScriptedAdapter,
) -> Result<ExecutionOutcome, RuntimeFailure> {
    // The fallback authority remains outside this function until one candidate
    // is fully prepared. Capability misses can leave it intact; fatal errors
    // cannot be reinterpreted as misses.
    let mut misses = Vec::new();
    let selected = loop {
        let Some(candidate) = candidates.get(misses.len()) else {
            adapter.log.push("fallback-selected".into());
            return Ok(ExecutionOutcome::FallbackSelected { misses });
        };

        match adapter.prepare(candidate) {
            PrepareOutcome::Ready => break candidate,
            PrepareOutcome::CapabilityMiss => misses.push(candidate.id),
            PrepareOutcome::Fatal => {
                return Err(failure(
                    FailureStage::PipelinePreparation,
                    FailureKind::FatalPreparation,
                    false,
                    false,
                    None,
                ));
            }
        }
    };

    // Staleness is an invariant failure. A caller may explicitly restart a
    // top-level preflight, but this prepared launch never silently reroutes.
    if !adapter.revalidate_for_commit(selected) {
        return Err(failure(
            FailureStage::RoutingCommit,
            FailureKind::StalePreparedSelection,
            false,
            false,
            None,
        ));
    }

    // This transition consumes fallback authority.
    adapter.routing_commit(selected);
    let routing_committed = true;

    let mut resources = Vec::new();
    for output in &selected.outputs {
        adapter.allocate(output).map_err(|_| {
            failure(
                FailureStage::Allocation,
                FailureKind::AllocationFailure,
                routing_committed,
                false,
                None,
            )
        })?;
        resources.push(*output);
    }
    for scratch in &selected.scratch {
        adapter.allocate(scratch).map_err(|_| {
            failure(
                FailureStage::Allocation,
                FailureKind::AllocationFailure,
                routing_committed,
                false,
                None,
            )
        })?;
        resources.push(*scratch);
    }

    let mut enforcement_committed = false;
    if selected.validation == ValidationPlan::DevicePreScan {
        adapter.allocate("validation-record").map_err(|_| {
            failure(
                FailureStage::Allocation,
                FailureKind::AllocationFailure,
                routing_committed,
                false,
                None,
            )
        })?;
        resources.push("validation-record");
        enforcement_committed = true;

        adapter.encode("validator").map_err(|_| {
            failure(
                FailureStage::Encoding("validator"),
                FailureKind::EncodingFailure,
                routing_committed,
                enforcement_committed,
                None,
            )
        })?;
        let validation_receipt = adapter.submit("validation");
        adapter.retain_until(validation_receipt, &resources);

        // The ordering here is the contract: wait for this exact receipt, then
        // inspect its authoritative terminal status, then cohere and read.
        adapter.wait_exact(validation_receipt);
        if adapter.inspect_terminal_after_wait(validation_receipt) != TerminalStatus::Completed {
            return Err(failure(
                FailureStage::Completion(validation_receipt),
                FailureKind::DeviceExecutionFailure,
                routing_committed,
                enforcement_committed,
                Some("validator"),
            ));
        }
        adapter.establish_coherence(validation_receipt);
        if adapter.read_validation_record(validation_receipt) == ValidationRecord::Invalid {
            return Err(failure(
                FailureStage::ValidationReadback,
                FailureKind::SemanticValidationError,
                routing_committed,
                enforcement_committed,
                Some("validator"),
            ));
        }
    }

    let mut last_encoded = None;
    for stage in &selected.stages {
        adapter.encode(stage).map_err(|_| {
            failure(
                FailureStage::Encoding(stage),
                FailureKind::EncodingFailure,
                routing_committed,
                enforcement_committed,
                last_encoded,
            )
        })?;
        last_encoded = Some(*stage);
    }

    let result_receipt = adapter.submit("result");
    adapter.retain_until(result_receipt, &resources);
    adapter.publish(&selected.outputs, result_receipt);

    let outputs = selected
        .outputs
        .iter()
        .map(|key| PublishedOutput {
            key,
            dependency_receipt: result_receipt,
        })
        .collect();
    Ok(ExecutionOutcome::Published(Published {
        variant: selected.id,
        outputs,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(id: &'static str) -> Candidate {
        Candidate {
            id,
            stages: vec!["stage-0", "stage-1"],
            scratch: vec!["scratch"],
            outputs: vec!["scores", "summary"],
            validation: ValidationPlan::None,
        }
    }

    #[test]
    fn typed_capability_miss_routes_before_commit() {
        let candidates = [candidate("vector"), candidate("scalar")];
        let mut adapter = ScriptedAdapter::default();
        adapter
            .preparation
            .insert("vector", PrepareOutcome::CapabilityMiss);

        let outcome = execute(&candidates, &mut adapter).unwrap();
        let ExecutionOutcome::Published(published) = outcome else {
            panic!("expected publication");
        };
        assert_eq!(published.variant, "scalar");
        assert!(adapter
            .log
            .iter()
            .any(|event| event == "prepare:vector:CapabilityMiss"));
        assert!(!adapter
            .log
            .iter()
            .any(|event| event == "routing-commit:vector"));
        assert_eq!(
            adapter
                .log
                .iter()
                .filter(|event| event.starts_with("routing-commit:"))
                .count(),
            1
        );
    }

    #[test]
    fn all_capability_misses_leave_fallback_before_work() {
        let candidates = [candidate("vector"), candidate("scalar")];
        let mut adapter = ScriptedAdapter::default();
        adapter
            .preparation
            .insert("vector", PrepareOutcome::CapabilityMiss);
        adapter
            .preparation
            .insert("scalar", PrepareOutcome::CapabilityMiss);

        assert_eq!(
            execute(&candidates, &mut adapter).unwrap(),
            ExecutionOutcome::FallbackSelected {
                misses: vec!["vector", "scalar"]
            }
        );
        assert!(!adapter
            .log
            .iter()
            .any(|event| event.starts_with("allocate:") || event.starts_with("encode:")));
    }

    #[test]
    fn fatal_pipeline_error_does_not_fallback() {
        let candidates = [candidate("broken"), candidate("scalar")];
        let mut adapter = ScriptedAdapter::default();
        adapter.preparation.insert("broken", PrepareOutcome::Fatal);

        let error = execute(&candidates, &mut adapter).unwrap_err();
        assert_eq!(error.kind, FailureKind::FatalPreparation);
        assert!(!error.routing_committed);
        assert!(!adapter
            .log
            .iter()
            .any(|event| event == "prepare:scalar:Ready" || event == "fallback-selected"));
    }

    #[test]
    fn stale_preparation_fails_before_allocation_without_rerouting() {
        let candidates = [candidate("vector"), candidate("scalar")];
        let mut adapter = ScriptedAdapter {
            stale_at_commit: true,
            ..Default::default()
        };

        let error = execute(&candidates, &mut adapter).unwrap_err();
        assert_eq!(error.kind, FailureKind::StalePreparedSelection);
        assert!(!error.routing_committed);
        assert!(!adapter
            .log
            .iter()
            .any(|event| event.starts_with("allocate:")));
        assert!(!adapter
            .log
            .iter()
            .any(|event| event == "prepare:scalar:Ready" || event == "fallback-selected"));
    }

    #[test]
    fn allocation_failure_is_postcommit_and_cannot_fallback() {
        let candidates = [candidate("selected"), candidate("fallback-candidate")];
        let mut adapter = ScriptedAdapter {
            allocation_failure: Some("scratch"),
            ..Default::default()
        };

        let error = execute(&candidates, &mut adapter).unwrap_err();
        assert_eq!(error.kind, FailureKind::AllocationFailure);
        assert!(error.routing_committed);
        assert!(adapter
            .log
            .iter()
            .any(|event| event == "routing-commit:selected"));
        assert!(!adapter
            .log
            .iter()
            .any(|event| event == "prepare:fallback-candidate:Ready"));
    }

    #[test]
    fn partial_encoding_failure_is_terminal_for_the_route() {
        let candidates = [candidate("selected"), candidate("fallback-candidate")];
        let mut adapter = ScriptedAdapter {
            encoding_failure: Some("stage-1"),
            ..Default::default()
        };

        let error = execute(&candidates, &mut adapter).unwrap_err();
        assert_eq!(error.stage, FailureStage::Encoding("stage-1"));
        assert_eq!(error.last_encoded_stage, Some("stage-0"));
        assert!(error.routing_committed);
        assert!(!adapter.log.iter().any(
            |event| event == "prepare:fallback-candidate:Ready" || event == "fallback-selected"
        ));
    }

    #[test]
    fn validation_checks_exact_post_wait_status_before_readback() {
        let mut validated = candidate("validated");
        validated.validation = ValidationPlan::DevicePreScan;
        let mut adapter = ScriptedAdapter {
            validation_terminal: TerminalStatus::Error,
            ..Default::default()
        };

        let error = execute(&[validated], &mut adapter).unwrap_err();
        assert_eq!(error.kind, FailureKind::DeviceExecutionFailure);
        assert!(error.routing_committed);
        assert!(error.enforcement_committed);
        let wait = adapter
            .log
            .iter()
            .position(|event| event == "wait-exact:1")
            .unwrap();
        let terminal = adapter
            .log
            .iter()
            .position(|event| event == "inspect-terminal-after-wait:1:Error")
            .unwrap();
        assert!(wait < terminal);
        assert!(!adapter
            .log
            .iter()
            .any(|event| event.starts_with("cohere:") || event.starts_with("read:")));
    }

    #[test]
    fn semantic_validation_error_never_falls_back_or_encodes_results() {
        let mut validated = candidate("validated");
        validated.validation = ValidationPlan::DevicePreScan;
        let mut adapter = ScriptedAdapter {
            validation_record: ValidationRecord::Invalid,
            ..Default::default()
        };

        let error = execute(&[validated, candidate("alternate")], &mut adapter).unwrap_err();
        assert_eq!(error.kind, FailureKind::SemanticValidationError);
        assert!(error.routing_committed);
        assert!(error.enforcement_committed);
        assert!(!adapter
            .log
            .iter()
            .any(|event| event == "encode:stage-0" || event == "prepare:alternate:Ready"));
    }

    #[test]
    fn canonical_stage_and_named_output_order_are_preserved() {
        let selected = candidate("selected");
        let mut adapter = ScriptedAdapter::default();
        let outcome = execute(&[selected], &mut adapter).unwrap();
        let ExecutionOutcome::Published(published) = outcome else {
            panic!("expected publication");
        };

        let stage_0 = adapter
            .log
            .iter()
            .position(|event| event == "encode:stage-0")
            .unwrap();
        let stage_1 = adapter
            .log
            .iter()
            .position(|event| event == "encode:stage-1")
            .unwrap();
        assert!(stage_0 < stage_1);
        assert_eq!(
            published
                .outputs
                .iter()
                .map(|output| output.key)
                .collect::<Vec<_>>(),
            vec!["scores", "summary"]
        );
        assert!(published
            .outputs
            .iter()
            .all(|output| output.dependency_receipt == 1));
    }

    #[test]
    fn scratch_is_retained_against_the_result_receipt() {
        let mut adapter = ScriptedAdapter::default();
        execute(&[candidate("selected")], &mut adapter).unwrap();
        let submit = adapter
            .log
            .iter()
            .position(|event| event == "submit:result:1")
            .unwrap();
        let retain = adapter
            .log
            .iter()
            .position(|event| event == "retain:1:scores,summary,scratch")
            .unwrap();
        let publish = adapter
            .log
            .iter()
            .position(|event| event == "publish:scores,summary:1")
            .unwrap();
        assert!(submit < retain && retain < publish);
    }

    #[test]
    fn pipeline_cache_identity_is_device_and_specialization_scoped() {
        let base = PipelineCacheKey {
            device: LiveDeviceKey {
                provider: "metal",
                runtime_instance: 7,
                device_token: 2,
                context_scope: 11,
            },
            payload_digest: "payload-a",
            entry: "reduce",
            specialization: vec![32],
            descriptor_digest: "descriptor-a",
            runtime_mode: "default",
        };
        let mut other_device = base.clone();
        other_device.device.runtime_instance = 8;
        let mut other_specialization = base.clone();
        other_specialization.specialization = vec![64];

        assert_ne!(base, other_device);
        assert_ne!(base, other_specialization);
    }
}
