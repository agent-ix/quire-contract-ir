# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX

"""Artifact custody and execution resource controls; no compiler proof claim."""
import gzip
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch
import run_coverage_census as runner


class OriginalSelection(unittest.TestCase):
    def test_exact_subset_follows_actual_inventory(self):
        self.assertEqual(runner.select_original_tests(["a", "b", "c"], ["c", "a"]), ["a", "c"])

    def test_unknown_duplicate_zero_and_control_mixing_fail_closed(self):
        for compiled, selected, controls, classes in [
            (["a"], ["typo"], False, False),
            (["a"], ["a", "a"], False, False),
            ([], ["a"], False, False),
            (["a"], ["a"], True, False),
            (["a"], ["a"], False, True),
        ]:
            with self.subTest(compiled=compiled, selected=selected, controls=controls, classes=classes):
                with self.assertRaises(runner.RunError):
                    runner.select_original_tests(compiled, selected, controls, classes)

    def test_exact_control_subset_is_distinct_from_original_and_full_modes(self):
        inventory = list(runner.CONTROL_CASES)
        selected = inventory[:1]
        self.assertEqual(runner.select_control_tests(inventory, selected), selected)
        self.assertEqual(runner.select_control_tests(inventory, []), inventory)
        for names in (["unknown"], selected * 2):
            with self.assertRaises(runner.RunError):
                runner.select_control_tests(inventory, names)
        with self.assertRaises(runner.RunError):
            runner.select_control_tests([], selected)
        metadata = runner.selection_metadata(SimpleNamespace(controls=True, class_controls=False, control_test=selected))
        self.assertEqual(metadata["scope"], "selected-only reader entry controls")
        self.assertFalse(metadata["full_corpus"])
        self.assertFalse(metadata["complete_declared_inventory"])

    def test_selected_completion_never_claims_full_corpus(self):
        metadata = runner.selection_metadata(SimpleNamespace(original_test=["a"], controls=False, class_controls=False))
        self.assertEqual(metadata["scope"], "selected-only original tests")
        self.assertFalse(metadata["full_corpus"])

    def test_unfiltered_selection_preserves_inventory_and_scope(self):
        inventory = ["b", "a"]
        self.assertEqual(runner.select_original_tests(inventory, []), inventory)
        metadata = runner.selection_metadata(SimpleNamespace(original_test=[], controls=False, class_controls=False))
        self.assertTrue(metadata["complete_declared_inventory"])
        self.assertFalse(metadata["full_corpus"])


class CompletionCustody(unittest.TestCase):
    def test_empty_completion_manifest_cannot_emit_success(self):
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "manifest.json"
            manifest.write_text(json.dumps({"schema":"ac511-full-completion/v1", "runs":[]}))
            with self.assertRaisesRegex(runner.RunError, "exactly two"):
                runner.complete_full_manifest(manifest)

    def test_support_controls_are_explicitly_scoped_without_corpus_credit(self):
        metadata = runner.selection_metadata(SimpleNamespace(controls=True, class_controls=False))
        self.assertEqual(metadata["scope"], "executed malformed-input reader entry controls only")
        self.assertFalse(metadata["full_corpus"])
        self.assertFalse(metadata["complete_declared_inventory"])


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

    def test_timeout_records_incomplete_and_terminates_only_owned_process(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / "slow-original"
            binary.write_text("#!/usr/bin/python3\nimport time\nprint('test original::slow ... ',flush=True)\ntime.sleep(60)\n")
            binary.chmod(0o700)
            args = SimpleNamespace(out=root,mode="sample",repo=root,test_timeout_seconds=0.05)
            with patch.object(runner,"LOCKED",[]), patch.object(runner,"validate_snapshot"), patch.object(runner,"require_space"):
                output,report,_,_ = runner.execute_one(args,binary,{"source_head":"unit-artifact-control"},{},
                                                     "original::slow",0,preserve_failure=True)
            self.assertTrue(report["timed_out"])
            self.assertNotEqual(report["exit_code"],0)
            self.assertFalse(report["passed_tests"])
            self.assertTrue((output / "timeout.json").exists())
            self.assertTrue(any("timeout; incomplete" in error for error in report["evidence_errors"]))

    def test_config_never_copies_registry_credentials(self):
        result = runner.safe_environment({"CARGO_BUILD_JOBS": "2", "CARGO_REGISTRIES_PRIVATE_TOKEN": "secret"})
        self.assertEqual(result, {"CARGO_BUILD_JOBS": "2"})


if __name__ == "__main__":
    unittest.main()
