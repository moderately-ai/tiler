"""Perturbation tests for the self-contained embedding driver.

Every check the driver acts on is run here against a case that must fail, and
watched failing. The reason is specific rather than general: this driver's two
load-bearing predicates are a *deletion* check and an *absence* check, and both
belong to the family that reads as success when it does not run at all. A
deletion check that passes because the path was wrong proves nothing, and an
"expansion names no path" check that passes because it was handed empty text
proves less.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

MODULE_PATH = Path(__file__).with_name("self_contained.py")
SPEC = importlib.util.spec_from_file_location("embedding_self_contained", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
driver = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = driver
SPEC.loader.exec_module(driver)

#: An expansion shaped like the real one: a byte-string literal and nothing that
#: reaches outside the file.
CLEAN = '#![feature(prelude_import)]\npub const EMBEDDED: &[u8] = b"\\x00\\x01ab";\n'


def test_a_clean_expansion_is_accepted(tmp_path: Path) -> None:
    assert driver.expansion_is_self_contained(CLEAN, tmp_path) == []


def test_an_expansion_without_a_byte_literal_is_rejected(tmp_path: Path) -> None:
    reasons = driver.expansion_is_self_contained("fn main() {}\n", tmp_path)
    assert reasons == ["no byte-string literal in the expansion"]


def test_every_forbidden_form_is_rejected(tmp_path: Path) -> None:
    """Parametrized over the whole list, because the list is the claim.

    A form added to `FORBIDDEN_IN_EXPANSION` without a matching rejection would
    leave a way out of the crate reachable while this file still passed, so the
    population is enumerated and counted rather than sampled.
    """
    forms = driver.FORBIDDEN_IN_EXPANSION
    assert len(forms) == 6, "the population this test covers is every forbidden form, counted"
    for form in forms:
        reasons = driver.expansion_is_self_contained(CLEAN + f"// {form}\n", tmp_path)
        # Membership rather than equality: `option_env!` contains `env!`, so one
        # occurrence legitimately reports two reasons. Asserting equality here
        # is what surfaced that, and the containment is not a defect — both
        # reasons are true of the text.
        assert f"expansion names `{form}`" in reasons, form


def test_an_expansion_naming_the_run_path_is_rejected(tmp_path: Path) -> None:
    reasons = driver.expansion_is_self_contained(CLEAN + f'// {tmp_path}\n', tmp_path)
    assert reasons == [f"expansion names the run path `{tmp_path}`"]


def test_the_payload_is_one_literal_and_its_length_is_measured() -> None:
    assert driver.byte_literal(CLEAN) == 'b"\\x00\\x01ab"'


def test_a_literal_containing_an_escaped_quote_is_not_ended_early() -> None:
    """The scan must survive the payload's own bytes.

    A real envelope contains `"` and `\\` bytes, and a scan that stopped at the
    first quote would report a length that is not the literal's — and, worse,
    would then see the remainder as a second literal and reject a correct
    expansion for the wrong reason.
    """
    source = 'pub const X: &[u8] = b"a\\"b\\\\c";\n'
    assert driver.byte_literal(source) == 'b"a\\"b\\\\c"'


def test_a_per_byte_representation_is_rejected() -> None:
    """The adverse control the cost note named: one integer literal per byte.

    Every other check in this driver passes on such an expansion — it is
    self-contained, it links, it runs — so this is the only place the accepted
    representation is actually asserted.
    """
    per_byte = "pub const X: &[u8] = &[0u8, 1u8, 97u8, 98u8];\n"
    with pytest.raises(driver.ScenarioFailure, match="exactly one per payload"):
        driver.byte_literal(per_byte)


def test_two_literals_are_rejected() -> None:
    with pytest.raises(driver.ScenarioFailure, match="holds 2 byte-string literals"):
        driver.byte_literal(CLEAN + 'pub const Y: &[u8] = b"zz";\n')


def test_deleting_a_populated_tree_reports_what_it_removed(tmp_path: Path) -> None:
    root = tmp_path / "artifacts"
    (root / "nested").mkdir(parents=True)
    (root / "one").write_bytes(b"a")
    (root / "nested" / "two").write_bytes(b"b")

    assert driver.delete_tree_provably("case", root) == 2
    assert not root.exists()


def test_deleting_a_path_that_held_nothing_fails(tmp_path: Path) -> None:
    """The exact trap: a wrong path makes an after-check pass for free.

    Requiring the path to hold files *first* is what turns a typo into a
    failure here instead of into a green row later.
    """
    with pytest.raises(driver.ScenarioFailure, match="mistyped path"):
        driver.delete_tree_provably("typo", tmp_path / "never-existed")

    empty = tmp_path / "empty"
    empty.mkdir()
    with pytest.raises(driver.ScenarioFailure, match="prove nothing"):
        driver.delete_tree_provably("empty", empty)


def test_census_distinguishes_absent_from_empty_from_populated(tmp_path: Path) -> None:
    assert driver.census(tmp_path / "absent") == 0
    (tmp_path / "empty").mkdir()
    assert driver.census(tmp_path / "empty") == 0
    (tmp_path / "one").write_bytes(b"x")
    assert driver.census(tmp_path) == 1


def test_the_checksum_matches_the_one_the_consumer_recomputes() -> None:
    """A known vector, so the oracle is not merely self-consistent.

    The consumer's `main` implements the same function in Rust with no
    dependency; if these two ever disagree, the run's comparison would be
    between two different functions and would fail for a reason that had nothing
    to do with the payload.
    """
    assert driver.fnv1a(b"") == 0xCBF29CE484222325
    assert driver.fnv1a(b"a") == 0xAF63DC4C8601EC8C
    assert driver.fnv1a(b"foobar") == 0x85944171F73967E8


def test_consumer_output_is_checked_against_the_producers_bytes() -> None:
    oracle = (4, driver.fnv1a(b"abcd"))
    assert driver.require_carries(f"slot=a len=4 fnv1a={oracle[1]:016x}", "a", oracle) == 4

    with pytest.raises(driver.ScenarioFailure, match="but the producer wrote"):
        driver.require_carries("slot=a len=5 fnv1a=0000000000000000", "a", oracle)
    with pytest.raises(driver.ScenarioFailure, match="unrecognized consumer output"):
        driver.require_carries("slot=a len=4", "a", oracle)
    with pytest.raises(driver.ScenarioFailure, match="unrecognized consumer output"):
        driver.require_carries(f"slot=b len=4 fnv1a={oracle[1]:016x}", "a", oracle)


def test_the_declared_invocation_counts_have_not_drifted() -> None:
    """The driver's expected expansion count and the fixture's must agree.

    Without this, lowering a consumer's invocation count would make every
    "expected exactly N expansions" check assert a number the fixture no longer
    produces — and the run would fail with a message about concurrency rather
    than about the drift that caused it.
    """
    driver.check_declared_invocations()


def test_an_expansion_count_that_drifted_is_caught(tmp_path: Path, monkeypatch) -> None:
    """The paired negative: the drift check must be able to say no."""
    fixture = tmp_path / "fixture"
    for consumer in ("consumer-a", "consumer-b"):
        source = fixture / consumer / "src"
        source.mkdir(parents=True)
        (source / "main.rs").write_text("pub const INVOCATIONS: usize = 99;\n")
    monkeypatch.setattr(driver, "WORKSPACE", fixture)
    with pytest.raises(driver.ScenarioFailure, match="drifted"):
        driver.check_declared_invocations()
