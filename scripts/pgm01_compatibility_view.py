#!/usr/bin/env python3
"""Read this repository's immutable PGM-01 history through the shared mapping.

FR-009-AC-5, FR-022-AC-4, FR-022-AC-5. Every legacy record in `evidence/` is
read through `engineering_assurance.verification_semantics.map_pgm01_bytes` —
the FR-010 read-only compatibility view — and nothing else. This file owns no
mapping rules of its own; it owns the census of what this repository has, the
expected outcome for each case, and the proof that reading changed nothing.

Four things it deliberately is not:

- not a verifier: it asserts declared expectations, and a record's outcome is
  whatever the shared mapping says it is;
- not a writer: the evidence tree is digested before and after every run and a
  single changed byte is a failure;
- not a translator: an outcome of `unreadable` or `incompatible` is reported as
  itself and never repaired into a readable one;
- not an aggregator: there is no overall verdict, only a per-case outcome and
  the count of cases whose outcome differed from the one declared.

    python3 scripts/pgm01_compatibility_view.py [--json]
    python3 scripts/pgm01_compatibility_view.py --mutation-probes

Exit status: 0 when every case matched its declared expectation and the
evidence tree is unchanged; 1 when a case did not match or bytes moved; 2 when
the pinned mapping could not be loaded, because a census that could not consult
the authority has nothing to report.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import sys
from pathlib import Path
from typing import Any, Callable

ROOT = Path(__file__).resolve().parent.parent
EVIDENCE = ROOT / "evidence"
FIXTURES = ROOT / "tests/fixtures/legacy-compat"
EXPECTATIONS_PATH = FIXTURES / "expectations.json"
EXPECTATIONS_VERSION = "quire-contract-ir.legacy-compat-expectations/v1"
REPORT_VERSION = "quire-contract-ir.pgm01-compatibility-census/v1"

# Every distinct check state the census must be able to show, and the case that
# shows it. A state with no case is a state nobody has demonstrated.
REQUIRED_STATES = ("passed", "failed", "skipped", "inconclusive", "unavailable")


class CompatibilityError(RuntimeError):
    """The pinned compatibility mapping is unavailable or misdeclared."""


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_mapper() -> Callable[..., dict[str, Any]]:
    try:
        from engineering_assurance.verification_semantics import map_pgm01_bytes
    except ImportError as error:
        raise CompatibilityError(
            "engineering-assurance is not importable; install "
            f"requirements-assurance.txt into this interpreter ({sys.executable}): {error}"
        ) from error
    return map_pgm01_bytes


def evidence_census() -> dict[str, str]:
    """Digest every file under `evidence/`, so a write anywhere is detectable."""
    return {
        path.relative_to(ROOT).as_posix(): sha256(path.read_bytes())
        for path in sorted(EVIDENCE.rglob("*"))
        if path.is_file()
    }


def load_expectations() -> dict[str, Any]:
    try:
        declared = json.loads(EXPECTATIONS_PATH.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise CompatibilityError(f"cannot read declared expectations: {error}") from error
    if declared.get("schema_version") != EXPECTATIONS_VERSION:
        raise CompatibilityError(f"unknown expectations schema: {declared.get('schema_version')!r}")
    return declared


def serialize(document: Any) -> bytes:
    """Reproduce the retained records' exact serialization."""
    return (json.dumps(document, indent=2) + "\n\n").encode("utf-8")


def pointer_get(node: Any, parts: list[str]) -> Any:
    for part in parts:
        node = node[int(part)] if isinstance(node, list) else node[part]
    return node


def pointer_set(document: Any, pointer: str, value: Any) -> None:
    parts = pointer.strip("/").split("/")
    node = pointer_get(document, parts[:-1])
    last = parts[-1]
    if isinstance(node, list):
        node[int(last)] = value
    else:
        node[last] = value


