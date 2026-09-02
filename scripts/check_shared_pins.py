#!/usr/bin/env python3
"""Classify the local shared-assurance toolchain against the accepted matrix.

FR-022-AC-1. This script observes; it does not judge. Every verdict comes from
`engineering_assurance.compatibility`, which is the accepted authority on which
component versions are pinned. Restating those rules here would create a second
authority that drifts from the first, which is the failure mode this campaign
exists to remove.

What this file owns is narrow and local:

- how each component is observed on this machine, and
- the digests of the Engineering Assurance artifacts this repository reads.

A component that cannot be observed is `unknown`, never a pass. A consumed
artifact whose bytes do not match `assurance/pins.json` fails closed, because a
drifted mapping is a mapping nobody accepted.

    python3 scripts/check_shared_pins.py [--json]

Human acceptance of the matrix is reported here and not gated on. Whether an
enforcing migration may begin is Engineering Assurance's question, a human
answered it in that repository, and the released 0.2.0 artifact installed here
predates the field that records the answer. Reading acceptance out of the
installed package would report a decision the package never carried, in either
direction. What this gate decides is narrower and entirely local: whether the
toolchain on this machine is the accepted one.

Exit status: 0 when every component is compatible, every consumed artifact
matches its pin, and no requirement names the internal mirror; 1 otherwise;
2 when Engineering Assurance itself cannot be loaded, because a gate that
cannot ask the authority has no answer to report.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
PINS_PATH = ROOT / "assurance/pins.json"
PINS_SCHEMA_VERSION = "quire-contract-ir.assurance-pins/v1"
SEMVER = re.compile(r"\b(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)\b")
FORBIDDEN_REGISTRY = "npm.ix"


class PinError(RuntimeError):
    """The pinned assurance toolchain cannot be resolved or read."""


def load_pins() -> dict[str, Any]:
    try:
        pins = json.loads(PINS_PATH.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise PinError(f"cannot read {PINS_PATH.name}: {error}") from error
    if pins.get("schema_version") != PINS_SCHEMA_VERSION:
        raise PinError(f"unknown pins schema: {pins.get('schema_version')!r}")
    return pins


def observe(command: list[str]) -> str | None:
    """Read one tool's self-reported version, or None if it cannot be read.

    Every failure path returns None rather than a guess: a missing binary, a
    non-zero exit, a timeout, and unparseable output are all "not observed".
    """
    if shutil.which(command[0]) is None:
        return None
    try:
        completed = subprocess.run(
            command, capture_output=True, text=True, timeout=60, check=False
        )
    except (OSError, subprocess.SubprocessError):
        return None
    if completed.returncode != 0:
        return None
    match = SEMVER.search(completed.stdout.strip())
    return match.group(1) if match else None


def observe_quire() -> str | None:
    """quire reports structured provenance; read the CLI version from it."""
    if shutil.which("quire") is None:
        return None
    try:
        completed = subprocess.run(
            ["quire", "provenance"], capture_output=True, text=True, timeout=60, check=False
        )
    except (OSError, subprocess.SubprocessError):
        return None
    if completed.returncode != 0:
        return None
    try:
        return str(json.loads(completed.stdout)["cli"]["version"])
    except (json.JSONDecodeError, KeyError, TypeError):
        return None


def observe_engineering_assurance() -> str | None:
    """The installed distribution's own declared version."""
    import importlib.metadata

    try:
        return importlib.metadata.version("engineering-assurance")
    except importlib.metadata.PackageNotFoundError:
        return None


def package_root() -> Path:
    import engineering_assurance

    return Path(engineering_assurance.__file__).resolve().parent


def artifact_digest_mismatches(pins: dict[str, Any]) -> list[str]:
    """Re-hash every consumed Engineering Assurance artifact that pins a digest.

    An artifact with no recorded digest is skipped by design and says so in the
    pins file; an artifact that records one and does not match is a mismatch,
    and a recorded artifact that is absent is a mismatch too. Absent is not
    "somebody else's tree" here — this repository named the file it reads.
    """
    root = package_root()
    mismatches: list[str] = []
    for artifact in pins["engineering_assurance"]["consumed_artifacts"]:
        expected = artifact.get("sha256")
        if expected is None:
            continue
        path = root / artifact["path"]
        if not path.is_file():
            mismatches.append(f"{artifact['path']}: absent from the installed distribution")
            continue
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != expected:
            mismatches.append(f"{artifact['path']}: {actual}, pinned {expected}")
    return mismatches


