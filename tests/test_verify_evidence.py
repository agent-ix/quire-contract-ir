"""Unit tests for the revision-scoped evidence verifier."""

from __future__ import annotations

import copy
import hashlib
import json
import unittest
from collections.abc import Callable
from pathlib import Path

from scripts.verify_evidence import (
    EVIDENCE_SCHEMA,
    ROOT,
    EvidenceError,
    choose_unique_match,
    parse_external_checksum_lines,
    safe_relative_path,
    sha256,
    validate_manifest_schema,
    load_evidence_corrections,
    verify_input_checksums,
    verify_output_entries,
)


def assert_evidence_error(message: str, callback: Callable[[], object]) -> None:
    try:
        callback()
    except EvidenceError as error:
        if message not in str(error):
            raise AssertionError(f"expected {message!r}, got {error!r}") from error
    else:
        raise AssertionError(f"expected EvidenceError containing {message!r}")


def test_evidence_verifier_detects_head_and_current_input_drift() -> None:
    """TC-013. Trace: TC-013, FR-009-AC-3."""
    payload = b"exact subject input"
    manifest = {
        "subjectRevision": "a" * 40,
        "inputChecksums": {
            "spec/example.md": hashlib.sha256(payload).hexdigest(),
        },
    }
    actual = verify_input_checksums(
        manifest,
        head_reader=lambda _revision, _path: payload,
        worktree_reader=lambda _path: payload,
        input_set_reader=lambda: {Path("spec/example.md")},
        worktree_set_reader=lambda: {Path("spec/example.md")},
    )
    if actual != 1:
        raise AssertionError(f"expected one verified input, got {actual}")

    wrong_record = copy.deepcopy(manifest)
    wrong_record["inputChecksums"]["spec/example.md"] = "0" * 64
    assert_evidence_error(
        "HEAD input checksum mismatch",
        lambda: verify_input_checksums(
            wrong_record,
            head_reader=lambda _revision, _path: payload,
            worktree_reader=lambda _path: payload,
            input_set_reader=lambda: {Path("spec/example.md")},
            worktree_set_reader=lambda: {Path("spec/example.md")},
        ),
    )
    assert_evidence_error(
        "current input checksum mismatch",
        lambda: verify_input_checksums(
            manifest,
            head_reader=lambda _revision, _path: payload,
            worktree_reader=lambda _path: b"drifted current input",
            input_set_reader=lambda: {Path("spec/example.md")},
            worktree_set_reader=lambda: {Path("spec/example.md")},
        ),
    )


def test_evidence_verifier_rejects_incomplete_input_coverage_and_unsafe_paths() -> None:
    """TC-013. Trace: TC-013, FR-009-AC-3."""
    payload = b"exact subject input"
    manifest = {
        "subjectRevision": "a" * 40,
        "inputChecksums": {
            "spec/example.md": hashlib.sha256(payload).hexdigest(),
        },
    }
    assert_evidence_error(
        "do not cover the current HEAD tree",
        lambda: verify_input_checksums(
            manifest,
            head_reader=lambda _revision, _path: payload,
            worktree_reader=lambda _path: payload,
            input_set_reader=lambda: {
                Path("spec/example.md"),
                Path("scripts/missing.py"),
            },
            worktree_set_reader=lambda: {Path("spec/example.md")},
        ),
    )
    assert_evidence_error(
        "unsafe evidence path",
        lambda: safe_relative_path("../escape"),
    )
    assert_evidence_error(
        "unsafe evidence path",
        lambda: parse_external_checksum_lines([f"{'0' * 64}  /absolute"]),
    )


