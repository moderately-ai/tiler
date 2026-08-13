//! Retained JSON record and custody hashes.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::population::{CompilerSelection, DescriptorShape, PipelineSpec, REPETITIONS};
#[cfg(test)]
use crate::population::{METRIC, PIPELINE_COUNT, PIPELINES, RECORD_SCHEMA};

/// SHA-256 of `bytes` as lowercase hex.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Directory that contains `Cargo.toml` and `kernels/`.
#[must_use]
pub fn spike_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// SHA-256 of one kernel file.
#[must_use]
pub fn kernel_sha256(root: &Path, name: &str) -> String {
    let path = root.join("kernels").join(format!("{name}.metal"));
    sha256_hex(&fs::read(path).expect("kernel source is readable"))
}

/// SHA-256 of every `src/*.rs` file concatenated in name order.
#[must_use]
pub fn harness_source_sha256(root: &Path) -> String {
    let mut paths: Vec<PathBuf> = fs::read_dir(root.join("src"))
        .expect("src/ is readable")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .collect();
    paths.sort();
    let mut hasher = Sha256::new();
    for path in paths {
        let name = path
            .file_name()
            .expect("a src path has a file name")
            .to_string_lossy();
        hasher.update(format!("// FILE: {name}\n").as_bytes());
        hasher.update(fs::read(path).expect("a src file is readable"));
    }
    format!("{:x}", hasher.finalize())
}

/// SHA-256 of the spike `Cargo.lock`.
#[must_use]
pub fn cargo_lock_sha256(root: &Path) -> String {
    sha256_hex(&fs::read(root.join("Cargo.lock")).expect("Cargo.lock is readable"))
}

/// SHA-256 of the running executable.
#[must_use]
pub fn executable_sha256() -> String {
    let path = std::env::current_exe().expect("the running executable has a path");
    sha256_hex(&fs::read(path).expect("the running executable is readable"))
}

/// Observed host and toolchain. Every field is required and non-empty.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Environment {
    /// `xcrun metal --version` first line.
    pub offline_metal: String,
    /// `xcrun metallib -version` first line.
    pub offline_linker: String,
    /// `xcodebuild -version` collapsed to one line.
    pub offline_xcode: String,
    /// `xcrun --sdk macosx --show-sdk-version`.
    pub offline_sdk_version: String,
    /// `xcrun --sdk macosx --show-sdk-build-version`.
    pub offline_sdk_build: String,
    /// `rustc -vV`.
    pub rustc_verbose: String,
    /// `sw_vers -productVersion`.
    pub platform_version: String,
    /// `sw_vers -buildVersion`.
    pub platform_build: String,
    /// `uname -m`.
    pub architecture: String,
    /// `MTLDevice.name`.
    pub device: String,
    /// `MTLDevice.registryID` in hex.
    pub device_registry_id: String,
    /// Whether `supportsFamily(Apple9)` was true.
    pub apple9: bool,
    /// `MTLDevice.maxBufferLength`.
    pub max_buffer_length: u64,
    /// `uptime` load averages, recorded rather than gated.
    pub load_averages: String,
}

impl Environment {
    /// Canonical JSON used as the environment custody subject.
    #[must_use]
    pub fn custody_digest(&self) -> String {
        sha256_hex(&serde_json::to_vec(self).expect("environment serializes"))
    }
}

/// Source, lock, and executable custody.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[expect(
    clippy::struct_field_names,
    reason = "each field is a SHA-256 of a different custody subject"
)]
pub struct Custody {
    /// Concatenated `src/*.rs` digest.
    pub harness_source_sha256: String,
    /// `Cargo.lock` digest.
    pub cargo_lock_sha256: String,
    /// Per-kernel source digests keyed by kernel name.
    pub kernel_sha256: BTreeMap<String, String>,
    /// Digest of the measuring executable at process start.
    pub starting_executable_sha256: String,
    /// Digest of the same path after the last preparation.
    pub ending_executable_sha256: String,
}

/// One compile attempt of one kernel under one selection.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompileObservation {
    /// `ok` or `failed`.
    pub status: String,
    /// Linked metallib digest when `status` is `ok`.
    pub metallib_sha256: Option<String>,
    /// Compiler stderr when `status` is `failed`.
    pub stderr: Option<String>,
}

