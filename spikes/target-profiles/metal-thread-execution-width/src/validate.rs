//! Unchanged assertions over a retained width record.

use std::collections::BTreeSet;
use std::path::Path;

use crate::population::{
    METRIC, PIPELINE_COUNT, PIPELINES, RECORD_SCHEMA, REPETITIONS, enumeration_counts, spec_by_id,
};
use crate::record::{
    PipelineRecord, WidthRecord, cargo_lock_sha256, harness_source_sha256, kernel_sha256,
};

/// One validation failure. The text is the assertion a perturbation must trip.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationFailure {
    /// Exact message retained in the ticket when a perturbation is watched.
    pub message: String,
}

impl ValidationFailure {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Checks a retained record against the freeze and the tree.
#[must_use]
pub fn validate(record: &WidthRecord, root: &Path) -> Vec<ValidationFailure> {
    let mut failures = Vec::new();
    if record.schema != RECORD_SCHEMA {
        failures.push(ValidationFailure::new(format!(
            "schema is {}, expected {RECORD_SCHEMA}",
            record.schema
        )));
    }
    if record.metric != METRIC {
        failures.push(ValidationFailure::new(format!(
            "metric is {}, expected {METRIC}",
            record.metric
        )));
    }
    if record.repetitions != REPETITIONS {
        failures.push(ValidationFailure::new(format!(
            "repetitions is {}, expected {REPETITIONS}",
            record.repetitions
        )));
    }
    if record.frozen_pipeline_count != PIPELINE_COUNT {
        failures.push(ValidationFailure::new(format!(
            "frozen pipeline count is {}, expected {PIPELINE_COUNT}",
            record.frozen_pipeline_count
        )));
    }
    let (kernels, selections, shapes) = enumeration_counts();
    if kernels == 0 || selections == 0 || shapes == 0 {
        failures.push(ValidationFailure::new("a freeze enumeration is empty"));
    }
    validate_environment(record, &mut failures);
    validate_custody(record, root, &mut failures);
    validate_pipeline_identities(record, &mut failures);
    validate_result_population(record, &mut failures);
    validate_verdict(record, &mut failures);
    failures
}

fn validate_environment(record: &WidthRecord, failures: &mut Vec<ValidationFailure>) {
    let env = &record.environment;
    for (key, value) in [
        ("offline_metal", env.offline_metal.as_str()),
        ("offline_linker", env.offline_linker.as_str()),
        ("offline_xcode", env.offline_xcode.as_str()),
        ("offline_sdk_version", env.offline_sdk_version.as_str()),
        ("offline_sdk_build", env.offline_sdk_build.as_str()),
        ("rustc_verbose", env.rustc_verbose.as_str()),
        ("platform_version", env.platform_version.as_str()),
        ("platform_build", env.platform_build.as_str()),
        ("architecture", env.architecture.as_str()),
        ("device", env.device.as_str()),
        ("device_registry_id", env.device_registry_id.as_str()),
        ("load_averages", env.load_averages.as_str()),
    ] {
        if value.is_empty() {
            failures.push(ValidationFailure::new(format!(
                "environment.{key} is missing or empty"
            )));
        }
    }
    if !env.apple9 {
        failures.push(ValidationFailure::new(
            "environment.apple9 is not true; the record is not an Apple9 observation",
        ));
    }
    if record.environment_sha256 != env.custody_digest() {
        failures.push(ValidationFailure::new(
            "environment digest does not match the recorded environment subject",
        ));
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn validate_custody(record: &WidthRecord, root: &Path, failures: &mut Vec<ValidationFailure>) {
    let expected_harness = harness_source_sha256(root);
    if record.custody.harness_source_sha256 != expected_harness {
        failures.push(ValidationFailure::new(
            "harness source digest does not match the tree",
        ));
    }
    let expected_lock = cargo_lock_sha256(root);
    if record.custody.cargo_lock_sha256 != expected_lock {
        failures.push(ValidationFailure::new(
            "Cargo.lock digest does not match the tree",
        ));
    }
    if !is_sha256(&record.custody.starting_executable_sha256)
        || !is_sha256(&record.custody.ending_executable_sha256)
    {
        failures.push(ValidationFailure::new(
            "executable digest is not a 64-hex custody value",
        ));
    }
    if record.custody.starting_executable_sha256 != record.custody.ending_executable_sha256 {
        failures.push(ValidationFailure::new(
            "ending executable digest does not match retained custody",
        ));
    }
    for spec in PIPELINES {
        let name = spec.kernel.name();
        let expected = kernel_sha256(root, name);
        match record.custody.kernel_sha256.get(name) {
            Some(observed) if observed == &expected => {}
            Some(_) => failures.push(ValidationFailure::new(format!(
                "kernel {name} digest does not match the tree"
            ))),
            None => failures.push(ValidationFailure::new(format!(
                "kernel {name} digest is missing from custody"
            ))),
        }
    }
}

fn validate_pipeline_identities(record: &WidthRecord, failures: &mut Vec<ValidationFailure>) {
    let frozen: BTreeSet<String> = PIPELINES.iter().map(PipelineSpecExt::id).collect();
    let mut seen = BTreeSet::new();
    for pipeline in &record.pipelines {
        if !frozen.contains(&pipeline.id) {
            failures.push(ValidationFailure::new(format!(
                "pipeline identity {} is not in the frozen population",
                pipeline.id
            )));
            continue;
        }
        if !seen.insert(pipeline.id.clone()) {
            failures.push(ValidationFailure::new(format!(
                "pipeline identity {} is duplicated",
                pipeline.id
            )));
        }
        let Some(spec) = spec_by_id(&pipeline.id) else {
            continue;
        };
        let expected_flags: Vec<String> = spec
            .compiler
            .metal_flags()
            .iter()
            .map(|flag| (*flag).to_owned())
            .collect();
        if pipeline.kernel != spec.kernel.name()
            || pipeline.compiler_selection != spec.compiler.name()
            || pipeline.descriptor != spec.descriptor.name()
            || pipeline.role != spec.role.name()
            || pipeline.required != spec.required
            || pipeline.compiler_flags != expected_flags
        {
            failures.push(ValidationFailure::new(format!(
                "pipeline identity {} does not match the frozen spec",
                pipeline.id
            )));
        }
    }
    for id in frozen {
        if !seen.contains(&id) {
            failures.push(ValidationFailure::new(format!(
                "frozen pipeline {id} is missing from the record"
            )));
        }
    }
}

trait PipelineSpecExt {
    fn id(&self) -> String;
}

impl PipelineSpecExt for crate::population::PipelineSpec {
    fn id(&self) -> String {
        crate::population::PipelineSpec::id(*self)
    }
}

fn validate_result_population(record: &WidthRecord, failures: &mut Vec<ValidationFailure>) {
    if record.pipelines.len() != PIPELINE_COUNT {
        failures.push(ValidationFailure::new(format!(
            "result population has {} pipelines, expected {PIPELINE_COUNT}",
            record.pipelines.len()
        )));
    }
    for pipeline in &record.pipelines {
        validate_one_population(pipeline, failures);
    }
}

fn validate_one_population(pipeline: &PipelineRecord, failures: &mut Vec<ValidationFailure>) {
    if pipeline.compile.status == "ok" {
        if pipeline.preparations.len() != REPETITIONS {
            failures.push(ValidationFailure::new(format!(
                "result population for {} has {} preparations, expected {REPETITIONS}",
                pipeline.id,
                pipeline.preparations.len()
            )));
        }
        for (index, prep) in pipeline.preparations.iter().enumerate() {
            let expected = u32::try_from(index + 1).expect("repetition fits u32");
            if prep.repetition != expected {
                failures.push(ValidationFailure::new(format!(
                    "result population for {} repetition order is not 1..{REPETITIONS}",
                    pipeline.id
                )));
            }
            if pipeline.required && prep.status != "ok" {
                failures.push(ValidationFailure::new(format!(
                    "required pipeline {} did not prepare",
                    pipeline.id
                )));
            }
            if prep.status == "ok" && prep.thread_execution_width.is_none() {
                failures.push(ValidationFailure::new(format!(
                    "result population for {} is missing thread_execution_width",
                    pipeline.id
                )));
            }
        }
    } else if pipeline.required {
        failures.push(ValidationFailure::new(format!(
            "required pipeline {} did not compile",
            pipeline.id
        )));
    } else if !pipeline.preparations.is_empty() {
        failures.push(ValidationFailure::new(format!(
            "result population for {} retained preparations after a failed compile",
            pipeline.id
        )));
    }
}

fn validate_verdict(record: &WidthRecord, failures: &mut Vec<ValidationFailure>) {
    let derived = WidthRecord::derive_verdict(&record.pipelines);
    if record.verdict != derived {
        failures.push(ValidationFailure::new(
            "verdict does not match the retained observations; no modal, first, or fallback width is admissible",
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{accepted_fixture, spike_root};

    #[test]
    fn the_accepted_fixture_validates() {
        let root = spike_root();
        let record = accepted_fixture(&root);
        assert_eq!(validate(&record, &root), Vec::new());
    }

    #[test]
    fn pipeline_identity_perturbation_fails() {
        let root = spike_root();
        let mut record = accepted_fixture(&root);
        record.pipelines[0].id = "not-a-frozen-identity".to_owned();
        let messages: Vec<_> = validate(&record, &root)
            .into_iter()
            .map(|failure| failure.message)
            .collect();
        assert!(
            messages.iter().any(|message| message
                == "pipeline identity not-a-frozen-identity is not in the frozen population"),
            "{messages:?}"
        );
    }

    #[test]
    fn result_population_perturbation_fails() {
        let root = spike_root();
        let mut record = accepted_fixture(&root);
        let required = record
            .pipelines
            .iter_mut()
            .find(|pipeline| pipeline.required)
            .expect("the freeze has a required pipeline");
        let id = required.id.clone();
        required.preparations.pop();
        record.verdict = WidthRecord::derive_verdict(&record.pipelines);
        let messages: Vec<_> = validate(&record, &root)
            .into_iter()
            .map(|failure| failure.message)
            .collect();
        assert!(
            messages.iter().any(|message| message
                == &format!(
                    "result population for {id} has 2 preparations, expected {REPETITIONS}"
                )),
            "{messages:?}"
        );
    }

    #[test]
    fn environment_perturbation_fails() {
        let root = spike_root();
        let mut record = accepted_fixture(&root);
        record.environment.device = "Apple M4 Max".to_owned();
        let messages: Vec<_> = validate(&record, &root)
            .into_iter()
            .map(|failure| failure.message)
            .collect();
        assert!(
            messages.iter().any(|message| {
                message == "environment digest does not match the recorded environment subject"
            }),
            "{messages:?}"
        );
    }

    #[test]
    fn executable_custody_perturbation_fails() {
        let root = spike_root();
        let mut record = accepted_fixture(&root);
        record.custody.ending_executable_sha256 = "f".repeat(64);
        let messages: Vec<_> = validate(&record, &root)
            .into_iter()
            .map(|failure| failure.message)
            .collect();
        assert!(
            messages.iter().any(|message| {
                message == "ending executable digest does not match retained custody"
            }),
            "{messages:?}"
        );
    }
}
