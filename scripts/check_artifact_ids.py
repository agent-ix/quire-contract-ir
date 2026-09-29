#!/usr/bin/env python3
"""Fail when spec artifact IDs collide or a relocation map disagrees with the tree.

The repository-local check ADR-0056 requires until `quire validate` reports
these defects itself (FR-345). It reads only local files and, when `SPEC_BASE`
names a revision, that revision's Git objects.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from collections import Counter
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPEC = "spec"
EXCLUDED = "spec/reviews/"
RELOCATIONS = "spec/relocations/"
MAP_HEADER = ["old_path", "new_path", "old_id", "new_id"]
NONE = "-"
HEADING = re.compile(r"^(#{1,6})\s+(.*?)\s*$")
FRONTMATTER_ID = re.compile(r"^id:\s*['\"]?([^'\"\s]+)['\"]?\s*$")
FRONTMATTER_TYPE = re.compile(r"^type:\s*['\"]?([^'\"\s]+)['\"]?\s*$")
TC_ID = re.compile(r"TC-\d+")
TM_ID = re.compile(r"TM-\d+")
BLOCK_RANGE = re.compile(r"([A-Za-z]+)-(\d+)\.\.([A-Za-z]+)-(\d+)")


@dataclass(frozen=True)
class Site:
    """One `path:line`, prefixed by the base commit when it was read from `SPEC_BASE`."""

    path: str
    line: int
    base: str | None = None

    def __str__(self) -> str:
        where = f"{self.path}:{self.line}"
        return f"{self.base}:{where}" if self.base else where


@dataclass(frozen=True)
class Document:
    path: str
    text: str
    base: str | None = None

    def frontmatter(self) -> list[tuple[int, str]]:
        """The frontmatter lines with their 1-based line numbers, or none."""
        lines = self.text.splitlines()
        if not lines or lines[0].strip() != "---":
            return []
        body = []
        for number, line in enumerate(lines[1:], start=2):
            if line.strip() == "---":
                return body
            body.append((number, line))
        return []

    def field(self, pattern: re.Pattern[str]) -> tuple[str, int] | None:
        """The first frontmatter value `pattern` matches, with its line."""
        for number, line in self.frontmatter():
            match = pattern.match(line)
            if match:
                return match.group(1), number
        return None

    def kind(self) -> str | None:
        found = self.field(FRONTMATTER_TYPE)
        return found[0] if found else None

    def site(self, line: int) -> Site:
        return Site(self.path, line, self.base)


def table_rows(document: Document, heading: str) -> list[tuple[int, list[str]]]:
    """Rows of the tables under the level-2 `heading`.

    Each table's header row is returned with line 0 so a caller can map
    columns; body rows carry their 1-based line number.
    """
    rows: list[tuple[int, list[str]]] = []
    inside = False
    in_table = False
    for number, line in enumerate(document.text.splitlines(), start=1):
        match = HEADING.match(line)
        if match:
            if len(match.group(1)) <= 2:
                inside = match.group(1) == "##" and match.group(2) == heading
            in_table = False
            continue
        if not inside:
            continue
        if not line.lstrip().startswith("|"):
            in_table = False
            continue
        if set(line.replace("|", "").strip()) <= {"-", ":", " "}:
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        rows.append((number if in_table else 0, cells))
        in_table = True
    return rows


def spec_documents_on_disk(root: Path) -> list[Document]:
    """Every spec artifact in the working tree: Markdown under spec/ outside spec/reviews/."""
    documents = []
    for path in sorted((root / SPEC).rglob("*.md")):
        relative = path.relative_to(root).as_posix()
        if relative.startswith(EXCLUDED) or not path.is_file():
            continue
        documents.append(Document(relative, path.read_text(encoding="utf-8")))
    return documents


def declared_ids(documents: list[Document]) -> dict[str, list[Site]]:
    """The first frontmatter `id:` of each spec artifact."""
    declared: dict[str, list[Site]] = {}
    for document in documents:
        found = document.field(FRONTMATTER_ID)
        if found:
            identifier, line = found
            declared.setdefault(identifier, []).append(document.site(line))
    return declared


def matrix_test_cases(documents: list[Document]) -> dict[str, list[Site]]:
    """TC IDs leading a `Test Case Summary` row of a `TestMatrix` document."""
    declared: dict[str, list[Site]] = {}
    for document in documents:
        if document.kind() != "TestMatrix":
            continue
        for line, cells in table_rows(document, "Test Case Summary"):
            if line and cells and TC_ID.fullmatch(cells[0]):
                declared.setdefault(cells[0], []).append(document.site(line))
    return declared


def duplicates(kind: str, declared: dict[str, list[Site]]) -> list[str]:
    return [
        f"{identifier} is declared by more than one {kind}: "
        + ", ".join(str(site) for site in sites)
        for identifier, sites in sorted(declared.items())
        if len(sites) > 1
    ]


def id_block_findings(documents: list[Document]) -> tuple[list[str], int]:
    """Overlapping `## ID Blocks` rows of one family in the master document."""
    findings: list[str] = []
    blocks: list[tuple[str, int, int, Site]] = []
    for document in documents:
        if document.kind() != "master-requirements":
            continue
        columns: list[str] = []
        for line, cells in table_rows(document, "ID Blocks"):
            if not line:
                columns = cells
                continue
            row = dict(zip(columns, cells))
            family, value = row.get("Family", ""), row.get("Range", "")
            match = BLOCK_RANGE.fullmatch(value)
            if not match or match.group(1) != family or match.group(3) != family:
                findings.append(
                    f"ID block {value!r} is not a {family}-N..{family}-M range: "
                    f"{document.site(line)}"
                )
                continue
            low, high = int(match.group(2)), int(match.group(4))
            if low > high:
                findings.append(f"ID block {value} is empty: {document.site(line)}")
                continue
            blocks.append((family, low, high, document.site(line)))
    for index, (family, low, high, site) in enumerate(blocks):
        for other_family, other_low, other_high, other in blocks[index + 1 :]:
            if family == other_family and low <= other_high and other_low <= high:
                findings.append(
                    f"ID blocks of family {family} overlap from "
                    f"{family}-{max(low, other_low)} to {family}-{min(high, other_high)}: "
                    f"{site}, {other}"
                )
    return findings, len(blocks)


@dataclass(frozen=True)
class MapRow:
    old_path: str
    new_path: str
    old_id: str
    new_id: str
    site: Site


def read_map(path: str, text: str) -> tuple[list[MapRow], list[str]]:
    """The rows of one relocation map and the defects of its own shape."""
    findings: list[str] = []
    lines = [
        (number, line)
        for number, line in enumerate(text.splitlines(), start=1)
        if line.strip()
    ]
    if not lines or lines[0][1].split("\t") != MAP_HEADER:
        line = lines[0][0] if lines else 1
        return [], [
            f"relocation map header is not the tab-separated "
            f"{' '.join(MAP_HEADER)}: {Site(path, line)}"
        ]
    rows: list[MapRow] = []
    seen: dict[str, Site] = {}
    previous: MapRow | None = None
    for number, line in lines[1:]:
        site = Site(path, number)
        cells = line.split("\t")
        if len(cells) != len(MAP_HEADER) or not all(cells):
            findings.append(f"relocation map row does not have four columns: {site}")
            continue
        row = MapRow(*cells, site=site)
        if previous and (row.old_path, row.new_path) < (previous.old_path, previous.new_path):
            findings.append(
                "relocation map rows are not sorted by old_path then new_path: "
                f"{site} sorts before {previous.site}"
            )
        if row.old_path == NONE:
            if row.old_id != NONE:
                findings.append(f"added-file row has old_id {row.old_id}, not -: {site}")
            if not TM_ID.fullmatch(row.new_id):
                findings.append(f"added-file row has new_id {row.new_id}, not a TM ID: {site}")
        elif row.old_path in seen:
            findings.append(
                f"relocation map repeats old_path {row.old_path}: {seen[row.old_path]}, {site}"
            )
        else:
            seen[row.old_path] = site
        rows.append(row)
        previous = row
    return rows, findings


def tree_findings(rows: list[MapRow], root: Path, documents: dict[str, Document]) -> list[str]:
    """An added map's every `new_path` exists and declares its `new_id`."""
    findings = []
    for row in rows:
        if not (root / row.new_path).exists():
            findings.append(f"new_path {row.new_path} is not in the tree: {row.site}")
            continue
        if row.new_id == NONE:
            continue
        document = documents.get(row.new_path)
        found = document.field(FRONTMATTER_ID) if document else None
        if not found:
            findings.append(
                f"new_path {row.new_path} declares no id, not new_id {row.new_id}: {row.site}"
            )
        elif found[0] != row.new_id:
            findings.append(
                f"new_path {row.new_path} declares {found[0]}, not new_id {row.new_id}: "
                f"{row.site}, {document.site(found[1])}"
            )
    return findings


