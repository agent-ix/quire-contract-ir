#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX
"""Check actual QSpec read invocations from an unfiltered llvm-cov export.

This is an executed-corpus proof, not the AC-513 static reachability proof.
A missed alias, function item or external helper contributes a production
entry without a corresponding measured test boundary and fails the census.
The trusted test helpers retain their original source and semantic oracles.
"""

import argparse
import json
from pathlib import Path
import re


class CensusError(ValueError):
    """Compiler evidence is incomplete, ambiguous, or contradicts the boundary."""


# These are operation boundaries, not source-file exclusions. Each successful
# entry performs exactly one corresponding real API call; the actual helper
# source and calibration tests must be reviewed with this arithmetic.
BOUNDARIES = {
    ("tests/it/support/qspec_read_proof.rs", "read_and_account"): "plain",
    ("tests/it/support/read_observation.rs", "measure_at"): "budgeted",
    ("tests/it/support/qspec_read_proof.rs", "measure_seeded"): "budgeted",
    ("tests/it/support/qspec_read_proof.rs", "read_budget_and_account"): "budgeted",
    ("tests/it/support/qspec_read_proof.rs",
     "caller_budget_observation_preserves_prior_charges_and_outer_capture"): "budgeted-calibration",
}
PRODUCTION = {
    ("crates/quire-contract-model/src/checked_package/v2/budgeted_read.rs", "read"): "plain",
    ("crates/quire-contract-model/src/checked_package/v2/budgeted_read.rs", "read_with_budget"): "budgeted",
}


def closure_symbol(name):
    # V0 closure paths start NC (including generic closure instantiations).
    # Legacy demangled names carry the closure component explicitly.
    return (name.startswith("_RNC") or "::{closure" in name
            or "{{closure}}" in name or "$u7b$$u7b$closure" in name)


def function_line(source, name):
    matches = [i for i, line in enumerate(source.splitlines(), 1)
               if re.search(r"\bfn\s+" + re.escape(name) + r"\s*(?:[<(])", line)]
    if len(matches) != 1:
        raise CensusError(f"expected one source function {name}, found {matches}")
    return matches[0]


def symbol_mentions(name, function):
    # Both rustc V0/legacy symbols retain the source identifier text; closure
    # instances are excluded separately, so their enclosing name cannot count.
    # Generic V0 instantiations append encoded type arguments immediately
    # after the identifier. Source-definition regions disambiguate names.
    return function in name


def tally(export, root):
    if export.get("type") != "llvm.coverage.json.export":
        raise CensusError("expected the compiler's full llvm-cov JSON export")
    if not export.get("data"):
        raise CensusError("empty compiler coverage export")
    wanted = {**BOUNDARIES, **PRODUCTION}
    result = {key: 0 for key in wanted}
    definitions = {key: function_line((root / key[0]).read_text(), key[1])
                   for key in wanted}
    seen = set()
    names = set()
    for unit in export["data"]:
        if not unit.get("functions"):
            raise CensusError("per-function evidence absent (do not use --skip-functions)")
        for function in unit["functions"]:
            name = function["name"]
            identity = (name, tuple(function.get("filenames", [])))
            if identity in names:
                raise CensusError(f"duplicate function evidence {name}")
            names.add(identity)
            if closure_symbol(name):
                continue
            for key in wanted:
                file, source_name = key
                files = function.get("filenames", [])
                matching = [i for i, filename in enumerate(files)
                            if (Path(filename) if Path(filename).is_absolute() else root / filename).resolve()
                            == (root / file).resolve()]
                if not matching or not symbol_mentions(name, source_name):
                    continue
                # Region file ids are compiler supplied. Require this exact
                # source definition, not an equally named function elsewhere.
                regions = [region for region in function.get("regions", [])
                           if len(region) >= 8 and region[5] in matching]
                line = definitions[key]
                if not any(region[0] <= line <= region[2] for region in regions):
                    continue
                count = function.get("count")
                if not isinstance(count, int) or count < 0:
                    raise CensusError(f"invalid compiler entry count for {name}")
                result[key] += count
                seen.add(key)
    # A zero-use optional caller-budget helper is allowed only when its
    # compiler function record exists with count zero. No missing-file waiver.
    missing = set(wanted) - seen
    if missing:
        raise CensusError(f"missing compiled function records: {sorted(missing)}")
    return result


