//! Independent verification of retained timing and RSS custody.

use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::measure::parse_time_l_rss_exact;

/// Versioned meaning of the retained raw timing and RSS fields.
pub const CUSTODY_SCHEMA: &str = "tiler.physical-frontier-measurement-custody.v1";

/// Counts independently reconstructed from one retained record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerificationSummary {
    /// Sample rows whose summaries were recomputed.
    pub samples: usize,
    /// Ordered raw duration values consumed by recomputation.
    pub timing_values: usize,
    /// RSS rows whose complete `time -l` stderr was reparsed.
    pub rss_rows: usize,
}

/// Reads and verifies one retained JSON record.
pub fn verify_record_path(path: &Path) -> Result<VerificationSummary, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("cannot read custody record {}: {error}", path.display()))?;
    verify_record_text(&text)
}

/// Recomputes every summary and RSS value from the raw values retained in JSON.
pub fn verify_record_text(text: &str) -> Result<VerificationSummary, String> {
    let root: Value = serde_json::from_str(text)
        .map_err(|error| format!("custody record is not valid JSON: {error}"))?;
    let root = root
        .as_object()
        .ok_or_else(|| "custody record root must be an object".to_owned())?;
    let custody = root
        .get("custody")
        .and_then(Value::as_object)
        .ok_or_else(|| "custody metadata is absent".to_owned())?;
    expect_string(custody.get("schema"), "custody.schema", CUSTODY_SCHEMA)?;
    let samples = root
        .get("samples")
        .and_then(Value::as_array)
        .ok_or_else(|| "samples must be an array".to_owned())?;

    let mut timing_values = 0_usize;
    let mut rss_rows = 0_usize;
    for (index, sample) in samples.iter().enumerate() {
        let sample = sample
            .as_object()
            .ok_or_else(|| format!("sample {index} must be an object"))?;
        let name = sample
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("sample {index} has no string name"))?;
        let index_u64 = u64::try_from(index).map_err(|_| "sample index exceeds u64".to_owned())?;
        expect_u64(
            sample.get("sample_index"),
            &format!("{name}.sample_index"),
            index_u64,
        )?;
        verify_series_key(index, name, sample)?;
        let repeats = required_u64(sample.get("repeats"), &format!("{name}.repeats"))?;
        let durations = sample
            .get("timed_durations_ns")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("{name}.timed_durations_ns must be an array"))?;
        if durations.len() != usize::try_from(repeats).unwrap_or(usize::MAX) {
            return Err(format!(
                "{name} retained {} timing values for repeats={repeats}",
                durations.len()
            ));
        }
        if durations.is_empty() {
            return Err(format!("{name} retained no timing values"));
        }
        let ordered = durations
            .iter()
            .enumerate()
            .map(|(duration_index, value)| {
                required_u64(
                    Some(value),
                    &format!("{name}.timed_durations_ns[{duration_index}]"),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        timing_values = timing_values.saturating_add(ordered.len());
        verify_summary(name, sample, &ordered)?;

        match sample.get("rss") {
            Some(Value::Object(rss)) => {
                rss_rows = rss_rows.saturating_add(1);
                verify_rss(name, sample, rss)?;
            }
            Some(Value::Null) => {
                if !matches!(sample.get("peak_rss_bytes"), Some(Value::Null)) {
                    return Err(format!("{name} has peak RSS without retained RSS custody"));
                }
            }
            _ => return Err(format!("{name}.rss must be an object or null")),
        }
    }

    let summary = VerificationSummary {
        samples: samples.len(),
        timing_values,
        rss_rows,
    };
    expect_u64(
        custody.get("verified_samples"),
        "custody.verified_samples",
        u64::try_from(summary.samples).map_err(|_| "sample count exceeds u64".to_owned())?,
    )?;
    expect_u64(
        custody.get("verified_timing_values"),
        "custody.verified_timing_values",
        u64::try_from(summary.timing_values)
            .map_err(|_| "timing value count exceeds u64".to_owned())?,
    )?;
    expect_u64(
        custody.get("verified_rss_rows"),
        "custody.verified_rss_rows",
        u64::try_from(summary.rss_rows).map_err(|_| "RSS row count exceeds u64".to_owned())?,
    )?;
    Ok(summary)
}

/// Exports independently diffable raw timing and RSS artifacts from a record.
pub fn export_raw_artifacts(
    record_path: &Path,
    timing_path: &Path,
    rss_path: &Path,
) -> Result<(), String> {
    let text = fs::read_to_string(record_path).map_err(|error| {
        format!(
            "cannot read custody record {}: {error}",
            record_path.display()
        )
    })?;
    verify_record_text(&text)?;
    let root: Value = serde_json::from_str(&text)
        .map_err(|error| format!("custody record is not valid JSON: {error}"))?;
    let (timing, rss) = render_raw_artifacts(&root)?;
    write_artifact(timing_path, timing.as_bytes())?;
    write_artifact(rss_path, rss.as_bytes())?;
    verify_raw_artifacts(record_path, timing_path, rss_path)
}

/// Compares retained raw artifacts byte-for-byte with their generated source.
pub fn verify_raw_artifacts(
    record_path: &Path,
    timing_path: &Path,
    rss_path: &Path,
) -> Result<(), String> {
    let text = fs::read_to_string(record_path).map_err(|error| {
        format!(
            "cannot read custody record {}: {error}",
            record_path.display()
        )
    })?;
    verify_record_text(&text)?;
    let root: Value = serde_json::from_str(&text)
        .map_err(|error| format!("custody record is not valid JSON: {error}"))?;
    let (expected_timing, expected_rss) = render_raw_artifacts(&root)?;
    expect_file_text(timing_path, "raw timing artifact", &expected_timing)?;
    expect_file_text(rss_path, "raw RSS artifact", &expected_rss)
}

/// Writes an annotated semantic copy and its generated-artifact SHA-256 custody.
pub fn annotate_record(
    generated_path: &Path,
    annotated_path: &Path,
    timing_path: &Path,
    rss_path: &Path,
) -> Result<(), String> {
    verify_raw_artifacts(generated_path, timing_path, rss_path)?;
    let generated = fs::read(generated_path).map_err(|error| {
        format!(
            "cannot read generated record {}: {error}",
            generated_path.display()
        )
    })?;
    let mut root: Value = serde_json::from_slice(&generated)
        .map_err(|error| format!("generated record is not valid JSON: {error}"))?;
    let annotation = json!({
        "status": "live-request-wide-custodial-evidence",
        "generated_json": file_name(generated_path)?,
        "raw_timing_artifact": file_name(timing_path)?,
        "raw_rss_artifact": file_name(rss_path)?,
        "artifact_sha256": {
            "generated_json": sha256_hex(&generated),
            "raw_timing_artifact": sha256_file(timing_path)?,
            "raw_rss_artifact": sha256_file(rss_path)?,
        }
    });
    root.as_object_mut()
        .ok_or_else(|| "generated record root must be an object".to_owned())?
        .insert("evidence_annotation".to_owned(), annotation);
    let mut annotated = serde_json::to_string_pretty(&root)
        .map_err(|error| format!("cannot render annotated record: {error}"))?;
    annotated.push('\n');
    write_artifact(annotated_path, annotated.as_bytes())?;
    verify_evidence(generated_path, annotated_path, timing_path, rss_path)
}

/// Verifies hashes, annotation-only differences, and every retained raw value.
pub fn verify_evidence(
    generated_path: &Path,
    annotated_path: &Path,
    timing_path: &Path,
    rss_path: &Path,
) -> Result<(), String> {
    verify_raw_artifacts(generated_path, timing_path, rss_path)?;
    let generated = fs::read(generated_path).map_err(|error| {
        format!(
            "cannot read generated record {}: {error}",
            generated_path.display()
        )
    })?;
    let annotated = fs::read(annotated_path).map_err(|error| {
        format!(
            "cannot read annotated record {}: {error}",
            annotated_path.display()
        )
    })?;
    let generated_value: Value = serde_json::from_slice(&generated)
        .map_err(|error| format!("generated record is not valid JSON: {error}"))?;
    let mut annotated_value: Value = serde_json::from_slice(&annotated)
        .map_err(|error| format!("annotated record is not valid JSON: {error}"))?;
    verify_record_text(
        std::str::from_utf8(&annotated)
            .map_err(|error| format!("annotated record is not UTF-8: {error}"))?,
    )?;
    let annotation = annotated_value
        .as_object_mut()
        .and_then(|root| root.remove("evidence_annotation"))
        .ok_or_else(|| "annotated record has no evidence_annotation".to_owned())?;
    if annotated_value != generated_value {
        return Err("annotated measurement fields differ from generated record".to_owned());
    }
    let annotation = annotation
        .as_object()
        .ok_or_else(|| "evidence_annotation must be an object".to_owned())?;
    expect_string(
        annotation.get("status"),
        "evidence_annotation.status",
        "live-request-wide-custodial-evidence",
    )?;
    expect_string(
        annotation.get("generated_json"),
        "evidence_annotation.generated_json",
        file_name(generated_path)?,
    )?;
    expect_string(
        annotation.get("raw_timing_artifact"),
        "evidence_annotation.raw_timing_artifact",
        file_name(timing_path)?,
    )?;
    expect_string(
        annotation.get("raw_rss_artifact"),
        "evidence_annotation.raw_rss_artifact",
        file_name(rss_path)?,
    )?;
    let hashes = annotation
        .get("artifact_sha256")
        .and_then(Value::as_object)
        .ok_or_else(|| "evidence_annotation.artifact_sha256 must be an object".to_owned())?;
    expect_string(
        hashes.get("generated_json"),
        "artifact_sha256.generated_json",
        &sha256_hex(&generated),
    )?;
    expect_string(
        hashes.get("raw_timing_artifact"),
        "artifact_sha256.raw_timing_artifact",
        &sha256_file(timing_path)?,
    )?;
    expect_string(
        hashes.get("raw_rss_artifact"),
        "artifact_sha256.raw_rss_artifact",
        &sha256_file(rss_path)?,
    )?;
    Ok(())
}

fn render_raw_artifacts(root: &Value) -> Result<(String, String), String> {
    let samples = root
        .get("samples")
        .and_then(Value::as_array)
        .ok_or_else(|| "samples must be an array".to_owned())?;
    let mut timing = String::from("sample_index\tseries_key\tobservation_index\tduration_ns\n");
    let mut rss = String::new();
    for sample in samples {
        let sample = sample
            .as_object()
            .ok_or_else(|| "sample must be an object".to_owned())?;
        let sample_index = required_u64(sample.get("sample_index"), "sample_index")?;
        let series_key = sample
            .get("series_key")
            .and_then(Value::as_str)
            .ok_or_else(|| "series_key must be a string".to_owned())?;
        let durations = sample
            .get("timed_durations_ns")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("{series_key}.timed_durations_ns must be an array"))?;
        for (observation_index, duration) in durations.iter().enumerate() {
            let duration = required_u64(Some(duration), "duration_ns")?;
            timing.push_str(&format!(
                "{sample_index}\t{series_key}\t{observation_index}\t{duration}\n"
            ));
        }
        if let Some(Value::Object(rss_custody)) = sample.get("rss") {
            let row = json!({
                "sample_index": sample_index,
                "series_key": series_key,
                "rss": rss_custody,
            });
            rss.push_str(
                &serde_json::to_string(&row)
                    .map_err(|error| format!("cannot render raw RSS row: {error}"))?,
            );
            rss.push('\n');
        }
    }
    Ok((timing, rss))
}

