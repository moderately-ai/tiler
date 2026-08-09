use super::*;

/// The workspace root, two levels above the facade crate.
pub(super) fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("the facade crate sits two levels below the workspace root")
        .to_path_buf();
    let manifest = root.join("Cargo.toml");
    let text = read(&manifest);
    assert!(
        text.contains("[workspace]"),
        "{} declares no workspace",
        manifest.display(),
    );
    root
}

/// Cargo's actual workspace packages and target roots, cross-checked against
/// the explicit root-member list.
pub(super) fn workspace_population(root: &Path) -> Result<WorkspacePopulation, String> {
    let canonical_root = root.canonicalize().map_err(|error| {
        format!(
            "unsafe-sites.{}: workspace root is not canonical: {error}",
            root.display()
        )
    })?;
    let explicit_paths = explicit_member_paths(root);
    let mut explicit_roots = BTreeSet::new();
    for member in &explicit_paths {
        let directory = root.join(member);
        let canonical = directory.canonicalize().map_err(|error| {
            format!(
                "unsafe-sites.{}: explicit workspace member `{member}` is not a readable \
                 directory: {error}",
                root.join("Cargo.toml").display(),
            )
        })?;
        if !canonical.starts_with(&canonical_root) {
            return Err(format!(
                "unsafe-sites.{}: explicit workspace member `{member}` resolves outside the \
                 workspace root",
                root.join("Cargo.toml").display(),
            ));
        }
        if !explicit_roots.insert(canonical) {
            return Err(format!(
                "unsafe-sites.{}: explicit workspace member paths alias one directory",
                root.join("Cargo.toml").display(),
            ));
        }
    }

    // This is the repository's existing workspace-population authority. Cargo
    // metadata resolves manifests only; it does not build a target and cannot
    // recursively run this integration test. Dependencies are included because
    // proc-macro target kind is a property of the dependency package, not of a
    // workspace member's dependency declaration.
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--locked", "--format-version", "1"])
        .current_dir(&canonical_root)
        .output()
        .map_err(|error| format!("unsafe-site census: cargo metadata could not run: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "unsafe-site census: cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr),
        ));
    }
    let metadata: Value = serde_json::from_slice(&output.stdout).map_err(|error| {
        format!("unsafe-site census: cargo metadata emitted invalid JSON: {error}")
    })?;
    let metadata_root = metadata_string(&metadata, "workspace_root", "metadata root")?;
    let metadata_root = Path::new(metadata_root).canonicalize().map_err(|error| {
        format!("unsafe-site census: metadata workspace root is not canonical: {error}")
    })?;
    if metadata_root != canonical_root {
        return Err(format!(
            "unsafe-site census: cargo metadata described workspace root {}, expected {}",
            metadata_root.display(),
            canonical_root.display(),
        ));
    }

    let member_ids: BTreeSet<&str> = metadata_array(&metadata, "workspace_members", "metadata")?
        .iter()
        .map(|id| {
            id.as_str().ok_or_else(|| {
                "unsafe-site census: a workspace member ID is not a string".to_owned()
            })
        })
        .collect::<Result<_, _>>()?;
    if member_ids.len() < MEMBER_POPULATION_FLOOR {
        return Err(format!(
            "unsafe-site census: cargo metadata yielded {} package(s), below the floor of \
             {MEMBER_POPULATION_FLOOR}",
            member_ids.len(),
        ));
    }

    let mut actual_roots = BTreeSet::new();
    let mut target_roots = BTreeSet::new();
    let mut doctest_roots = BTreeSet::new();
    let mut doctest_package_roots = BTreeSet::new();
    let mut reopenable_packages = Vec::new();
    let mut target_count = 0_usize;
    for package in metadata_array(&metadata, "packages", "metadata")? {
        let id = metadata_string(package, "id", "package")?;
        if !member_ids.contains(id) {
            continue;
        }
        let manifest = Path::new(metadata_string(package, "manifest_path", id)?);
        let package_root = manifest
            .parent()
            .ok_or_else(|| format!("unsafe-site census: {id} has no manifest parent"))?
            .canonicalize()
            .map_err(|error| {
                format!("unsafe-site census: {id}'s manifest root is not canonical: {error}")
            })?;
        if !package_root.starts_with(&canonical_root) {
            return Err(format!(
                "unsafe-site census: workspace package {id} lives outside the workspace root at {}",
                package_root.display(),
            ));
        }
        actual_roots.insert(package_root.clone());
        let name = metadata_string(package, "name", id)?;
        let reopenable = manifest_has_local_unsafe_deny(manifest)?;
        let mut package_targets = BTreeSet::new();

        let targets = metadata_array(package, "targets", id)?;
        if targets.is_empty() {
            return Err(format!(
                "unsafe-site census: workspace package {id} has no Cargo targets"
            ));
        }
        for target in targets {
            target_count += 1;
            let target_name = metadata_string(target, "name", "Cargo target")?;
            let path = Path::new(metadata_string(target, "src_path", "Cargo target")?);
            let canonical = path.canonicalize().map_err(|error| {
                format!(
                    "unsafe-site census: Cargo target root {} is not a readable file: {error}",
                    path.display(),
                )
            })?;
            if !canonical.starts_with(&package_root) {
                return Err(format!(
                    "unsafe-site census: Cargo target root {} escapes owning package {}",
                    canonical.display(),
                    package_root.display(),
                ));
            }
            if !target_roots.insert(canonical.clone()) {
                return Err(format!(
                    "unsafe-site census: Cargo target root {} is compiled as more than one \
                     target; permission identity would be ambiguous",
                    canonical.display(),
                ));
            }
            package_targets.insert((
                target_name.to_owned(),
                relative_display(&canonical_root, &canonical),
            ));
            let doctest = target
                .get("doctest")
                .and_then(Value::as_bool)
                .ok_or_else(|| {
                    format!("unsafe-site census: Cargo target in {id} has no boolean `doctest`")
                })?;
            if doctest {
                let kinds = metadata_array(target, "kind", "Cargo target")?;
                let supported = kinds.len() == 1
                    && kinds[0]
                        .as_str()
                        .is_some_and(|kind| matches!(kind, "lib" | "proc-macro"));
                if !supported {
                    return Err(format!(
                        "unsafe-site census: doctest-enabled target {} in {id} has unsupported \
                         kind(s) {kinds:?}; its extracted-crate unsafe boundary is unknown",
                        canonical.display(),
                    ));
                }
                let text = std::fs::read_to_string(&canonical).map_err(|error| {
                    format!(
                        "unsafe-site census: doctest root {} is not readable UTF-8: {error}",
                        canonical.display(),
                    )
                })?;
                let sentinel_count = text
                    .lines()
                    .filter(|line| line.trim() == DOCTEST_UNSAFE_SENTINEL)
                    .count();
                if sentinel_count != 1 {
                    return Err(format!(
                        "unsafe-sites.{}: doctest-enabled Cargo target must contain exactly one \
                         `{DOCTEST_UNSAFE_SENTINEL}` crate-root sentinel; found {sentinel_count}",
                        relative_display(&canonical_root, &canonical),
                    ));
                }
                let relative = relative_display(&canonical_root, &canonical);
                let root_sentinel_count = root_doctest_sentinel_count(&relative, &text)?;
                if root_sentinel_count != 1 {
                    return Err(format!(
                        "unsafe-sites.{relative}: doctest unsafe sentinel must be in the Cargo target's \
                         crate-level attribute population; a nested module \
                         attribute does not govern every extracted doctest crate; found \
                         {root_sentinel_count}",
                    ));
                }
                doctest_roots.insert(canonical);
                doctest_package_roots.insert(package_root.clone());
            }
        }
        if reopenable {
            reopenable_packages.push(ReopenablePackage {
                id: id.to_owned(),
                name: name.to_owned(),
                targets: package_targets,
            });
        }
    }
    if actual_roots.len() != member_ids.len() {
        return Err(format!(
            "unsafe-site census: cargo metadata named {} workspace member ID(s) but {} package \
             object(s) resolved",
            member_ids.len(),
            actual_roots.len(),
        ));
    }

    let metadata_only: Vec<String> = actual_roots
        .difference(&explicit_roots)
        .map(|path| relative_display(&canonical_root, path))
        .collect();
    let explicit_only: Vec<String> = explicit_roots
        .difference(&actual_roots)
        .map(|path| relative_display(&canonical_root, path))
        .collect();
    if !metadata_only.is_empty() || !explicit_only.is_empty() {
        return Err(format!(
            "unsafe-site census: explicit root members and cargo metadata workspace packages \
             differ; implicit/metadata-only: {metadata_only:?}; explicit-only: {explicit_only:?}",
        ));
    }
    if target_count < TARGET_POPULATION_FLOOR {
        return Err(format!(
            "unsafe-site census: cargo metadata yielded {target_count} target(s), below the \
             floor of {TARGET_POPULATION_FLOOR}",
        ));
    }
    if doctest_roots.len() < DOCTEST_ROOT_FLOOR {
        return Err(format!(
            "unsafe-site census: cargo metadata yielded {} doctest root(s), below the floor of \
             {DOCTEST_ROOT_FLOOR}",
            doctest_roots.len(),
        ));
    }
    reopenable_packages.sort_by(|left, right| left.name.cmp(&right.name));
    if reopenable_packages.len() != 2 {
        return Err(format!(
            "unsafe-site census: expected the two checked local `unsafe_code = \"deny\"` \
             packages, found {:?}",
            reopenable_packages
                .iter()
                .map(|package| package.name.as_str())
                .collect::<Vec<_>>(),
        ));
    }
    reject_direct_proc_macro_dependencies(&metadata, &reopenable_packages)?;
    require_exact_tensor_facade_dependency(&metadata, &member_ids)?;

    Ok(WorkspacePopulation {
        member_roots: actual_roots.into_iter().collect(),
        target_roots: target_roots.into_iter().collect(),
        target_count,
        doctest_roots: doctest_roots.into_iter().collect(),
        doctest_package_roots: doctest_package_roots.into_iter().collect(),
        reopenable_packages,
    })
}