def check(export, root):
    counts = tally(export, root)
    def count(file, name):
        return counts[(file, name)]
    wrapper = "tests/it/support/qspec_read_proof.rs"
    production = "crates/quire-contract-model/src/checked_package/v2/budgeted_read.rs"
    plain = count(production, "read")
    budgeted = count(production, "read_with_budget")
    measured_plain = count(wrapper, "read_and_account")
    probes = count("tests/it/support/read_observation.rs", "measure_at")
    seeded_probes = count(wrapper, "measure_seeded")
    caller_budget = count(wrapper, "read_budget_and_account")
    calibration = count(wrapper, "caller_budget_observation_preserves_prior_charges_and_outer_capture")
    if measured_plain + caller_budget == 0 or probes + seeded_probes == 0:
        raise CensusError("the original-driver corpus did not execute measured read boundaries")
    if plain != measured_plain:
        raise CensusError(f"unaccounted plain entries: production {plain}, measured boundaries {measured_plain}")
    if seeded_probes < 6 * caller_budget:
        raise CensusError("caller-budget entries lack first/repeated checked OFF/count/record snapshot reads")
    expected_budgeted = plain + probes + seeded_probes + caller_budget + calibration
    if budgeted != expected_budgeted:
        raise CensusError(f"unaccounted budgeted entries: production {budgeted}, expected {expected_budgeted} "
                          f"(plain {plain} + probes {probes} + seeded-probes {seeded_probes} + caller-budget {caller_budget} + calibration {calibration})")
    return {"scope": "executed source-snapshot reader invocation census only",
            "plain": plain, "budgeted": budgeted, "measured_plain": measured_plain,
            "measured_probes": probes, "seeded_probes": seeded_probes, "caller_budget": caller_budget,
            "private_counter_calibration": calibration,
            "static_ac513_proven": False}


def validate_provenance(export, root, intended, records):
    for record in records:
        if record.get("test") not in intended:
            raise CensusError("runtime record names a test outside the exact intended original set")
        file = Path(record.get("driver", ""))
        file = (file if file.is_absolute() else root / file).resolve()
        if not file.is_relative_to(root):
            raise CensusError("external runtime callsite needs its own compiled source evidence")
        line, column = record.get("line"), record.get("column")
        if not isinstance(line, int) or not isinstance(column, int) or line <= 0 or column <= 0:
            raise CensusError("invalid captured callsite coordinates")
        source = file.read_text().splitlines()
        if line > len(source):
            raise CensusError("captured callsite is outside the frozen source")
        spelling = source[line - 1]
        names = ("read_budget_and_account",) if record.get("entry") == "caller-budget" else ("read_and_account", "fixture_read")
        if re.search(r"\bfn\s+", spelling) or not any(re.search(r"\b" + name + r"\s*\(", spelling) for name in names):
            raise CensusError("captured location is not an actual instrumented invocation (function-item shim is unresolved)")
        executed = False
        for unit in export["data"]:
            for function in unit.get("functions", []):
                indices = {i for i, filename in enumerate(function.get("filenames", []))
                           if Path(filename).resolve() == file}
                for region in function.get("regions", []):
                    if (len(region) >= 8 and region[5] in indices and region[4] > 0
                            and (region[0], region[1]) <= (line, column) <= (region[2], region[3])):
                        executed = True
        if not executed:
            raise CensusError("runtime callsite has no executed compiler source region")