/// One `newComputePipelineState` attempt.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreparationObservation {
    /// 1-based repetition index.
    pub repetition: u32,
    /// `ok` or `failed`.
    pub status: String,
    /// The metric, present only on `ok`.
    pub thread_execution_width: Option<u64>,
    /// Corroborating prepared fact, not a substitute.
    pub max_total_threads_per_threadgroup: Option<u64>,
    /// Corroborating prepared fact, not a substitute.
    pub static_threadgroup_memory_length: Option<u64>,
    /// Prepare error when `status` is `failed`.
    pub error: Option<String>,
}

/// One frozen identity and every observation taken for it.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PipelineRecord {
    /// `{kernel}/{compiler}/{descriptor}`.
    pub id: String,
    /// Role label from the freeze.
    pub role: String,
    /// Kernel file stem / entry point.
    pub kernel: String,
    /// Arithmetic label.
    pub arithmetic: String,
    /// Operation-family label.
    pub operation_family: String,
    /// Compilation-selection identity.
    pub compiler_selection: String,
    /// Exact `xcrun metal` flags.
    pub compiler_flags: Vec<String>,
    /// Descriptor-shape identity.
    pub descriptor: String,
    /// Whether a compile/prepare failure aborts the run.
    pub required: bool,
    /// Kernel source digest.
    pub source_sha256: String,
    /// Compile observation.
    pub compile: CompileObservation,
    /// One observation per repetition when compile succeeded.
    pub preparations: Vec<PreparationObservation>,
}

/// Derived from the retained observations. Never a modal or first-value field.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Verdict {
    /// How many frozen identities compiled and prepared every repetition.
    pub prepared_ok_count: usize,
    /// Identities whose compile failed. Optional only.
    pub compile_failed: Vec<String>,
    /// Identities whose compile succeeded but a prepare failed.
    pub prepare_failed: Vec<String>,
    /// Sorted unique prepared widths. Empty if nothing prepared.
    pub widths_observed: Vec<u64>,
    /// True only when every successful preparation reported the same width.
    pub all_prepared_widths_equal: bool,
}

/// The complete retained record.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WidthRecord {
    /// Schema id.
    pub schema: String,
    /// Exact metric name.
    pub metric: String,
    /// Independent constructions per identity.
    pub repetitions: usize,
    /// Frozen population size.
    pub frozen_pipeline_count: usize,
    /// Host and toolchain.
    pub environment: Environment,
    /// Digest of the environment object.
    pub environment_sha256: String,
    /// Source and executable custody.
    pub custody: Custody,
    /// Every frozen identity, in freeze order.
    pub pipelines: Vec<PipelineRecord>,
    /// Derived equality or variation.
    pub verdict: Verdict,
}

impl WidthRecord {
    /// Builds the derived verdict from the pipeline rows. No substitution.
    #[must_use]
    pub fn derive_verdict(pipelines: &[PipelineRecord]) -> Verdict {
        let mut widths = BTreeSet::new();
        let mut prepared_ok = 0;
        let mut compile_failed = Vec::new();
        let mut prepare_failed = Vec::new();
        for pipeline in pipelines {
            if pipeline.compile.status != "ok" {
                compile_failed.push(pipeline.id.clone());
                continue;
            }
            let ok = pipeline
                .preparations
                .iter()
                .filter(|prep| prep.status == "ok")
                .count();
            if ok == REPETITIONS && pipeline.preparations.len() == REPETITIONS {
                prepared_ok += 1;
            } else {
                prepare_failed.push(pipeline.id.clone());
            }
            for prep in &pipeline.preparations {
                if let Some(width) = prep.thread_execution_width {
                    widths.insert(width);
                }
            }
        }
        let widths_observed: Vec<u64> = widths.into_iter().collect();
        let all_prepared_widths_equal = widths_observed.len() == 1;
        Verdict {
            prepared_ok_count: prepared_ok,
            compile_failed,
            prepare_failed,
            widths_observed,
            all_prepared_widths_equal,
        }
    }
}

/// Applies a descriptor shape to a Metal pipeline descriptor.
pub fn apply_descriptor(descriptor: &metal::ComputePipelineDescriptor, shape: DescriptorShape) {
    if let Some(max) = shape.max_total_threads() {
        descriptor.set_max_total_threads_per_threadgroup(max);
    }
    if shape.multiple_of_width() {
        descriptor.set_thread_group_size_is_multiple_of_thread_execution_width(true);
    }
}

