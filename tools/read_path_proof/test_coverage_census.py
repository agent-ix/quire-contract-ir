# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX

"""Arithmetic/parser controls only; actual rustc profile proof is separate."""
import copy
from pathlib import Path
import unittest

import coverage_census as census

ROOT = Path(__file__).resolve().parents[2]


def export_counts(plain=3, probes=20, caller=2, calibration=1):
    counts = {
        ("tests/it/support/qspec_read_proof.rs", "read_and_account"): plain,
        ("tests/it/support/read_observation.rs", "measure_at"): probes,
        ("tests/it/support/qspec_read_proof.rs", "read_budget_and_account"): caller,
        ("tests/it/support/qspec_read_proof.rs",
         "caller_budget_observation_preserves_prior_charges_and_outer_capture"): calibration,
        ("crates/quire-contract-model/src/checked_package/v2/budgeted_read.rs", "read"): plain,
        ("crates/quire-contract-model/src/checked_package/v2/budgeted_read.rs", "read_with_budget"):
            plain + probes + caller + calibration,
    }
    functions = []
    for (file, function), count in counts.items():
        line = census.function_line((ROOT / file).read_text(), function)
        functions.append({"name": f"_RNv{len(function)}{function}", "count": count,
                          "filenames": [str(ROOT / file)],
                          "regions": [[line, 1, line + 1, 1, count, 0, 0, 0]]})
    return {"type": "llvm.coverage.json.export", "data": [{"functions": functions}]}


class ActualEntryCensus(unittest.TestCase):
    def test_accounted_invocations_match(self):
        result = census.check(export_counts(), ROOT)
        self.assertEqual(result["budgeted"], 26)
        self.assertFalse(result["static_ac513_proven"])

    def test_executed_unwrapped_alias_or_function_item_fails(self):
        data = export_counts()
        for function in data["data"][0]["functions"]:
            if function["filenames"][0].endswith("/v2/budgeted_read.rs"):
                function["count"] += 1
        with self.assertRaisesRegex(census.CensusError, "unaccounted plain"):
            census.check(data, ROOT)

    def test_executed_unwrapped_budgeted_callback_fails(self):
        data = export_counts()
        data["data"][0]["functions"][-1]["count"] += 1
        with self.assertRaisesRegex(census.CensusError, "unaccounted budgeted"):
            census.check(data, ROOT)

    def test_closure_entry_does_not_double_count_its_parent(self):
        data = export_counts()
        closure = copy.deepcopy(data["data"][0]["functions"][0])
        closure["name"] = "_RNCNv16read_and_account0"
        closure["count"] = 100
        data["data"][0]["functions"].append(closure)
        self.assertEqual(census.check(data, ROOT)["measured_plain"], 3)

    def test_missing_compiled_function_is_not_zero_evidence(self):
        data = export_counts()
        data["data"][0]["functions"].pop()
        with self.assertRaisesRegex(census.CensusError, "missing compiled function"):
            census.check(data, ROOT)

    def test_no_original_driver_execution_cannot_pass(self):
        with self.assertRaisesRegex(census.CensusError, "did not execute"):
            census.check(export_counts(plain=0, probes=0, caller=0, calibration=0), ROOT)

    def test_generic_symbol_is_resolved_at_exact_source_definition(self):
        data = export_counts()
        data["data"][0]["functions"][1]["name"] = "_RINv10measure_atNt13DriverCounterE"
        self.assertEqual(census.check(data, ROOT)["measured_probes"], 20)


class FullRunEvidence(unittest.TestCase):
    def report(self):
        return {"intended_tests": ["original::first", "original::second"],
                "passed_tests": ["original::first", "original::second"],
                "failed_tests": [], "ignored_tests": [], "exit_code": 0, "test_threads": 1}

    def records(self):
        base = {"driver": "tests/conformance_qspec/main.rs", "line": 1,
                "test": "original::first", "mode": "sample"}
        return [dict(base) for _ in range(3)] + [dict(base, entry="caller-budget") for _ in range(2)]

    def test_sample_is_not_normative_or_static_proof(self):
        result = census.check_run(export_counts(), ROOT, self.report(), self.records(), "sample")
        self.assertFalse(result["normative_boundary_proof"])
        self.assertFalse(result["static_ac513_proven"])

    def test_zero_or_omitted_original_tests_fail(self):
        report = self.report()
        report["passed_tests"] = []
        with self.assertRaisesRegex(census.CensusError, "every intended"):
            census.check_run(export_counts(), ROOT, report, self.records(), "sample")

    def test_missing_runtime_sites_fail(self):
        with self.assertRaisesRegex(census.CensusError, "records do not match"):
            census.check_run(export_counts(), ROOT, self.report(), self.records()[:-1], "sample")

    def test_full_label_without_native_event_oracle_fails(self):
        records = [dict(row, mode="full") for row in self.records()]
        with self.assertRaisesRegex(census.CensusError, "passive-event normative"):
            census.check_run(export_counts(), ROOT, self.report(), records, "full")


if __name__ == "__main__":
    unittest.main()
