#!/usr/bin/env python3
"""Verify retained PGM-01 outputs and their exact subject inputs."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any, Callable


ROOT = Path(__file__).resolve().parent.parent
DIGEST = re.compile(r"^[0-9a-f]{64}$")
REVISION = re.compile(r"^[0-9a-f]{40}$")
CHECKSUM_LINE = re.compile(r"^([0-9a-f]{64})  (.+)$")
BlobReader = Callable[[str, Path], bytes]
WorktreeReader = Callable[[Path], bytes]


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


def read_worktree_file(path: Path) -> bytes:
    try:
        return (ROOT / path).read_bytes()
    except OSError as error:
        raise EvidenceError(f"cannot read current input {path}: {error}") from error


# Implements: FR-009
def verify_input_checksums(
    manifest: dict[str, Any],
    subject_reader: BlobReader = read_subject_blob,
    worktree_reader: WorktreeReader = read_worktree_file,
) -> int:
    revision = manifest.get("subjectRevision")
    if not isinstance(revision, str) or not REVISION.fullmatch(revision):
        raise EvidenceError("subjectRevision must be a full lowercase Git revision")
    checksums = manifest.get("inputChecksums")
    if not isinstance(checksums, dict) or not checksums:
        raise EvidenceError("inputChecksums must be a non-empty object")
    for raw_path, expected in checksums.items():
        if not isinstance(raw_path, str) or not isinstance(expected, str):
            raise EvidenceError("inputChecksums entries must map paths to digests")
        if not DIGEST.fullmatch(expected):
            raise EvidenceError(f"invalid recorded input digest for {raw_path}")
        path = safe_relative_path(raw_path)
        subject_actual = sha256(subject_reader(revision, path))
        if subject_actual != expected:
            raise EvidenceError(
                f"subject input checksum mismatch for {path}: "
                f"expected {expected}, got {subject_actual}"
            )
        worktree_actual = sha256(worktree_reader(path))
        if worktree_actual != expected:
            raise EvidenceError(
                f"current input checksum mismatch for {path}: "
                f"expected {expected}, got {worktree_actual}"
            )
    return len(checksums)


def parse_external_checksums(path: Path) -> dict[Path, str]:
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as error:
        raise EvidenceError(f"cannot read external checksum file {path}: {error}") from error
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


def verify_retained_outputs(
    manifest: dict[str, Any], checksum_file: Path
) -> int:
    outputs = manifest.get("outputs")
    if not isinstance(outputs, list) or not all(isinstance(item, str) for item in outputs):
        raise EvidenceError("outputs must be an array of paths")
    output_paths = {safe_relative_path(item) for item in outputs}
    try:
        checksum_relative = checksum_file.relative_to(ROOT)
    except ValueError as error:
        raise EvidenceError("external checksum file must be inside the repository") from error
    if checksum_relative not in output_paths:
        raise EvidenceError("outputs must name the external checksum file")
    expected_entries = output_paths - {checksum_relative}
    recorded_entries = parse_external_checksums(checksum_file)
    if set(recorded_entries) != expected_entries:
        raise EvidenceError("external checksum entries do not match retained outputs")
    for path, expected in recorded_entries.items():
        try:
            actual = sha256((ROOT / path).read_bytes())
        except OSError as error:
            raise EvidenceError(f"cannot read retained output {path}: {error}") from error
        if actual != expected:
            raise EvidenceError(
                f"retained output checksum mismatch for {path}: "
                f"expected {expected}, got {actual}"
            )
    return len(recorded_entries)


def verify_record(record: Path) -> tuple[int, int]:
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
    revision = manifest.get("subjectRevision")
    expected_record_id = f"pgm-01-{str(revision)[:7]}"
    if manifest.get("recordId") != record.name or record.name != expected_record_id:
        raise EvidenceError("record path, recordId, and subjectRevision do not agree")
    checksum_file = record.parent / f"{record.name}.sha256"
    output_count = verify_retained_outputs(manifest, checksum_file)
    input_count = verify_input_checksums(manifest)
    return output_count, input_count


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("record", type=Path, help="revision-scoped evidence directory")
    args = parser.parse_args()
    try:
        outputs, inputs = verify_record(args.record)
    except EvidenceError as error:
        print(f"evidence verification error: {error}", file=sys.stderr)
        return 1
    print(
        f"PGM-01 evidence: {outputs}/{outputs} retained outputs and "
        f"{inputs}/{inputs} subject/current inputs matched"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
