"""Tests for the repository-local artifact-ID and relocation-map check (FR-345)."""

import contextlib
import io
import pathlib
import subprocess
import tempfile
import unittest

from scripts.check_artifact_ids import ROOT, check, main

HEADER = "old_path\tnew_path\told_id\tnew_id\n"


def artifact(identifier: str, kind: str = "FR", body: str = "") -> str:
    return f"---\nid: {identifier}\ntitle: t\ntype: {kind}\n---\n# {identifier}\n{body}"


def matrix(identifier: str, *test_cases: str, coverage: str = "") -> str:
    rows = "".join(f"| {tc} | title | Test | P0 | FR-001 | ✅ implemented |\n" for tc in test_cases)
    return (
        f"---\nid: {identifier}\ntitle: t\ntype: TestMatrix\n---\n# Matrix\n\n"
        "## Functional Requirement Coverage\n\n"
        "| Functional Req | Acceptance Criteria | Test Cases | Status |\n|---|---|---|---|\n"
        f"{coverage}\n"
        "## Test Case Summary\n\n"
        "| Test ID | Title | Type | Priority | Traces To | Status |\n"
        "|---|---|---|---|---|---|\n"
        f"{rows}"
    )


def master(*blocks: str) -> str:
    rows = "".join(f"| {row} |\n" for row in blocks)
    return (
        "---\ntype: master-requirements\nname: t\n---\n# Master\n\n## ID Blocks\n\n"
        "| Family | Range | Holder | Status |\n| --- | --- | --- | --- |\n"
        f"{rows}\n## References\n"
    )


def relocation_map(*rows: str) -> str:
    return HEADER + "".join(f"{row}\n" for row in rows)


class Tree:
    """A scratch repository whose spec tree the check reads."""

    def __init__(self, root: pathlib.Path) -> None:
        self.root = root

    def write(self, path: str, text: str) -> None:
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text, encoding="utf-8")

    def move(self, old: str, new: str) -> None:
        target = self.root / new
        target.parent.mkdir(parents=True, exist_ok=True)
        (self.root / old).rename(target)

    def remove(self, path: str) -> None:
        (self.root / path).unlink()

    def git(self, *arguments: str) -> str:
        return subprocess.run(
            [
                "git",
                "-c", "user.name=test",
                "-c", "user.email=test@example.invalid",
                "-c", "commit.gpgsign=false",
                "-C", str(self.root),
                *arguments,
            ],
            capture_output=True,
            text=True,
            check=True,
        ).stdout.strip()

    def commit(self) -> str:
        """Commit the tree as it stands and return the commit id, the SPEC_BASE."""
        if not (self.root / ".git").exists():
            self.git("init", "--quiet")
        self.git("add", "--all")
        self.git("commit", "--quiet", "--allow-empty", "-m", "base")
        return self.git("rev-parse", "HEAD")

    def findings(self, base: str | None = None) -> list[str]:
        return check(self.root, base)[0]


@contextlib.contextmanager
def scratch_tree():
    with tempfile.TemporaryDirectory(prefix="quire-artifact-ids-") as directory:
        tree = Tree(pathlib.Path(directory))
        tree.write("spec/functional/FR-001-a.md", artifact("FR-001"))
        tree.write("spec/test-matrix.md", matrix("TM-001", "TC-001"))
        yield tree


def restructure_base(tree: Tree) -> str:
    """A flat base tree for a structural change, committed."""
    tree.write("spec/functional/FR-002-b.md", artifact("FR-002"))
    tree.write("spec/functional/FR-003-c.md", artifact("FR-003"))
    tree.write("spec/contract-test-matrix.md", matrix("TM-002", "TC-002", "TC-003"))
    return tree.commit()


def restructure(tree: Tree) -> list[str]:
    """Move the base tree into subsystems and return the complete map's rows."""
    tree.move("spec/functional/FR-001-a.md", "spec/core/functional/FR-001-a.md")
    tree.move("spec/functional/FR-002-b.md", "spec/core/functional/FR-002-b.md")
    tree.move("spec/functional/FR-003-c.md", "spec/emit/functional/FR-003-c.md")
    tree.move("spec/test-matrix.md", "spec/core/matrix/tests.md")
    tree.move("spec/contract-test-matrix.md", "spec/emit/matrix/tests.md")
    tree.write("spec/tests.md", artifact("TM-003", "TestMatrixIndex"))
    return [
        "-\tspec/tests.md\t-\tTM-003",
        "spec/contract-test-matrix.md\tspec/emit/matrix/tests.md\tTM-002\tTM-002",
        "spec/functional/FR-001-a.md\tspec/core/functional/FR-001-a.md\tFR-001\tFR-001",
        "spec/functional/FR-002-b.md\tspec/core/functional/FR-002-b.md\tFR-002\tFR-002",
        "spec/functional/FR-003-c.md\tspec/emit/functional/FR-003-c.md\tFR-003\tFR-003",
        "spec/test-matrix.md\tspec/core/matrix/tests.md\tTM-001\tTM-001",
    ]