fn write_artifact(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    fs::write(path, bytes).map_err(|error| format!("cannot write {}: {error}", path.display()))
}

fn expect_file_text(path: &Path, kind: &str, expected: &str) -> Result<(), String> {
    let observed = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {kind} {}: {error}", path.display()))?;
    if observed == expected {
        Ok(())
    } else {
        Err(format!("{kind} does not match generated record"))
    }
}

fn file_name(path: &Path) -> Result<&str, String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("{} has no UTF-8 file name", path.display()))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("cannot read artifact {}: {error}", path.display()))?;
    Ok(sha256_hex(&bytes))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn verify_summary(
    name: &str,
    sample: &serde_json::Map<String, Value>,
    ordered: &[u64],
) -> Result<(), String> {
    let mut sorted = ordered.to_vec();
    sorted.sort_unstable();
    let count = sorted.len();
    let total = sorted
        .iter()
        .fold(0_u128, |sum, value| sum.saturating_add(u128::from(*value)));
    let expected = [
        ("min_us", sorted[0] / 1_000),
        ("median_us", sorted[count / 2] / 1_000),
        (
            "p90_us",
            sorted[count.saturating_mul(9).saturating_sub(1) / 10] / 1_000,
        ),
        ("max_us", sorted[count - 1] / 1_000),
        (
            "mean_us",
            u64::try_from(total / u128::try_from(count).unwrap_or(u128::MAX)).unwrap_or(u64::MAX)
                / 1_000,
        ),
    ];
    for (field, value) in expected {
        expect_u64(sample.get(field), &format!("{name}.{field}"), value)?;
    }
    Ok(())
}