def test_evidence_verifier_authenticates_outputs_and_checksum_file_to_head() -> None:
    """TC-013. Trace: TC-013, FR-009-AC-3."""
    checksum_path = Path("evidence/pgm-01-example.sha256")
    manifest_path = Path("evidence/pgm-01-example/manifest.json")
    summary_path = Path("evidence/pgm-01-example/validation-summary.md")
    payloads = {
        checksum_path: b"external checksums",
        manifest_path: b"manifest",
        summary_path: b"summary",
    }
    manifest = {
        "outputs": [str(manifest_path), str(summary_path), str(checksum_path)],
    }
    entries = {
        manifest_path: hashlib.sha256(payloads[manifest_path]).hexdigest(),
        summary_path: hashlib.sha256(payloads[summary_path]).hexdigest(),
    }
    actual = verify_output_entries(
        manifest,
        checksum_path,
        entries,
        current_reader=lambda path: payloads[path],
        head_reader=lambda _revision, path: payloads[path],
    )
    if actual != 3:
        raise AssertionError(f"expected three verified retained outputs, got {actual}")
    assert_evidence_error(
        "differs from the current HEAD blob",
        lambda: verify_output_entries(
            manifest,
            checksum_path,
            entries,
            current_reader=lambda path: payloads[path],
            head_reader=lambda _revision, path: (
                b"tampered HEAD checksum" if path == checksum_path else payloads[path]
            ),
        ),
    )
    assert_evidence_error(
        "entries do not match retained outputs",
        lambda: verify_output_entries(
            manifest,
            checksum_path,
            {manifest_path: entries[manifest_path]},
            current_reader=lambda path: payloads[path],
            head_reader=lambda _revision, path: payloads[path],
        ),
    )


def test_evidence_verifier_requires_one_matching_record() -> None:
    """TC-013. Trace: TC-013, FR-009-AC-3."""
    only = (Path("evidence/pgm-01-only"), 2, 64)
    if choose_unique_match([only]) != only:
        raise AssertionError("expected the sole current record")
    assert_evidence_error(
        "multiple current evidence records",
        lambda: choose_unique_match([only, (Path("evidence/pgm-01-other"), 2, 64)]),
    )


def test_evidence_manifest_schema_rejects_missing_required_provenance() -> None:
    """TC-013. Trace: TC-013, FR-009-AC-3."""
    schema_bytes = (ROOT / EVIDENCE_SCHEMA).read_bytes()
    schema = json.loads(schema_bytes)
    required_tools = set(schema["properties"]["environment"]["required"])
    expected_tools = {
        "platform",
        "rustc",
        "cargo",
        "cargoDeny",
        "python",
        "jsonschema",
        "rfc3339Validator",
        "rfc3986Validator",
        "quire",
        "codeReviewSessions",
    }
    if required_tools != expected_tools:
        raise AssertionError(f"unexpected required tool identities: {required_tools}")
    incomplete = {
        "schemaIdentity": {
            "path": EVIDENCE_SCHEMA.as_posix(),
            "sha256": sha256(schema_bytes),
        }
    }
    assert_evidence_error(
        "evidence manifest schema violation",
        lambda: validate_manifest_schema(incomplete),
    )


def test_evidence_verifier_enforces_append_only_correction() -> None:
    """TC-022. Trace: TC-022, FR-009-AC-4, NFR-004-AC-4."""
    corrections = load_evidence_corrections()
    if corrections != {"pgm-01-568bd05": ["COR-001"]}:
        raise AssertionError(f"unexpected enforced corrections: {corrections}")


def load_tests(
    _loader: unittest.TestLoader,
    tests: unittest.TestSuite,
    _pattern: str | None,
) -> unittest.TestSuite:
    tests.addTest(
        unittest.FunctionTestCase(
            test_evidence_verifier_detects_head_and_current_input_drift
        )
    )
    tests.addTest(
        unittest.FunctionTestCase(
            test_evidence_verifier_rejects_incomplete_input_coverage_and_unsafe_paths
        )
    )
    tests.addTest(
        unittest.FunctionTestCase(
            test_evidence_verifier_authenticates_outputs_and_checksum_file_to_head
        )
    )
    tests.addTest(
        unittest.FunctionTestCase(
            test_evidence_verifier_requires_one_matching_record
        )
    )
    tests.addTest(
        unittest.FunctionTestCase(
            test_evidence_manifest_schema_rejects_missing_required_provenance
        )
    )
    tests.addTest(
        unittest.FunctionTestCase(
            test_evidence_verifier_enforces_append_only_correction
        )
    )
    return tests


if __name__ == "__main__":
    unittest.main()
