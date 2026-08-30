#!/usr/bin/env python3
"""Validate the PGM-01 v1 schema contract and conformance corpus."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import re
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parent.parent
SCHEMA_PATH = ROOT / "schemas" / "derivation-evidence-envelope-v1.schema.json"
MANIFEST_PATH = ROOT / "corpus" / "governance" / "manifest.json"
SCHEMA_VERSION = "quire.derivation-evidence/v1"
HEX_64 = re.compile(r"^[0-9a-f]{64}$")
REVISION = re.compile(r"^[0-9a-f]{40,64}$")
REVERSE_DNS = re.compile(r"^[a-z0-9]+(?:[.-][a-z0-9]+)+$")
REVIEWER = re.compile(r"^@[A-Za-z0-9-]+$")
RESULTS = {
    "conclusive",
    "inconclusive",
    "unsupported",
    "rejected",
    "timed-out",
    "pending",
    "error",
}


def issue(code: str, path: str, message: str) -> dict[str, str]:
    return {"code": code, "path": path, "message": message}


def require_object(
    value: Any, path: str, required: set[str], allowed: set[str], errors: list[dict[str, str]]
) -> dict[str, Any] | None:
    if not isinstance(value, dict):
        errors.append(issue("INVALID_TYPE", path, "expected object"))
        return None
    for name in sorted(required - value.keys()):
        code = "MISSING_BACKEND" if path == "$" and name == "backend" else "MISSING_FIELD"
        errors.append(issue(code, f"{path}.{name}", "required field is absent"))
    for name in sorted(value.keys() - allowed):
        errors.append(issue("UNKNOWN_FIELD", f"{path}.{name}", "field is not defined by v1"))
    return value


def require_string(value: Any, path: str, errors: list[dict[str, str]]) -> None:
    if not isinstance(value, str) or not value:
        errors.append(issue("INVALID_STRING", path, "expected a non-empty string"))


def validate_digest(value: Any, path: str, errors: list[dict[str, str]]) -> None:
    obj = require_object(value, path, {"algorithm", "value"}, {"algorithm", "value"}, errors)
    if obj is None:
        return
    if obj.get("algorithm") != "sha256" or not isinstance(obj.get("value"), str) or not HEX_64.fullmatch(obj["value"]):
        errors.append(issue("INVALID_DIGEST", path, "expected sha256 and 64 lowercase hexadecimal digits"))


def validate_schema_identity(value: Any, path: str, errors: list[dict[str, str]]) -> None:
    obj = require_object(value, path, {"id", "version", "digest"}, {"id", "version", "digest"}, errors)
    if obj is None:
        return
    require_string(obj.get("id"), f"{path}.id", errors)
    if not isinstance(obj.get("version"), str) or not re.fullmatch(r"v[1-9][0-9]*", obj["version"]):
        errors.append(issue("INVALID_SCHEMA_IDENTITY", f"{path}.version", "expected an explicit vN major"))
    if "digest" in obj:
        validate_digest(obj["digest"], f"{path}.digest", errors)


def validate_artifact(value: Any, path: str, errors: list[dict[str, str]]) -> None:
    fields = {"role", "uri", "mediaType", "schema", "contentDigest"}
    obj = require_object(value, path, fields, fields, errors)
    if obj is None:
        return
    for name in ("role", "uri", "mediaType"):
        require_string(obj.get(name), f"{path}.{name}", errors)
    if "schema" in obj:
        validate_schema_identity(obj["schema"], f"{path}.schema", errors)
    if "contentDigest" in obj:
        validate_digest(obj["contentDigest"], f"{path}.contentDigest", errors)


def validate_envelope(document: Any) -> list[dict[str, str]]:
    errors: list[dict[str, str]] = []
    if not isinstance(document, dict):
        return [issue("INVALID_TYPE", "$", "expected object")]
    if document.get("schemaVersion") != SCHEMA_VERSION:
        return [issue("UNSUPPORTED_SCHEMA", "$.schemaVersion", f"expected {SCHEMA_VERSION}")]

    required = {
        "schemaVersion", "recordId", "recordedAt", "producer", "inputs", "backend", "outputs",
        "parametersDigest", "environment", "provenance", "result",
    }
    top = require_object(document, "$", required, required | {"extensions"}, errors)
    assert top is not None
    require_string(top.get("recordId"), "$.recordId", errors)
    timestamp = top.get("recordedAt")
    try:
        if not isinstance(timestamp, str):
            raise ValueError
        dt.datetime.fromisoformat(timestamp.replace("Z", "+00:00"))
    except ValueError:
        errors.append(issue("INVALID_TIMESTAMP", "$.recordedAt", "expected an RFC 3339 date-time"))

    producer_fields = {"name", "version", "sourceRevision", "executableDigest", "invocation"}
    producer = require_object(top.get("producer"), "$.producer", producer_fields, producer_fields, errors)
    if producer is not None:
        require_string(producer.get("name"), "$.producer.name", errors)
        require_string(producer.get("version"), "$.producer.version", errors)
        if not isinstance(producer.get("sourceRevision"), str) or not REVISION.fullmatch(producer["sourceRevision"]):
            errors.append(issue("INVALID_REVISION", "$.producer.sourceRevision", "expected a 40-64 digit lowercase revision"))
        if "executableDigest" in producer:
            validate_digest(producer["executableDigest"], "$.producer.executableDigest", errors)
        invocation = producer.get("invocation")
        if not isinstance(invocation, list) or not invocation or not all(isinstance(part, str) for part in invocation):
            errors.append(issue("INVALID_INVOCATION", "$.producer.invocation", "expected a non-empty string array"))

    for collection_name in ("inputs", "outputs"):
        values = top.get(collection_name)
        if not isinstance(values, list) or not values:
            errors.append(issue("INVALID_ARTIFACTS", f"$.{collection_name}", "expected a non-empty array"))
        else:
            for index, value in enumerate(values):
                validate_artifact(value, f"$.{collection_name}[{index}]", errors)

    if "backend" in top:
        backend = top["backend"]
        if isinstance(backend, dict) and backend.get("kind") == "none":
            fields = {"kind", "reason"}
            obj = require_object(backend, "$.backend", fields, fields, errors)
            if obj is not None:
                require_string(obj.get("reason"), "$.backend.reason", errors)
        else:
            fields = {"kind", "name", "version", "executableDigest", "configurationDigest"}
            obj = require_object(backend, "$.backend", fields, fields, errors)
            if obj is not None:
                if obj.get("kind") not in {"engine", "tool"}:
                    errors.append(issue("INVALID_BACKEND", "$.backend.kind", "expected none, engine, or tool"))
                require_string(obj.get("name"), "$.backend.name", errors)
                require_string(obj.get("version"), "$.backend.version", errors)
                for name in ("executableDigest", "configurationDigest"):
                    if name in obj:
                        validate_digest(obj[name], f"$.backend.{name}", errors)

    if "parametersDigest" in top:
        validate_digest(top["parametersDigest"], "$.parametersDigest", errors)

    environment_fields = {"targetTriple", "operatingSystem", "toolchain", "dependenciesDigest"}
    environment = require_object(top.get("environment"), "$.environment", environment_fields, environment_fields, errors)
    if environment is not None:
        for name in ("targetTriple", "operatingSystem", "toolchain"):
            require_string(environment.get(name), f"$.environment.{name}", errors)
        if "dependenciesDigest" in environment:
            validate_digest(environment["dependenciesDigest"], "$.environment.dependenciesDigest", errors)

    provenance_fields = {"repository", "sourceRevision", "candidateRevision", "contributionMethod", "reviewers"}
    provenance = require_object(top.get("provenance"), "$.provenance", provenance_fields, provenance_fields, errors)
    if provenance is not None:
        require_string(provenance.get("repository"), "$.provenance.repository", errors)
        for name in ("sourceRevision", "candidateRevision"):
            value = provenance.get(name)
            if not isinstance(value, str) or not REVISION.fullmatch(value):
                errors.append(issue("INVALID_REVISION", f"$.provenance.{name}", "expected a 40-64 digit lowercase revision"))
        if provenance.get("contributionMethod") not in {"human", "agent-assisted", "generated", "mixed"}:
            errors.append(issue("INVALID_PROVENANCE", "$.provenance.contributionMethod", "unknown method"))
        reviewers = provenance.get("reviewers")
        if not isinstance(reviewers, list) or not reviewers or not all(isinstance(name, str) and REVIEWER.fullmatch(name) for name in reviewers):
            errors.append(issue("INVALID_REVIEWERS", "$.provenance.reviewers", "expected at least one @owner"))

    result_fields = {"status", "summary", "requirementRefs"}
    result = require_object(top.get("result"), "$.result", result_fields, result_fields, errors)
    if result is not None:
        if result.get("status") not in RESULTS:
            errors.append(issue("INVALID_RESULT", "$.result.status", "unknown terminal state"))
        require_string(result.get("summary"), "$.result.summary", errors)
        refs = result.get("requirementRefs")
        if not isinstance(refs, list) or not all(isinstance(ref, str) and ref for ref in refs):
            errors.append(issue("INVALID_REQUIREMENT_REFS", "$.result.requirementRefs", "expected a string array"))

    extensions = top.get("extensions", {})
    if not isinstance(extensions, dict) or not all(REVERSE_DNS.fullmatch(name) for name in extensions):
        errors.append(issue("INVALID_EXTENSION", "$.extensions", "keys must be reverse-DNS names"))
    return errors


def load_json(path: Path) -> Any:
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def check_schema_contract() -> None:
    schema = load_json(SCHEMA_PATH)
    expected_required = {
        "schemaVersion", "recordId", "recordedAt", "producer", "inputs", "backend", "outputs",
        "parametersDigest", "environment", "provenance", "result",
    }
    if schema.get("additionalProperties") is not False:
        raise ValueError("v1 schema must reject unknown top-level fields")
    if set(schema.get("required", [])) != expected_required:
        raise ValueError("v1 schema required fields diverge from PGM-01")
    if schema.get("properties", {}).get("schemaVersion", {}).get("const") != SCHEMA_VERSION:
        raise ValueError("v1 schema identity diverges from PGM-01")
    digest = schema.get("definitions", {}).get("digest", {})
    if digest.get("properties", {}).get("algorithm", {}).get("const") != "sha256":
        raise ValueError("v1 digest algorithm must be sha256")


def validate_manifest() -> dict[str, Any]:
    check_schema_contract()
    manifest = load_json(MANIFEST_PATH)
    if manifest.get("schemaVersion") != "quire.governance-corpus/v1":
        raise ValueError("unsupported corpus manifest")
    results = []
    listed: set[Path] = set()
    for case in manifest.get("cases", []):
        path = MANIFEST_PATH.parent / case["path"]
        listed.add(path.resolve())
        errors = validate_envelope(load_json(path))
        actual_valid = not errors
        matched = actual_valid is case["valid"]
        expected_code = case.get("expectedCode")
        if expected_code is not None:
            matched = matched and expected_code in {error["code"] for error in errors}
        results.append({"path": case["path"], "valid": actual_valid, "errors": errors, "matched": matched})
    available = {path.resolve() for path in MANIFEST_PATH.parent.glob("*/*.json")}
    if available != listed:
        raise ValueError("corpus manifest must list every fixture exactly once")
    return {
        "schemaVersion": "quire.governance-validation-report/v1",
        "schema": str(SCHEMA_PATH.relative_to(ROOT)),
        "cases": results,
        "matched": all(result["matched"] for result in results),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", action="store_true", help="emit the complete report as JSON")
    parser.add_argument("--fixture", type=Path, help="validate one envelope instead of the corpus")
    args = parser.parse_args()
    try:
        check_schema_contract()
        if args.fixture:
            errors = validate_envelope(load_json(args.fixture))
            print(json.dumps({"valid": not errors, "errors": errors}, indent=2, sort_keys=True))
            return 0 if not errors else 1
        report = validate_manifest()
    except (OSError, KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"governance validation error: {error}", file=sys.stderr)
        return 2
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        matched = sum(case["matched"] for case in report["cases"])
        print(f"PGM-01 governance corpus: {matched}/{len(report['cases'])} cases matched")
        for case in report["cases"]:
            state = "valid" if case["valid"] else ",".join(error["code"] for error in case["errors"])
            print(f"  {case['path']}: {state}")
    return 0 if report["matched"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
