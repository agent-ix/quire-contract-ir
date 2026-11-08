# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX

"""Artifact custody and execution resource controls; no compiler proof claim."""
import gzip
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch
import run_coverage_census as runner


class ArtifactCustody(unittest.TestCase):
    def test_archive_retains_entire_unfiltered_export_losslessly(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            data = b'{"unfiltered":true,"functions":[1,2,3]}\n' * 100000
            (root / "coverage.json").write_bytes(data)
            runner.archive_export(root)
            self.assertFalse((root / "coverage.json").exists())
            with gzip.open(root / "coverage.json.gz", "rb") as archive:
                self.assertEqual(archive.read(), data)

    def test_low_space_stops_without_removing_prior_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            evidence = root / "prior.profraw"
            evidence.write_bytes(b"untouched")
            with patch.object(runner.shutil, "disk_usage", return_value=type("Usage", (), {"free": 1})()):
                with self.assertRaisesRegex(runner.RunError, "preserve artifacts and stop"):
                    runner.require_space(root, "before test")
            self.assertEqual(evidence.read_bytes(), b"untouched")

    def test_owned_target_rejects_symlink_to_another_worktree(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo, other = root / "owned", root / "other"
            repo.mkdir(); other.mkdir()
            (repo / "target").symlink_to(other, target_is_directory=True)
            with self.assertRaisesRegex(runner.RunError, "symlink"):
                runner.owned_target(repo, repo / "target" / "proof")

    def test_failed_original_process_is_retained_for_remaining_corpus(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / "red-original"
            binary.write_text("#!/usr/bin/python3\nimport os, pathlib\n"
                              "pathlib.Path(os.environ['LLVM_PROFILE_FILE'].replace('%p',str(os.getpid())).replace('%m','unit')).write_bytes(b'profile-preserved')\n"
                              "print('test original::red ... FAILED')\nraise SystemExit(1)\n")
            binary.chmod(0o700)
            args = SimpleNamespace(out=root, mode="sample", repo=root)
            with patch.object(runner, "LOCKED", []), patch.object(runner, "validate_snapshot"), patch.object(runner, "require_space"):
                output, report, profiles, records = runner.execute_one(
                    args, binary, {"source_head": "unit-artifact-control"}, {},
                    "original::red", 0, preserve_failure=True)
            self.assertEqual(report["exit_code"], 1)
            self.assertEqual(report["failed_tests"], ["original::red"])
            self.assertTrue(report["evidence_errors"])
            self.assertEqual(profiles[0].read_bytes(), b"profile-preserved")
            self.assertIn("FAILED", (output / "run.log").read_text())
            self.assertEqual(records, [])

    def test_config_never_copies_registry_credentials(self):
        result = runner.safe_environment({"CARGO_BUILD_JOBS": "2", "CARGO_REGISTRIES_PRIVATE_TOKEN": "secret"})
        self.assertEqual(result, {"CARGO_BUILD_JOBS": "2"})


if __name__ == "__main__":
    unittest.main()
