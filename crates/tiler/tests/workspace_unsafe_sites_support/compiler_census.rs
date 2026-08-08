use super::*;

/// Runs rustc over every ordinary/test target of the only two packages where
/// item attributes can reopen unsafe code, then inventories the compiler's
/// post-expansion unsafe-code diagnostics.
pub(super) fn expanded_unsafe_operations(
    root: &Path,
    population: &WorkspacePopulation,
) -> Result<ExpandedOperations, String> {
    let mut command = Command::new(env!("CARGO"));
    command
        .args([
            "check",
            "--locked",
            "--all-targets",
            "--message-format=json",
        ])
        .arg("--target-dir")
        .arg(root.join("target/workspace-unsafe-sites-expanded"));
    for package in &population.reopenable_packages {
        command.args(["-p", &package.name]);
    }
    let output = command
        .current_dir(root)
        .env("RUSTFLAGS", "--force-warn=unsafe-code")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .output()
        .map_err(|error| format!("nested cargo check could not run: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "nested cargo check failed with {}:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
        ));
    }

    let package_by_id: BTreeMap<&str, &str> = population
        .reopenable_packages
        .iter()
        .map(|package| (package.id.as_str(), package.name.as_str()))
        .collect();
    let mut found = BTreeMap::new();
    let mut compiled_targets = BTreeSet::new();
    let mut saw_build_finished = false;
    for (line_number, line) in output.stdout.split(|byte| *byte == b'\n').enumerate() {
        if line.is_empty() {
            continue;
        }
        let message: Value = serde_json::from_slice(line).map_err(|error| {
            format!(
                "nested cargo check emitted invalid JSON on line {}: {error}",
                line_number + 1,
            )
        })?;
        if message.get("reason").and_then(Value::as_str) == Some("build-finished") {
            saw_build_finished = message
                .get("success")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            continue;
        }
        if message.get("reason").and_then(Value::as_str) == Some("compiler-artifact") {
            let package_id = metadata_string(&message, "package_id", "compiler artifact")?;
            if let Some(package_name) = package_by_id.get(package_id) {
                let target = message
                    .get("target")
                    .ok_or_else(|| "compiler artifact has no `target` object".to_owned())?;
                compiled_targets.insert((
                    (*package_name).to_owned(),
                    metadata_string(target, "name", "compiler artifact target")?.to_owned(),
                    normalized_compiler_path(
                        root,
                        metadata_string(target, "src_path", "compiler artifact target")?,
                    )?,
                ));
            }
            continue;
        }
        if message.get("reason").and_then(Value::as_str) != Some("compiler-message") {
            continue;
        }
        let package_id = metadata_string(&message, "package_id", "compiler message")?;
        let Some(package_name) = package_by_id.get(package_id) else {
            continue;
        };
        let diagnostic = message
            .get("message")
            .ok_or_else(|| "compiler message has no `message` object".to_owned())?;
        if diagnostic
            .get("code")
            .and_then(|code| code.get("code"))
            .and_then(Value::as_str)
            != Some("unsafe_code")
        {
            continue;
        }
        let level = metadata_string(diagnostic, "level", "unsafe-code diagnostic")?;
        if level != "warning" {
            return Err(format!(
                "compiler unsafe-code diagnostic for {package_name} had level `{level}`, \
                 expected force-warn `warning`",
            ));
        }
        let target = message
            .get("target")
            .ok_or_else(|| "compiler message has no `target` object".to_owned())?;
        let target_name = metadata_string(target, "name", "compiler target")?;
        let target_source = normalized_compiler_path(
            root,
            metadata_string(target, "src_path", "compiler target")?,
        )?;
        let primary_spans: Vec<&Value> = metadata_array(diagnostic, "spans", "diagnostic")?
            .iter()
            .filter(|span| {
                span.get("is_primary")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            })
            .collect();
        if primary_spans.len() != 1 {
            return Err(format!(
                "compiler unsafe-code diagnostic for {package_name}/{target_name} has {} primary \
                 spans; source identity is ambiguous",
                primary_spans.len(),
            ));
        }
        let operation_source = normalized_compiler_path(
            root,
            metadata_string(primary_spans[0], "file_name", "primary diagnostic span")?,
        )?;
        *found
            .entry((
                (*package_name).to_owned(),
                target_name.to_owned(),
                target_source,
                operation_source,
            ))
            .or_insert(0) += 1;
    }
    if !saw_build_finished {
        return Err("nested cargo check emitted no successful build-finished record".to_owned());
    }
    let expected_targets: BTreeSet<(String, String, String)> = population
        .reopenable_packages
        .iter()
        .flat_map(|package| {
            package
                .targets
                .iter()
                .map(|(name, source)| (package.name.clone(), name.clone(), source.clone()))
        })
        .collect();
    if compiled_targets != expected_targets {
        return Err(format!(
            "nested cargo check did not reach the exact metadata target population; compiled \
             {compiled_targets:?}, expected {expected_targets:?}",
        ));
    }
    Ok(found)
}

/// Normalizes one compiler JSON path to the workspace-relative identity pinned
/// by this gate. A diagnostic whose primary source is not a local readable file
/// cannot silently become an admitted operation.
pub(super) fn normalized_compiler_path(root: &Path, path: &str) -> Result<String, String> {
    let path = Path::new(path);
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    let canonical = candidate.canonicalize().map_err(|error| {
        format!(
            "compiler source identity `{}` is not a readable local file: {error}",
            candidate.display(),
        )
    })?;
    if !canonical.starts_with(root) {
        return Err(format!(
            "compiler source identity {} escapes workspace root {}",
            canonical.display(),
            root.display(),
        ));
    }
    Ok(relative_display(root, &canonical))
}
