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
        ("tests/it/support/qspec_read_proof.rs", "measure_seeded"): 6*caller,
        ("tests/it/support/qspec_read_proof.rs",
         "caller_budget_observation_preserves_prior_charges_and_outer_capture"): calibration,
        ("crates/quire-contract-model/src/checked_package/v2/budgeted_read.rs", "read"): plain,
        ("crates/quire-contract-model/src/checked_package/v2/budgeted_read.rs", "read_with_budget"):
            plain + probes + 6*caller + caller + calibration,
    }
    functions = []
    for (file, function), count in counts.items():
        line = census.function_line((ROOT / file).read_text(), function)
        functions.append({"name": f"_RNv{len(function)}{function}", "count": count,
                          "filenames": [str(ROOT / file)],
                          "regions": [[line, 1, line + 1, 1, count, 0, 0, 0]]})
    source_sites = []
    for file, text in [("tests/conformance_qspec/main.rs", "crate::read_and_account("),
                       ("tests/it/checked_package_v2_graph_read_storage.rs", "crate::qspec_read_proof::read_budget_and_account(")]:
        lines = (ROOT / file).read_text().splitlines()
        line = next(i for i, value in enumerate(lines, 1) if text in value)
        source_sites.append((file, line, len(lines[line - 1]) + 1))
    functions.append({"name": "executed_original_driver", "count": 3,
                      "filenames": [str(ROOT / file) for file, _, _ in source_sites],
                      "regions": [[line, 1, line, end, 3, i, 0, 0]
                                  for i, (_, line, end) in enumerate(source_sites)]})
    return {"type": "llvm.coverage.json.export", "data": [{"functions": functions}]}


class ActualEntryCensus(unittest.TestCase):
    def test_accounted_invocations_match(self):
        result = census.check(export_counts(), ROOT)
        self.assertEqual(result["budgeted"], 38)
        self.assertFalse(result["static_ac513_proven"])

    def test_compiler_path_attribute_spelling_resolves_exact_source(self):
        data = export_counts()
        for row in data["data"][0]["functions"]:
            row["filenames"] = [name.replace("tests/it/support/", "tests/conformance_qspec/../it/support/")
                                for name in row["filenames"]]
        self.assertEqual(census.check(data, ROOT)["measured_probes"], 20)
        next(row for row in data["data"][0]["functions"] if "read_with_budget" in row["name"])["count"] += 1
        with self.assertRaisesRegex(census.CensusError, "unaccounted budgeted"):
            census.check(data, ROOT)

    def test_equal_suffix_in_other_source_tree_cannot_supply_boundary(self):
        data = export_counts()
        row = data["data"][0]["functions"][0]
        row["filenames"] = [str(ROOT.parent / "foreign-tree/tests/it/support/qspec_read_proof.rs")]
        with self.assertRaisesRegex(census.CensusError, "missing compiled function"):
            census.check(data, ROOT)

    def test_seeded_original_entry_requires_six_checked_snapshot_reads(self):
        data=export_counts(plain=0,probes=0,caller=1,calibration=1)
        self.assertEqual(census.check(data,ROOT)["budgeted"],8)
        for row in data["data"][0]["functions"]:
            if "measure_seeded" in row["name"]:
                row["count"]=5
        with self.assertRaisesRegex(census.CensusError,"first/repeated checked"):
            census.check(data,ROOT)

    def test_executed_unwrapped_alias_or_function_item_fails(self):
        data = export_counts()
        for function in data["data"][0]["functions"]:
            if function["filenames"][0].endswith("/v2/budgeted_read.rs"):
                function["count"] += 1
        with self.assertRaisesRegex(census.CensusError, "unaccounted plain"):
            census.check(data, ROOT)

    def test_executed_unwrapped_budgeted_callback_fails(self):
        data = export_counts()
        next(row for row in data["data"][0]["functions"] if "read_with_budget" in row["name"])["count"] += 1
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
        data["data"][0]["functions"] = [row for row in data["data"][0]["functions"] if "read_with_budget" not in row["name"]]
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
        def site(file, spelling, entry):
            lines = (ROOT / file).read_text().splitlines()
            line = next(i for i, value in enumerate(lines, 1) if spelling in value)
            return {"driver": file, "line": line, "column": lines[line - 1].index("crate::") + 1,
                    "test": "original::first", "mode": "sample", "entry": entry, "snapshot_replay": "exact-entire-caller-state"}
        plain = site("tests/conformance_qspec/main.rs", "crate::read_and_account(", "plain")
        caller = site("tests/it/checked_package_v2_graph_read_storage.rs", "crate::qspec_read_proof::read_budget_and_account(", "caller-budget")
        return [dict(plain) for _ in range(3)] + [dict(caller) for _ in range(2)]

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

    def test_runtime_test_must_be_in_exact_intended_set(self):
        records = self.records()
        records[0]["test"] = "unselected::other"
        with self.assertRaisesRegex(census.CensusError, "outside the exact intended"):
            census.check_run(export_counts(), ROOT, self.report(), records, "sample")

    def test_captured_location_must_be_executed_real_invocation(self):
        records = self.records()
        records[0]["line"] = 1
        with self.assertRaisesRegex(census.CensusError, "not an actual instrumented"):
            census.check_run(export_counts(), ROOT, self.report(), records, "sample")