/// Proves the `tiler_macros` spelling behind the facade re-export resolves to
/// the one workspace-owned procedural-macro package rather than a Cargo rename.
pub(super) fn require_exact_tensor_facade_dependency(
    metadata: &Value,
    member_ids: &BTreeSet<&str>,
) -> Result<(), String> {
    let packages = metadata_array(metadata, "packages", "metadata")?;
    let package_ids = |name: &str| {
        packages
            .iter()
            .filter(|package| {
                package
                    .get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|found| found == name)
                    && package
                        .get("id")
                        .and_then(Value::as_str)
                        .is_some_and(|id| member_ids.contains(id))
            })
            .filter_map(|package| package.get("id").and_then(Value::as_str))
            .collect::<Vec<_>>()
    };
    let facades = package_ids("tiler");
    let producers = package_ids("tiler-macros");
    if facades.len() != 1 || producers.len() != 1 {
        return Err(format!(
            "unsafe-site tensor facade identity requires one workspace `tiler` and one \
             workspace `tiler-macros` package; found {facades:?} and {producers:?}",
        ));
    }

    let resolve = metadata
        .get("resolve")
        .ok_or_else(|| "unsafe-site census: cargo metadata has no resolve graph".to_owned())?;
    let node = metadata_array(resolve, "nodes", "metadata resolve graph")?
        .iter()
        .find(|node| node.get("id").and_then(Value::as_str) == Some(facades[0]))
        .ok_or_else(|| "unsafe-site tensor facade has no resolve node".to_owned())?;
    let bindings = metadata_array(node, "deps", "tiler resolve node")?
        .iter()
        .filter(|dependency| dependency.get("name").and_then(Value::as_str) == Some("tiler_macros"))
        .map(|dependency| metadata_string(dependency, "pkg", "tiler_macros dependency"))
        .collect::<Result<Vec<_>, _>>()?;
    if bindings.as_slice() != producers.as_slice() {
        return Err(format!(
            "unsafe-site tensor facade dependency identity changed: Cargo binding \
             `tiler_macros` resolves to {bindings:?}, expected workspace producer {producers:?}",
        ));
    }
    let shadowed_builtins = metadata_array(node, "deps", "tiler resolve node")?
        .iter()
        .filter_map(|dependency| dependency.get("name").and_then(Value::as_str))
        .filter(|name| matches!(*name, "core" | "std"))
        .collect::<Vec<_>>();
    if !shadowed_builtins.is_empty() {
        return Err(format!(
            "unsafe-site compiler diagnostic identity changed: facade Cargo dependency \
             binding(s) {shadowed_builtins:?} shadow compiler namespaces used by the exact \
             `core::compile_error` re-export",
        ));
    }
    Ok(())
}