class BaseError(RuntimeError):
    """`SPEC_BASE` names no commit, or its objects cannot be read."""


def git(root: Path, *arguments: str, stdin: bytes | None = None) -> bytes:
    result = subprocess.run(
        ["git", "-C", str(root), *arguments], input=stdin, capture_output=True, check=False
    )
    if result.returncode != 0:
        raise BaseError(
            f"git {' '.join(arguments)} exited {result.returncode}: "
            f"{result.stderr.decode(errors='replace').strip()}"
        )
    return result.stdout


@dataclass(frozen=True)
class Base:
    commit: str
    paths: list[str]
    documents: list[Document]


def read_base(root: Path, base: str) -> Base:
    """The `spec/` paths and spec artifacts at the commit `base` names."""
    try:
        commit = git(root, "rev-parse", "--verify", "--quiet", f"{base}^{{commit}}")
    except BaseError as error:
        raise BaseError(f"SPEC_BASE {base!r} does not name a commit") from error
    commit_id = commit.decode().strip()
    listing = git(root, "ls-tree", "-r", "-z", "--name-only", commit_id, "--", SPEC)
    paths = [path for path in listing.decode().split("\0") if path]
    wanted = [path for path in paths if path.endswith(".md") and not path.startswith(EXCLUDED)]
    label = commit_id[:12]
    documents = []
    if wanted:
        output = git(
            root,
            "cat-file",
            "--batch",
            stdin="".join(f"{commit_id}:{path}\n" for path in wanted).encode(),
        )
        offset = 0
        for path in wanted:
            end = output.index(b"\n", offset)
            size = int(output[offset:end].split()[2])
            text = output[end + 1 : end + 1 + size].decode("utf-8")
            documents.append(Document(path, text, label))
            offset = end + 1 + size + 1
    return Base(commit_id, paths, documents)