/// Flags as owned strings for the record.
#[must_use]
pub fn compiler_flag_strings(selection: CompilerSelection) -> Vec<String> {
    selection
        .metal_flags()
        .iter()
        .map(|flag| (*flag).to_owned())
        .collect()
}

/// Completes a pipeline row's identity fields from the freeze.
#[must_use]
pub fn identity_record(spec: PipelineSpec, source_sha256: String) -> PipelineRecord {
    PipelineRecord {
        id: spec.id(),
        role: spec.role.name().to_owned(),
        kernel: spec.kernel.name().to_owned(),
        arithmetic: spec.kernel.arithmetic().to_owned(),
        operation_family: spec.kernel.operation_family().to_owned(),
        compiler_selection: spec.compiler.name().to_owned(),
        compiler_flags: compiler_flag_strings(spec.compiler),
        descriptor: spec.descriptor.name().to_owned(),
        required: spec.required,
        source_sha256,
        compile: CompileObservation {
            status: "unrecorded".to_owned(),
            metallib_sha256: None,
            stderr: None,
        },
        preparations: Vec::new(),
    }
}

/// Skeleton record used by validation tests. Not a measured result.
#[cfg(test)]
#[must_use]
pub fn accepted_fixture(root: &Path) -> WidthRecord {
    let mut kernel_sha256_map = BTreeMap::new();
    for spec in PIPELINES {
        kernel_sha256_map
            .entry(spec.kernel.name().to_owned())
            .or_insert_with(|| kernel_sha256(root, spec.kernel.name()));
    }
    let environment = Environment {
        offline_metal: "Apple metal version 32023.883 (metalfe-32023.883)".to_owned(),
        offline_linker: "AIR-LLD 32023.883 (metalfe-32023.883)".to_owned(),
        offline_xcode: "Xcode 26.6 Build version 17F113".to_owned(),
        offline_sdk_version: "26.5".to_owned(),
        offline_sdk_build: "25F70".to_owned(),
        rustc_verbose: "rustc 1.99.0-nightly (fixture)".to_owned(),
        platform_version: "27.0".to_owned(),
        platform_build: "26A5388g".to_owned(),
        architecture: "arm64".to_owned(),
        device: "Apple M3 Pro".to_owned(),
        device_registry_id: "0x1".to_owned(),
        apple9: true,
        max_buffer_length: 1,
        load_averages: "0.00 0.00 0.00".to_owned(),
    };
    let exe = "0".repeat(64);
    let pipelines: Vec<PipelineRecord> = PIPELINES
        .iter()
        .map(|spec| {
            let mut row = identity_record(*spec, kernel_sha256(root, spec.kernel.name()));
            if spec.required {
                row.compile = CompileObservation {
                    status: "ok".to_owned(),
                    metallib_sha256: Some("a".repeat(64)),
                    stderr: None,
                };
                row.preparations = (1..=REPETITIONS)
                    .map(|repetition| PreparationObservation {
                        repetition: u32::try_from(repetition).expect("repetition fits u32"),
                        status: "ok".to_owned(),
                        thread_execution_width: Some(32),
                        max_total_threads_per_threadgroup: Some(1024),
                        static_threadgroup_memory_length: Some(0),
                        error: None,
                    })
                    .collect();
            } else {
                row.compile = CompileObservation {
                    status: "failed".to_owned(),
                    metallib_sha256: None,
                    stderr: Some("optional compile failed in fixture".to_owned()),
                };
            }
            row
        })
        .collect();
    let verdict = WidthRecord::derive_verdict(&pipelines);
    WidthRecord {
        schema: RECORD_SCHEMA.to_owned(),
        metric: METRIC.to_owned(),
        repetitions: REPETITIONS,
        frozen_pipeline_count: PIPELINE_COUNT,
        environment_sha256: environment.custody_digest(),
        environment,
        custody: Custody {
            harness_source_sha256: harness_source_sha256(root),
            cargo_lock_sha256: cargo_lock_sha256(root),
            kernel_sha256: kernel_sha256_map,
            starting_executable_sha256: exe.clone(),
            ending_executable_sha256: exe,
        },
        pipelines,
        verdict,
    }
}