class FullCorpusCompletion(unittest.TestCase):
    def bundle(self):
        cohort = {key: "unit-cohort" for key in census.COHORT_FIELDS}
        row = {"test": "original", "driver": "original.rs", "line": 7,
               "proof_schema": "native-budget-events/v1", "native_budget_events": [], "boundary_results": [],
               "complete_reference": {"native_budget": "unlimited", "outcome": "incomplete",
                   "prior_work": 0, "prior_read_storage": 0, "work": 0, "read_storage": 0,
                   "requested": 0, "logical_storage_delta": 0, "exact_work_limit": 0,
                   "exact_read_storage_limit": 0, "exact_outcome_equal": True,
                   "exact_counts_equal": True, "complete_repeat_equal": True, "above_work_limit": 1,
                   "above_read_storage_limit": 1, "above_outcome_equal": True, "above_counts_equal": True, "above_budget_equal": True}}
        runs = []
        for corpus in ("qspec", "tc048"):
            runs.append({"config": dict(cohort, corpus=corpus, mode="full", original_tests=[],
                         complete_declared_inventory=True, intended_tests=["original"],
                         auxiliary_tests=[], original_driver_tests=["original"]),
                         "declared_inventory": ["original"], "records": [copy.deepcopy(row)],
                         "report": {"intended_tests": ["original"], "passed_tests": ["original"], "exit_code": 0},
                         "census": {"normative_boundaries_verified": True, "original_tests_passed": 1,
                                    "measured_plain": 1, "caller_budget": 0}})
        return {"schema": "ac511-full-completion/v1", "runs": runs,
                "canonical_config": cohort,
                "canonical_classes": {"qualified": True,
                   "classes": [{"class": name, "qualified": True} for name in census.CANONICAL_CLASS_SITES],
                   "negative_controls": [{"test": name, "rejected": True, "rust_oracle_passed": True} for name in
                     ("negative_empty_string_is_not_text_growth", "negative_empty_array_is_not_retained_children",
                      "negative_scratch_child_is_not_retained_children")]}}

    def complete_bundle(self):
        bundle = self.bundle()
        row = bundle["runs"][0]["records"][0]
        row["native_budget_events"] = [
            {"resource": "ReadStorage", "amount": 8, "consumed_before": i * 8, "admitted": True,
             "allocation_class": "walker_growth" if i < 3 else "exact_reservation",
             "locus": {"source": "PackageRead(Graph, 0)", "form": "walk" if i < 3 else "exact_number"}}
            for i in range(4)]
        row["boundary_results"] = [
            dict(event, event_index=i, exact_bound=(i + 1) * 8, one_below_bound=(i + 1) * 8 - 1,
                 exact_event_admitted=True, above_bound=(i + 1) * 8 + 1, above_event_admitted=True,
                 denial={"outcome":"exhausted","resource":"ReadStorage","amount":8,"configured_bound":(i+1)*8-1,
                         "locus":event["locus"],"entire_repeat_budget_equal":True},
                 one_below_event_denied=True, full_repeat_equal=True, accounting_equal=True)
            for i, event in enumerate(row["native_budget_events"])]
        row["designated_growth_case"] = {"test": "original", "driver": "original.rs", "line": 7,
            "provider": "unit-schema-provider", "input_binding":"exact-borrowed-input-evidence-and-callsite", "classes": {"walker_growth": "walker_growth", "exact_reservation": "exact_reservation"}}
        row["complete_reference"].update(read_storage=32, requested=32, exact_read_storage_limit=32,
                                          outcome="refused", refusal_code="UnitRefusal", above_read_storage_limit=33)
        row["provider_obligation_evidence"] = [
            {"role": role, "source": {"file": "original.rs", "line": 7, "symbol": "unit_schema_provider"},
             "input_binding":"exact-borrowed-input-evidence-and-callsite", "fact_source":"unit schema input assertion",
             "reservation_locus": {"source":"PackageRead(Graph, 0)","form":"walk" if role=="term_walk_multigrowth" else "exact_number"},
             "input_facts": {field: minimum}, "request_event_indices": [0, 1, 2] if role == "term_walk_multigrowth" else [3],
             "reservation_source": "unit schema reservation source", "same_walk_instance_source": "unit schema single walk"}
            for role, (field, minimum) in census.REQUIRED_PROVIDER_FACTS.items()]
        bundle["expected_refusal_codes"] = ["UnitRefusal"]
        return bundle

    def test_complete_schema_passes_only_with_all_obligations_and_refusals(self):
        bundle = self.complete_bundle()
        result = census.validate_full_completion(bundle)
        self.assertTrue(result["full_corpus"])
        self.assertFalse(result["static_ac513_proven"])
        bundle["expected_refusal_codes"].append("MissingActualRefusal")
        with self.assertRaisesRegex(census.CensusError, "every source refusal code"):
            census.validate_full_completion(bundle)

    def test_unknown_reservation_classes_remain_unknown_without_waiving_boundaries(self):
        row = self.complete_bundle()["runs"][0]["records"][0]
        row.pop("designated_growth_case")
        for event in row["native_budget_events"]:
            event["allocation_class"] = None
        for result in row["boundary_results"]:
            result["allocation_class"] = None
        row["boundary_results"] = [row["boundary_results"][i] for i in (0, 2, 3)]
        census.validate_normative_boundaries(row)
        row["boundary_results"].pop()
        with self.assertRaisesRegex(census.CensusError, "missing normative"):
            census.validate_normative_boundaries(row)

    def test_relabeling_a_growth_event_at_an_unrelated_role_locus_fails(self):
        bundle = self.complete_bundle()
        row = bundle["runs"][0]["records"][0]
        row["provider_obligation_evidence"][-1]["reservation_locus"] = {
            "source":"PackageRead(Graph, 99)","form":"exact_number"}
        with self.assertRaisesRegex(census.CensusError, "actual selected source-bound"):
            census.validate_full_completion(bundle)

    def test_missing_walk_instance_and_omitted_intermediate_boundary_fail(self):
        for remove in ("walk-instance", "intermediate-event"):
            bundle = self.complete_bundle()
            row = bundle["runs"][0]["records"][0]
            if remove == "walk-instance":
                for evidence in row["provider_obligation_evidence"]:
                    evidence.pop("same_walk_instance_source", None)
            else:
                row["boundary_results"].pop(1)
            with self.assertRaises(census.CensusError):
                census.validate_full_completion(bundle)

    def test_empty_provider_roles_never_complete(self):
        with self.assertRaisesRegex(census.CensusError, "missing real adverse/growth"):
            census.validate_full_completion(self.bundle())

    def test_one_corpus_selected_subset_and_failed_original_fail_closed(self):
        for mutation in (lambda b: b["runs"].pop(),
                         lambda b: b["runs"][0]["config"].update(original_tests=["original"]),
                         lambda b: b["runs"][0]["report"].update(exit_code=1),
                         lambda b: b["runs"][0].update(declared_inventory=["original", "omitted"])):
            bundle = self.bundle(); mutation(bundle)
            with self.assertRaises(census.CensusError):
                census.validate_full_completion(bundle)

    def test_foreign_or_missing_class_cohort_fails(self):
        for mutation in (lambda b: b["canonical_config"].update(source_head="old-source"),
                         lambda b: b["canonical_classes"].update(qualified=False)):
            bundle = self.bundle(); mutation(bundle)
            with self.assertRaisesRegex(census.CensusError, "same-cohort"):
                census.validate_full_completion(bundle)

    def test_provider_label_without_real_size_fact_or_source_fails(self):
        bundle = self.bundle()
        bundle["runs"][0]["records"][0]["provider_obligation_evidence"] = [{"role": "reference_chain"}]
        with self.assertRaisesRegex(census.CensusError, "source-bound measured input"):
            census.validate_full_completion(bundle)

    def test_large_graph_is_not_evidence_of_one_growing_term_walk(self):
        bundle = self.bundle()
        bundle["runs"][0]["records"][0]["provider_obligation_evidence"] = [{
            "role": "term_walk_multigrowth", "source": {"file": "original.rs", "line": 7, "symbol": "graph_builder"},
            "input_binding":"exact-borrowed-input-evidence-and-callsite", "fact_source":"unit size assertion",
            "input_facts": {"same_term_walk_stack_growth_requests": 100000}}]
        with self.assertRaisesRegex(census.CensusError, "actual selected source-bound reservations"):
            census.validate_full_completion(bundle)