/// Refuses a direct proc-macro package edge from either reopenable package as
/// an early diagnostic. The all-member source-language census is the closure
/// authority because transitive and re-exported macros need not be direct
/// edges.
pub(super) fn reject_direct_proc_macro_dependencies(
    metadata: &Value,
    reopenable_packages: &[ReopenablePackage],
) -> Result<(), String> {
    let packages = metadata_array(metadata, "packages", "metadata")?;
    let mut proc_macro_ids = BTreeSet::new();
    let mut names = BTreeMap::new();
    for package in packages {
        let id = metadata_string(package, "id", "package")?;
        names.insert(id, metadata_string(package, "name", id)?);
        let is_proc_macro = metadata_array(package, "targets", id)?
            .iter()
            .any(|target| {
                metadata_array(target, "kind", "Cargo target")
                    .is_ok_and(|kinds| kinds.iter().any(|kind| kind.as_str() == Some("proc-macro")))
            });
        if is_proc_macro {
            proc_macro_ids.insert(id);
        }
    }
    let resolve = metadata
        .get("resolve")
        .ok_or_else(|| "unsafe-site census: cargo metadata has no resolve graph".to_owned())?;
    let nodes = metadata_array(resolve, "nodes", "metadata resolve graph")?;
    for package in reopenable_packages {
        let node = nodes
            .iter()
            .find(|node| node.get("id").and_then(Value::as_str) == Some(package.id.as_str()))
            .ok_or_else(|| {
                format!(
                    "unsafe-site census: resolve graph has no node for reopenable package {}",
                    package.name,
                )
            })?;
        for dependency in metadata_array(node, "deps", &package.name)? {
            let dependency_id = metadata_string(dependency, "pkg", "resolved dependency")?;
            if proc_macro_ids.contains(dependency_id) {
                return Err(format!(
                    "unsafe-site census: reopenable package `{}` directly depends on proc-macro \
                     package `{}`; external macro expansion suppresses unsafe-code diagnostics \
                     and is outside the admitted source-language boundary",
                    package.name,
                    names.get(dependency_id).copied().unwrap_or(dependency_id),
                ));
            }
        }
    }
    Ok(())
}