def derive(source: bytes, derivation: dict[str, Any]) -> bytes:
    """Re-apply one declared edit to the real source bytes."""
    if derivation.get("operation") == "truncate":
        return source[: derivation["bytes"]]
    document = json.loads(source)
    pointer = derivation["pointer"]
    current = pointer_get(document, pointer.strip("/").split("/"))
    if current != derivation["from"]:
        raise CompatibilityError(
            f"declared derivation {pointer} reads {current!r}, not {derivation['from']!r}"
        )
    pointer_set(document, pointer, derivation["to"])
    return serialize(document)


def verify_derivations(declared: dict[str, Any]) -> list[str]:
    """Re-derive every constructed fixture and reject one that drifted.

    Without this, a fixture could be hand-edited into any shape at all and still
    claim in its own metadata to be one named change to a real record. The claim
    is checkable, so it is checked.
    """
    source_path = ROOT / declared["source"]["path"]
    source = source_path.read_bytes()
    failures: list[str] = []
    if sha256(source) != declared["source"]["sha256"]:
        failures.append(f"{declared['source']['path']}: source record digest changed")
        return failures
    if serialize(json.loads(source)) != source:
        failures.append(f"{declared['source']['path']}: source record no longer round-trips")
        return failures
    for fixture in declared["cases"]:
        derivation = fixture.get("derivation")
        if derivation is None:
            continue
        expected = derive(source, derivation)
        actual = (FIXTURES / fixture["file"]).read_bytes()
        if actual != expected:
            failures.append(
                f"{fixture['id']}: retained bytes are not the declared derivation of "
                f"{declared['source']['path']}"
            )
    return failures


def historical_records() -> list[Path]:
    records = sorted(EVIDENCE.glob("pgm-01-*/manifest.json"))
    if not records:
        raise CompatibilityError("no historical PGM-01 record was found to read")
    return records


def superseded_records() -> dict[str, list[str]]:
    """Which historical records a retained correction says something about.

    This reports; it does not gate. A record an append-only correction names is
    `suspect` support for any current claim — a fourth thing, distinct from
    passed, failed, and unavailable — and saying so is not the same as the
    deleted verifier's refusal to let it back a decision. Nothing here decides
    anything; a reader is told which records carry a correction against them.
    """
    corrections: dict[str, list[str]] = {}
    for path in sorted((EVIDENCE / "corrections").glob("COR-*.json")):
        try:
            record = json.loads(path.read_bytes())
        except json.JSONDecodeError as error:
            raise CompatibilityError(f"retained correction {path.name} is unreadable: {error}")
        for claim in record.get("affectedClaims", []):
            corrections.setdefault(claim["record"], []).append(record["recordId"])
    return corrections


def evaluate(
    mapper: Callable[..., dict[str, Any]],
    *,
    bind_expected_digest: bool = True,
) -> list[dict[str, Any]]:
    """Map every real record and every declared fixture, and report each case.

    `bind_expected_digest` exists for the mutation probes. Binding the digest is
    what makes a tampered record report as tampered rather than as a readable
    one, so a probe that unbinds it must be seen to turn the tamper case red.
    """
    declared = load_expectations()
    corrections = superseded_records()
    cases: list[dict[str, Any]] = []

    for record in historical_records():
        raw = record.read_bytes()
        view = mapper(raw)
        relative = record.relative_to(ROOT).as_posix()
        superseding = corrections.get(record.parent.name, [])
        cases.append(
            {
                "case": record.parent.name,
                "kind": "historical",
                "support_status": "suspect" if superseding else "uncorrected",
                "superseded_by": superseding,
                "source": relative,
                "source_digest": sha256(raw),
                "outcome": view["outcome"],
                "expected_outcome": "lossy",
                "mapped_states": sorted(
                    {
                        mapping["value"]
                        for mapping in view["mappings"]
                        if mapping["target_field"] == "state"
                    }
                ),
                "mappings": len(view["mappings"]),
                "unmapped_fields": len(view["unmapped_fields"]),
                "digest_preserved": view["source_digest"] == sha256(raw),
            }
        )

    for fixture in declared["cases"]:
        # A declared case may point at a retained fixture or at a real file in
        # the repository. The second form is how a record that is not a PGM-01
        # evidence record at all gets shown being refused as one.
        base = ROOT if fixture.get("root") == "repository" else FIXTURES
        path = base / fixture["file"]
        try:
            raw = path.read_bytes()
        except OSError as error:
            raise CompatibilityError(f"declared fixture {fixture['file']} is unreadable: {error}")
        expected_digest = None
        if fixture.get("bind_digest") is not None and bind_expected_digest:
            expected_digest = fixture["bind_digest"]
        view = mapper(raw, expected_digest=expected_digest)
        cases.append(
            {
                "case": fixture["id"],
                "kind": fixture["kind"],
                "source": path.relative_to(ROOT).as_posix(),
                "source_digest": sha256(raw),
                "outcome": view["outcome"],
                "expected_outcome": fixture["expected_outcome"],
                "mapped_states": sorted(
                    {
                        mapping["value"]
                        for mapping in view["mappings"]
                        if mapping["target_field"] == "state"
                    }
                ),
                "mappings": len(view["mappings"]),
                "unmapped_fields": len(view["unmapped_fields"]),
                "digest_preserved": view["source_digest"] == sha256(raw),
                "requires_states": fixture.get("requires_states", []),
                "why": fixture["why"],
            }
        )

    for case in cases:
        required = set(case.get("requires_states", []))
        case["matched"] = (
            case["outcome"] == case["expected_outcome"]
            and case["digest_preserved"]
            and required <= set(case["mapped_states"])
        )
    return cases


