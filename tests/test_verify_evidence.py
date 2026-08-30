"""Unit tests for the revision-scoped evidence verifier."""

from __future__ import annotations

import copy
import hashlib
import unittest
from collections.abc import Callable

from scripts.verify_evidence import EvidenceError, verify_input_checksums


def assert_evidence_error(message: str, callback: Callable[[], object]) -> None:
    try:
        callback()
    except EvidenceError as error:
        if message not in str(error):
            raise AssertionError(f"expected {message!r}, got {error!r}") from error
    else:
        raise AssertionError(f"expected EvidenceError containing {message!r}")


def test_evidence_verifier_detects_subject_and_current_input_drift() -> None:
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
        subject_reader=lambda _revision, _path: payload,
        worktree_reader=lambda _path: payload,
    )
    if actual != 1:
        raise AssertionError(f"expected one verified input, got {actual}")

    wrong_record = copy.deepcopy(manifest)
    wrong_record["inputChecksums"]["spec/example.md"] = "0" * 64
    assert_evidence_error(
        "subject input checksum mismatch",
        lambda: verify_input_checksums(
            wrong_record,
            subject_reader=lambda _revision, _path: payload,
            worktree_reader=lambda _path: payload,
        ),
    )
    assert_evidence_error(
        "current input checksum mismatch",
        lambda: verify_input_checksums(
            manifest,
            subject_reader=lambda _revision, _path: payload,
            worktree_reader=lambda _path: b"drifted current input",
        ),
    )


def load_tests(
    _loader: unittest.TestLoader,
    tests: unittest.TestSuite,
    _pattern: str | None,
) -> unittest.TestSuite:
    tests.addTest(
        unittest.FunctionTestCase(
            test_evidence_verifier_detects_subject_and_current_input_drift
        )
    )
    return tests


if __name__ == "__main__":
    unittest.main()
