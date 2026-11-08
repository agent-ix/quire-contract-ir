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


class PersistedEvidence(unittest.TestCase):
    def test_support_controls_are_explicitly_scoped_without_corpus_credit(self):
        metadata = runner.selection_metadata(SimpleNamespace(controls=True, class_controls=False))
        self.assertEqual(metadata["scope"], "executed malformed-input reader entry controls only")
        self.assertFalse(metadata["full_corpus"])
        self.assertFalse(metadata["complete_declared_inventory"])

    def test_class_proof_keeps_verdicts_and_counts_without_facts_or_reports(self):
        entry = {"facts": {"test": "canonical_class_denial::x"}, "counts": {"value": 1}, "directory": "/d"}
        proof = runner.persisted_class_proof({
            "schema": "s", "qualified": True, "scope": "c", "configuration": "config.json",
            "static_ac513_proven": False, "canonical_source": "/src",
            "classes": [{"class": "text", "qualified": True, "kernel_allow": entry, "whole_deny": None}],
            "negative_controls": [], "runs": [{"test": "canonical_class_denial::x", "directory": "/d",
                                               "rust_oracle_passed": True}]})
        self.assertEqual(proof["classes"][0]["kernel_allow"], {"counts": {"value": 1}})
        self.assertIsNone(proof["classes"][0]["whole_deny"])
        self.assertEqual(proof["runs"], [{"test": "canonical_class_denial::x", "rust_oracle_passed": True}])
        self.assertNotIn("facts", json.dumps(proof))

    def test_persisted_summary_keeps_counts_without_test_identity(self):
        summary = runner.persisted_test_summary({
            "intended_tests": ["module::case"],
            "passed_tests": ["module::case"],
            "failed_tests": [],
            "ignored_tests": [],
            "scope": "selected-only original tests",
            "full_corpus": False,
        })
        self.assertEqual(summary["test_count"], 1)
        self.assertEqual(summary["passed_count"], 1)
        self.assertNotIn("module::case", json.dumps(summary))
        self.assertNotIn("intended_tests", summary)


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
                    args, binary, {}, {},
                    "original::red", 0, preserve_failure=True)
            self.assertEqual(report["exit_code"], 1)
            self.assertEqual(report["failed_tests"], ["original::red"])
            self.assertTrue(report["evidence_errors"])
            self.assertEqual(profiles[0].read_bytes(), b"profile-preserved")
            summary = json.loads((output / "test-report.json").read_text())
            self.assertEqual(summary["failed_count"], 1)
            self.assertFalse((output / "run.log").exists())
            self.assertFalse((output / "driver-sites.jsonl").exists())
            self.assertEqual(records, [])

    def test_failed_corpus_run_names_the_failing_test_and_its_error(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / "red-original"
            binary.write_text("#!/usr/bin/python3\nprint('test original::red ... FAILED')\nprint('assertion failed: reader')\nraise SystemExit(1)\n")
            binary.chmod(0o700)
            args = SimpleNamespace(out=root, mode="sample", repo=root, original_test=[], corpus="qspec")
            with patch.object(runner, "LOCKED", []), patch.object(runner, "validate_snapshot"), patch.object(runner, "require_space"):
                with self.assertRaises(runner.RunError) as raised:
                    runner.run_original_corpus(args, binary, {}, {}, ["original::red"])
            message = str(raised.exception)
            self.assertIn("1 of 1 intended original tests failed: original::red", message)
            self.assertIn("assertion failed: reader", message)
            self.assertNotIn("retained", message)
            summary = json.loads((root / "test-report.json").read_text())
            self.assertEqual(summary["failed_count"], 1)

    def test_timeout_records_incomplete_and_terminates_only_owned_process(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / "slow-original"
            binary.write_text("#!/usr/bin/python3\nimport time\nprint('test original::slow ... ',flush=True)\ntime.sleep(60)\n")
            binary.chmod(0o700)
            args = SimpleNamespace(out=root,mode="sample",repo=root,test_timeout_seconds=0.05)
            with patch.object(runner,"LOCKED",[]), patch.object(runner,"validate_snapshot"), patch.object(runner,"require_space"):
                output,report,_,_ = runner.execute_one(args,binary,{}, {},
                                                     "original::slow",0,preserve_failure=True)
            self.assertTrue(report["timed_out"])
            self.assertNotEqual(report["exit_code"],0)
            self.assertFalse(report["passed_tests"])
            self.assertTrue((output / "timeout.json").exists())
            self.assertFalse((output / "run.log").exists())
            self.assertTrue(any("timeout; incomplete" in error for error in report["evidence_errors"]))

    def test_config_never_copies_registry_credentials(self):
        result = runner.safe_environment({"CARGO_BUILD_JOBS": "2", "CARGO_REGISTRIES_PRIVATE_TOKEN": "secret"})
        self.assertEqual(result, {"CARGO_BUILD_JOBS": "2"})


if __name__ == "__main__":
    unittest.main()