def validate_normative_boundaries(record):
    events = record.get("native_budget_events")
    results = record.get("boundary_results")
    if (record.get("proof_schema") != "native-budget-events/v1"
            or not isinstance(events, list) or not isinstance(results, list)):
        raise CensusError("full proof requires actual passive-event normative boundary results")
    complete=record.get("complete_reference",{})
    counters=("prior_work","prior_read_storage","work","read_storage","requested")
    if (complete.get("native_budget")!="unlimited"
            or complete.get("outcome") not in {"admitted","refused","incomplete","catalog-unavailable"}
            or any(type(complete.get(key)) is not int or complete[key]<0 for key in counters)
            or complete.get("logical_storage_delta")!=0
            or not complete.get("exact_outcome_equal") or not complete.get("exact_counts_equal")
            or not complete.get("complete_repeat_equal")):
        raise CensusError("full proof lacks measured unlimited complete-reference/exact-repeat evidence")
    if any(type(event.get("amount")) is not int or not 0 <= event["amount"] < 2**128 for event in events):
        raise CensusError("invalid full native event amount")
    if any(not event.get("admitted") for event in events):
        raise CensusError("complete-reference events contain an early native-budget denial")
    work=sum(event["amount"] for event in events if event.get("resource")=="Work")
    storage=sum(event["amount"] for event in events if event.get("resource")=="ReadStorage")
    if (complete["work"]!=complete["prior_work"]+work
            or complete["read_storage"]!=complete["prior_read_storage"]+storage
            or complete["requested"]!=storage
            or complete.get("exact_work_limit")!=complete["work"]
            or complete.get("exact_read_storage_limit")!=complete["read_storage"]):
        raise CensusError("unlimited complete-reference totals disagree with actual native events")
    selected = set()
    stage_last, class_first, class_last = {}, {}, {}
    designated = record.get("designated_growth_case")
    exhaustive = {}
    if designated is not None:
        if (designated.get("test") != record.get("test")
                or designated.get("driver") != record.get("driver")
                or designated.get("line") != record.get("line")
                or not designated.get("provider")
                or not isinstance(designated.get("classes"), dict)):
            raise CensusError("designated growth case lacks exact original-provider provenance")
        requested = designated.get("requested_classes", designated["classes"])
        if (not isinstance(requested, dict) or not requested
                or any(kind not in {"walker_growth", "exact_reservation", "exact_limb"}
                       for kind in requested.values())):
            raise CensusError("designated growth classes require source-derived walker/limb roles")
        exhaustive = {name: role for name, role in requested.items()
                      if any(event.get("admitted") and event.get("amount", 0) > 0
                             and event.get("resource") == "ReadStorage"
                             and event.get("allocation_class") == name for event in events)}
        if designated["classes"] != exhaustive:
            raise CensusError("designated reached classes do not match actual native reservations")
    seen_exhaustive = set()
    for index, event in enumerate(events):
        if not event.get("admitted") or event.get("amount", 0) == 0:
            continue
        resource = event.get("resource")
        if resource == "LogicalStorage":
            raise CensusError("a read admitted native logical-storage consumption")
        if resource not in {"Work", "ReadStorage"}:
            raise CensusError("unknown native observer resource")
        locus = event.get("locus") or {}
        match = re.fullmatch(r"PackageRead\(([A-Za-z]+), [0-9]+\)", locus.get("source", ""))
        if not match or not locus.get("form"):
            raise CensusError("native event lacks a genuine package-read stage/form")
        allocation_class = event.get("allocation_class")
        if resource == "ReadStorage" and not allocation_class:
            raise CensusError("native reservation has no source-derived allocation class")
        if resource == "ReadStorage" and allocation_class in exhaustive:
            selected.add(index)
            seen_exhaustive.add(allocation_class)
        stage_last[(resource, match[1])] = index
        key = (resource, match[1], locus["form"], allocation_class)
        class_first.setdefault(key, index)
        class_last[key] = index
    if seen_exhaustive != set(exhaustive):
        raise CensusError("designated growth class has no actual native reservation")
    selected.update(stage_last.values())
    selected.update(class_first.values())
    selected.update(class_last.values())
    if complete["requested"] > 0 and not selected:
        raise CensusError("nonempty read has no actual event-selected boundaries")
    observed = set()
    for result in results:
        index = result.get("event_index")
        if index not in selected or index in observed:
            raise CensusError("duplicate or unselected normative boundary result")
        observed.add(index)
        event = events[index]
        next_bound = event["consumed_before"] + event["amount"]
        if (result.get("exact_bound") != next_bound or result.get("one_below_bound") != next_bound - 1
                or not result.get("exact_event_admitted") or not result.get("one_below_event_denied")
                or not result.get("full_repeat_equal") or not result.get("accounting_equal")
                or result.get("amount") != event["amount"] or result.get("locus") != event["locus"]
                or result.get("allocation_class") != event.get("allocation_class")):
            raise CensusError("normative boundary outcome does not match its actual typed event")
    if observed != selected:
        raise CensusError("missing normative stage/class/walker/exact boundary results")


