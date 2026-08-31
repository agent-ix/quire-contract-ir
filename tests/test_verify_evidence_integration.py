"""Git/filesystem integration test for the retained-evidence verifier."""

from __future__ import annotations

import hashlib
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import verify_evidence


def run_git(root: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def test_evidence_verifier_uses_head_tree_without_subject_ancestry() -> None:
    """TC-013. Trace: TC-013, FR-009-AC-3."""
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        run_git(root, "init", "-q")
        run_git(root, "config", "user.name", "Evidence Test")
        run_git(root, "config", "user.email", "evidence-test@example.invalid")
        payload = b"committed candidate input\n"
        (root / "candidate.txt").write_bytes(payload)
        run_git(root, "add", "candidate.txt")
        run_git(root, "commit", "-q", "-m", "candidate")

        manifest = {
            "subjectRevision": "f" * 40,
            "inputChecksums": {
                "candidate.txt": hashlib.sha256(payload).hexdigest(),
            },
        }
        with patch.object(verify_evidence, "ROOT", root):
            count = verify_evidence.verify_input_checksums(manifest)
            if count != 1:
                raise AssertionError(f"expected one verified input, got {count}")

            (root / "added.txt").write_text("new untracked input\n", encoding="utf-8")
            with unittest.TestCase().assertRaisesRegex(
                verify_evidence.EvidenceError,
                "worktree input set differs from current HEAD",
            ):
                verify_evidence.verify_input_checksums(manifest)

            run_git(root, "add", "added.txt")
            run_git(root, "commit", "-q", "-m", "add uncovered input")
            with unittest.TestCase().assertRaisesRegex(
                verify_evidence.EvidenceError,
                "do not cover the current HEAD tree",
            ):
                verify_evidence.verify_input_checksums(manifest)


def test_evidence_verifier_selects_only_the_unique_valid_record() -> None:
    """TC-013. Trace: TC-013, FR-009-AC-3."""
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        stale = root / "evidence/pgm-01-stale"
        current = root / "evidence/pgm-01-current"
        stale.mkdir(parents=True)
        current.mkdir(parents=True)
        (stale / "manifest.json").write_text("{}\n", encoding="utf-8")
        (current / "manifest.json").write_text("{}\n", encoding="utf-8")

        def verify_one(record: Path) -> tuple[int, int, str]:
            if record.name == stale.name:
                raise verify_evidence.EvidenceError("stale record")
            return 4, 66, "f" * 40

        with patch.object(verify_evidence, "ROOT", root), patch.object(
            verify_evidence, "verify_record", side_effect=verify_one
        ):
            selected = verify_evidence.select_current_record()
            if selected[0].name != current.name:
                raise AssertionError(f"expected current record, got {selected[0]}")

            with patch.object(
                verify_evidence,
                "verify_record",
                side_effect=verify_evidence.EvidenceError("all stale"),
            ), unittest.TestCase().assertRaisesRegex(
                verify_evidence.EvidenceError,
                "no evidence record matches the current candidate",
            ):
                verify_evidence.select_current_record()

            with patch.object(
                verify_evidence,
                "verify_record",
                return_value=(4, 66, "f" * 40),
            ), unittest.TestCase().assertRaisesRegex(
                verify_evidence.EvidenceError,
                "multiple current evidence records",
            ):
                verify_evidence.select_current_record()


def load_tests(
    _loader: unittest.TestLoader,
    tests: unittest.TestSuite,
    _pattern: str | None,
) -> unittest.TestSuite:
    tests.addTest(
        unittest.FunctionTestCase(
            test_evidence_verifier_uses_head_tree_without_subject_ancestry
        )
    )
    tests.addTest(
        unittest.FunctionTestCase(
            test_evidence_verifier_selects_only_the_unique_valid_record
        )
    )
    return tests


if __name__ == "__main__":
    unittest.main()
