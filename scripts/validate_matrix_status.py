#!/usr/bin/env python3
"""Fail closed when a matrix marks verification complete without an executable test."""

from __future__ import annotations

import argparse
import ast
import re
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
# `#[test]` followed by any further outer attributes (for example `#[trace]`),
# in either order relative to them, and then the traced function.
RUST_TEST = re.compile(r"#\[test\]\s*(?:#\[[^\]]*\]\s*)*fn\s+tc_(\d{3})(?:_|\b)")
POLICY_AC = re.compile(r"PGM-\d+-R\d+-AC-\d+")
REQUIREMENT_ID = re.compile(r"(?:FR|NFR)-\d{3}")
RETIRED_HEADING = re.compile(r"^#{2,4}\s+Retired criteria\s*$", re.M)
SPEC_DIRECTORIES = ("spec/contract", "spec/functional", "spec/interface", "spec/nonfunctional")


def rows(document: str) -> list[list[str]]:
    parsed = []
    for line in document.splitlines():
        if not line.startswith("|") or set(line.replace("|", "").strip()) <= {"-"}:
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if cells and cells[0] not in {
            "ID",
            "Test ID",
            "Functional Req",
            "Stakeholder Req",
        }:
            parsed.append(cells)
    return parsed


def referenced_tests(value: str) -> set[str]:
    result = {f"TC-{number}" for number in TEST_ID.findall(value)}
    for start, end in TEST_RANGE.findall(value):
        result.update(f"TC-{number:03d}" for number in range(int(start), int(end) + 1))
    return result


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
            body = RETIRED_HEADING.split(document, maxsplit=1)[0]
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


def executable_tests(root: Path = ROOT) -> set[str]:
    result = set()
    for path in (root / "tests").glob("*.rs"):
        result.update(f"TC-{number}" for number in RUST_TEST.findall(path.read_text()))
    for path in (root / "tests").glob("*.py"):
        module = ast.parse(path.read_text(encoding="utf-8"))
        has_function_loader = any(
            isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
            and node.name == "load_tests"
            for node in module.body
        )
        for node in module.body:
            if (
                has_function_loader
                and isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
                and node.name.startswith("test_")
            ):
                result.update(referenced_tests(ast.get_docstring(node) or ""))
            if isinstance(node, ast.ClassDef) and any(
                (isinstance(base, ast.Name) and base.id == "TestCase")
                or (isinstance(base, ast.Attribute) and base.attr == "TestCase")
                for base in node.bases
            ):
                for method in node.body:
                    if isinstance(
                        method, (ast.FunctionDef, ast.AsyncFunctionDef)
                    ) and method.name.startswith("test_"):
                        result.update(referenced_tests(ast.get_docstring(method) or ""))
    return result


def validate_criterion_citations(
    documents: list[str], declared: dict[str, list[str]]
) -> list[str]:
    """Fail closed when a matrix row cites fewer criteria than it must.

    `quire coverage` reports a row backed when the criteria the row *names* are
    backed, so a row that silently omits one of its requirement's live criteria
    reads green while that criterion is verified by nothing.
    """
    failures = []
    for row in (row for document in documents for row in rows(document)):
        if not row or not REQUIREMENT_ID.fullmatch(row[0]) or row[0] not in declared:
            continue
        live = set(declared[row[0]])
        if not live:
            continue
        cited = cited_criteria(row[0], row[1])
        if not cited:
            # Sections that verify by method rather than by criterion id — the
            # non-functional table — name no criterion at all, by design.
            continue
        for criterion in sorted(live - cited):
            failures.append(f"{row[0]} omits live criterion {criterion}")
        for criterion in sorted(cited - live):
            failures.append(f"{row[0]} cites unknown or retired criterion {criterion}")
    return failures


def validate_documents(documents: list[str], executable: set[str]) -> list[str]:
    parsed = [row for document in documents for row in rows(document)]
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
    failures = validate_documents(documents, executable_tests(root))
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