class DesignatedGrowthEvidence(unittest.TestCase):
    def test_missing_intermediate_walker_or_limb_boundary_fails(self):
        for role in ("walker_growth", "exact_reservation", "exact_limb"):
            events = [{"resource": "ReadStorage", "amount": 8, "consumed_before": i * 8,
                       "admitted": True, "locus": {"source": "PackageRead(Graph, 0)", "form": "reserve"},
                       "allocation_class": "actual_source_class"} for i in range(3)]
            record = {"proof_schema": "native-budget-events/v1", "read_storage": 24,
                      "test": "actual::case", "driver": "actual.rs", "line": 17,
                      "designated_growth_case": {"test": "actual::case", "driver": "actual.rs", "line": 17,
                           "provider": "original_source_builder", "input_binding":"exact-borrowed-input-evidence-and-callsite", "classes": {"actual_source_class": role}},
                      "native_budget_events": events,
                      "original_context_outcome":"budget-incomplete",
                      "complete_reference":{"native_budget":"unlimited","outcome":"admitted",
                        "prior_work":0,"prior_read_storage":0,"work":0,"read_storage":24,"requested":24,
                        "logical_storage_delta":0,"exact_work_limit":0,"exact_read_storage_limit":24,
                        "exact_outcome_equal":True,"exact_counts_equal":True,"complete_repeat_equal":True,
                        "above_work_limit":1,"above_read_storage_limit":25,"above_outcome_equal":True,"above_counts_equal":True,"above_budget_equal":True}}
            def result(i):
                e = events[i]
                return {"event_index": i, "exact_bound": (i+1)*8, "one_below_bound": (i+1)*8-1,
                        "above_bound": (i+1)*8+1, "above_event_admitted": True,
                        "denial":{"outcome":"exhausted","resource":"ReadStorage","amount":8,"configured_bound":(i+1)*8-1,
                                  "locus":e["locus"],"entire_repeat_budget_equal":True},
                        "exact_event_admitted": True, "one_below_event_denied": True,
                        "full_repeat_equal": True, "accounting_equal": True,
                        "amount": 8, "locus": e["locus"], "allocation_class": e["allocation_class"]}
            record["boundary_results"] = [result(i) for i in range(3)]
            census.validate_normative_boundaries(record)
            record["boundary_results"] = [result(0), result(2)]
            with self.assertRaisesRegex(census.CensusError, "missing normative"):
                census.validate_normative_boundaries(record)


