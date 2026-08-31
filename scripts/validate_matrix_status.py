#!/usr/bin/env python3
"""Fail closed when a matrix marks verification complete without an executable test."""

from __future__ import annotations

import ast
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MATRICES = (ROOT / "spec/test-matrix.md", ROOT / "spec/contract-test-matrix.md")
TEST_ID = re.compile(r"TC-(\d{3})")
TEST_RANGE = re.compile(r"TC-(\d{3})\s+through\s+TC-(\d{3})")
RUST_TEST = re.compile(r"#\[test\]\s*fn\s+tc_(\d{3})(?:_|\b)")


def rows(document: str) -> list[list[str]]:
    parsed = []
    for line in document.splitlines():
        if not line.startswith("|") or set(line.replace("|", "").strip()) <= {"-"}:
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if cells and cells[0] not in {"Test ID", "Functional Req", "Stakeholder Req"}:
            parsed.append(cells)
    return parsed


def referenced_tests(value: str) -> set[str]:
    result = {f"TC-{number}" for number in TEST_ID.findall(value)}
    for start, end in TEST_RANGE.findall(value):
        result.update(f"TC-{number:03d}" for number in range(int(start), int(end) + 1))
    return result


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
    return failures


def main() -> int:
    failures = validate_documents(
        [path.read_text(encoding="utf-8") for path in MATRICES], executable_tests()
    )
    if failures:
        for failure in failures:
            print(f"matrix status error: {failure}", file=sys.stderr)
        return 1
    print("matrix status census: every ✅ row resolves to completed executable tests")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
