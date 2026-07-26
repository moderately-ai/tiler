"""Mutation tests for the fail-closed Rust workspace gate."""

from __future__ import annotations

import copy
import json
import os
import shutil
import sys
from pathlib import Path

import pytest

REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = REPOSITORY_ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS))

import check_rust  # noqa: E402
import check_workspace  # noqa: E402


@pytest.fixture(scope="module")
def metadata() -> dict[str, object]:
    """Load the unmodified locked metadata once through the canonical pin."""
    toolchain = check_workspace.configured_toolchain(REPOSITORY_ROOT)
    return check_workspace.cargo_metadata(REPOSITORY_ROOT, toolchain)


def contract_copy(tmp_path: Path, metadata: dict[str, object]) -> tuple[Path, dict[str, object]]:
    """Copy only governed manifests and rewrite metadata paths to that copy."""
    root = tmp_path / "repo"
    root.mkdir()
    for name in ("Cargo.toml", "rustfmt.toml", "rust-toolchain.toml"):
        shutil.copy2(REPOSITORY_ROOT / name, root / name)
    for relative in check_workspace.EXPECTED_MEMBERS:
        destination = root / relative
        destination.mkdir(parents=True)
        shutil.copy2(REPOSITORY_ROOT / relative / "Cargo.toml", destination / "Cargo.toml")
    encoded = json.dumps(metadata).replace(str(REPOSITORY_ROOT), str(root))
    return root, json.loads(encoded)


def replace(path: Path, old: str, new: str) -> None:
    """Apply one exact textual manifest mutation."""
    source = path.read_text()
    assert old in source
    path.write_text(source.replace(old, new, 1))


def errors_for(root: Path, metadata: dict[str, object]) -> str:
    """Render contract errors for concise assertions."""
    return "\n".join(check_workspace.validate_manifest_contract(root, metadata))


def test_current_workspace_contract_is_complete(metadata: dict[str, object]) -> None:
    """Keep the checked-in manifests and resolved target graph canonical."""
    assert check_workspace.validate_manifest_contract(REPOSITORY_ROOT, metadata) == []


@pytest.mark.parametrize(
    ("relative", "old", "new", "category"),
    (
        ("Cargo.toml", 'resolver = "3"', 'resolver = "2"', "workspace.resolver"),
        (
            "Cargo.toml",
            'missing_docs = "warn"',
            'missing_docs = "allow"',
            "workspace.lints.rust",
        ),
        (
            "Cargo.toml",
            'unsafe_code = "forbid"',
            'unsafe_code = "allow"',
            "workspace.lints.rust",
        ),
        (
            "rustfmt.toml",
            'edition = "2024"',
            "disable_all_formatting = true",
            "rustfmt.config",
        ),
        (
            "crates/tiler-artifact/Cargo.toml",
            "[lints]\nworkspace = true",
            "[lints]\nworkspace = false",
            "package.tiler-artifact.lints",
        ),
        (
            "crates/tiler-artifact/Cargo.toml",
            "version.workspace = true",
            'version = "0.0.0"',
            "package.tiler-artifact.manifest",
        ),
    ),
)
def test_manifest_policy_mutations_fail_at_the_contract_boundary(
    tmp_path: Path,
    metadata: dict[str, object],
    relative: str,
    old: str,
    new: str,
    category: str,
) -> None:
    root, copied_metadata = contract_copy(tmp_path, metadata)
    replace(root / relative, old, new)
    assert category in errors_for(root, copied_metadata)


@pytest.mark.parametrize(
    ("field", "value"),
    (
        ("optional", True),
        ("target", "cfg(unix)"),
        ("rename", "renamed-ir"),
        ("kind", "dev"),
        ("uses_default_features", False),
        ("features", ["bypass"]),
    ),
)
def test_dependency_semantics_cannot_hide_behind_the_same_name_set(
    tmp_path: Path,
    metadata: dict[str, object],
    field: str,
    value: object,
) -> None:
    root, copied_metadata = contract_copy(tmp_path, metadata)
    packages = copied_metadata["packages"]
    artifact = next(package for package in packages if package["name"] == "tiler-artifact")
    artifact["dependencies"][0][field] = value
    assert "package.tiler-artifact.dependencies" in errors_for(root, copied_metadata)