if __name__ == "__main__":
    unittest.main()

class CanonicalClassSafety(unittest.TestCase):
    def source(self, directory):
        source=Path(directory)/"read.rs"
        source.write_text("impl Parser {\n"+"\n".join(" fn "+name+"() {}" for name in census.CANONICAL_PATH_FUNCTIONS)+"\n"+"\n".join(expr for _,expr in census.CANONICAL_CLASS_SITES.values())+"\n}")
        return source

    def test_empty_text_candidate_cannot_be_bound_by_positive_append_coverage(self):
        import tempfile
        with tempfile.TemporaryDirectory() as directory:
            counts=dict.fromkeys(census.CANONICAL_PATH_FUNCTIONS,0)
            counts.update(value=1,append=1,node=1)
            with self.assertRaisesRegex(census.CensusError,"before-denial Parser path"):
                census.bind_canonical_denial("text",counts,{"outcome":"reservation-denied","calls":1},self.source(directory))

    def test_retained_children_require_actual_predecessors_and_close_before_parent_node(self):
        import tempfile
        with tempfile.TemporaryDirectory() as directory:
            counts=dict.fromkeys(census.CANONICAL_PATH_FUNCTIONS,0)
            counts.update(value=2,node=1,complete=1,close=1)
            result=census.bind_canonical_denial("children",counts,{"outcome":"reservation-denied","calls":4},self.source(directory))
            self.assertEqual(result["granted_predecessor_callbacks"],3)
            for change in ({"close":0},{"node":2}):
                bad=dict(counts,**change)
                with self.assertRaises(census.CensusError):
                    census.bind_canonical_denial("children",bad,{"outcome":"reservation-denied","calls":4},self.source(directory))

    def test_completion_rejects_empty_omitted_unverified_or_unrun_negative_controls(self):
        self.assertFalse(census.canonical_classes_complete([],[]))
        classes=[{"class":name,"qualified":True} for name in census.CANONICAL_CLASS_SITES]
        negatives=[{"test":name,"rejected":True,"rust_oracle_passed":True} for name in
                   ["negative_empty_string_is_not_text_growth","negative_empty_array_is_not_retained_children","negative_scratch_child_is_not_retained_children"]]
        self.assertTrue(census.canonical_classes_complete(classes,negatives))
        self.assertFalse(census.canonical_classes_complete(classes[:-1],negatives))
        self.assertFalse(census.canonical_classes_complete(classes,negatives[:-1]))
        negatives[0]["rust_oracle_passed"]=False
        self.assertFalse(census.canonical_classes_complete(classes,negatives))