def check_run(export, root, report, records, mode):
    result = check(export, root)
    intended = report.get("intended_tests", [])
    passed = report.get("passed_tests", [])
    if (report.get("exit_code") != 0 or report.get("test_threads") != 1
            or not intended or len(set(intended)) != len(intended)
            or set(intended) != set(passed) or len(passed) != len(intended)
            or report.get("failed_tests") or report.get("ignored_tests")):
        raise CensusError("not every intended original-driver test executed successfully once")
    validate_provenance(export, root, set(intended), records)
    plain = [record for record in records if record.get("entry", "plain") == "plain"]
    caller = [record for record in records if record.get("entry") == "caller-budget"]
    if len(plain) != result["measured_plain"] or len(caller) != result["caller_budget"]:
        raise CensusError("temporary original-site records do not match compiler boundary counts")
    if any(record.get("snapshot_replay") != "exact-entire-caller-state" for record in caller):
        raise CensusError("caller-budget capture lacks exact entire-state replay evidence")
    if any(record.get("mode") != mode for record in plain):
        raise CensusError("mixed or missing proof modes in original-driver records")
    if any(not record.get("driver") or not record.get("line") or not record.get("test")
           for record in records):
        raise CensusError("missing original driver/test/callsite provenance")
    if mode == "full":
        # A caller-budget original entry is not a probe exemption. It must
        # carry actual full-boundary evidence too; core-only captures stay RED.
        requested_roles, reached_roles = set(), set()
        for record in records:
            validate_normative_boundaries(record)
            designated=record.get("designated_growth_case")
            if designated:
                requested_roles.update(designated.get("requested_classes",designated["classes"]).values())
                reached_roles.update(designated["classes"].values())
        if not requested_roles <= reached_roles:
            raise CensusError("a designated source growth role was never reached by any original input")
    result.update({"original_tests_passed": len(passed), "mode": mode,
                   "normative_boundary_proof": mode == "full"})
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--export", type=Path, required=True)
    parser.add_argument("--source-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--test-report", type=Path, required=True)
    parser.add_argument("--driver-log", type=Path, required=True)
    parser.add_argument("--mode", choices=["sample", "full"], required=True)
    args = parser.parse_args()
    try:
        result = check_run(
            json.loads(args.export.read_text()), args.source_root.resolve(),
            json.loads(args.test_report.read_text()),
            [json.loads(line) for line in args.driver_log.read_text().splitlines() if line.strip()],
            args.mode,
        )
    except (CensusError, OSError, ValueError, KeyError) as error:
        args.output.write_text(json.dumps({"passed": False, "error": str(error)}, indent=2) + "\n")
        raise SystemExit(str(error)) from error
    args.output.write_text(json.dumps({"passed": True, **result}, indent=2) + "\n")


if __name__ == "__main__":
    main()

# This is an execution-path binding for minimal class-denial controls, not a
# reader reachability proof or a per-callback class inference from byte sizes.
CANONICAL_CLASS_SITES = {
    "arena": ("node", ".grow(&mut self.slots, 1)"),
    "text": ("append", ".grow_text(&mut self.text, piece.len())"),
    "stack": ("value", ".grow(&mut self.stack, 1)"),
    "children": ("close", ".grow(&mut self.items, items.len())"),
    "duplicate_sort": ("check_unique_names", ".grow(&mut self.order, children.len())"),
}
CANONICAL_PATH_FUNCTIONS = ("value", "node", "complete", "close", "append", "member_name", "check_unique_names", "pointer")