def test_unlisted_external_dependency_is_rejected(
    tmp_path: Path, metadata: dict[str, object]
) -> None:
    root, copied_metadata = contract_copy(tmp_path, metadata)
    packages = copied_metadata["packages"]
    artifact = next(package for package in packages if package["name"] == "tiler-artifact")
    extra = copy.deepcopy(artifact["dependencies"][0])
    extra.update(name="num-traits", source=check_workspace.CRATES_IO, req="^0.2.19", path=None)
    artifact["dependencies"].append(extra)
    assert "package.tiler-artifact.dependencies" in errors_for(root, copied_metadata)


@pytest.mark.parametrize(("field", "value"), (("test", False), ("doctest", False), ("doc", False)))
def test_library_target_enablement_is_required(
    tmp_path: Path,
    metadata: dict[str, object],
    field: str,
    value: object,
) -> None:
    root, copied_metadata = contract_copy(tmp_path, metadata)
    packages = copied_metadata["packages"]
    artifact = next(package for package in packages if package["name"] == "tiler-artifact")
    artifact["targets"][0][field] = value
    assert "package.tiler-artifact.targets" in errors_for(root, copied_metadata)


def test_which_integration_tests_exist_is_not_pinned(
    tmp_path: Path, metadata: dict[str, object]
) -> None:
    """Adding or removing a `tests/*.rs` is ordinary work, not a contract change.

    The set used to be pinned, which made writing a Tiler test an edit to this
    checker and rejected it with a whole-target-list diff naming no cause.
    Cargo discovers these targets and runs them; a new one is not a defect.
    """
    root, copied_metadata = contract_copy(tmp_path, metadata)
    ir = next(package for package in copied_metadata["packages"] if package["name"] == "tiler-ir")
    ir["targets"] = [target for target in ir["targets"] if target["name"] != "typed_handles"]
    assert "package.tiler-ir.targets" not in errors_for(root, copied_metadata)


def test_a_target_that_is_compiled_but_never_run_is_rejected(
    tmp_path: Path, metadata: dict[str, object]
) -> None:
    """What the open target set still refuses, in both of its shapes.

    `test = false` leaves the source present and compiling while removing it
    from every run, which is indistinguishable from a passing suite. On the
    primary target it also disables doctests and documentation for a whole
    package.
    """
    root, copied_metadata = contract_copy(tmp_path, metadata)
    packages = copied_metadata["packages"]
    runner = next(package for package in packages if package["name"] == "tiler-prototype-run")
    runner["targets"][0]["test"] = False
    ir = next(package for package in packages if package["name"] == "tiler-ir")
    integration = next(target for target in ir["targets"] if target["kind"] == ["test"])
    integration["test"] = False
    rendered = errors_for(root, copied_metadata)
    assert "package.tiler-prototype-run.targets" in rendered
    assert "compiled but never run" in rendered


@pytest.mark.parametrize(
    "name",
    (
        "RUSTUP_TOOLCHAIN",
        "RUSTUP_HOME",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTC",
        "RUSTC_WRAPPER",
        "RUSTDOC",
        "RUSTDOCFLAGS",
        "RUSTFMT",
        "CLIPPY_DRIVER",
        "CARGO_BUILD_TARGET",
        "CARGO_BUILD_RUSTDOC",
        "CARGO_BUILD_RUSTDOCFLAGS",
        "CARGO_PROFILE_RELEASE_LTO",
        "CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER",
        "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS",
    ),
)
def test_hostile_environment_controls_are_rejected(name: str) -> None:
    assert check_rust.hostile_environment({name: "bypass"}) == [name]
    with pytest.raises(check_rust.GateFailure, match="environment.hostile"):
        check_rust.sanitized_environment({name: "bypass"})


def test_cargo_home_and_target_directory_are_sanitized(tmp_path: Path) -> None:
    environment = check_rust.sanitized_environment(
        {"PATH": os.environ["PATH"], "CARGO_HOME": str(tmp_path), "CARGO_TARGET_DIR": "elsewhere"}
    )
    assert environment["CARGO_HOME"] == str(Path.home() / ".cargo")
    assert environment["CARGO_TARGET_DIR"] == str(check_rust.ROOT / "target")
    assert str(tmp_path) not in environment["PATH"]
    assert environment["PATH"].endswith("/usr/bin:/bin")