fn verify_rss(
    name: &str,
    sample: &serde_json::Map<String, Value>,
    rss: &serde_json::Map<String, Value>,
) -> Result<(), String> {
    let command = rss
        .get("command")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{name}.rss.command must be an array"))?;
    if command.first().and_then(Value::as_str) != Some("/usr/bin/time")
        || command.get(1).and_then(Value::as_str) != Some("-l")
    {
        return Err(format!(
            "{name}.rss.command is not the retained time -l invocation"
        ));
    }
    verify_rss_subject(name, sample, command)?;
    if rss.get("child_exit_success").and_then(Value::as_bool) != Some(true) {
        return Err(format!("{name} RSS child did not exit successfully"));
    }
    if rss.get("child_exit_code").and_then(Value::as_i64) != Some(0) {
        return Err(format!("{name} RSS child exit code is not zero"));
    }
    let stderr = rss
        .get("time_stderr")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{name}.rss.time_stderr must be a string"))?;
    let reparsed = parse_time_l_rss_exact(stderr).map_err(|error| format!("{name} {error}"))?;
    expect_u64(
        rss.get("parsed_peak_rss_bytes"),
        &format!("{name}.rss.parsed_peak_rss_bytes"),
        reparsed,
    )?;
    expect_u64(
        sample.get("peak_rss_bytes"),
        &format!("{name}.peak_rss_bytes"),
        reparsed,
    )
}

