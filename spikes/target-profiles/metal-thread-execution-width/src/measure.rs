//! Offline compile and prepare. No dispatch. The metric is a pipeline property.

use std::fs;
use std::process::Command;

use metal::{ComputePipelineDescriptor, Device, MTLGPUFamily};

use crate::population::{METRIC, PIPELINE_COUNT, RECORD_SCHEMA};
use crate::population::{PIPELINES, REPETITIONS};
use crate::record::{
    CompileObservation, Custody, Environment, PipelineRecord, PreparationObservation, WidthRecord,
    apply_descriptor, cargo_lock_sha256, executable_sha256, harness_source_sha256, identity_record,
    kernel_sha256, spike_root,
};

/// Compiles every frozen identity, prepares each required pipeline three times, and records every width.
pub fn measure() -> WidthRecord {
    let root = spike_root();
    let starting_executable = executable_sha256();
    let device = Device::system_default().expect("this host exposes a default Metal device");
    assert!(
        device.supports_family(MTLGPUFamily::Apple9),
        "the default device is not Apple9; this measurement does not run"
    );
    let environment = observe_environment(&device);
    let mut kernel_hashes = std::collections::BTreeMap::new();
    for spec in PIPELINES {
        kernel_hashes
            .entry(spec.kernel.name().to_owned())
            .or_insert_with(|| kernel_sha256(&root, spec.kernel.name()));
    }
    let mut pipelines = Vec::with_capacity(PIPELINES.len());
    for spec in PIPELINES {
        pipelines.push(measure_one(&device, &root, *spec, &kernel_hashes));
    }
    let ending_executable = executable_sha256();
    let verdict = WidthRecord::derive_verdict(&pipelines);
    WidthRecord {
        schema: RECORD_SCHEMA.to_owned(),
        metric: METRIC.to_owned(),
        repetitions: REPETITIONS,
        frozen_pipeline_count: PIPELINE_COUNT,
        environment_sha256: environment.custody_digest(),
        environment,
        custody: Custody {
            harness_source_sha256: harness_source_sha256(&root),
            cargo_lock_sha256: cargo_lock_sha256(&root),
            kernel_sha256: kernel_hashes,
            starting_executable_sha256: starting_executable,
            ending_executable_sha256: ending_executable,
        },
        pipelines,
        verdict,
    }
}

fn measure_one(
    device: &Device,
    root: &std::path::Path,
    spec: crate::population::PipelineSpec,
    kernel_hashes: &std::collections::BTreeMap<String, String>,
) -> PipelineRecord {
    let source_sha256 = kernel_hashes
        .get(spec.kernel.name())
        .expect("every kernel was hashed")
        .clone();
    let mut row = identity_record(spec, source_sha256);
    match compile_kernel(root, spec) {
        Ok(object) => {
            row.compile = CompileObservation {
                status: "ok".to_owned(),
                metallib_sha256: Some(crate::record::sha256_hex(&object)),
                stderr: None,
            };
            for repetition in 1..=REPETITIONS {
                row.preparations
                    .push(prepare_once(device, spec, &object, repetition));
            }
            if spec.required {
                let failed = row
                    .preparations
                    .iter()
                    .any(|prep| prep.status != "ok" || prep.thread_execution_width.is_none());
                assert!(
                    !failed,
                    "required pipeline {} did not prepare; the run has no equality claim",
                    spec.id()
                );
            }
        }
        Err(stderr) => {
            assert!(
                !spec.required,
                "required pipeline {} did not compile: {stderr}",
                spec.id()
            );
            row.compile = CompileObservation {
                status: "failed".to_owned(),
                metallib_sha256: None,
                stderr: Some(stderr),
            };
        }
    }
    row
}