MAP = "spec/relocations/2026-09-28-subsystems.tsv"


class DuplicateArtifactIdTests(unittest.TestCase):
    def test_two_spec_artifacts_declaring_one_id_fail_with_both_paths(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-1."""
        with scratch_tree() as tree:
            tree.write("spec/contract/FR-001-copy.md", artifact("FR-001"))
            self.assertEqual(
                tree.findings(),
                [
                    "FR-001 is declared by more than one spec artifact: "
                    "spec/contract/FR-001-copy.md:2, spec/functional/FR-001-a.md:2"
                ],
            )

    def test_prose_plans_and_reviews_do_not_declare_spec_ids(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-1."""
        with scratch_tree() as tree:
            tree.write("spec/functional/FR-002-b.md", artifact("FR-002", body="id: FR-001\n"))
            for path in (
                "plan/PLAN-001-a/index.md",
                "plan/PLAN-001-b/index.md",
                "reviews/a.md",
                "reviews/b.md",
                "spec/reviews/a.md",
                "spec/reviews/b.md",
            ):
                tree.write(path, artifact("SR-001", "SpecReview"))
            tree.write("plan/PLAN-002/index.md", artifact("FR-001", "Plan"))
            tree.write("spec/reviews/c.md", artifact("FR-001", "SpecReview"))
            self.assertEqual(tree.findings(), [])


class DuplicateTestCaseRowTests(unittest.TestCase):
    def test_tc_leading_two_rows_in_one_matrix_fails_with_each_line(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-2."""
        with scratch_tree() as tree:
            tree.write("spec/test-matrix.md", matrix("TM-001", "TC-001", "TC-002", "TC-001"))
            self.assertEqual(
                tree.findings(),
                [
                    "TC-001 is declared by more than one Test Case Summary row: "
                    "spec/test-matrix.md:17, spec/test-matrix.md:19"
                ],
            )

    def test_tc_leading_rows_in_two_matrices_fails_with_each_line(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-2."""
        with scratch_tree() as tree:
            tree.write("spec/contract-test-matrix.md", matrix("TM-002", "TC-001"))
            self.assertEqual(
                tree.findings(),
                [
                    "TC-001 is declared by more than one Test Case Summary row: "
                    "spec/contract-test-matrix.md:17, spec/test-matrix.md:17"
                ],
            )

    def test_citing_a_tc_in_another_table_does_not_declare_it(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-2."""
        with scratch_tree() as tree:
            tree.write(
                "spec/test-matrix.md",
                matrix("TM-001", "TC-001", coverage="| TC-001 | FR-001-AC-1 | TC-001 | ok |\n"),
            )
            tree.write(
                "spec/functional/FR-002-b.md",
                artifact(
                    "FR-002",
                    body="\n## Test Case Summary\n\n| Test ID | Title |\n|---|---|\n| TC-001 | x |\n",
                ),
            )
            self.assertEqual(tree.findings(), [])


class IdBlockTests(unittest.TestCase):
    def test_overlapping_blocks_of_one_family_fail_naming_both_rows(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-3."""
        with scratch_tree() as tree:
            tree.write(
                "spec/spec.md",
                master(
                    "FR | FR-900..FR-919 | `a` | open",
                    "FR | FR-910..FR-929 | `b` | open",
                ),
            )
            self.assertEqual(
                tree.findings(),
                [
                    "ID blocks of family FR overlap from FR-910 to FR-919: "
                    "spec/spec.md:11, spec/spec.md:12"
                ],
            )

    def test_adjacent_ranges_and_other_families_pass(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-3."""
        with scratch_tree() as tree:
            tree.write(
                "spec/spec.md",
                master(
                    "FR | FR-900..FR-919 | `a` | open",
                    "FR | FR-920..FR-929 | `b` | open",
                    "TC | TC-900..TC-919 | `a` | closed",
                ),
            )
            self.assertEqual(tree.findings(), [])


class RelocationMapShapeTests(unittest.TestCase):
    def test_wrong_header_fails_at_its_line(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-4."""
        with scratch_tree() as tree:
            tree.write(MAP, "old_path\tnew_path\tnew_id\told_id\n")
            self.assertEqual(
                tree.findings(),
                [
                    "relocation map header is not the tab-separated "
                    f"old_path new_path old_id new_id: {MAP}:1"
                ],
            )

    def test_unsorted_and_repeated_rows_fail_at_their_lines(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-4."""
        with scratch_tree() as tree:
            tree.write(
                MAP,
                relocation_map(
                    "spec/b.md\tspec/x/b.md\t-\t-",
                    "spec/a.md\tspec/x/a.md\t-\t-",
                    "spec/b.md\tspec/y/b.md\t-\t-",
                ),
            )
            self.assertEqual(
                tree.findings(),
                [
                    "relocation map rows are not sorted by old_path then new_path: "
                    f"{MAP}:3 sorts before {MAP}:2",
                    f"relocation map repeats old_path spec/b.md: {MAP}:2, {MAP}:4",
                ],
            )

    def test_added_file_rows_need_dash_old_id_and_a_tm_new_id(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-4."""
        with scratch_tree() as tree:
            tree.write(
                MAP,
                relocation_map("-\tspec/tests.md\tTM-001\tTM-003", "-\tspec/x.md\t-\tFR-050"),
            )
            self.assertEqual(
                tree.findings(),
                [
                    f"added-file row has old_id TM-001, not -: {MAP}:2",
                    f"added-file row has new_id FR-050, not a TM ID: {MAP}:3",
                ],
            )

    def test_added_map_new_path_must_exist_and_declare_new_id(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-4."""
        with scratch_tree() as tree:
            base = restructure_base(tree)
            rows = restructure(tree)
            rows[2] = "spec/functional/FR-001-a.md\tspec/core/functional/FR-001-z.md\tFR-001\tFR-001"
            rows[3] = "spec/functional/FR-002-b.md\tspec/core/functional/FR-002-b.md\tFR-002\tFR-009"
            tree.write(MAP, relocation_map(*rows))
            findings = tree.findings(base)
            self.assertIn(
                f"new_path spec/core/functional/FR-001-z.md is not in the tree: {MAP}:4", findings
            )
            self.assertIn(
                "new_path spec/core/functional/FR-002-b.md declares FR-002, not new_id "
                f"FR-009: {MAP}:5, spec/core/functional/FR-002-b.md:2",
                findings,
            )

    def test_a_map_already_at_spec_base_is_not_held_against_the_tree(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-4.

        A historical map records an earlier move; a later restructure moves its
        new paths again, and the map must not fail for that.
        """
        with scratch_tree() as tree:
            tree.write(
                "spec/relocations/2026-01-01-old.tsv",
                relocation_map("spec/old/FR-001-a.md\tspec/gone/FR-001-a.md\tFR-001\tFR-007"),
            )
            base = tree.commit()
            tree.write("spec/functional/FR-010-new.md", artifact("FR-010"))
            self.assertEqual(tree.findings(base), [])


class StructuralModeTests(unittest.TestCase):
    def test_complete_map_with_additions_and_a_collision_renumbering_passes(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-5."""
        with scratch_tree() as tree:
            tree.write("spec/contract/FR-002-dup.md", artifact("FR-002"))
            base = restructure_base(tree)
            rows = restructure(tree)
            tree.move("spec/contract/FR-002-dup.md", "spec/emit/functional/FR-004-dup.md")
            tree.write("spec/emit/functional/FR-004-dup.md", artifact("FR-004"))
            tree.write("spec/emit/matrix/other.md", artifact("TM-004", "TestMatrix"))
            rows.append("-\tspec/emit/matrix/other.md\t-\tTM-004")
            rows.append(
                "spec/contract/FR-002-dup.md\tspec/emit/functional/FR-004-dup.md\tFR-002\tFR-004"
            )
            tree.write(MAP, relocation_map(*sorted(rows)))
            findings, summary = check(tree.root, base)
            self.assertEqual(findings, [])
            self.assertIn("match SPEC_BASE through 1 added relocation map", summary)

    def assert_restructure_fails(self, mutate, expected: str) -> None:
        with scratch_tree() as tree:
            base = restructure_base(tree)
            rows = restructure(tree)
            rows = mutate(tree, rows) or rows
            tree.write(MAP, relocation_map(*rows))
            findings = tree.findings(base)
            self.assertTrue(
                any(expected in finding for finding in findings),
                f"{expected!r} not in {findings}",
            )

    def test_dropping_an_id_fails(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-5."""

        def drop(tree: Tree, rows: list[str]) -> list[str]:
            tree.remove("spec/emit/functional/FR-003-c.md")
            return [row for row in rows if "FR-003" not in row]

        self.assert_restructure_fails(drop, "spec-artifact ID FR-003 is declared 0 time(s)")

    def test_adding_a_non_tm_id_fails(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-5."""
        self.assert_restructure_fails(
            lambda tree, rows: tree.write("spec/core/functional/FR-050-x.md", artifact("FR-050")),
            "spec-artifact ID FR-050 is declared 1 time(s) in the tree, but SPEC_BASE with "
            "the added relocation maps applied declares it 0 time(s): "
            "spec/core/functional/FR-050-x.md:2",
        )

    def test_adding_a_tm_id_without_a_map_row_fails(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-5."""
        self.assert_restructure_fails(
            lambda tree, rows: tree.write("spec/emit/matrix/x.md", artifact("TM-009", "TestMatrix")),
            "spec-artifact ID TM-009 is declared 1 time(s)",
        )

    def test_renumbering_without_a_map_row_fails(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-5."""
        self.assert_restructure_fails(
            lambda tree, rows: tree.write("spec/emit/functional/FR-003-c.md", artifact("FR-030")),
            "spec-artifact ID FR-003 is declared 0 time(s)",
        )

    def test_renaming_a_file_without_a_map_row_fails(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-5."""

        def unlisted(tree: Tree, rows: list[str]) -> list[str]:
            return [row for row in rows if not row.startswith("spec/functional/FR-003")]

        with scratch_tree() as tree:
            base = restructure_base(tree)
            rows = unlisted(tree, restructure(tree))
            tree.write(MAP, relocation_map(*rows))
            label = base[:12]
            self.assertEqual(
                tree.findings(base),
                [
                    f"{label}:spec/functional/FR-003-c.md moved and has 0 rows in the "
                    "added relocation maps, not one"
                ],
            )

    def test_removing_a_tc_row_without_a_map_row_fails(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-5."""
        with scratch_tree() as tree:
            base = restructure_base(tree)
            rows = restructure(tree)
            tree.write("spec/emit/matrix/tests.md", matrix("TM-002", "TC-002"))
            tree.write(MAP, relocation_map(*rows))
            label = base[:12]
            self.assertEqual(
                tree.findings(base),
                [
                    "matrix TC row TC-003 is declared 0 time(s) in the tree, but SPEC_BASE "
                    "with the added relocation maps applied declares it 1 time(s): "
                    f"{label}:spec/contract-test-matrix.md:18"
                ],
            )


class OrdinaryChangeAndGateTests(unittest.TestCase):
    def test_ordinary_change_adding_ids_passes_without_an_added_map(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-6."""
        with scratch_tree() as tree:
            base = tree.commit()
            tree.write("spec/functional/FR-002-b.md", artifact("FR-002"))
            tree.write("spec/test-matrix.md", matrix("TM-001", "TC-001", "TC-002"))
            self.assertEqual(tree.findings(base), [])

    def test_clean_tree_exits_zero_with_one_summary_line(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-6."""
        with scratch_tree() as tree:
            stdout, stderr = io.StringIO(), io.StringIO()
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                self.assertEqual(main(["--root", str(tree.root)]), 0)
            self.assertEqual(stderr.getvalue(), "")
            self.assertEqual(
                stdout.getvalue(),
                "artifact id check: 2 spec-artifact IDs, 1 matrix TC rows, 0 ID blocks "
                "and 0 relocation map(s), no defect\n",
            )

    def test_a_defect_exits_non_zero_on_stderr(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-6."""
        with scratch_tree() as tree:
            tree.write("spec/functional/FR-001-b.md", artifact("FR-001"))
            stdout, stderr = io.StringIO(), io.StringIO()
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                self.assertEqual(main(["--root", str(tree.root)]), 1)
            self.assertEqual(stdout.getvalue(), "")
            self.assertIn("artifact id error: FR-001 is declared", stderr.getvalue())

    def test_make_spec_fails_when_the_check_fails(self) -> None:
        """TC-224. Trace: TC-224, FR-345-AC-6."""
        result = subprocess.run(
            ["make", "--no-print-directory", "-C", str(ROOT), "spec", "SPEC_BASE=no-such-revision"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "artifact id error: SPEC_BASE 'no-such-revision' does not name a commit",
            result.stderr,
        )
        # The check runs first, so `quire validate` never starts.
        self.assertNotIn("validate", result.stdout)


if __name__ == "__main__":
    unittest.main()
