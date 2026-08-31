"""Git/filesystem integration test for the retained-evidence verifier."""

from __future__ import annotations

import hashlib
import json
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
        subject_revision = run_git(root, "rev-parse", "HEAD")

        manifest = {
            "subjectRevision": subject_revision,
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
                "subject input tree differs from current HEAD",
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


def test_evidence_corrections_fail_closed_on_identity_integrity_and_target() -> None:
    """TC-022. Trace: TC-022, FR-009-AC-4, NFR-004-AC-4."""
    source_schema = (
        verify_evidence.ROOT / verify_evidence.CORRECTION_SCHEMA
    ).read_bytes()

    def write_case(root: Path, correction: dict[str, object], checksum: str) -> None:
        schema_path = root / verify_evidence.CORRECTION_SCHEMA
        schema_path.parent.mkdir(parents=True)
        schema_path.write_bytes(source_schema)
        correction_path = root / "evidence/corrections/COR-001-test.json"
        correction_path.parent.mkdir(parents=True)
        correction_path.write_text(json.dumps(correction), encoding="utf-8")
        checksum_path = correction_path.with_suffix(".sha256")
        checksum_path.write_text(
            f"{checksum}  {correction_path.relative_to(root).as_posix()}\n",
            encoding="utf-8",
        )

    base: dict[str, object] = {
        "schemaVersion": "quire.evidence-correction/v1",
        "recordId": "COR-001",
        "recordedAt": "2026-08-31T00:39:11Z",
        "repository": "https://github.com/agent-ix/quire-contract-ir",
        "affectedClaims": [
            {
                "record": "pgm-01-abcdef0",
                "claim": "code-review status pass",
                "location": "https://github.com/agent-ix/quire-contract-ir/pull/12",
            }
        ],
        "correctedStatus": "inconclusive",
        "findingRefs": [
            "https://github.com/agent-ix/quire-contract-ir/pull/12#review"
        ],
        "basis": "Formal review contradicts the pass claim.",
        "immutability": "The original bytes remain unchanged.",
        "decisionEffect": "The affected record cannot support a decision.",
    }

    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        write_case(root, base, "0" * 64)
        with patch.object(verify_evidence, "ROOT", root), unittest.TestCase().assertRaisesRegex(
            verify_evidence.EvidenceError, "checksum mismatch"
        ):
            verify_evidence.load_evidence_corrections()

    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        payload = json.dumps(base).encode()
        write_case(root, base, hashlib.sha256(payload).hexdigest())
        correction_path = root / "evidence/corrections/COR-001-test.json"
        checksum_path = correction_path.with_suffix(".sha256")
        checksum_path.write_text(
            f"{hashlib.sha256(correction_path.read_bytes()).hexdigest()}  "
            f"{correction_path.relative_to(root).as_posix()}\n",
            encoding="utf-8",
        )
        with patch.object(verify_evidence, "ROOT", root), unittest.TestCase().assertRaisesRegex(
            verify_evidence.EvidenceError, "unavailable record"
        ):
            verify_evidence.load_evidence_corrections()

    traversal = json.loads(json.dumps(base))
    traversal["affectedClaims"][0]["record"] = "../pgm-01-abcdef0"
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        write_case(root, traversal, "0" * 64)
        correction_path = root / "evidence/corrections/COR-001-test.json"
        checksum_path = correction_path.with_suffix(".sha256")
        checksum_path.write_text(
            f"{hashlib.sha256(correction_path.read_bytes()).hexdigest()}  "
            f"{correction_path.relative_to(root).as_posix()}\n",
            encoding="utf-8",
        )
        with patch.object(verify_evidence, "ROOT", root), unittest.TestCase().assertRaisesRegex(
            verify_evidence.EvidenceError, "schema violation"
        ):
            verify_evidence.load_evidence_corrections()


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
    tests.addTest(
        unittest.FunctionTestCase(
            test_evidence_corrections_fail_closed_on_identity_integrity_and_target
        )
    )
    return tests


if __name__ == "__main__":
    unittest.main()