def canonical_path_counts(export, source):
    lines = source.read_text().splitlines()
    definitions = {}
    for name in CANONICAL_PATH_FUNCTIONS:
        found = [i for i, line in enumerate(lines, 1)
                 if re.match(r"\s*fn\s+" + name + r"\s*\(", line)]
        if len(found) != 1:
            raise CensusError(f"canonical Parser source identity ambiguous/missing: {name}")
        definitions[name] = found[0]
    counts = dict.fromkeys(definitions, 0)
    seen = set()
    for unit in export.get("data", []):
        for function in unit.get("functions", []):
            if closure_symbol(function["name"]) or "Parser" not in function["name"]:
                continue
            indices = {i for i, name in enumerate(function.get("filenames", [])) if Path(name).resolve() == source.resolve()}
            for name, line in definitions.items():
                if not symbol_mentions(function["name"], name):
                    continue
                if not any(len(region) >= 8 and region[5] in indices and region[0] <= line <= region[2]
                           for region in function.get("regions", [])):
                    continue
                counts[name] += function["count"]
                seen.add(name)
    if seen != set(definitions):
        raise CensusError(f"missing compiled canonical Parser identities: {sorted(set(definitions)-seen)}")
    return counts


def bind_canonical_denial(provider, counts, facts, source):
    # These constraints distinguish the last actual reservation caller from
    # earlier granted callers and later callers which have not been entered.
    expected = {
        "arena": {"value":1,"node":1,"complete":0,"close":0,"append":0,"member_name":0,"check_unique_names":0,"pointer":0},
        "text": {"value":1,"node":0,"complete":0,"close":0,"append":1,"member_name":0,"check_unique_names":0,"pointer":0},
        "stack": {"value":1,"node":0,"complete":0,"close":0,"append":0,"member_name":0,"check_unique_names":0,"pointer":0},
        "children": {"value":2,"node":1,"complete":1,"close":1,"append":0,"member_name":0,"check_unique_names":0,"pointer":0},
        "duplicate_sort": {"value":3,"node":2,"complete":2,"close":1,"member_name":2,"check_unique_names":1,"pointer":0},
    }[provider]
    predecessors = {"arena":0,"text":0,"stack":0,"children":3,"duplicate_sort":3}[provider]
    if facts.get("outcome") != "reservation-denied" or facts.get("calls") != predecessors+1:
        raise CensusError("actual denied callback does not follow the genuine required predecessor path")
    if any(counts[name] != amount for name, amount in expected.items()):
        raise CensusError(f"canonical before-denial Parser path does not identify {provider}: {counts}")
    function, expression = CANONICAL_CLASS_SITES[provider]
    source_lines = source.read_text().splitlines()
    sites = [(i, line.index(expression)+1) for i,line in enumerate(source_lines,1) if expression in line]
    if len(sites) != 1:
        raise CensusError("canonical reservation expression source identity changed/ambiguous")
    return {"source":str(source),"qualified_function":"quire_canonical::read::Parser::"+function,
            "reservation_expression":expression,"line":sites[0][0],"column":sites[0][1],
            "before_denial_counts":counts,"granted_predecessor_callbacks":predecessors}


def canonical_classes_complete(classes, negatives):
    expected=set(CANONICAL_CLASS_SITES)
    required_negatives={"negative_empty_string_is_not_text_growth",
                        "negative_empty_array_is_not_retained_children",
                        "negative_scratch_child_is_not_retained_children"}
    return (len(classes)==len(expected) and {item.get("class") for item in classes}==expected
            and all(item.get("qualified") is True for item in classes)
            and len(negatives)==len(required_negatives)
            and {item.get("test") for item in negatives}==required_negatives
            and all(item.get("rejected") is True and item.get("rust_oracle_passed") is True
                    for item in negatives))
