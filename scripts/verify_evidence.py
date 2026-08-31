#!/usr/bin/env python3
"""Verify retained PGM-01 outputs and the exact committed candidate tree."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any, Callable

from jsonschema import Draft7Validator, FormatChecker


ROOT = Path(__file__).resolve().parent.parent
DIGEST = re.compile(r"^[0-9a-f]{64}$")
REVISION = re.compile(r"^[0-9a-f]{40}$")
CHECKSUM_LINE = re.compile(r"^([0-9a-f]{64})  (.+)$")
BlobReader = Callable[[str, Path], bytes]
WorktreeReader = Callable[[Path], bytes]
InputSetReader = Callable[[], set[Path]]
EVIDENCE_SCHEMA = Path("schemas/pgm01-evidence-v1.schema.json")
CORRECTION_SCHEMA = Path("schemas/evidence-correction-v1.schema.json")


class EvidenceError(ValueError):
    """Raised when retained evidence is inconsistent or stale."""


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def safe_relative_path(value: str) -> Path:
    path = Path(value)
    if path.is_absolute() or not path.parts or ".." in path.parts:
        raise EvidenceError(f"unsafe evidence path: {value!r}")
    return path


def read_subject_blob(revision: str, path: Path) -> bytes:
    result = subprocess.run(
        ["git", "cat-file", "blob", f"{revision}:{path.as_posix()}"],
        cwd=ROOT,
        check=False,
        capture_output=True,
    )
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        raise EvidenceError(f"cannot read {path} at {revision}: {detail}")
    return result.stdout


def read_head_inputs() -> set[Path]:
    result = subprocess.run(
        ["git", "ls-tree", "-r", "-z", "--name-only", "HEAD"],
        cwd=ROOT,
        check=False,
        capture_output=True,
    )
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        raise EvidenceError(f"cannot enumerate current HEAD tree: {detail}")
    paths = set()
    for raw_path in result.stdout.split(b"\0"):
        if not raw_path:
            continue
        path = safe_relative_path(raw_path.decode("utf-8"))
        if path.parts[0] != "evidence":
            paths.add(path)
    if not paths:
        raise EvidenceError("current HEAD tree contains no non-evidence inputs")
    return paths


def read_worktree_inputs() -> set[Path]:
    result = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=ROOT,
        check=False,
        capture_output=True,
    )
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        raise EvidenceError(f"cannot enumerate current worktree: {detail}")
    paths = set()
    for raw_path in result.stdout.split(b"\0"):
        if not raw_path:
            continue
        path = safe_relative_path(raw_path.decode("utf-8"))
        if path.parts[0] != "evidence":
            paths.add(path)
    return paths


def read_worktree_file(path: Path) -> bytes:
    try:
        return (ROOT / path).read_bytes()
    except OSError as error:
        raise EvidenceError(f"cannot read current input {path}: {error}") from error


# Implements: FR-009
def verify_input_checksums(
    manifest: dict[str, Any],
    head_reader: BlobReader = read_subject_blob,
    worktree_reader: WorktreeReader = read_worktree_file,
    input_set_reader: InputSetReader = read_head_inputs,
    worktree_set_reader: InputSetReader = read_worktree_inputs,
) -> int:
    revision = manifest.get("subjectRevision")
    if not isinstance(revision, str) or not REVISION.fullmatch(revision):
        raise EvidenceError("subjectRevision must be a full lowercase Git revision")
    checksums = manifest.get("inputChecksums")
    if not isinstance(checksums, dict) or not checksums:
        raise EvidenceError("inputChecksums must be a non-empty object")
    normalized_checksums: dict[Path, str] = {}
    for raw_path, expected in checksums.items():
        if not isinstance(raw_path, str) or not isinstance(expected, str):
            raise EvidenceError("inputChecksums entries must map paths to digests")
        if not DIGEST.fullmatch(expected):
            raise EvidenceError(f"invalid recorded input digest for {raw_path}")
        path = safe_relative_path(raw_path)
        if path in normalized_checksums:
            raise EvidenceError(f"duplicate normalized input path: {path}")
        normalized_checksums[path] = expected
    required_inputs = input_set_reader()
    if set(normalized_checksums) != required_inputs:
        missing = sorted(str(path) for path in required_inputs - set(normalized_checksums))
        extra = sorted(str(path) for path in set(normalized_checksums) - required_inputs)
        raise EvidenceError(
            f"inputChecksums do not cover the current HEAD tree; "
            f"missing={missing}, extra={extra}"
        )
    worktree_inputs = worktree_set_reader()
    if worktree_inputs != required_inputs:
        missing = sorted(str(path) for path in required_inputs - worktree_inputs)
        untracked = sorted(str(path) for path in worktree_inputs - required_inputs)
        raise EvidenceError(
            f"worktree input set differs from current HEAD; "
            f"missing={missing}, untracked={untracked}"
        )
    for path, expected in normalized_checksums.items():
        head_actual = sha256(head_reader("HEAD", path))
        if head_actual != expected:
            raise EvidenceError(
                f"HEAD input checksum mismatch for {path}: "
                f"expected {expected}, got {head_actual}"
            )
        worktree_actual = sha256(worktree_reader(path))
        if worktree_actual != expected:
            raise EvidenceError(
                f"current input checksum mismatch for {path}: "
                f"expected {expected}, got {worktree_actual}"
            )
    return len(normalized_checksums)


def validate_manifest_schema(manifest: dict[str, Any]) -> None:
    identity = manifest.get("schemaIdentity")
    if not isinstance(identity, dict):
        raise EvidenceError("schemaIdentity must be an object")
    if identity.get("path") != EVIDENCE_SCHEMA.as_posix():
        raise EvidenceError(f"schemaIdentity.path must be {EVIDENCE_SCHEMA}")
    try:
        schema_bytes = (ROOT / EVIDENCE_SCHEMA).read_bytes()
        schema = json.loads(schema_bytes)
    except (OSError, json.JSONDecodeError) as error:
        raise EvidenceError(f"cannot load evidence schema: {error}") from error
    if identity.get("sha256") != sha256(schema_bytes):
        raise EvidenceError("evidence schema identity digest mismatch")
    try:
        Draft7Validator.check_schema(schema)
    except Exception as error:
        raise EvidenceError(f"invalid evidence schema: {error}") from error
    errors = sorted(
        Draft7Validator(schema, format_checker=FormatChecker()).iter_errors(manifest),
        key=lambda error: (list(error.absolute_path), error.message),
    )
    if errors:
        first = errors[0]
        location = "/".join(str(part) for part in first.absolute_path) or "<root>"
        raise EvidenceError(f"evidence manifest schema violation at {location}: {first.message}")


def parse_external_checksum_lines(lines: list[str]) -> dict[Path, str]:
    entries: dict[Path, str] = {}
    for line in lines:
        match = CHECKSUM_LINE.fullmatch(line)
        if match is None:
            raise EvidenceError(f"invalid external checksum line: {line!r}")
        entry = safe_relative_path(match.group(2))
        if entry in entries:
            raise EvidenceError(f"duplicate external checksum entry: {entry}")
        entries[entry] = match.group(1)
    if not entries:
        raise EvidenceError("external checksum file must not be empty")
    return entries


def parse_external_checksums(path: Path) -> dict[Path, str]:
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as error:
        raise EvidenceError(f"cannot read external checksum file {path}: {error}") from error
    return parse_external_checksum_lines(lines)


# Implements: FR-009
def load_evidence_corrections() -> dict[str, list[str]]:
    correction_paths = sorted((ROOT / "evidence/corrections").glob("COR-*.json"))
    if not correction_paths:
        return {}
    try:
        schema = json.loads((ROOT / CORRECTION_SCHEMA).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise EvidenceError(f"cannot load evidence correction schema: {error}") from error
    try:
        Draft7Validator.check_schema(schema)
    except Exception as error:
        raise EvidenceError(f"invalid evidence correction schema: {error}") from error

    corrected: dict[str, list[str]] = {}
    for path in correction_paths:
        relative = path.relative_to(ROOT)
        try:
            payload = path.read_bytes()
            correction = json.loads(payload)
        except (OSError, json.JSONDecodeError) as error:
            raise EvidenceError(f"cannot load evidence correction {relative}: {error}") from error
        errors = sorted(
            Draft7Validator(schema, format_checker=FormatChecker()).iter_errors(correction),
            key=lambda error: (list(error.absolute_path), error.message),
        )
        if errors:
            first = errors[0]
            location = "/".join(str(part) for part in first.absolute_path) or "<root>"
            raise EvidenceError(
                f"evidence correction schema violation in {relative} at "
                f"{location}: {first.message}"
            )
        record_id = correction["recordId"]
        if not path.name.startswith(f"{record_id}-"):
            raise EvidenceError(f"evidence correction filename does not match {record_id}")
        checksum_path = path.with_suffix(".sha256")
        entries = parse_external_checksums(checksum_path)
        if entries != {relative: sha256(payload)}:
            raise EvidenceError(f"evidence correction checksum mismatch: {relative}")
        for claim in correction["affectedClaims"]:
            affected = claim["record"]
            if not (ROOT / "evidence" / affected / "manifest.json").is_file():
                raise EvidenceError(
                    f"evidence correction {record_id} names unavailable record {affected}"
                )
            corrected.setdefault(affected, []).append(record_id)
    return corrected


def verify_output_entries(
    manifest: dict[str, Any],
    checksum_relative: Path,
    recorded_entries: dict[Path, str],
    current_reader: WorktreeReader = read_worktree_file,
    head_reader: BlobReader = read_subject_blob,
) -> int:
    outputs = manifest.get("outputs")
    if not isinstance(outputs, list) or not all(isinstance(item, str) for item in outputs):
        raise EvidenceError("outputs must be an array of paths")
    output_paths = [safe_relative_path(item) for item in outputs]
    if len(set(output_paths)) != len(output_paths):
        raise EvidenceError("outputs must not contain duplicate paths")
    output_set = set(output_paths)
    if checksum_relative not in output_set:
        raise EvidenceError("outputs must name the external checksum file")
    expected_entries = output_set - {checksum_relative}
    if set(recorded_entries) != expected_entries:
        raise EvidenceError("external checksum entries do not match retained outputs")
    for path in output_set:
        current = current_reader(path)
        if current != head_reader("HEAD", path):
            raise EvidenceError(f"retained output differs from the current HEAD blob: {path}")
        if path in recorded_entries:
            expected = recorded_entries[path]
            actual = sha256(current)
            if actual != expected:
                raise EvidenceError(
                    f"retained output checksum mismatch for {path}: "
                    f"expected {expected}, got {actual}"
                )
    return len(output_set)


def verify_retained_outputs(
    manifest: dict[str, Any], checksum_file: Path
) -> int:
    try:
        checksum_relative = checksum_file.relative_to(ROOT)
    except ValueError as error:
        raise EvidenceError("external checksum file must be inside the repository") from error
    recorded_entries = parse_external_checksums(checksum_file)
    return verify_output_entries(manifest, checksum_relative, recorded_entries)


def verify_record(record: Path) -> tuple[int, int, str]:
    record = record.resolve()
    if not record.is_relative_to(ROOT):
        raise EvidenceError("evidence record must be inside the repository")
    manifest_path = record / "manifest.json"
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise EvidenceError(f"cannot load evidence manifest {manifest_path}: {error}") from error
    if not isinstance(manifest, dict):
        raise EvidenceError("evidence manifest must be an object")
    validate_manifest_schema(manifest)
    revision = manifest.get("subjectRevision")
    if not isinstance(revision, str) or not REVISION.fullmatch(revision):
        raise EvidenceError("subjectRevision must be a full lowercase Git revision")
    expected_record_id = f"pgm-01-{str(revision)[:7]}"
    if manifest.get("recordId") != record.name or record.name != expected_record_id:
        raise EvidenceError("record path, recordId, and subjectRevision do not agree")
    corrections = load_evidence_corrections()
    if record.name in corrections:
        correction_ids = ", ".join(corrections[record.name])
        raise EvidenceError(
            f"record has an append-only correction and cannot support a current "
            f"decision: {correction_ids}"
        )
    checksum_file = record.parent / f"{record.name}.sha256"
    output_count = verify_retained_outputs(manifest, checksum_file)
    input_count = verify_input_checksums(manifest)
    return output_count, input_count, revision


def choose_unique_match(
    matches: list[tuple[Path, int, int]],
) -> tuple[Path, int, int]:
    if len(matches) == 1:
        return matches[0]
    if len(matches) > 1:
        names = ", ".join(match[0].name for match in matches)
        raise EvidenceError(f"multiple current evidence records match: {names}")
    raise EvidenceError("no evidence record matches the current candidate")


def select_current_record() -> tuple[Path, int, int]:
    matches: list[tuple[Path, int, int]] = []
    failures = []
    for manifest_path in sorted((ROOT / "evidence").glob("pgm-01-*/manifest.json")):
        record = manifest_path.parent
        try:
            outputs, inputs, _revision = verify_record(record)
        except EvidenceError as error:
            failures.append(f"{record.name}: {error}")
        else:
            matches.append((record, outputs, inputs))
    if not matches:
        detail = "; ".join(failures) if failures else "no records found"
        raise EvidenceError(f"no evidence record matches the current candidate: {detail}")
    return choose_unique_match(matches)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "record",
        nargs="?",
        type=Path,
        help="revision-scoped evidence directory; omit to discover the unique current record",
    )
    args = parser.parse_args()
    try:
        if args.record is None:
            record, outputs, inputs = select_current_record()
        else:
            record = args.record
            outputs, inputs, _revision = verify_record(record)
    except EvidenceError as error:
        print(f"evidence verification error: {error}", file=sys.stderr)
        return 1
    print(
        f"PGM-01 evidence {record.name}: {outputs}/{outputs} retained outputs and "
        f"{inputs}/{inputs} HEAD/worktree inputs matched; "
        f"{sum(len(items) for items in load_evidence_corrections().values())} "
        "correction claim(s) enforced"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