/// Whether one member manifest locally replaces the workspace's forbidden
/// unsafe lint with the checked, reopenable `deny` boundary.
pub(super) fn manifest_has_local_unsafe_deny(manifest: &Path) -> Result<bool, String> {
    let text = std::fs::read_to_string(manifest).map_err(|error| {
        format!(
            "unsafe-site census: member manifest {} is not readable UTF-8: {error}",
            manifest.display(),
        )
    })?;
    let mut in_rust_lints = false;
    let mut found = false;
    for line in text.lines() {
        let code = manifest_code(line);
        let trimmed = code.trim();
        if trimmed.starts_with('[') {
            in_rust_lints = trimmed == "[lints.rust]";
            continue;
        }
        if in_rust_lints && trimmed.starts_with("unsafe_code") {
            if trimmed != "unsafe_code = \"deny\"" {
                return Err(format!(
                    "unsafe-site census: {} has unsupported local unsafe lint syntax `{trimmed}`",
                    manifest.display(),
                ));
            }
            if found {
                return Err(format!(
                    "unsafe-site census: {} repeats its local unsafe lint",
                    manifest.display(),
                ));
            }
            found = true;
        }
    }
    Ok(found)
}

/// The member paths declared literally by the root manifest.
///
/// This narrow parser intentionally recognizes only the table-and-array form
/// the repository uses. Cargo metadata is cross-checked against it, so an
/// implicit path member cannot hide behind the parser's literal boundary.
pub(super) fn explicit_member_paths(root: &Path) -> Vec<String> {
    let manifest = root.join("Cargo.toml");
    let text = read(&manifest);
    let lines: Vec<&str> = text.lines().collect();
    let mut in_workspace = false;

    for (index, line) in lines.iter().enumerate() {
        let trimmed = manifest_code(line).trim().to_owned();
        if trimmed.starts_with('[') {
            in_workspace = trimmed == "[workspace]";
            continue;
        }
        if !in_workspace {
            continue;
        }
        let Some(value) = trimmed.strip_prefix("members") else {
            continue;
        };
        let value = value.trim_start().strip_prefix('=').unwrap_or_else(|| {
            panic!(
                "{}:{}: `members` has no `=` and cannot be read",
                manifest.display(),
                index + 1,
            )
        });

        let mut array = value.to_owned();
        let mut cursor = index;
        while !array.contains(']') {
            cursor += 1;
            assert!(
                cursor < lines.len(),
                "{}:{}: the `members` array never closes",
                manifest.display(),
                index + 1,
            );
            array.push('\n');
            array.push_str(&manifest_code(lines[cursor]));
        }
        let members = quoted_values(&array, &manifest, index + 1);
        let unique: BTreeSet<&str> = members.iter().map(String::as_str).collect();
        assert_eq!(
            unique.len(),
            members.len(),
            "{}:{}: the member list repeats a path",
            manifest.display(),
            index + 1,
        );
        return members;
    }

    panic!(
        "{} has no `members` key under `[workspace]`; the unsafe-site scan has no roots",
        manifest.display(),
    );
}

