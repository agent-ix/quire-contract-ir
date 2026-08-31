#!/usr/bin/env python3
"""Validate PGM-01 fixtures with the published Draft 7 schema itself."""

from __future__ import annotations

import argparse
import copy
import importlib.metadata
import json
import platform
import re
import sys
from pathlib import Path
from typing import Any, Callable

import jsonschema
from jsonschema import Draft7Validator, FormatChecker


ROOT = Path(__file__).resolve().parent.parent
SCHEMA_PATH = ROOT / "schemas" / "derivation-evidence-envelope-v1.schema.json"
MANIFEST_PATH = ROOT / "corpus" / "governance" / "manifest.json"
PYTHON_VERSION = (3, 10, 12)
JSONSCHEMA_VERSION = "3.2.0"
FORMAT_DEPENDENCIES = {
    "rfc3339-validator": "0.1.4",
    "rfc3986-validator": "0.1.1",
}
REQUIRED_FORMATS = {"date-time", "uri", "uri-reference"}
FORMAT_CHECKER = FormatChecker()
MISSING_CODES = {
    "producer": "MISSING_PRODUCER",
    "inputs": "MISSING_INPUTS",
    "backend": "MISSING_BACKEND",
    "outputs": "MISSING_OUTPUTS",
    "schema": "MISSING_SCHEMA_IDENTITY",
}
REQUIRED_PROPERTY = re.compile(r"^'([^']+)' is a required property$")


def load_json(path: Path) -> Any:
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def check_runtime() -> None:
    if sys.version_info[:3] != PYTHON_VERSION:
        expected = ".".join(str(part) for part in PYTHON_VERSION)
        raise RuntimeError(f"expected Python {expected}, found {platform.python_version()}")
    if jsonschema.__version__ != JSONSCHEMA_VERSION:
        raise RuntimeError(
            f"expected jsonschema {JSONSCHEMA_VERSION}, found {jsonschema.__version__}"
        )
    for package, expected in FORMAT_DEPENDENCIES.items():
        try:
            actual = importlib.metadata.version(package)
        except importlib.metadata.PackageNotFoundError as error:
            raise RuntimeError(f"required format validator {package} is not installed") from error
        if actual != expected:
            raise RuntimeError(f"expected {package} {expected}, found {actual}")
    missing_formats = REQUIRED_FORMATS.difference(FORMAT_CHECKER.checkers)
    if missing_formats:
        missing = ", ".join(sorted(missing_formats))
        raise RuntimeError(f"required JSON Schema format checkers are unavailable: {missing}")


def load_schema() -> dict[str, Any]:
    schema = load_json(SCHEMA_PATH)
    Draft7Validator.check_schema(schema)
    return schema


# Implements: FR-008
def classify_error(error: jsonschema.ValidationError) -> str:
    path = list(error.absolute_path)
    if error.validator == "const" and path == ["schemaVersion"]:
        return "UNSUPPORTED_SCHEMA"
    if error.validator == "required":
        match = REQUIRED_PROPERTY.fullmatch(error.message)
        if match and match.group(1) in MISSING_CODES:
            return MISSING_CODES[match.group(1)]
    if error.validator == "pattern" and path and path[-1] == "value":
        return "INVALID_DIGEST"
    return "SCHEMA_VIOLATION"


# Implements: FR-008
def validate_envelope(
    schema: dict[str, Any], document: Any
) -> list[dict[str, Any]]:
    validator = Draft7Validator(schema, format_checker=FORMAT_CHECKER)
    errors = sorted(
        validator.iter_errors(document),
        key=lambda error: (list(error.absolute_path), error.message),
    )
    return [
        {
            "code": classify_error(error),
            "path": "$"
            + "".join(
                f"[{part}]" if isinstance(part, int) else f".{part}"
                for part in error.absolute_path
            ),
            "message": error.message,
        }
        for error in errors
    ]


def validate_manifest(schema: dict[str, Any]) -> dict[str, Any]:
    manifest = load_json(MANIFEST_PATH)
    if manifest.get("schemaVersion") != "quire.governance-corpus/v1":
        raise ValueError("unsupported corpus manifest")
    results = []
    listed: set[Path] = set()
    for case in manifest.get("cases", []):
        path = MANIFEST_PATH.parent / case["path"]
        listed.add(path.resolve())
        errors = validate_envelope(schema, load_json(path))
        actual_valid = not errors
        matched = actual_valid is case["valid"]
        expected_code = case.get("expectedCode")
        if expected_code is not None:
            matched = matched and expected_code in {error["code"] for error in errors}
        results.append(
            {
                "path": case["path"],
                "valid": actual_valid,
                "errors": errors,
                "matched": matched,
            }
        )
    available = {path.resolve() for path in MANIFEST_PATH.parent.glob("*/*.json")}
    if available != listed:
        raise ValueError("corpus manifest must list every fixture exactly once")
    return {
        "schemaVersion": "quire.governance-validation-report/v1",
        "schema": str(SCHEMA_PATH.relative_to(ROOT)),
        "cases": results,
        "matched": all(result["matched"] for result in results),
    }


