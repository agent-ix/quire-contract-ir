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
# `#[trace(...)]` is the declared binding between a test and the ids it
# verifies (see `ix_trace_rs::trace`): comma-separated string-literal
# arguments, in any order, mixing exactly the id shapes each names — a `TC-`
# id and zero or more acceptance-criterion ids such as `FR-047-AC-1`. This
# mirrors how `quire coverage` itself reads the attribute: whichever quoted
# argument is `TC-\d{3}` shaped is the bound test case; nothing about the
# decorated function's own name is part of that reading.
RUST_TRACE_ATTR = re.compile(r"#\[trace\((?P<args>.*?)\)\]", re.S)
RUST_TRACE_ARG = re.compile(r'"([^"]*)"')
RUST_TRACE_TC_ID = re.compile(r"\ATC-(\d{3})\Z")
# One `#[test]` item: every attribute contiguous with it — including
# `#[trace(...)]`, in whatever order they were written — up through the `fn`
# it decorates. The captured name exists only so a disagreement or an absent
# trace can be reported; it is never consulted to bind coverage.
RUST_TEST_UNIT = re.compile(
    r"(?P<attrs>(?:#\[[^\[\]]*\]\s*)+)fn\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)"
)
RUST_NAME_TC_ID = re.compile(r"\Atc_(\d{3})(?:_|\b)")
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


def rust_test_bindings(path: Path) -> tuple[set[str], list[str]]:
    """TC ids `path`'s `#[test]` functions bind, and any binding failures.

    Binding comes only from a test's own `#[trace(...)]` — never from its
    function name. A `tc_NNN_*` name is inspected only to report the two
    failure modes a name-driven reading used to hide: a test traced to a
    different TC than its name suggests (the credited TC silently moves when
    the function is renamed), and a test carrying no trace at all (which
    binds nothing, however its name reads).
    """
    text = path.read_text(encoding="utf-8")
    bound: set[str] = set()
    failures: list[str] = []
    for unit in RUST_TEST_UNIT.finditer(text):
        attrs, name = unit.group("attrs"), unit.group("name")
        if "#[test]" not in attrs:
            continue
        trace_ids = sorted(
            {
                id_match.group(1)
                for trace in RUST_TRACE_ATTR.finditer(attrs)
                for arg in RUST_TRACE_ARG.findall(trace.group("args"))
                for id_match in [RUST_TRACE_TC_ID.match(arg)]
                if id_match
            }
        )
        bound.update(f"TC-{number}" for number in trace_ids)
        name_match = RUST_NAME_TC_ID.match(name)
        if name_match is None:
            continue
        name_id = name_match.group(1)
        line = text.count("\n", 0, unit.start("name")) + 1
        if not trace_ids:
            failures.append(
                f"{path}:{line} {name} is named for TC-{name_id} but carries no "
                f"#[trace]; it contributes no coverage for TC-{name_id}"
            )
        elif name_id not in trace_ids:
            traced = ", ".join(f"TC-{number}" for number in trace_ids)
            failures.append(
                f"{path}:{line} {name} is named for TC-{name_id} but #[trace] "
                f"binds {traced}; coverage follows the trace, not the name"
            )
    return bound, failures


def executable_tests(root: Path = ROOT) -> tuple[set[str], list[str]]:
    result = set()
    failures: list[str] = []
    for path in (root / "tests").glob("*.rs"):
        bound, rust_failures = rust_test_bindings(path)
        result.update(bound)
        failures.extend(rust_failures)
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
    return result, failures


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
    executable, trace_failures = executable_tests(root)
    failures = list(trace_failures)
    failures.extend(validate_documents(documents, executable))
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