/// Collects every Rust source beneath every actual member plus every Cargo
/// target root, including target roots whose extension is not `.rs`.
pub(super) fn workspace_sources(
    member_roots: &[PathBuf],
    target_roots: &[PathBuf],
) -> Vec<PathBuf> {
    let mut all = target_roots.to_vec();
    for directory in member_roots {
        let mut sources = Vec::new();
        collect_rust_sources(directory, &mut sources);
        assert!(
            !sources.is_empty(),
            "workspace member `{}` contributes no Rust source file; a member omitted from \
             the walk would otherwise look safely empty",
            directory.display(),
        );
        all.extend(sources);
    }
    all.sort();
    all.dedup();
    all
}

/// One required string property from metadata JSON.
pub(super) fn metadata_string<'a>(
    value: &'a Value,
    key: &str,
    owner: &str,
) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("unsafe-site census: {owner} has no string `{key}`"))
}

/// One required array property from metadata JSON.
pub(super) fn metadata_array<'a>(
    value: &'a Value,
    key: &str,
    owner: &str,
) -> Result<&'a [Value], String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("unsafe-site census: {owner} has no array `{key}`"))
}

/// A stable workspace-relative display path.
pub(super) fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Recursively collects Rust source and rejects symlinks at the scan boundary.
pub(super) fn collect_rust_sources(directory: &Path, into: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", directory.display()));
    for entry in entries {
        let entry = entry.expect("a workspace source directory entry is readable");
        let path = entry.path();
        let kind = entry
            .file_type()
            .unwrap_or_else(|error| panic!("{} has a readable file type: {error}", path.display()));
        assert!(
            !kind.is_symlink(),
            "{} is a symlink inside a workspace member; following it could escape or duplicate \
             the governed source population",
            path.display(),
        );
        if kind.is_dir() {
            collect_rust_sources(&path, into);
        } else if kind.is_file()
            && path.extension().and_then(|extension| extension.to_str()) == Some("rs")
        {
            into.push(path);
        }
    }
}

