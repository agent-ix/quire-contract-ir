#!/usr/bin/env python3
"""Fail closed when a matrix marks verification complete without an executable test."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
STATUS_DOCUMENTS = (
    Path("spec/test-matrix.md"),
    Path("spec/contract-test-matrix.md"),
    Path("spec/program/PGM-01-governance.md"),
)
TEST_ID = re.compile(r"TC-(\d{3})")
TEST_RANGE = re.compile(r"TC-(\d{3})\s+through\s+TC-(\d{3})")
POLICY_AC = re.compile(r"PGM-\d+-R\d+-AC-\d+")
REQUIREMENT_ID = re.compile(r"(?:FR|NFR)-\d{3}")
RETIRED_HEADING = re.compile(r"^#{2,4}\s+Retired criteria\b.*$", re.I)
SPEC_DIRECTORIES = ("spec/contract", "spec/functional", "spec/interface", "spec/nonfunctional")
HEADING = re.compile(r"^(#{1,6})\s+(.*?)\s*$")
# The heading text (matched case-insensitively, by substring) marking the one
# table kind that verifies by method rather than by acceptance-criterion id.
NON_FUNCTIONAL_SECTION = "non-functional"


def rows(document: str) -> list[tuple[str, list[str]]]:
    """Table rows in `document`, each paired with its nearest heading.

    The heading identifies the row's table *kind* (for example "Functional
    Requirement Coverage" versus "Non-Functional Requirement Coverage"), so a
    caller can key behavior on what table a row lives in rather than guessing
    from the row's own content.
    """
    parsed: list[tuple[str, list[str]]] = []
    section = ""
    for line in document.splitlines():
        heading = HEADING.match(line)
        if heading:
            section = heading.group(2)
            continue
        if not line.startswith("|") or set(line.replace("|", "").strip()) <= {"-"}:
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if cells and cells[0] not in {
            "ID",
            "Test ID",
            "Functional Req",
            "Stakeholder Req",
        }:
            parsed.append((section, cells))
    return parsed


def referenced_tests(value: str) -> set[str]:
    result = {f"TC-{number}" for number in TEST_ID.findall(value)}
    for start, end in TEST_RANGE.findall(value):
        result.update(f"TC-{number:03d}" for number in range(int(start), int(end) + 1))
    return result


def _without_retired_sections(document: str) -> str:
    """`document` with every `Retired criteria` subsection's body removed.

    A retired subsection runs from its own heading up to (but not including)
    the next heading at the same or a shallower level, wherever that falls —
    not "everything after the first match". Content before, between, or
    after retired subsections is kept, so a live criterion declared later in
    the document (for example after a `## Dependencies` section that follows
    a `### Retired criteria` subsection) is not silently dropped.
    """
    kept = []
    skip_at_or_below: int | None = None
    for line in document.splitlines(keepends=True):
        heading = HEADING.match(line)
        if heading:
            level = len(heading.group(1))
            if skip_at_or_below is not None and level <= skip_at_or_below:
                skip_at_or_below = None
            if skip_at_or_below is None and RETIRED_HEADING.match(line):
                skip_at_or_below = level
                continue
        if skip_at_or_below is not None:
            continue
        kept.append(line)
    return "".join(kept)


def live_criteria(root: Path = ROOT) -> dict[str, list[str]]:
    """Acceptance criterion ids each requirement document still declares.

    Criteria under a `Retired criteria` heading are deliberately withdrawn and
    are not live, so a matrix row that omits them is correct rather than
    under-citing.
    """
    declared: dict[str, list[str]] = {}
    for directory in SPEC_DIRECTORIES:
        for path in sorted((root / directory).glob("*.md")):
            document = path.read_text(encoding="utf-8")
            identifier = re.search(r"^id:\s*(\S+)", document, re.M)
            if not identifier or not REQUIREMENT_ID.fullmatch(identifier.group(1)):
                continue
            requirement = identifier.group(1)
            body = _without_retired_sections(document)
            criteria = re.findall(rf"\|\s*({requirement}-AC-\d+)\s*\|", body)
            declared[requirement] = sorted(set(criteria))
    return declared


def cited_criteria(requirement: str, cell: str) -> set[str]:
    """Criterion ids a matrix cell names, expanding `X through Y` ranges."""
    numbered = rf"{requirement}-AC-(\d+)"
    cited = set()
    for start, end in re.findall(rf"{numbered}\s+through\s+{numbered}", cell):
        cited.update(
            f"{requirement}-AC-{number}" for number in range(int(start), int(end) + 1)
        )
    cited.update(re.findall(rf"{requirement}-AC-\d+", cell))
    return cited


class QuireCoverageError(RuntimeError):
    """`quire coverage` could not be run, or did not return a parseable report."""


def run_quire_coverage(root: Path, *, quire_bin: str = "quire") -> dict:
    """The `CoverageReport` `quire coverage --scope <root> --json` emits.

    This is the *only* reader of test/code trace bindings: whichever forms
    `quire`'s declared trace-tag grammar honours (`#[trace(...)]`, the
    doc-comment and line-comment legacy forms, and any others a module
    declares) are the forms this script honours, because this script never
    parses Rust or Python source itself. `quire` also walks the whole
    scope — `tests/**/*.rs` and `src/`, not just `tests/*.rs` — so a
    production `#[trace]` (for example `src/predicate/artifacts.rs`) counts
    here too.

    Without `--strict`, `quire coverage` always exits 0 — the rollup is a
    report, not a gate (this script and `make spec`'s separate `--strict`
    invocation are the gates) — so a non-zero exit here means the command
    itself failed, not that a row is unbacked; that is never swallowed.
    """
    try:
        result = subprocess.run(
            [quire_bin, "coverage", "--scope", str(root), "--json"],
            capture_output=True,
            text=True,
            check=False,
        )
    except FileNotFoundError as error:
        raise QuireCoverageError(f"could not run {quire_bin!r}: {error}") from error
    if result.returncode != 0:
        raise QuireCoverageError(
            f"`{quire_bin} coverage --scope {root} --json` exited "
            f"{result.returncode}:\n{result.stderr}"
        )
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise QuireCoverageError(
            f"`{quire_bin} coverage --json` did not emit valid JSON: {error}\n"
            f"stderr:\n{result.stderr}"
        ) from error


def executable_test_ids(report: dict) -> set[str]:
    """TC ids `quire coverage` reports as backed by a real test-case symbol.

    `minted_targets` carries every id `quire` minted from the corpus, each
    with the `backed` verdict it computed — for a `test-case` target that
    verdict already reconciles every declared binding form, in every
    scanned language and file, exactly as `quire coverage --strict` gates
    on. Reading it here is the whole point of "one reader, one fact": this
    function invents no coverage of its own, in either direction.
    """
    return {
        target["id"]
        for target in report.get("minted_targets", [])
        if target.get("target") == "test-case" and target.get("backed")
    }


def validate_criterion_citations(
    documents: list[str], declared: dict[str, list[str]]
) -> list[str]:
    """Fail closed when a matrix row cites fewer criteria than it must.

    `quire coverage` reports a row backed when the criteria the row *names* are
    backed, so a row that silently omits one of its requirement's live criteria
    reads green while that criterion is verified by nothing.
    """
    failures = []
    for section, row in (
        item for document in documents for item in rows(document)
    ):
        if not row or not REQUIREMENT_ID.fullmatch(row[0]) or row[0] not in declared:
            continue
        live = set(declared[row[0]])
        if not live:
            continue
        cited = cited_criteria(row[0], row[1])
        if NON_FUNCTIONAL_SECTION not in section.lower():
            # The non-functional table verifies by method, not by criterion
            # id, so it names no criterion at all, by design — keyed on the
            # table the row lives in, not on whether the row is empty, so a
            # functional row that cites nothing still fails below. This
            # exemption covers only the omission direction: a non-functional
            # row that does cite a criterion is still held to naming a live
            # one, below.
            for criterion in sorted(live - cited):
                failures.append(f"{row[0]} omits live criterion {criterion}")
        for criterion in sorted(cited - live):
            failures.append(f"{row[0]} cites unknown or retired criterion {criterion}")
    return failures


def validate_documents(documents: list[str], executable: set[str]) -> list[str]:
    parsed = [row for document in documents for _, row in rows(document)]
    summaries = {
        row[0]: row[-1]
        for row in parsed
        if row and re.fullmatch(r"TC-\d{3}", row[0])
    }
    failures = []
    for test_id, status in sorted(summaries.items()):
        if status.startswith("✅") and test_id not in executable:
            failures.append(f"{test_id} is complete but has no executable test")
    for row in parsed:
        if not row or not row[-1].startswith("✅") or re.fullmatch(r"TC-\d{3}", row[0]):
            continue
        references = referenced_tests(" ".join(row[1:-1]))
        if not references:
            failures.append(f"{row[0]} is complete but references no test")
        for test_id in sorted(references):
            if not summaries.get(test_id, "").startswith("✅"):
                failures.append(f"{row[0]} is complete but {test_id} is not complete")
    for row in parsed:
        if not row or not POLICY_AC.fullmatch(row[0]):
            continue
        references = referenced_tests(" ".join(row[1:]))
        if not references:
            failures.append(f"{row[0]} cites no test")
        for test_id in sorted(references):
            if not summaries.get(test_id, "").startswith("✅"):
                failures.append(f"{row[0]} cites incomplete or unknown {test_id}")
            elif test_id not in executable:
                failures.append(f"{row[0]} cites non-executable {test_id}")
    return failures


# Implements: NFR-004-AC-5.
def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=ROOT)
    arguments = parser.parse_args(argv)
    root = arguments.root.resolve()
    documents = [(root / path).read_text(encoding="utf-8") for path in STATUS_DOCUMENTS]
    executable = executable_test_ids(run_quire_coverage(root))
    failures = validate_documents(documents, executable)
    failures.extend(validate_criterion_citations(documents, live_criteria(root)))
    if failures:
        for failure in failures:
            print(f"matrix status error: {failure}", file=sys.stderr)
        return 1
    print(
        "matrix status census: every ✅ row and PGM acceptance citation "
        "resolves to a completed test case with a declared test symbol, and "
        "every requirement row cites exactly its live acceptance criteria"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