fn verify_rss_subject(
    name: &str,
    sample: &serde_json::Map<String, Value>,
    command: &[Value],
) -> Result<(), String> {
    let request_wide = sample
        .get("request_wide")
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("{name}.request_wide must be a boolean"))?;
    let targets = required_u64(sample.get("targets"), &format!("{name}.targets"))?;
    let providers = required_u64(
        sample.get("extra_providers"),
        &format!("{name}.extra_providers"),
    )?;
    let kind = sample
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{name}.kind must be a string"))?;
    let program_kind = sample
        .get("program_kind")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{name}.program_kind must be a string"))?;
    let observed = command
        .iter()
        .skip(3)
        .map(|value| value.as_str().unwrap_or("<non-string>"))
        .collect::<Vec<_>>();
    let targets = targets.to_string();
    let providers = providers.to_string();
    let expected = if request_wide {
        vec![
            "child-request-measure",
            name,
            targets.as_str(),
            providers.as_str(),
            kind,
            program_kind,
        ]
    } else {
        vec!["child-measure", name, providers.as_str(), kind]
    };
    if observed == expected {
        Ok(())
    } else {
        Err(format!(
            "{name} RSS command subject mismatch: expected={expected:?} observed={observed:?}"
        ))
    }
}

fn verify_series_key(
    index: usize,
    name: &str,
    sample: &serde_json::Map<String, Value>,
) -> Result<(), String> {
    let targets = required_u64(sample.get("targets"), &format!("{name}.targets"))?;
    let providers = required_u64(
        sample.get("extra_providers"),
        &format!("{name}.extra_providers"),
    )?;
    let kind = sample
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{name}.kind must be a string"))?;
    let program_kind = sample
        .get("program_kind")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{name}.program_kind must be a string"))?;
    let expected = format!(
        "{index}:{name}:targets={targets}:providers={providers}:kind={kind}:program={program_kind}"
    );
    expect_string(
        sample.get("series_key"),
        &format!("{name}.series_key"),
        &expected,
    )
}

