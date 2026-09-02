"""Gates for the pinned shared-assurance path (FR-022).

Each test runs the real thing. The compatibility view and the pin classifier
need `engineering-assurance`, which lives in its own interpreter because the
PGM-01 Draft 7 lane pins jsonschema 3.2.0 and Engineering Assurance declares
4.x; these tests invoke that interpreter rather than importing across the two.

A missing assurance interpreter fails these tests. It does not skip them: a
gate that quietly stands down when its dependency is absent reports the same
green as one that ran, and this whole migration exists because that is not
acceptable.

The chain report is expensive to produce and every chain test reads the same
one, so it is built once and cached for the module.
"""

from __future__ import annotations

import json
import os
import subprocess
import unittest
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
ASSURANCE_PYTHON = Path(
    os.environ.get("ASSURANCE_PYTHON", ROOT / ".venv-assurance/bin/python")
)
CONFORMANCE_RESULT = ROOT / "target/assurance/conformance.jsonl"
QUIRE_EXPORT = ROOT / "target/assurance/quire-static-export.json"

# Every state the migration contract requires this repository to demonstrate,
# and the gate that demonstrates it. A state with no home here is a state
# nobody showed.
REQUIRED_STATES = {
    "pass": "chain",
    "fail": "chain",
    "unavailable": "chain",
    "not-computed": "chain",
    "partial": "chain",
    "stale": "chain",
    "tampered": "chain",
    "unsupported": "compatibility",
    "inconclusive": "compatibility",
    "malformed": "compatibility",
    "suspect": "compatibility",
    "vacuous": "adapter",
}

_CACHE: dict[str, Any] = {}


def assurance_interpreter() -> Path:
    if not ASSURANCE_PYTHON.is_file():
        raise AssertionError(
            f"the pinned assurance interpreter is missing at {ASSURANCE_PYTHON}. "
            "Run `make assurance-env`. This is a failure and not a skip: a gate that "
            "stands down when its dependency is absent reports the same green as one "
            "that ran."
        )
    return ASSURANCE_PYTHON


def run_assurance(*arguments: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(assurance_interpreter()), *arguments],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )


def pin_report() -> dict[str, Any]:
    if "pins" not in _CACHE:
        completed = run_assurance("scripts/check_shared_pins.py", "--json")
        if completed.returncode == 2:
            raise AssertionError(f"the pin classifier could not run: {completed.stderr}")
        _CACHE["pins"] = json.loads(completed.stdout)
        _CACHE["pins_exit"] = completed.returncode
    return _CACHE["pins"]


def compatibility_report() -> dict[str, Any]:
    if "compatibility" not in _CACHE:
        completed = run_assurance("scripts/pgm01_compatibility_view.py", "--json")
        if completed.returncode != 0:
            raise AssertionError(f"the compatibility census failed: {completed.stderr}")
        _CACHE["compatibility"] = json.loads(completed.stdout)
    return _CACHE["compatibility"]