def mirror_references() -> list[str]:
    """Report any requirement or config that names the internal mirror.

    The accepted matrix forbids it in any requirement, pin, lockfile, or
    `.npmrc`: the mirror is unreachable from CI and lags the public registry, so
    a pin that names it can neither be installed nor believed.
    """
    offenders = []
    for name in (
        "requirements-assurance.txt",
        "requirements-governance.txt",
        "assurance/pins.json",
        ".npmrc",
        "Cargo.lock",
    ):
        path = ROOT / name
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        for number, line in enumerate(text.splitlines(), start=1):
            stripped = line.strip()
            if FORBIDDEN_REGISTRY not in stripped:
                continue
            # The rule itself is stated in prose in the pins file; a sentence
            # forbidding the mirror is not a use of the mirror.
            if '"registry"' in stripped or "must not appear" in stripped:
                continue
            offenders.append(f"{name}:{number}: {stripped}")
    return offenders


def classify() -> dict[str, Any]:
    pins = load_pins()
    try:
        from engineering_assurance.compatibility import accepted, classify_all, load_matrix
    except ImportError as error:
        raise PinError(
            "engineering-assurance is not importable; install "
            "requirements-assurance.txt into this interpreter "
            f"({sys.executable}): {error}"
        ) from error

    matrix = load_matrix()
    observed = {
        "quire-cli": observe_quire(),
        "quoin": observe(["quoin", "--version"]),
        "ix-flow": observe(["ix-flow", "--version"]),
        "engineering-assurance": observe_engineering_assurance(),
    }
    classifications = classify_all(matrix, observed)
    mismatches = artifact_digest_mismatches(pins)
    offenders = mirror_references()
    versions_ok = accepted(classifications) and not mismatches and not offenders
    acceptance = matrix["accepted"]
    return {
        "schemaVersion": "quire-contract-ir.shared-pin-report/v1",
        "interpreter": sys.executable,
        # Reported, not gated, and the difference matters. Whether an enforcing
        # migration may begin is Engineering Assurance's question and a human
        # answered it there; the released 0.2.0 artifact this repository
        # installs predates the field that records the answer, so reading
        # acceptance out of it would report a decision the artifact never
        # carried. What this gate decides is narrower and entirely local: is the
        # toolchain on this machine the accepted one.
        "acceptance_state": acceptance.get("state"),
        "acceptance_recorded_here": bool(acceptance.get("accepted_by")),
        "acceptance_authority": "agent-ix/engineering-assurance, docs/compatibility-matrix.md",
        "versions_compatible": versions_ok,
        "accepted": versions_ok,
        "components": [
            {
                "component": item.component,
                "observed": item.observed,
                "expected": item.expected,
                "verdict": item.verdict,
                "reason": item.reason,
            }
            for item in classifications
        ],
        "artifact_mismatches": mismatches,
        "mirror_references": offenders,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Classify the shared-assurance toolchain.")
    parser.add_argument("--json", action="store_true", help="emit the report as JSON")
    args = parser.parse_args()
    try:
        report = classify()
    except PinError as error:
        print(f"shared pin error: {error}", file=sys.stderr)
        return 2
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        for item in report["components"]:
            print(f"{item['verdict']:<12} {item['component']:<24} {item['reason']}")
        for mismatch in report["artifact_mismatches"]:
            print(f"{'mismatch':<12} {mismatch}")
        for offender in report["mirror_references"]:
            print(f"{'mirror':<12} {offender}")
        print()
        print(
            f"human acceptance, as the installed release records it: "
            f"{report['acceptance_state']} "
            f"(authority: {report['acceptance_authority']})"
        )
        print("toolchain gate: " + ("satisfied" if report["accepted"] else "NOT satisfied"))
    return 0 if report["accepted"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