/// Scans every initial source and follows literal local source-loading edges.
pub(super) fn scan_files(
    root: &Path,
    member_roots: &[PathBuf],
    target_roots: &[PathBuf],
    doctest_package_roots: &[PathBuf],
    sources: &[PathBuf],
) -> (Scan, usize) {
    let mut whole = Scan::default();
    let mut queue: VecDeque<PathBuf> = sources.iter().cloned().collect();
    let mut seen = BTreeSet::new();
    let mut nonstandard_loaders: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();

    while let Some(source) = queue.pop_front() {
        if !seen.insert(source.clone()) {
            continue;
        }
        let relative = source
            .strip_prefix(root)
            .expect("a member source lies under the workspace root")
            .to_string_lossy()
            .replace('\\', "/");
        let text = read(&source);
        let scan = scan_text(&relative, &text);
        whole.errors.extend(scan.errors);
        whole.builtin_macros.extend(scan.builtin_macros);
        whole.builtin_attributes.extend(scan.builtin_attributes);
        whole.builtin_derives.extend(scan.builtin_derives);
        whole.local_macro_rules.extend(scan.local_macro_rules);
        whole.proc_macro_exporters.extend(scan.proc_macro_exporters);
        whole.facade_reexports += scan.facade_reexports;
        whole.facade_diagnostic_reexports += scan.facade_diagnostic_reexports;
        for (invocation_path, count) in scan.tensor_invocations {
            *whole.tensor_invocations.entry(invocation_path).or_default() += count;
        }
        if doctest_package_roots
            .iter()
            .any(|package| source.starts_with(package))
        {
            let documentation = scan_rustdoc_code(&relative, &text);
            whole.errors.extend(documentation.errors);
            whole.builtin_macros.extend(documentation.macros);
            whole.builtin_attributes.extend(documentation.attributes);
            whole.builtin_derives.extend(documentation.derives);
            for (invocation_path, count) in documentation.tensor_invocations {
                *whole
                    .rustdoc_tensor_invocations
                    .entry(invocation_path)
                    .or_default() += count;
            }
        }
        for load in scan.loads {
            match resolve_source_load(root, member_roots, target_roots, &source, &load) {
                Ok(loaded) => {
                    nonstandard_loaders
                        .entry(loaded.clone())
                        .or_default()
                        .push(format!("{relative}:{} via {}", load.line, load.kind));
                    queue.push_back(loaded);
                }
                Err(error) => whole.errors.push(error),
            }
        }
        for (key, reason) in scan.sites {
            if whole.sites.insert(key.clone(), reason).is_some() {
                whole.errors.push(format!(
                    "unsafe-sites.{}: `{}` is reported twice",
                    key.0, key.1,
                ));
            }
        }
    }

    for (loaded, loaders) in nonstandard_loaders {
        let relative = relative_display(root, &loaded);
        for ((path, item), _) in whole
            .sites
            .iter()
            .filter(|((path, _), _)| path == &relative)
        {
            whole.errors.push(format!(
                "unsafe-sites.{path}: `{item}` carries a permission in a source reached through \
                 include!/#[path] ({loaders:?}); nonstandard loads can duplicate semantic sites \
                 and are outside the file-root pin boundary",
            ));
        }
    }
    (whole, seen.len())
}

/// Resolves one literal source-loading edge and keeps it inside a governed
/// workspace package. Canonicalization collapses aliases; the queue's visited
/// set terminates cycles.
pub(super) fn resolve_source_load(
    root: &Path,
    member_roots: &[PathBuf],
    target_roots: &[PathBuf],
    source: &Path,
    load: &SourceLoad,
) -> Result<PathBuf, String> {
    if load.kind == "#[path]"
        && source.file_name().and_then(|name| name.to_str()) != Some("mod.rs")
        && !target_roots.iter().any(|target| target == source)
    {
        return Err(format!(
            "unsafe-sites.{}:{}: #[path] in a non-mod.rs module source is unsupported; rustc's \
             module-directory rules depend on semantic context that this lexical inventory \
             refuses to guess",
            relative_display(root, source),
            load.line,
        ));
    }
    let candidate = source
        .parent()
        .expect("a source file has a parent")
        .join(&load.literal);
    let canonical = candidate.canonicalize().map_err(|error| {
        format!(
            "unsafe-sites.{}:{}: {} source `{}` is not a readable file: {error}",
            relative_display(root, source),
            load.line,
            load.kind,
            load.literal,
        )
    })?;
    if !canonical.is_file() {
        return Err(format!(
            "unsafe-sites.{}:{}: {} source `{}` is not a file",
            relative_display(root, source),
            load.line,
            load.kind,
            load.literal,
        ));
    }
    if !member_roots
        .iter()
        .any(|member| canonical.starts_with(member))
    {
        return Err(format!(
            "unsafe-sites.{}:{}: {} source `{}` resolves outside every governed workspace \
             package to {}",
            relative_display(root, source),
            load.line,
            load.kind,
            load.literal,
            canonical.display(),
        ));
    }
    Ok(canonical)
}
