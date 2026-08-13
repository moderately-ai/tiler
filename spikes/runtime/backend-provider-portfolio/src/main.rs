//! Exercise standard Metal, a custom Metal physical provider, and CPU in one portfolio.

mod compile;
mod cpu;
mod metal;
mod portfolio;
mod provider;
mod semantic;

use std::path::PathBuf;
use std::process::ExitCode;

use tiler_artifact::program::{ArtifactBuildError, RecordedArtifactProgramIdentity};
use tiler_cache::expansion::ExpansionCache;
use tiler_runtime::load::{ExecutionEnvironment, LoadRejection, VariantIneligibility};

use crate::compile::{
    offered_custom, physical_provenance, selected_custom, selected_governed, selected_plan,
};
use crate::cpu::SOLE_DELIVERY;

fn main() -> ExitCode {
    match run(std::env::args().nth(1).map(PathBuf::from)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(record: Option<PathBuf>) -> Result<(), String> {
    println!("# backend-provider-portfolio");
    println!("base\t612468048d541a1017640fc5dcbe5ff9160716cf");

    let program = semantic::program();
    let (reference, reference_identity) = semantic::reference_bits(&program);
    println!(
        "semantic\t{}x{} f32 pointwise (input * 2.0) * 1.0",
        semantic::ROWS,
        semantic::COLUMNS
    );
    println!("reference.identity_bytes\t{}", reference_identity.len());

    let compiled = compile::compile_portfolio(&program)?;
    let with_custom = physical_provenance(&compiled.with_custom);
    let without_custom = physical_provenance(&compiled.without_custom);
    println!(
        "compile.with_custom.offered\t{}",
        with_custom.offered.join(",")
    );
    for alternative in &with_custom.alternatives {
        println!(
            "compile.with_custom.alternative\t{}\t{}",
            alternative.stable_id,
            alternative.selected.join(",")
        );
    }
    println!(
        "compile.without_custom.offered\t{}",
        without_custom.offered.join(",")
    );
    if !offered_custom(&compiled.with_custom) {
        return Err(
            "the custom provider was not named by Compilation::offered_physical_providers".into(),
        );
    }
    if offered_custom(&compiled.without_custom) {
        return Err(
            "removing the custom provider still named it in the offered physical environment"
                .into(),
        );
    }
    if !selected_governed(&compiled.without_custom) {
        return Err(
            "the governed provider was not selected after the custom provider was removed".into(),
        );
    }
    println!(
        "compile.custom_offered_and_removable\t{}",
        selected_custom(&compiled.with_custom)
    );
    println!("compile.governed_survives_removal\ttrue");

    let metal_plan = selected_plan(&compiled.with_custom)?;
    let cpu_plan = selected_plan(&compiled.without_custom)?;
    let cpu = cpu::assemble(&program, cpu_plan).map_err(|error| error.to_string())?;
    println!("cpu.assemble_plan_artifact\t{} bytes", cpu.bytes.len());

    let cache_root = std::env::temp_dir().join(format!(
        "tiler-backend-provider-portfolio-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&cache_root).map_err(|error| error.to_string())?;
    let cache = ExpansionCache::open(&cache_root);
    let metal_payload = match metal::assemble(&cache, &program, metal_plan, &compiled.declaration) {
        Ok(produced) => {
            println!(
                "metal.accept_or_publish_metal_plan\t{} bytes",
                produced.bytes.len()
            );
            Some(produced)
        }
        Err(metal::MetalError::Unavailable(message)) => {
            println!("metal.unavailable\t{message}");
            None
        }
        Err(error) => return Err(error.to_string()),
    };

    let shared = compiled
        .declaration
        .target_profile_ref()
        .map_err(|error| error.to_string())?;
    let dtype = metal::dtype_dispatch(&compiled.declaration);
    let metal_content = metal_payload
        .as_ref()
        .map_or_else(|| cpu.content.clone(), |produced| produced.content.clone());
    // The mixed-target control uses a deliberately different descriptor so
    // `check_subject` is the thing that refuses, not a missing payload.
    let mixed = portfolio::refuse_mixed_targets(
        &program,
        &compiled.with_custom,
        metal_plan,
        &compiled.without_custom,
        cpu_plan,
        &metal_content,
        &cpu,
    )
    .map_err(|error| error.to_string())?;
    assert!(matches!(mixed, ArtifactBuildError::TargetProfileMismatch));
    println!("portfolio.mixed_target\tTargetProfileMismatch");

    let mut metal_run = MetalRun::Unavailable("not packaged".into());
    let cpu_bits;
    let mut portfolio_bytes = 0;
    let mut cross_family = Vec::new();

    if let Some(produced) = metal_payload.as_ref() {
        let packaged = portfolio::assemble_shared(
            &program,
            &compiled.with_custom,
            metal_plan,
            &compiled.without_custom,
            cpu_plan,
            shared.clone(),
            &produced.content,
            &cpu,
        )
        .map_err(|error| error.to_string())?;
        portfolio_bytes = packaged.bytes.len();
        println!("portfolio.shared_target\t{} bytes", packaged.bytes.len());

        let cpu_env = ExecutionEnvironment {
            target_profile: shared.clone(),
            backend: cpu::backend(),
            representation: cpu::representation(),
            dtype_dispatch: dtype.clone(),
        };
        let metal_env = ExecutionEnvironment {
            target_profile: shared.clone(),
            backend: metal::backend(),
            representation: metal::representation(),
            dtype_dispatch: dtype.clone(),
        };

        // Present each *family's own* assembled artifact under the other
        // environment. Against the combined portfolio the loader would select
        // the matching family instead of refusing; the cross-family control is
        // that a Metal-only member under a CPU host, and a CPU-only member
        // under a Metal host, refuse in preflight as UnsupportedRepresentation.
        let metal_under_cpu =
            metal::preflight_refusal(&produced.bytes, &produced.expected, &cpu_env)
                .map_err(|error| error.to_string())?;
        let cpu_only_expected = RecordedArtifactProgramIdentity::from_bytes(
            cpu.artifact.canonical_identity().as_bytes(),
        )
        .map_err(|error| error.to_string())?;
        let cpu_under_metal = cpu::preflight_refusal(&cpu.bytes, &cpu_only_expected, &metal_env)
            .map_err(|error| error.to_string())?;
        if !matches_unsupported(&metal_under_cpu) {
            return Err(format!(
                "Metal under CPU environment refused as {metal_under_cpu}, not UnsupportedRepresentation"
            ));
        }
        if !matches_unsupported(&cpu_under_metal) {
            return Err(format!(
                "CPU under Metal environment refused as {cpu_under_metal}, not UnsupportedRepresentation"
            ));
        }
        println!("preflight.metal_under_cpu\t{metal_under_cpu}");
        println!("preflight.cpu_under_metal\t{cpu_under_metal}");
        cross_family.push(metal_under_cpu.to_string());
        cross_family.push(cpu_under_metal.to_string());

        cpu_bits = Some(
            cpu::route_and_compare(
                &packaged.bytes,
                &packaged.expected,
                shared.clone(),
                dtype.clone(),
                &reference,
            )
            .map_err(|error| error.to_string())?,
        );
        println!(
            "cpu.route_with_adapter\t{} elements agree with tiler-reference",
            cpu_bits.as_ref().map_or(0, Vec::len)
        );

        metal_run = match metal::route_and_compare(
            &packaged.bytes,
            &packaged.expected,
            shared,
            dtype,
            &reference,
        ) {
            Ok(bits) => {
                println!(
                    "metal.route_with_adapter\t{} elements agree with tiler-reference",
                    bits.len()
                );
                MetalRun::Executed(bits)
            }
            Err(metal::MetalError::Unavailable(message)) => {
                println!("metal.unavailable\t{message}");
                MetalRun::Unavailable(message)
            }
            Err(error) => return Err(error.to_string()),
        };

        probe_fail_closed(&packaged.bytes, &packaged.expected, &cpu_env)?;
    } else {
        let bits = cpu::route_and_compare(
            &cpu.bytes,
            &RecordedArtifactProgramIdentity::from_bytes(
                cpu.artifact.canonical_identity().as_bytes(),
            )
            .map_err(|error| error.to_string())?,
            compiled
                .declaration
                .target_profile_ref()
                .map_err(|error| error.to_string())?,
            dtype,
            &reference,
        )
        .map_err(|error| error.to_string())?;
        println!(
            "cpu.route_with_adapter\t{} elements agree with tiler-reference (CPU-only artifact; Metal payload unavailable)",
            bits.len()
        );
        cpu_bits = Some(bits);
    }

    let _ = std::fs::remove_dir_all(&cache_root);

    if let Some(path) = record {
        write_fixture(
            &path,
            &reference,
            cpu_bits.as_deref(),
            &metal_run,
            &with_custom,
            &without_custom,
            portfolio_bytes,
            &cross_family,
            reference_identity.len(),
        )?;
        println!("fixture\t{}", path.display());
    }
    Ok(())
}

enum MetalRun {
    Executed(Vec<u32>),
    Unavailable(String),
}

fn matches_unsupported(rejection: &LoadRejection) -> bool {
    metal::is_unsupported_representation(rejection)
}

fn probe_fail_closed(
    bytes: &[u8],
    expected: &RecordedArtifactProgramIdentity,
    cpu_env: &ExecutionEnvironment,
) -> Result<(), String> {
    let mut damaged = bytes.to_vec();
    if let Some(byte) = damaged.get_mut(bytes.len() / 2) {
        *byte ^= 0xff;
    }
    match tiler_runtime::load::DecodedProgram::decode(&damaged, SOLE_DELIVERY) {
        Err(rejection) => println!("probe.flipped_byte\t{rejection}"),
        Ok(_) => return Err("flipping an interior byte did not refuse decode".into()),
    }
    match tiler_runtime::load::DecodedProgram::decode(&bytes[..bytes.len() / 2], SOLE_DELIVERY) {
        Err(rejection) => println!("probe.truncated\t{rejection}"),
        Ok(_) => return Err("truncating the envelope did not refuse decode".into()),
    }
    let foreign = ExecutionEnvironment {
        target_profile: cpu_env.target_profile.clone(),
        backend: tiler_artifact::program::BackendKey::new("tiler.test.other")
            .expect("a governed backend key"),
        representation: cpu_env.representation.clone(),
        dtype_dispatch: cpu_env.dtype_dispatch.clone(),
    };
    let refusal =
        cpu::preflight_refusal(bytes, expected, &foreign).map_err(|error| error.to_string())?;
    if !matches!(
        sole_ineligibility(&refusal),
        Some(VariantIneligibility::UnsupportedRepresentation { .. })
    ) && !matches_unsupported(&refusal)
    {
        return Err(format!(
            "a foreign backend was refused as {refusal}, not UnsupportedRepresentation"
        ));
    }
    println!("probe.foreign_backend\t{refusal}");
    Ok(())
}

fn sole_ineligibility(rejection: &LoadRejection) -> Option<&VariantIneligibility> {
    let LoadRejection::NoEligibleVariant { filtered, .. } = rejection else {
        return None;
    };
    filtered.first().map(|filtered| &filtered.reason)
}

fn write_fixture(
    path: &PathBuf,
    reference: &[u32],
    cpu_bits: Option<&[u32]>,
    metal: &MetalRun,
    with_custom: &compile::PhysicalProvenance,
    without_custom: &compile::PhysicalProvenance,
    portfolio_bytes: usize,
    cross_family: &[String],
    reference_identity_bytes: usize,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let cpu_hex = cpu_bits.unwrap_or(reference);
    let metal_line = match metal {
        MetalRun::Executed(bits) => format!(
            "\"executed\", \"bits\": [{}]",
            bits.iter()
                .map(|value| format!("\"0x{value:08x}\""))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        MetalRun::Unavailable(message) => {
            format!("\"unavailable\", \"detail\": {}", json_string(message))
        }
    };
    let body = format!(
        "{{\n  \"base\": \"612468048d541a1017640fc5dcbe5ff9160716cf\",\n  \"program\": \"(input * 2.0) * 1.0\",\n  \"elements\": {},\n  \"reference_bits\": [{}],\n  \"cpu_bits\": [{}],\n  \"metal\": {{ \"status\": {metal_line} }},\n  \"offered_with_custom\": [{}],\n  \"offered_without_custom\": [{}],\n  \"portfolio_bytes\": {portfolio_bytes},\n  \"cross_family_refusals\": [{}],\n  \"reference_identity_bytes\": {reference_identity_bytes}\n}}\n",
        reference.len(),
        hex_list(reference),
        hex_list(cpu_hex),
        with_custom
            .offered
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(", "),
        without_custom
            .offered
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(", "),
        cross_family
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(", "),
    );
    std::fs::write(path, body).map_err(|error| error.to_string())
}

fn hex_list(bits: &[u32]) -> String {
    bits.iter()
        .map(|value| format!("\"0x{value:08x}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