def replace_required(
    definition: str, required: list[str]
) -> Callable[[dict[str, Any]], None]:
    def mutate(schema: dict[str, Any]) -> None:
        schema["definitions"][definition]["required"] = required

    return mutate


def replace_document_value(
    path: tuple[str | int, ...], value: Any
) -> Callable[[dict[str, Any]], None]:
    def mutate(document: dict[str, Any]) -> None:
        parent: Any = document
        for segment in path[:-1]:
            parent = parent[segment]
        parent[path[-1]] = value

    return mutate


def run_mutation_probes(schema: dict[str, Any]) -> dict[str, Any]:
    schema_probes = {
        "producer-required": replace_required("producer", ["name"]),
        "backend-required": replace_required("backend", ["kind"]),
        "output-required": replace_required("artifactOutput", ["role"]),
        "provenance-required": replace_required("provenance", ["repository"]),
    }
    results = []
    for name, mutate in schema_probes.items():
        candidate = copy.deepcopy(schema)
        mutate(candidate)
        Draft7Validator.check_schema(candidate)
        report = validate_manifest(candidate)
        detected = not report["matched"]
        results.append({"name": name, "detected": detected})
    format_probes = {
        "invalid-recorded-at": replace_document_value(("recordedAt",), ""),
        "invalid-repository-uri": replace_document_value(
            ("provenance", "repository"), "foo"
        ),
        "invalid-artifact-uri": replace_document_value(("inputs", 0, "uri"), "foo bar"),
    }
    valid_fixture = load_json(MANIFEST_PATH.parent / "valid" / "generated-oracle.json")
    for name, mutate in format_probes.items():
        candidate = copy.deepcopy(valid_fixture)
        mutate(candidate)
        results.append({"name": name, "detected": bool(validate_envelope(schema, candidate))})
    return {
        "schemaVersion": "quire.governance-mutation-report/v1",
        "probes": results,
        "detected": all(result["detected"] for result in results),
    }


def print_report(report: dict[str, Any], as_json: bool) -> None:
    if as_json:
        print(json.dumps(report, indent=2, sort_keys=True))
        return
    if "cases" in report:
        matched = sum(case["matched"] for case in report["cases"])
        print(f"PGM-01 Draft 7 corpus: {matched}/{len(report['cases'])} cases matched")
        for case in report["cases"]:
            codes = sorted({error["code"] for error in case["errors"]})
            state = "valid" if case["valid"] else ",".join(codes)
            print(f"  {case['path']}: {state}")
    else:
        detected = sum(probe["detected"] for probe in report["probes"])
        print(f"PGM-01 schema mutation probes: {detected}/{len(report['probes'])} detected")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", action="store_true", help="emit the complete report as JSON")
    parser.add_argument("--fixture", type=Path, help="validate one envelope instead of the corpus")
    parser.add_argument("--check-runtime", action="store_true", help="verify the declared Python lane")
    parser.add_argument(
        "--mutation-probes",
        action="store_true",
        help="prove weakened schemas fail the corpus gate",
    )
    args = parser.parse_args()
    try:
        check_runtime()
        if args.check_runtime:
            print(
                f"PGM-01 Python lane: {platform.python_version()}, "
                f"jsonschema {jsonschema.__version__}; formats "
                f"{','.join(sorted(REQUIRED_FORMATS))}"
            )
            return 0
        schema = load_schema()
        if args.fixture:
            errors = validate_envelope(schema, load_json(args.fixture))
            print(json.dumps({"valid": not errors, "errors": errors}, sort_keys=True))
            return 0 if not errors else 1
        if args.mutation_probes:
            report = run_mutation_probes(schema)
            print_report(report, args.json)
            return 0 if report["detected"] else 1
        report = validate_manifest(schema)
        print_report(report, args.json)
        return 0 if report["matched"] else 1
    except (
        OSError,
        KeyError,
        TypeError,
        ValueError,
        RuntimeError,
        json.JSONDecodeError,
        jsonschema.SchemaError,
    ) as error:
        print(f"governance validation error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