def observed_states(cases: list[dict[str, Any]]) -> set[str]:
    states: set[str] = set()
    for case in cases:
        states.update(case["mapped_states"])
    return states


def census(mapper: Callable[..., dict[str, Any]], **kwargs: Any) -> dict[str, Any]:
    before = evidence_census()
    drifted_derivations = verify_derivations(load_expectations())
    cases = evaluate(mapper, **kwargs)
    after = evidence_census()
    moved = sorted(
        path
        for path in set(before) | set(after)
        if before.get(path) != after.get(path)
    )
    states = observed_states(cases)
    missing_states = sorted(set(REQUIRED_STATES) - states)
    suspect = sorted(
        case["case"] for case in cases if case.get("support_status") == "suspect"
    )
    return {
        "suspect_records": suspect,
        "suspect_demonstrated": bool(suspect),
        "schemaVersion": REPORT_VERSION,
        "mapping_version": "engineering-assurance.pgm01-compatibility-view/v1",
        "cases": cases,
        "evidence_files_read": len(before),
        "evidence_bytes_moved": moved,
        "drifted_derivations": drifted_derivations,
        "observed_states": sorted(states),
        "missing_required_states": missing_states,
        "matched": all(case["matched"] for case in cases)
        and not moved
        and not missing_states
        and not drifted_derivations
        and bool(suspect),
    }