def test_ambient_home_cannot_redirect_governed_tools(tmp_path: Path) -> None:
    with pytest.raises(check_rust.GateFailure, match="HOME must identify"):
        check_rust.sanitized_environment({"HOME": str(tmp_path)})


def test_lockfile_mutation_is_a_typed_failure(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    lock = tmp_path / "Cargo.lock"
    lock.write_text("before")
    monkeypatch.setattr(check_rust, "LOCKFILES", (lock,))
    snapshot = check_rust.snapshot_lockfiles()
    lock.write_text("after")
    with pytest.raises(check_rust.GateFailure, match="lockfile.mutated"):
        check_rust.verify_lockfiles(snapshot)


def test_every_retained_spike_diagnostic_has_a_decided_posture() -> None:
    """Keep the checked-in tree conforming to the admission rule it declares."""
    check_rust.validate_spike_evidence_custody()
    governed = (*check_rust.GATED_SPIKE_WORKSPACES, *check_rust.OFF_PIN_SPIKE_WORKSPACES)
    assert check_rust.retained_diagnostics(check_rust.SPIKES)
    assert all(
        any(path.is_relative_to(workspace) for workspace in governed)
        for path in check_rust.retained_diagnostics(check_rust.SPIKES)
    )


def planted_spikes(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    """Point the custody predicate at a synthetic spike tree."""
    spikes = tmp_path / "spikes"
    admitted = spikes / "admitted"
    (admitted / "tests/ui/fail").mkdir(parents=True)
    (admitted / "tests/ui/fail/case.stderr").write_text("error: planted\n")
    off_pin = spikes / "off-pin"
    off_pin.mkdir()
    (off_pin / "held.stderr").write_text("error: planted\n")
    monkeypatch.setattr(check_rust, "ROOT", tmp_path)
    monkeypatch.setattr(check_rust, "SPIKES", spikes)
    monkeypatch.setattr(check_rust, "GATED_SPIKE_WORKSPACES", (admitted,))
    monkeypatch.setattr(check_rust, "OFF_PIN_SPIKE_WORKSPACES", {off_pin: "stable 1.89.0"})
    return spikes


def test_a_retained_diagnostic_outside_every_governed_workspace_is_rejected(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    spikes = planted_spikes(tmp_path, monkeypatch)
    check_rust.validate_spike_evidence_custody()

    (spikes / "newcomer").mkdir()
    (spikes / "newcomer/case.stderr").write_text("error: planted\n")
    with pytest.raises(check_rust.GateFailure, match="spike.ungoverned-evidence"):
        check_rust.validate_spike_evidence_custody()


def test_a_generated_target_tree_neither_supplies_nor_demands_evidence(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    spikes = planted_spikes(tmp_path, monkeypatch)
    # `trybuild` writes its scratch crate under the workspace `target/`, so a
    # rebuilt copy of every fixture diagnostic appears there. Treating one as a
    # retained claim would demand custody for a regenerable file.
    (spikes / "newcomer/target/tests").mkdir(parents=True)
    (spikes / "newcomer/target/tests/case.stderr").write_text("error: regenerated\n")

    check_rust.validate_spike_evidence_custody()


def test_an_off_pin_exclusion_that_retains_nothing_is_rejected(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    spikes = planted_spikes(tmp_path, monkeypatch)
    (spikes / "off-pin/held.stderr").unlink()
    with pytest.raises(check_rust.GateFailure, match="spike.stale-exclusion"):
        check_rust.validate_spike_evidence_custody()


def test_a_spike_run_that_exercised_no_fixture_cannot_report_agreement(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    workspace = tmp_path / "spikes/probe"
    (workspace / "crate/tests/ui/fail").mkdir(parents=True)
    (workspace / "crate/tests/ui/fail/case.rs").write_text("fn main() {}\n")
    monkeypatch.setattr(check_rust, "ROOT", tmp_path)

    check_rust.verify_fixture_coverage(
        workspace, "test tests/ui/fail/case.rs [should fail to compile] ... ok\n"
    )
    # The transcript a vacuous `trybuild` glob produces: an ordinary passing
    # test that names no case at all.
    with pytest.raises(check_rust.GateFailure, match="spike.fixtures"):
        check_rust.verify_fixture_coverage(workspace, "test result: ok. 1 passed; 0 failed;\n")

    (workspace / "crate/tests/ui/fail/case.rs").unlink()
    with pytest.raises(check_rust.GateFailure, match="retains no trybuild case"):
        check_rust.verify_fixture_coverage(workspace, "test result: ok. 1 passed; 0 failed;\n")


def planted_source(tmp_path: Path, body: str) -> Path:
    """Write one synthetic governed package holding a single Rust source file."""
    root = tmp_path / "repo"
    source = root / "prototypes/planted/src"
    source.mkdir(parents=True)
    (source / "buffer.rs").write_text(body, encoding="utf-8")
    return root


PLANTED_PACKAGES = {"planted": "prototypes/planted"}
PLANTED_MEMBERS: dict[str, object] = {"planted": {}}
PLANTED_PATH = "prototypes/planted/src/buffer.rs"
PLANTED_ITEM = "pub fn write(target: &Buffer)"
PLANTED_PIN = {(PLANTED_PATH, PLANTED_ITEM): "planted reason"}
# The admitted form, verbatim in shape: a four-line attribute whose `reason` is
# on its own line, followed by a second attribute before the signature.
ADMITTED_SITE = (
    "//! `unsafe_code` in prose must not register as a site.\n"
    "#[allow(\n"
    "    unsafe_code,\n"
    '    reason = "planted reason"\n'
    ")]\n"
    "#[must_use]\n"
    "pub fn write(target: &Buffer) {\n"
    "}\n"
)


def unsafe_errors(root: Path, admitted: dict[tuple[str, str], str]) -> str:
    """Render site-pin violations for concise assertions."""
    return "\n".join(
        check_workspace.validate_unsafe_site_pins(
            root,
            package_dirs=PLANTED_PACKAGES,
            admitted=admitted,
            diverging_members=PLANTED_MEMBERS,
        )
    )


def test_the_checked_in_unsafe_sites_match_their_pins() -> None:
    """Keep the tree conforming to the per-site half of ADR 0079.

    The scan result is compared to the pin as well as to the empty error list,
    because a scan that reached nothing would also report no violations against
    a table it never looked at.
    """
    assert check_workspace.validate_unsafe_site_pins(REPOSITORY_ROOT) == []
    found, errors = check_workspace.scan_unsafe_allow_sites(
        REPOSITORY_ROOT, check_workspace.PACKAGE_DIRS
    )
    assert errors == []
    assert found == check_workspace.ADMITTED_UNSAFE_SITES
    assert found


def test_the_admitted_multi_line_form_is_recognized_as_one_site(tmp_path: Path) -> None:
    """A per-line search splits the landed four-line attribute; this must not."""
    root = planted_source(tmp_path, ADMITTED_SITE)
    assert unsafe_errors(root, PLANTED_PIN) == ""
    found, errors = check_workspace.scan_unsafe_allow_sites(root, PLANTED_PACKAGES)
    assert errors == []
    assert found == PLANTED_PIN


def test_an_added_site_fails_until_it_is_pinned(tmp_path: Path) -> None:
    """ADR 0079 makes a third site a new decision, not an application of one."""
    added = ADMITTED_SITE + (
        '#[allow(unsafe_code, reason = "second")]\npub fn read(target: &Buffer) {\n}\n'
    )
    root = planted_source(tmp_path, added)
    assert "`pub fn read(target: &Buffer)` admits unsafe_code and is not pinned" in unsafe_errors(
        root, PLANTED_PIN
    )


def test_a_moved_site_fails_even_though_the_count_is_unchanged(tmp_path: Path) -> None:
    """The failure a count predicate cannot see: one site added, one removed."""
    moved = ADMITTED_SITE.replace("pub fn write(target: &Buffer)", "pub fn stash(target: &Buffer)")
    root = planted_source(tmp_path, moved)
    errors = unsafe_errors(root, PLANTED_PIN)
    assert "`pub fn stash(target: &Buffer)` admits unsafe_code and is not pinned" in errors
    assert f"pinned site `{PLANTED_ITEM}` is gone" in errors


def test_a_removed_site_fails_until_its_pin_is_removed(tmp_path: Path) -> None:
    root = planted_source(tmp_path, "pub fn write(target: &Buffer) {\n}\n")
    assert f"pinned site `{PLANTED_ITEM}` is gone" in unsafe_errors(root, PLANTED_PIN)


def test_a_weakened_reason_fails_although_the_site_is_unchanged(tmp_path: Path) -> None:
    """The `reason` is ADR 0079 item 3's second condition, so it is pinned too."""
    weakened = ADMITTED_SITE.replace('"planted reason"', '"it is fine"')
    root = planted_source(tmp_path, weakened)
    assert "states reason 'it is fine', pinned as 'planted reason'" in unsafe_errors(
        root, PLANTED_PIN
    )


def test_a_site_without_a_reason_is_rejected(tmp_path: Path) -> None:
    root = planted_source(tmp_path, "#[allow(unsafe_code)]\npub fn write(target: &Buffer) {\n}\n")
    assert "admits unsafe_code without the `reason`" in unsafe_errors(root, {})


def test_the_token_outside_a_recognized_attribute_is_reported(tmp_path: Path) -> None:
    """A `cfg_attr`, macro, or string literal must stop the gate, not pass unseen."""
    root = planted_source(
        tmp_path,
        '#[cfg_attr(target_os = "macos", allow(unsafe_code))]\n'
        "pub fn write(target: &Buffer) {\n}\n",
    )
    assert "`unsafe_code` appears outside a recognized" in unsafe_errors(root, {})


def test_prose_mentioning_the_lint_is_not_a_site(tmp_path: Path) -> None:
    """`buffer.rs`'s own module documentation names the lint three times."""
    root = planted_source(
        tmp_path,
        "//! The crate denies `unsafe_code`.\n// unsafe_code again.\npub fn write() {\n}\n",
    )
    assert unsafe_errors(root, {}) == ""


def test_a_bracket_inside_a_reason_does_not_end_the_attribute(tmp_path: Path) -> None:
    """Counting brackets inside a literal would close the span early."""
    body = ADMITTED_SITE.replace('"planted reason"', '"copy_nonoverlapping(src, dst) over &[f32]"')
    root = planted_source(tmp_path, body)
    found, errors = check_workspace.scan_unsafe_allow_sites(root, PLANTED_PACKAGES)
    assert errors == []
    assert found == {(PLANTED_PATH, PLANTED_ITEM): "copy_nonoverlapping(src, dst) over &[f32]"}


def test_a_multi_line_trailing_attribute_is_skipped_whole(tmp_path: Path) -> None:
    """Its continuation must not be mistaken for the admitted item's signature."""
    body = ADMITTED_SITE.replace(
        "#[must_use]\n", '#[cfg_attr(\n    target_os = "macos",\n    must_use\n)]\n'
    )
    root = planted_source(tmp_path, body)
    found, errors = check_workspace.scan_unsafe_allow_sites(root, PLANTED_PACKAGES)
    assert errors == []
    assert found == PLANTED_PIN


def test_a_pin_outside_a_diverging_member_is_rejected(tmp_path: Path) -> None:
    """A site can only compile where the workspace `forbid` was replaced."""
    root = planted_source(tmp_path, ADMITTED_SITE)
    errors = "\n".join(
        check_workspace.validate_unsafe_site_pins(
            root,
            package_dirs=PLANTED_PACKAGES,
            admitted=PLANTED_PIN,
            diverging_members={},
        )
    )
    assert 'inherits the workspace `unsafe_code = "forbid"`' in errors


FRAMING_PATH = "prototypes/planted/src/identity.rs"
FRAMING_ITEM = "pub fn push_len(bytes: &mut Vec<u8>, len: usize)"
FRAMING_PIN = {(FRAMING_PATH, FRAMING_ITEM): ("planted reason", "the sole framing")}
# The admitted form: a documented sink-plus-payload helper, with the citation
# the pin requires somewhere in its documentation.
ADMITTED_FRAMING = (
    "//! A module whose prose says `.len()` and `to_be_bytes()` without framing.\n"
    "\n"
    "/// Appends the canonical prefix.\n"
    "///\n"
    "/// This is the sole framing this package is permitted.\n"
    "#[inline]\n"
    "pub fn push_len(bytes: &mut Vec<u8>, len: usize) {\n"
    '    let len = u64::try_from(len).expect("64-bit");\n'
    "    bytes.extend_from_slice(&len.to_be_bytes());\n"
    "}\n"
    "\n"
    "#[cfg(test)]\n"
    "mod tests {\n"
    "    #[test]\n"
    "    fn the_prefix_is_eight_bytes() {\n"
    "        assert_eq!(subject.len() as u64, 8_u64.to_be_bytes().len() as u64);\n"
    "    }\n"
    "}\n"
)


def planted_framing(tmp_path: Path, body: str, *, name: str = "identity.rs") -> Path:
    """Write one synthetic governed package holding a single Rust source file."""
    root = tmp_path / "repo"
    source = root / "prototypes/planted/src"
    source.mkdir(parents=True, exist_ok=True)
    (source / name).write_text(body, encoding="utf-8")
    return root


def framing_errors(root: Path, admitted: dict[tuple[str, str], tuple[str, str]]) -> str:
    """Render length-framing violations for concise assertions."""
    return "\n".join(
        check_workspace.validate_length_framing_pins(
            root, package_dirs=PLANTED_PACKAGES, admitted=admitted
        )
    )


def test_the_checked_in_framing_sites_match_their_pins() -> None:
    """Keep the tree to one length framing per crate that is permitted one.

    The scan result is compared to the pin as well as to the empty error list,
    because a scan that reached nothing would also report no violations against
    a table it never looked at.
    """
    assert check_workspace.validate_length_framing_pins(REPOSITORY_ROOT) == []
    found, errors, population = check_workspace.scan_length_framing_sites(
        REPOSITORY_ROOT, check_workspace.PACKAGE_DIRS
    )
    assert errors == []
    assert set(found) == set(check_workspace.FRAMING_SITE_CITATIONS)
    assert found
    assert set(population) == set(check_workspace.PACKAGE_DIRS)
    assert all(count > 0 for count in population.values())


def test_the_admitted_form_is_recognized_and_its_tests_are_not(tmp_path: Path) -> None:
    """One site, and neither the module prose nor the test module beneath it."""
    root = planted_framing(tmp_path, ADMITTED_FRAMING)
    assert framing_errors(root, FRAMING_PIN) == ""
    found, errors, _population = check_workspace.scan_length_framing_sites(root, PLANTED_PACKAGES)
    assert errors == []
    assert set(found) == {(FRAMING_PATH, FRAMING_ITEM)}


def test_a_second_copy_in_a_permitted_crate_fails(tmp_path: Path) -> None:
    """The whole point of admitting a forced copy: a second one is a diff."""
    added = ADMITTED_FRAMING + (
        "/// A second framing.\n"
        "pub fn push_count(bytes: &mut Vec<u8>, count: usize) {\n"
        "    bytes.extend_from_slice(&(count as u64).to_be_bytes());\n"
        "}\n"
    )
    root = planted_framing(tmp_path, added)
    assert "`pub fn push_count(bytes: &mut Vec<u8>, count: usize)` frames a length" in (
        framing_errors(root, FRAMING_PIN)
    )


def framing_subjects(root: Path) -> set[str]:
    """Return the subjects the scan recognized, with no pin comparison."""
    found, errors, _population = check_workspace.scan_length_framing_sites(root, PLANTED_PACKAGES)
    assert errors == []
    return {subject for _path, subject in found}


def test_a_copy_under_any_helper_name_is_caught(tmp_path: Path) -> None:
    """The failure a name list cannot see: `encode_count` defeated the last one."""
    root = planted_framing(
        tmp_path,
        "/// Frames a length under a name no list held.\n"
        "fn scribble_ordinal(sink: &mut Vec<u8>, value: usize) {\n"
        "    sink.extend_from_slice(&(value as u64).to_be_bytes());\n"
        "}\n",
    )
    assert framing_subjects(root) == {"fn scribble_ordinal(sink: &mut Vec<u8>, value: usize)"}


def test_a_sink_helper_that_frames_nothing_is_not_a_site(tmp_path: Path) -> None:
    """The signature is necessary, not sufficient.

    `&mut Vec<u8>` plus one payload is also the shape of any ordinary
    serializer helper. Matching on it alone rejected a body that was a single
    `extend_from_slice` -- framing no length at all -- and told its author to
    call a framing primitive instead, which in a workspace this full of
    canonical encoding blocked the most common function there is to write.
    """
    root = planted_framing(
        tmp_path,
        "/// Appends a payload verbatim, framing nothing.\n"
        "fn append(sink: &mut Vec<u8>, value: &[u8]) {\n"
        "    sink.extend_from_slice(value);\n"
        "}\n",
    )
    assert framing_subjects(root) == set()


def test_a_helper_less_copy_wrapped_across_lines_is_caught(tmp_path: Path) -> None:
    """`rustfmt` wraps at 100 columns, which a per-line search cannot follow."""
    root = planted_framing(
        tmp_path,
        "fn encode(output: &mut Vec<u8>, shape: &Shape) {\n"
        "    output.extend_from_slice(\n"
        "        &u64::try_from(shape.rank())\n"
        "            .unwrap_or(u64::MAX)\n"
        "            .to_be_bytes(),\n"
        "    );\n"
        "}\n",
    )
    assert framing_subjects(root) == {
        "output.extend_from_slice( &u64::try_from(shape.rank()) .unwrap_or(u64::MAX) "
        ".to_be_bytes(), );"
    }


def test_counting_and_comparison_are_not_framing(tmp_path: Path) -> None:
    """`tiler-cache` holds each of these, and rewriting them would be the defect."""
    root = planted_framing(
        tmp_path,
        "fn scan(bytes: &[u8], limit: u64) -> u64 {\n"
        "    if bytes.len() as u64 > limit {\n"
        "        return 0;\n"
        "    }\n"
        "    self.entries.len() as u64\n"
        "}\n",
    )
    assert framing_subjects(root) == set()


def test_a_renamed_framing_site_fails_even_though_the_count_is_unchanged(tmp_path: Path) -> None:
    """The failure a count predicate cannot see: one site added, one removed."""
    moved = ADMITTED_FRAMING.replace("fn push_len(", "fn push_prefix(")
    root = planted_framing(tmp_path, moved)
    errors = framing_errors(root, FRAMING_PIN)
    assert "`pub fn push_prefix(bytes: &mut Vec<u8>, len: usize)` frames a length" in errors
    assert f"pinned site `{FRAMING_ITEM}` is gone" in errors


def test_a_dropped_citation_fails_although_the_site_is_unchanged(tmp_path: Path) -> None:
    """The decision that forces a copy is the substance of admitting it."""
    weakened = ADMITTED_FRAMING.replace("the sole framing this package is permitted", "fine")
    root = planted_framing(tmp_path, weakened)
    assert "admitted on the strength of 'the sole framing'" in framing_errors(root, FRAMING_PIN)


def test_an_empty_population_fails_rather_than_reporting_success(tmp_path: Path) -> None:
    """Finding nothing is what a broken search does, so it cannot be a pass.

    This check exists partly because a sweep for these copies returned six clean
    results after `zsh` expanded an unquoted `--include` glob.
    """
    root = tmp_path / "repo"
    (root / "prototypes/planted/src").mkdir(parents=True)
    errors = framing_errors(root, {})
    assert "`planted` contributed no production source" in errors
    assert "the admitted table is empty" in errors


def test_a_scan_over_no_package_at_all_fails(tmp_path: Path) -> None:
    """A `package_dirs` that silently shrank must not read as a clean tree."""
    root = planted_framing(tmp_path, ADMITTED_FRAMING)
    errors = "\n".join(
        check_workspace.validate_length_framing_pins(root, package_dirs={}, admitted=FRAMING_PIN)
    )
    assert "no package was scanned" in errors
    assert f"pinned site `{FRAMING_ITEM}` is gone" in errors


def test_a_whole_file_test_module_is_excluded(tmp_path: Path) -> None:
    """Nine such files exist; `tests.rs` holds no `#[cfg(test)]` of its own."""
    root = planted_framing(tmp_path, "#[cfg(test)]\nmod checks;\n", name="lib.rs")
    planted_framing(
        tmp_path,
        "fn corrupt(bytes: &[u8]) -> Vec<u8> {\n"
        "    (bytes.len() as u64 + 1).to_be_bytes().to_vec()\n"
        "}\n",
        name="checks.rs",
    )
    assert framing_subjects(root) == set()


def test_an_unresolvable_test_module_stops_the_gate(tmp_path: Path) -> None:
    """A declaration whose file is missing must not leave the scan quietly short."""
    root = planted_framing(tmp_path, "#[cfg(test)]\nmod absent;\n", name="lib.rs")
    _found, errors, _population = check_workspace.scan_length_framing_sites(root, PLANTED_PACKAGES)
    assert any("`#[cfg(test)] mod absent;` resolves to no file" in error for error in errors)