fn required_u64(value: Option<&Value>, field: &str) -> Result<u64, String> {
    value
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("{field} must be an unsigned integer"))
}

fn expect_u64(value: Option<&Value>, field: &str, expected: u64) -> Result<(), String> {
    let observed = required_u64(value, field)?;
    if observed == expected {
        Ok(())
    } else {
        Err(format!("{field} expected={expected} observed={observed}"))
    }
}

fn expect_string(value: Option<&Value>, field: &str, expected: &str) -> Result<(), String> {
    let observed = value
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{field} must be a string"))?;
    if observed == expected {
        Ok(())
    } else {
        Err(format!(
            "{field} expected={expected:?} observed={observed:?}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"{
      "custody":{"schema":"tiler.physical-frontier-measurement-custody.v1","verified_samples":1,"verified_timing_values":5,"verified_rss_rows":1},
      "samples":[{
        "sample_index":0,"series_key":"0:row:targets=1:providers=2:kind=propose:program=five-op","name":"row","program_kind":"five-op","request_wide":false,"targets":1,"extra_providers":2,"kind":"propose","repeats":5,
        "timed_durations_ns":[5000,1000,3000,2000,4000],
        "min_us":1,"median_us":3,"p90_us":5,"max_us":5,"mean_us":3,
        "peak_rss_bytes":42,
        "rss":{"command":["/usr/bin/time","-l","/tmp/child","child-measure","row","2","propose"],"child_exit_success":true,"child_exit_code":0,"time_stderr":"42  maximum resident set size\n","parsed_peak_rss_bytes":42}
      }]
    }"#;

    #[test]
    fn retained_values_recompute_every_summary_and_rss() {
        assert_eq!(
            verify_record_text(GOOD),
            Ok(VerificationSummary {
                samples: 1,
                timing_values: 5,
                rss_rows: 1,
            })
        );
    }

    #[test]
    fn changed_raw_timing_fails_the_unchanged_summary_check() {
        let perturbed = GOOD.replacen("5000,1000", "9000,1000", 1);
        assert_eq!(
            verify_record_text(&perturbed),
            Err("row.p90_us expected=9 observed=5".to_owned())
        );
    }

    #[test]
    fn changed_rss_stderr_fails_the_unchanged_parsed_value_check() {
        let perturbed = GOOD.replacen("42  maximum", "43  maximum", 1);
        assert_eq!(
            verify_record_text(&perturbed),
            Err("row.rss.parsed_peak_rss_bytes expected=43 observed=42".to_owned())
        );
    }

    #[test]
    fn duplicate_rss_stderr_lines_fail_the_unchanged_parser() {
        let perturbed = GOOD.replacen(
            "42  maximum resident set size\\n",
            "42  maximum resident set size\\n43  maximum resident set size\\n",
            1,
        );
        assert_eq!(
            verify_record_text(&perturbed),
            Err("row retained time stderr has 2 maximum RSS lines".to_owned())
        );
    }

    #[test]
    fn changed_rss_subject_fails_the_unchanged_subject_check() {
        let perturbed = GOOD.replacen("child-measure", "child-request-measure", 1);
        assert_eq!(
            verify_record_text(&perturbed),
            Err("row RSS command subject mismatch: expected=[\"child-measure\", \"row\", \"2\", \"propose\"] observed=[\"child-request-measure\", \"row\", \"2\", \"propose\"]".to_owned())
        );
    }
}