def run_mutation_probes(mapper: Callable[..., dict[str, Any]]) -> dict[str, Any]:
    """Weaken each load-bearing check in turn and require the census to notice.

    A gate nobody has seen red is a gate nobody has tested. Each probe removes
    exactly one thing this census depends on and records whether the census
    still passed — and a probe that leaves it green is the failure being looked
    for, not a convenience.
    """
    probes: list[dict[str, Any]] = []

    unbound = census(mapper, bind_expected_digest=False)
    probes.append(
        {
            "name": "unbind-tamper-digest",
            "removes": "the expected-digest binding that makes an altered record report as tampered",
            "detected": not unbound["matched"],
        }
    )

    def collapsing_mapper(raw: bytes, **kwargs: Any) -> dict[str, Any]:
        view = mapper(raw, **kwargs)
        for mapping in view["mappings"]:
            if mapping["target_field"] == "state" and mapping["value"] in {
                "inconclusive",
                "skipped",
                "unavailable",
            }:
                mapping["value"] = "passed"
        return view

    collapsed = census(collapsing_mapper)
    probes.append(
        {
            "name": "collapse-non-success-states",
            "removes": "the distinctness of inconclusive, skipped, and unavailable from passed",
            "detected": not collapsed["matched"],
        }
    )

    def readable_mapper(raw: bytes, **kwargs: Any) -> dict[str, Any]:
        view = mapper(raw, **kwargs)
        if view["outcome"] in {"unreadable", "incompatible"}:
            view["outcome"] = "lossy"
        return view

    repaired = census(readable_mapper)
    probes.append(
        {
            "name": "repair-unreadable-outcome",
            "removes": "the refusal to turn an unreadable or unsupported record into a readable one",
            "detected": not repaired["matched"],
        }
    )

    def drifting_mapper(raw: bytes, **kwargs: Any) -> dict[str, Any]:
        view = mapper(raw, **kwargs)
        view["source_digest"] = "0" * 64
        return view

    drifted = census(drifting_mapper)
    probes.append(
        {
            "name": "drop-source-identity",
            "removes": "the check that a mapped view still carries its source's digest",
            "detected": not drifted["matched"],
        }
    )

    forged = copy.deepcopy(load_expectations())
    for fixture in forged["cases"]:
        if "derivation" in fixture and "pointer" in fixture["derivation"]:
            fixture["derivation"]["to"] = "a value nobody derived"
            break
    probes.append(
        {
            "name": "forge-declared-derivation",
            "removes": "the re-derivation that binds a constructed fixture to the real record it claims to come from",
            "detected": bool(verify_derivations(forged)),
        }
    )

    unlocked = copy.deepcopy(load_expectations())
    unlocked["source"]["sha256"] = "0" * 64
    probes.append(
        {
            "name": "unlock-source-record",
            "removes": "the digest that fixes which real record the fixtures are derived from",
            "detected": bool(verify_derivations(unlocked)),
        }
    )

    return {
        "schemaVersion": "quire-contract-ir.pgm01-compatibility-mutation-report/v1",
        "probes": probes,
        "detected": all(probe["detected"] for probe in probes),
    }


def print_census(report: dict[str, Any]) -> None:
    matched = sum(case["matched"] for case in report["cases"])
    print(
        f"PGM-01 compatibility census: {matched}/{len(report['cases'])} cases "
        f"matched their declared outcome"
    )
    for case in report["cases"]:
        states = ",".join(case["mapped_states"]) or "-"
        flag = " " if case["matched"] else "!"
        support = case.get("support_status", "")
        print(
            f" {flag} {case['case']:<44} {case['outcome']:<13} "
            f"states={states:<40} unmapped={case['unmapped_fields']} {support}"
        )
    print(
        f"suspect support (named by a retained correction): "
        f"{', '.join(report['suspect_records']) or 'none'}"
    )
    print(
        f"evidence: {report['evidence_files_read']} files read, "
        f"{len(report['evidence_bytes_moved'])} bytes moved"
    )
    for drift in report["drifted_derivations"]:
        print(f"derivation drift: {drift}")
    if report["missing_required_states"]:
        print(f"missing demonstrated states: {', '.join(report['missing_required_states'])}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit the report as JSON")
    parser.add_argument(
        "--mutation-probes",
        action="store_true",
        help="prove a weakened census fails",
    )
    args = parser.parse_args()
    try:
        mapper = load_mapper()
        if args.mutation_probes:
            report = run_mutation_probes(mapper)
            if args.json:
                print(json.dumps(report, indent=2, sort_keys=True))
            else:
                detected = sum(probe["detected"] for probe in report["probes"])
                print(
                    f"PGM-01 compatibility mutation probes: "
                    f"{detected}/{len(report['probes'])} detected"
                )
                for probe in report["probes"]:
                    mark = "detected" if probe["detected"] else "ESCAPED"
                    print(f"  {probe['name']:<32} {mark}: {probe['removes']}")
            return 0 if report["detected"] else 1
        report = census(mapper)
        if args.json:
            print(json.dumps(report, indent=2, sort_keys=True))
        else:
            print_census(report)
        return 0 if report["matched"] else 1
    except CompatibilityError as error:
        print(f"compatibility view error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