def structural_findings(
    root: Path, base: Base, documents: list[Document], added: list[MapRow]
) -> list[str]:
    """ID-set equality and rename coverage against `SPEC_BASE` (ADR-0056 gate rules 3, 4)."""
    findings = []
    # Counted, not a plain set: a collision renumbering turns one of two
    # declarations of an ID into a new ID and leaves the other in place.
    renames = [
        row
        for row in added
        if row.old_id != NONE and row.new_id != NONE and row.old_id != row.new_id
    ]
    additions = [row for row in added if row.old_id == NONE and row.new_id != NONE]
    # A TC renumbering renames the test case's own artifact and its Test Case
    # Summary row; no other rename touches a TC row.
    tc_renames = [row for row in renames if TC_ID.fullmatch(row.old_id)]
    for kind, reader, applied in (
        ("spec-artifact ID", declared_ids, [*renames, *additions]),
        ("matrix TC row", matrix_test_cases, tc_renames),
    ):
        before = reader(base.documents)
        after = reader(documents)
        expected = Counter({identifier: len(sites) for identifier, sites in before.items()})
        cited: dict[str, list[Site]] = {}
        for row in applied:
            if row.old_id != NONE:
                expected[row.old_id] -= 1
                cited.setdefault(row.old_id, []).append(row.site)
            expected[row.new_id] += 1
            cited.setdefault(row.new_id, []).append(row.site)
        for identifier in sorted(set(expected) | set(after)):
            want, have = expected[identifier], len(after.get(identifier, []))
            if want == have:
                continue
            sites = [
                *before.get(identifier, []),
                *cited.get(identifier, []),
                *after.get(identifier, []),
            ]
            findings.append(
                f"{kind} {identifier} is declared {have} time(s) in the tree, but "
                f"SPEC_BASE with the added relocation maps applied declares it {want} "
                "time(s): " + ", ".join(str(site) for site in sites)
            )
    rows_by_old: dict[str, list[MapRow]] = {}
    for row in added:
        rows_by_old.setdefault(row.old_path, []).append(row)
    label = base.commit[:12]
    for path in base.paths:
        if path.startswith(EXCLUDED) or (root / path).exists():
            continue
        rows = rows_by_old.get(path, [])
        if len(rows) != 1:
            where = ": " + ", ".join(str(row.site) for row in rows) if rows else ""
            findings.append(
                f"{label}:{path} moved and has {len(rows)} rows in the added "
                f"relocation maps, not one{where}"
            )
    return findings


def check(root: Path, base: str | None) -> tuple[list[str], str]:
    """Every finding for the tree at `root`, and the summary line for a clean tree."""
    documents = spec_documents_on_disk(root)
    ids = declared_ids(documents)
    test_cases = matrix_test_cases(documents)
    findings = duplicates("spec artifact", ids)
    findings += duplicates("Test Case Summary row", test_cases)
    block_findings, block_count = id_block_findings(documents)
    findings += block_findings

    maps_dir = root / RELOCATIONS
    map_paths = sorted(
        path.relative_to(root).as_posix() for path in maps_dir.rglob("*") if path.is_file()
    )
    maps: dict[str, list[MapRow]] = {}
    for path in map_paths:
        rows, shape = read_map(path, (root / path).read_text(encoding="utf-8"))
        maps[path] = rows
        findings += shape

    added: list[str] = []
    if base:
        try:
            spec_base = read_base(root, base)
        except BaseError as error:
            return findings + [str(error)], ""
        # A map already at SPEC_BASE is a historical record of an earlier move;
        # only the maps this change adds are held against the tree.
        added = [path for path in map_paths if path not in spec_base.paths]
        by_path = {document.path: document for document in documents}
        for path in added:
            findings += tree_findings(maps[path], root, by_path)
        if added:
            findings += structural_findings(
                root, spec_base, documents, [row for path in added for row in maps[path]]
            )

    structural = (
        f"; spec-artifact IDs and TC rows match SPEC_BASE through {len(added)} added "
        "relocation map(s)"
        if added
        else ""
    )
    summary = (
        f"artifact id check: {len(ids)} spec-artifact IDs, {len(test_cases)} matrix TC "
        f"rows, {block_count} ID blocks and {len(map_paths)} relocation map(s), no "
        f"defect{structural}"
    )
    return findings, summary


# Implements: FR-345.
def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    arguments = parser.parse_args(argv)
    findings, summary = check(arguments.root.resolve(), os.environ.get("SPEC_BASE") or None)
    if findings:
        for finding in findings:
            print(f"artifact id error: {finding}", file=sys.stderr)
        return 1
    print(summary)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