def chain_report() -> dict[str, Any]:
    if "chain" in _CACHE:
        return _CACHE["chain"]
    if not CONFORMANCE_RESULT.is_file() or not QUIRE_EXPORT.is_file():
        raise AssertionError(
            "the native producer outputs are missing. Run `make assurance-inputs`; "
            "nothing in the shared path produces them, by design."
        )
    revision = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True, check=True
    ).stdout.strip()
    completed = subprocess.run(
        [
            "python3",
            "scripts/assurance_chain.py",
            "--candidate-revision",
            revision,
            "--conformance",
            str(CONFORMANCE_RESULT),
            "--quire-export",
            str(QUIRE_EXPORT),
            "--json",
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        raise AssertionError(
            f"the change-assurance chain failed: {completed.stderr or completed.stdout}"
        )
    _CACHE["chain"] = json.loads(completed.stdout)
    return _CACHE["chain"]


def assert_equal(actual: Any, expected: Any, message: str) -> None:
    if actual != expected:
        raise AssertionError(f"{message}: expected {expected!r}, got {actual!r}")


def assert_true(value: Any, message: str) -> None:
    if not value:
        raise AssertionError(message)


def assert_false(value: Any, message: str) -> None:
    if value:
        raise AssertionError(message)


def test_shared_components_classify_against_the_accepted_matrix() -> None:
    """TC-029. Trace: TC-029, FR-022-AC-1."""
    report = pin_report()
    verdicts = {item["component"]: item["verdict"] for item in report["components"]}
    assert_equal(
        sorted(verdicts),
        ["engineering-assurance", "ix-flow", "quire-cli", "quoin"],
        "every pinned component must be classified, including one that is absent",
    )
    for component, verdict in verdicts.items():
        reasons = [i["reason"] for i in report["components"] if i["component"] == component]
        assert_equal(verdict, "compatible", f"{component} is {verdict}: {reasons}")
    assert_equal(report["artifact_mismatches"], [], "a consumed artifact drifted")
    assert_equal(
        report["mirror_references"],
        [],
        "the internal npm.ix mirror must not appear in any requirement or pin",
    )
    assert_true(report["accepted"], "the local toolchain gate must be satisfied")


def test_matrix_acceptance_is_reported_and_never_inferred() -> None:
    """TC-029. Trace: TC-029, FR-022-AC-1."""
    report = pin_report()
    # The installed release is the authority on what it records, and it is
    # reported verbatim. What must never happen is this repository reading an
    # approval out of an artifact that does not carry one.
    assert_true("acceptance_state" in report, "the acceptance state must be reported")
    if not report["acceptance_recorded_here"]:
        assert_true(
            report["acceptance_state"] != "accepted",
            "a state that reads as accepted with nobody named is not a record",
        )
    pins = json.loads((ROOT / "assurance/pins.json").read_text(encoding="utf-8"))
    assert_true(
        pins["known_drift"],
        "a pin whose release lags its acceptance record must say so in writing",
    )


def test_an_unobservable_component_is_unknown_and_never_a_pass() -> None:
    """TC-029. Trace: TC-029, FR-022-AC-1."""
    program = (
        "import json;"
        "from engineering_assurance.compatibility import classify, load_matrix;"
        "m=load_matrix();"
        "print(json.dumps({n: classify(m, n, None).verdict "
        "for n in ['quoin','quire-cli','ix-flow','engineering-assurance']}))"
    )
    completed = subprocess.run(
        [str(assurance_interpreter()), "-c", program],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    assert_equal(completed.returncode, 0, completed.stderr)
    for component, verdict in json.loads(completed.stdout).items():
        assert_equal(verdict, "unknown", f"an unobserved {component} must not read as a pass")


def test_a_drifted_consumed_artifact_fails_closed() -> None:
    """TC-029. Trace: TC-029, FR-022-AC-1."""
    program = (
        "import json,sys;"
        "sys.path.insert(0, 'scripts');"
        "import check_shared_pins as c;"
        "p=c.load_pins();"
        "p['engineering_assurance']['consumed_artifacts'][0]['sha256']='0'*64;"
        "print(json.dumps(c.artifact_digest_mismatches(p)))"
    )
    completed = subprocess.run(
        [str(assurance_interpreter()), "-c", program],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    assert_equal(completed.returncode, 0, completed.stderr)
    assert_true(
        json.loads(completed.stdout),
        "a drifted consumed artifact must be reported, not read past",
    )


def test_the_domain_result_reaches_quoin_through_the_declared_adapter() -> None:
    """TC-030. Trace: TC-030, FR-022-AC-2."""
    report = chain_report()
    assert_equal(
        report["conformance_protocol"],
        "quire.contract.conformance-jsonl/v1",
        "the adapter reads the runner's declared protocol",
    )
    probes = {probe["probe"]: probe for probe in report["adapter_probes"]}
    assert_equal(probes["accepts-the-real-run"]["exit"], 0, "the real run must transcribe")
    assert_equal(
        probes["refuses-a-vacuous-run"]["exit"], 1, "an empty run is not a clean run"
    )
    assert_equal(
        probes["refuses-a-foreign-protocol"]["exit"],
        1,
        "a stream that does not declare the runner's protocol is not the runner's output",
    )


def test_no_verdict_is_read_from_a_console_stream() -> None:
    """TC-030. Trace: TC-030, FR-022-AC-2."""
    # A verdict recovered from console text is a verdict the producer never
    # made. Reading a version string, or parsing a stream whose protocol the
    # producer declares on every row, is not that — so the check names verdict
    # words rather than banning every mention of stdout. A rule broad enough to
    # catch `SEMVER.search(stdout)` would be reworded away rather than obeyed.
    verdict_words = ("pass", "fail", "ok", "success", "error", "green", "red", "warning")
    offenders = []
    for path in sorted((ROOT / "scripts").glob("*.py")):
        for number, line in enumerate(
            path.read_text(encoding="utf-8").splitlines(), start=1
        ):
            stripped = line.strip()
            if stripped.startswith("#"):
                continue
            if "stdout" not in stripped and "stderr" not in stripped:
                continue
            lowered = stripped.lower()
            if any(
                f'"{word}" in ' in lowered or f"'{word}' in " in lowered
                for word in verdict_words
            ):
                offenders.append(f"{path.name}:{number}: {stripped}")
    assert_equal(
        offenders,
        [],
        "a verdict recovered from console text is a verdict the producer never made",
    )
    chain = (ROOT / "scripts/assurance_chain.py").read_text(encoding="utf-8")
    assert_true(
        "quire.contract.conformance-jsonl/v1" in chain,
        "the chain must bind to the runner's declared protocol",
    )


def test_the_static_export_is_retained_by_digest_without_execution() -> None:
    """TC-031. Trace: TC-031, FR-022-AC-3."""
    scenarios = {item["scenario"]: item for item in chain_report()["scenarios"]}
    retained = scenarios["retain-producer-output"]
    assert_true(retained["matched"], f"retention scenario did not match: {retained}")
    assert_true(
        retained["observed"]["retained_bytes_identical"],
        "Quoin must retain the producer's exact bytes",
    )
    assert_equal(retained["observed"]["export_intake"], 0, "the static export must retain")


def test_the_chain_refuses_to_run_a_producer() -> None:
    """TC-031. Trace: TC-031, FR-022-AC-3."""
    source = (ROOT / "scripts/assurance_chain.py").read_text(encoding="utf-8")
    assert_true(
        '["quoin", *arguments]' in source,
        "quoin must be the only command family the chain invokes",
    )
    for forbidden in ("cargo", "rustup", "quire-contract-conformance", "quire"):
        assert_false(
            f'subprocess.run(["{forbidden}"' in source,
            f"the chain must not invoke {forbidden}; the producer runs natively",
        )


def test_every_immutable_record_maps_read_only_and_keeps_its_digest() -> None:
    """TC-032. Trace: TC-032, FR-009-AC-5, FR-022-AC-4."""
    report = compatibility_report()
    assert_equal(
        report["evidence_bytes_moved"], [], "reading history must not write to it"
    )
    assert_equal(report["drifted_derivations"], [], "a fixture is not its declared derivation")
    historical = [case for case in report["cases"] if case["kind"] == "historical"]
    assert_equal(len(historical), 10, "every retained PGM-01 record must be read")
    for case in historical:
        assert_equal(case["outcome"], "lossy", f"{case['case']} outcome")
        assert_true(case["digest_preserved"], f"{case['case']} lost its source digest")
        assert_true(
            case["unmapped_fields"] > 0,
            f"{case['case']} claims a complete mapping of a record that has no "
            "producer version, no configuration digest, and no decision event",
        )


def test_unreadable_unsupported_and_tampered_keep_their_own_outcome() -> None:
    """TC-032. Trace: TC-032, FR-022-AC-4."""
    outcomes = {case["case"]: case["outcome"] for case in compatibility_report()["cases"]}
    assert_equal(outcomes["derived-tampered"], "incompatible", "a tampered record")
    assert_equal(outcomes["unsupported-schema"], "incompatible", "an unknown schema major")
    assert_equal(outcomes["malformed-v1"], "unreadable", "a malformed record")
    assert_equal(outcomes["unreadable-truncated"], "unreadable", "bytes that are not JSON")
    assert_equal(
        outcomes["derived-not-computed"],
        "unreadable",
        "PGM-01 v1 has no not-computed status and the mapping must refuse one rather "
        "than invent a translation",
    )
    assert_equal(
        outcomes["correction-record-is-not-an-evidence-record"],
        "incompatible",
        "a correction record is not a PGM-01 evidence record and is not read as one",
    )


def test_every_required_state_has_a_demonstrated_case() -> None:
    """TC-033. Trace: TC-033, FR-022-AC-5."""
    chain = chain_report()
    compatibility = compatibility_report()
    demonstrated = {
        "chain": set(chain["states_demonstrated"]),
        "compatibility": {
            "unsupported",
            "malformed",
            *(state for case in compatibility["cases"] for state in case["mapped_states"]),
            *({"suspect"} if compatibility["suspect_demonstrated"] else set()),
        },
        "adapter": {
            "vacuous"
            for probe in chain["adapter_probes"]
            if probe["probe"] == "refuses-a-vacuous-run" and probe["matched"]
        },
    }
    missing = [
        state for state, home in REQUIRED_STATES.items() if state not in demonstrated[home]
    ]
    assert_equal(missing, [], f"states with no demonstrated case: {missing}")


def test_no_non_success_state_collapses_into_another() -> None:
    """TC-033. Trace: TC-033, FR-022-AC-5."""
    scenarios = {item["scenario"]: item for item in chain_report()["scenarios"]}
    for name in ("attested-unavailable", "attested-not_computed"):
        observed = scenarios[name]["observed"]
        assert_false(observed["collapsed_to_pass"], f"{name} read as a pass")
        assert_false(observed["collapsed_to_fail"], f"{name} read as a failure")
        assert_true(
            observed["reason_names_the_state"],
            f"{name} lost its reason: {observed['reasons']}",
        )
    assert_false(
        scenarios["attested-failure"]["observed"]["collapsed_to_pass"],
        "a failing proof read as a pass",
    )
    assert_false(
        scenarios["stale-candidate-binding"]["observed"]["discharged"],
        "an attestation bound to another candidate discharged this one",
    )
    assert_false(
        scenarios["retained-bytes-changed-after-sealing"]["observed"]["retained"],
        "bytes that contradict their sealed digest were retained",
    )


def test_the_human_decision_is_absent_and_is_not_synthesized() -> None:
    """TC-033. Trace: TC-033, FR-009-AC-2, FR-022-AC-5."""
    scenarios = {item["scenario"]: item for item in chain_report()["scenarios"]}
    observed = scenarios["unattested-proof-and-absent-decision"]["observed"]
    assert_equal(observed["outcome"], "incomplete", "the receipt outcome")
    assert_equal(observed["review_reason"], "decision_missing", "the review reason")
    declaration = json.loads(
        (ROOT / "assurance/change-assurance.json").read_text(encoding="utf-8")
    )
    unknowns = {
        unknown["id"]: unknown for unknown in declaration["record"]["definition"]["unknowns"]
    }
    assert_equal(
        unknowns["UNKNOWN-human-decision-absent"]["disposition"], "open", "the disposition"
    )
    assert_equal(
        unknowns["UNKNOWN-human-decision-absent"]["owner"], "@kreneskyp", "the owner"
    )


def test_weakening_the_compatibility_census_turns_it_red() -> None:
    """TC-034. Trace: TC-034, FR-022-AC-6."""
    completed = run_assurance(
        "scripts/pgm01_compatibility_view.py", "--mutation-probes", "--json"
    )
    assert_equal(completed.returncode, 0, completed.stderr)
    report = json.loads(completed.stdout)
    assert_true(len(report["probes"]) >= 6, "too few probes to call this tested")
    escaped = [probe["name"] for probe in report["probes"] if not probe["detected"]]
    assert_equal(escaped, [], f"mutation probes escaped the census: {escaped}")


def test_every_negative_result_is_paired_with_a_positive_control() -> None:
    """TC-034. Trace: TC-034, FR-022-AC-6."""
    report = chain_report()
    assert_true(len(report["controls"]) >= 4, "too few controls")
    paired = {item["pairs_with"] for item in report["controls"]}
    for negative in (
        "retained-bytes-changed-after-sealing",
        "stale-candidate-binding",
        "refuse-an-edited-receipt",
        "attested-failure",
    ):
        assert_true(
            negative in paired,
            f"{negative} has no positive control, so it could be a step that never "
            "worked rather than a check that fired",
        )
    for control in report["controls"]:
        assert_true(control["matched"], f"control did not hold: {control}")


def load_tests(
    _loader: unittest.TestLoader,
    tests: unittest.TestSuite,
    _pattern: str | None,
) -> unittest.TestSuite:
    for name, function in sorted(globals().items()):
        if name.startswith("test_") and callable(function):
            tests.addTest(unittest.FunctionTestCase(function))
    return tests


if __name__ == "__main__":
    unittest.main()