fn compile_kernel(
    root: &std::path::Path,
    spec: crate::population::PipelineSpec,
) -> Result<Vec<u8>, String> {
    let directory = std::env::temp_dir().join(format!(
        "tiler-tew-{}-{}-{}",
        std::process::id(),
        spec.kernel.name(),
        spec.compiler.name()
    ));
    fs::create_dir_all(&directory).expect("the scratch directory is creatable");
    let source = root
        .join("kernels")
        .join(format!("{}.metal", spec.kernel.name()));
    let air = directory.join("probe.air");
    let metallib = directory.join("probe.metallib");
    let mut metal_args = vec!["--sdk".to_owned(), "macosx".to_owned(), "metal".to_owned()];
    metal_args.extend(
        spec.compiler
            .metal_flags()
            .iter()
            .map(|flag| (*flag).to_owned()),
    );
    metal_args.extend([
        "-c".to_owned(),
        source.to_str().expect("UTF-8 source path").to_owned(),
        "-o".to_owned(),
        air.to_str().expect("UTF-8 air path").to_owned(),
    ]);
    let compile = run_allow_fail("xcrun", &metal_args);
    if !compile.status {
        let _ = fs::remove_dir_all(&directory);
        return Err(compile.stderr);
    }
    let link = run_allow_fail(
        "xcrun",
        &[
            "--sdk".to_owned(),
            "macosx".to_owned(),
            "metallib".to_owned(),
            air.to_str().expect("UTF-8 air path").to_owned(),
            "-o".to_owned(),
            metallib.to_str().expect("UTF-8 metallib path").to_owned(),
        ],
    );
    if !link.status {
        let _ = fs::remove_dir_all(&directory);
        return Err(link.stderr);
    }
    let object = fs::read(&metallib).expect("the linked object is readable");
    fs::remove_dir_all(&directory).expect("the scratch directory is removable");
    Ok(object)
}

fn prepare_once(
    device: &Device,
    spec: crate::population::PipelineSpec,
    object: &[u8],
    repetition: usize,
) -> PreparationObservation {
    let repetition = u32::try_from(repetition).expect("repetition fits u32");
    let library = match device.new_library_with_data(object) {
        Ok(library) => library,
        Err(error) => {
            return failed_prep(repetition, error);
        }
    };
    let function = match library.get_function(spec.kernel.name(), None) {
        Ok(function) => function,
        Err(error) => return failed_prep(repetition, error),
    };
    let descriptor = ComputePipelineDescriptor::new();
    descriptor.set_compute_function(Some(&function));
    apply_descriptor(&descriptor, spec.descriptor);
    match device.new_compute_pipeline_state(&descriptor) {
        Ok(pipeline) => PreparationObservation {
            repetition,
            status: "ok".to_owned(),
            thread_execution_width: Some(pipeline.thread_execution_width()),
            max_total_threads_per_threadgroup: Some(pipeline.max_total_threads_per_threadgroup()),
            static_threadgroup_memory_length: Some(pipeline.static_threadgroup_memory_length()),
            error: None,
        },
        Err(error) => failed_prep(repetition, error),
    }
}

fn failed_prep(repetition: u32, error: impl std::fmt::Display) -> PreparationObservation {
    PreparationObservation {
        repetition,
        status: "failed".to_owned(),
        thread_execution_width: None,
        max_total_threads_per_threadgroup: None,
        static_threadgroup_memory_length: None,
        error: Some(error.to_string()),
    }
}

fn observe_environment(device: &Device) -> Environment {
    Environment {
        offline_metal: first_line(&run("xcrun", &["--sdk", "macosx", "metal", "--version"])),
        offline_linker: first_line(&run("xcrun", &["--sdk", "macosx", "metallib", "-version"])),
        offline_xcode: run("xcodebuild", &["-version"])
            .replace('\n', " ")
            .trim()
            .to_owned(),
        offline_sdk_version: first_line(&run("xcrun", &["--sdk", "macosx", "--show-sdk-version"])),
        offline_sdk_build: first_line(&run(
            "xcrun",
            &["--sdk", "macosx", "--show-sdk-build-version"],
        )),
        rustc_verbose: run("rustc", &["-vV"]),
        platform_version: first_line(&run("sw_vers", &["-productVersion"])),
        platform_build: first_line(&run("sw_vers", &["-buildVersion"])),
        architecture: first_line(&run("uname", &["-m"])),
        device: device.name().to_owned(),
        device_registry_id: format!("{:#x}", device.registry_id()),
        apple9: device.supports_family(MTLGPUFamily::Apple9),
        max_buffer_length: device.max_buffer_length(),
        load_averages: first_line(&run("sysctl", &["-n", "vm.loadavg"])),
    }
}

struct CommandResult {
    status: bool,
    stderr: String,
}

fn run_allow_fail(program: &str, arguments: &[String]) -> CommandResult {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("{program} {arguments:?} is runnable: {error}"));
    CommandResult {
        status: output.status.success(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn run(program: &str, arguments: &[&str]) -> String {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("{program} {arguments:?} is runnable: {error}"));
    assert!(
        output.status.success(),
        "{program} {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr),
    );
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    if text.trim().is_empty() {
        return String::from_utf8_lossy(&output.stderr).into_owned();
    }
    text
}

fn first_line(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or_default()
        .to_owned()
}
