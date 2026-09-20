"""Tests for fail-closed matrix status classification."""

import contextlib
import io
import pathlib
import tempfile
import unittest
from unittest import mock

from scripts.validate_matrix_status import (
    cited_criteria,
    executable_test_ids,
    live_criteria,
    main,
    validate_criterion_citations,
    validate_documents,
)


class MatrixStatusTests(unittest.TestCase):
    def test_rejects_complete_rows_backed_by_planned_tests(self) -> None:
        """TC-021. Trace: TC-021, NFR-004-AC-5."""
        document = """
| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| NFR-001 | NFR-001-AC-1 | TC-019 | ✅ Complete |

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-019 | portability | Analysis | P0 | NFR-001 | 🚧 planned |
"""
        failures = validate_documents([document], set())
        self.assertEqual(failures, ["NFR-001 is complete but TC-019 is not complete"])

        no_test = """
| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| NFR-001 | NFR-001-AC-1 | none | ✅ Complete |
"""
        self.assertEqual(
            validate_documents([no_test], set()),
            ["NFR-001 is complete but references no test"],
        )

    def test_rejects_policy_acceptance_citation_without_executable_test(self) -> None:
        """TC-021. Trace: TC-021, NFR-004-AC-5."""
        matrix = """
| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-026 | dispositions | Inspection | P0 | FR-021 | ✅ implemented |
"""
        policy = """
| ID | Criterion | Verification |
|---|---|---|
| PGM-01-R11-AC-1 | dispositions are linked | TC-026 |
"""
        self.assertEqual(
            validate_documents([matrix, policy], set()),
            [
                "TC-026 is complete but has no executable test",
                "PGM-01-R11-AC-1 cites non-executable TC-026",
            ],
        )
        self.assertEqual(validate_documents([matrix, policy], {"TC-026"}), [])

    def test_main_reads_a_real_tree_and_fails_closed(self) -> None:
        """TC-021. Trace: TC-021, NFR-004-AC-5.

        `main()` never parses Rust or Python itself; it consumes whatever
        `run_quire_coverage` returns, so this test controls that boundary
        directly rather than writing source files for a real `quire` to
        scan. Everything else — the matrix and policy documents, the
        criterion-citation check — is real.
        """
        matrix = """
| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-021 | matrix status | Test | P0 | NFR-004 | ✅ implemented |
"""
        policy = """
| ID | Criterion | Verification |
|---|---|---|
| PGM-01-R11-AC-1 | completed rows resolve | TC-021 |
"""
        with tempfile.TemporaryDirectory(prefix="quire-matrix-status-") as directory:
            root = pathlib.Path(directory)
            (root / "spec/program").mkdir(parents=True)
            (root / "spec/test-matrix.md").write_text(matrix, encoding="utf-8")
            (root / "spec/contract-test-matrix.md").write_text("", encoding="utf-8")
            (root / "spec/program/PGM-01-governance.md").write_text(
                policy, encoding="utf-8"
            )

            unbacked_report = {"minted_targets": []}
            stderr = io.StringIO()
            with mock.patch(
                "scripts.validate_matrix_status.run_quire_coverage",
                return_value=unbacked_report,
            ), contextlib.redirect_stderr(stderr):
                self.assertEqual(main(["--root", str(root)]), 1)
            self.assertIn(
                "TC-021 is complete but has no executable test", stderr.getvalue()
            )

            backed_report = {
                "minted_targets": [
                    {"id": "TC-021", "target": "test-case", "backed": True}
                ]
            }
            stdout = io.StringIO()
            with mock.patch(
                "scripts.validate_matrix_status.run_quire_coverage",
                return_value=backed_report,
            ), contextlib.redirect_stdout(stdout):
                self.assertEqual(main(["--root", str(root)]), 0)
            self.assertIn("declared test symbol", stdout.getvalue())

    def test_main_ties_the_coverage_verdict_to_the_exit_code(self) -> None:
        """TC-021. Trace: TC-021, NFR-004-AC-5.

        Regression for finding 5: a rewrite that computes `failures` from
        `validate_documents`/`validate_criterion_citations` but forgets to
        route them into `main()`'s return value (or a mutant that clears
        `failures` before the `if failures:` check) must fail this test.
        Mutation transcript recorded in the PR discussion: setting
        `failures = []` immediately before the check turns this failure
        into a false pass, which is exactly the regression this asserts
        against.
        """
        matrix = """
| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-900 | invented row | Test | P0 | NFR-004 | ✅ implemented |
"""
        with tempfile.TemporaryDirectory(prefix="quire-matrix-status-") as directory:
            root = pathlib.Path(directory)
            (root / "spec/program").mkdir(parents=True)
            (root / "spec/test-matrix.md").write_text(matrix, encoding="utf-8")
            (root / "spec/contract-test-matrix.md").write_text("", encoding="utf-8")
            (root / "spec/program/PGM-01-governance.md").write_text("", encoding="utf-8")

            with mock.patch(
                "scripts.validate_matrix_status.run_quire_coverage",
                return_value={"minted_targets": []},
            ):
                stderr = io.StringIO()
                with contextlib.redirect_stderr(stderr):
                    exit_code = main(["--root", str(root)])
                self.assertEqual(exit_code, 1)
                self.assertIn(
                    "matrix status error: TC-900 is complete but has no executable "
                    "test",
                    stderr.getvalue(),
                )

    def test_executable_test_ids_reads_backed_test_case_targets(self) -> None:
        """TC-021, IR-202 (quire-contract-ir#157). Trace: TC-021, NFR-004-AC-5.

        `executable_test_ids` is the whole binding boundary now: it credits
        exactly the ids `quire coverage --json` marked as a backed
        `test-case` target, and nothing else — not an unbacked test-case, not
        a backed target of a different kind (an acceptance criterion can
        share an id shape's neighborhood but is never a test case), and not
        an id `quire` never minted at all.
        """
        report = {
            "minted_targets": [
                {"id": "TC-044", "target": "test-case", "backed": True},
                {"id": "TC-047", "target": "test-case", "backed": True},
                {"id": "TC-049", "target": "test-case", "backed": False},
                {"id": "FR-035-AC-1", "target": "acceptance-criterion", "backed": True},
            ]
        }
        self.assertEqual(executable_test_ids(report), {"TC-044", "TC-047"})
        self.assertEqual(executable_test_ids({"minted_targets": []}), set())
        self.assertEqual(executable_test_ids({}), set())

    def test_rejects_rows_that_omit_a_live_acceptance_criterion(self) -> None:
        """TC-021. Trace: TC-021, NFR-004-AC-5, NFR-004-AC-7."""
        # A row can cite fewer criteria than its requirement declares and stay
        # green under both coverage gates, because coverage checks the criteria
        # the row names. This is the class issue #33 describes.
        document = """
| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-099 | FR-099-AC-2 | TC-001 | ✅ covered |
"""
        declared = {"FR-099": ["FR-099-AC-1", "FR-099-AC-2", "FR-099-AC-3"]}
        self.assertEqual(
            validate_criterion_citations([document], declared),
            [
                "FR-099 omits live criterion FR-099-AC-1",
                "FR-099 omits live criterion FR-099-AC-3",
            ],
        )

        # Citing every live criterion passes, whether spelled out or ranged.
        for cell in ("FR-099-AC-1, FR-099-AC-2, FR-099-AC-3", "FR-099-AC-1 through FR-099-AC-3"):
            complete = f"""
| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-099 | {cell} | TC-001 | ✅ covered |
"""
            self.assertEqual(validate_criterion_citations([complete], declared), [])

        # A row naming a criterion the document no longer declares — a retired
        # or misspelled id — is a failure in the other direction.
        stale = """
| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-099 | FR-099-AC-1 through FR-099-AC-4 | TC-001 | ✅ covered |
"""
        self.assertEqual(
            validate_criterion_citations([stale], declared),
            ["FR-099 cites unknown or retired criterion FR-099-AC-4"],
        )

        # A table that verifies by method rather than by criterion id names no
        # criterion at all and is not treated as omitting every one of them.
        # The exemption is keyed on the row's own table — the heading above
        # it — not on the cell being empty.
        by_method = """
## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| FR-099 | inspection | TC-001 | ✅ covered |
"""
        self.assertEqual(validate_criterion_citations([by_method], declared), [])

    def test_rejects_functional_rows_that_cite_zero_criteria(self) -> None:
        """TC-021. Trace: TC-021, NFR-004-AC-5, NFR-004-AC-7.

        The exemption for the non-functional table (which cites no criterion
        id, by design) must not also swallow a *functional* row that cites
        zero criteria. Citing one of three live criteria already failed as an
        omission; citing none of three must fail at least as loudly, not pass
        by accident because the cited set happened to be empty.
        """
        declared = {"FR-099": ["FR-099-AC-1", "FR-099-AC-2", "FR-099-AC-3"]}
        no_citation = """
## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-099 | see design doc | TC-001 | ✅ covered |
"""
        self.assertEqual(
            validate_criterion_citations([no_citation], declared),
            [
                "FR-099 omits live criterion FR-099-AC-1",
                "FR-099 omits live criterion FR-099-AC-2",
                "FR-099 omits live criterion FR-099-AC-3",
            ],
        )

        # Unheaded rows (no section context at all) are functional-table
        # shaped too, and must not fall into the non-functional exemption by
        # default.
        no_heading = """
| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-099 | see design doc | TC-001 | ✅ covered |
"""
        self.assertEqual(
            validate_criterion_citations([no_heading], declared),
            [
                "FR-099 omits live criterion FR-099-AC-1",
                "FR-099 omits live criterion FR-099-AC-2",
                "FR-099 omits live criterion FR-099-AC-3",
            ],
        )

    def test_rejects_non_functional_rows_that_cite_a_retired_criterion(self) -> None:
        """TC-021. Trace: TC-021, NFR-004-AC-7.

        NFR-004-AC-7's omission exemption for the non-functional table is
        scoped to the naming direction only: such a row need not name a live
        criterion at all. It is not scoped away from the unknown-or-retired
        direction — a non-functional row that does cite a criterion id must
        still cite one that is live, the same as a functional row.
        """
        declared = {"NFR-004": ["NFR-004-AC-1"]}
        cites_retired_and_unknown = """
## Non-Functional Requirement Coverage

| Non-Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| NFR-004 | NFR-004-AC-4, NFR-004-AC-99 | TC-021 | OK |
"""
        self.assertEqual(
            validate_criterion_citations([cites_retired_and_unknown], declared),
            [
                "NFR-004 cites unknown or retired criterion NFR-004-AC-4",
                "NFR-004 cites unknown or retired criterion NFR-004-AC-99",
            ],
        )

    def test_retired_criteria_are_not_live(self) -> None:
        """TC-021. Trace: TC-021, NFR-004-AC-5."""
        # Every FR-001, FR-009 and FR-022 criterion under a `Retired criteria`
        # heading is withdrawn with a recorded reason. A matrix row that omits
        # them is correct, and the checker must not report it.
        declared = live_criteria()
        self.assertEqual(declared["FR-001"], ["FR-001-AC-2"])
        self.assertEqual(declared["FR-009"], ["FR-009-AC-2", "FR-009-AC-6"])
        self.assertEqual(
            declared["FR-022"],
            [
                "FR-022-AC-1",
                "FR-022-AC-2",
                "FR-022-AC-3",
                "FR-022-AC-5",
                "FR-022-AC-6",
            ],
        )
        self.assertEqual(
            cited_criteria("FR-022", "FR-022-AC-1 through FR-022-AC-3, FR-022-AC-5, FR-022-AC-6"),
            set(declared["FR-022"]),
        )

    def test_retired_heading_matches_case_and_trailing_text(self) -> None:
        """TC-021. Trace: TC-021, NFR-004-AC-5.

        Regression for finding 7: `RETIRED_HEADING` was exact-string and
        case-sensitive with no trailing text, so `### Retired Criteria` or
        `### Retired criteria (FR-100-AC-1)` failed to match and their
        criteria counted as live — which then fails a correct matrix row
        that correctly omits them.
        """
        for heading in (
            "### Retired Criteria",
            "#### retired criteria",
            "### Retired criteria (FR-100-AC-1)",
        ):
            document = f"""---
id: FR-100
---
# FR-100: test

## Acceptance Criteria

| FR-100-AC-2 | condition | Test |

{heading}

`FR-100-AC-1` retired for reasons stated here.

| FR-100-AC-1 | condition | Test |

## Dependencies
"""
            with tempfile.TemporaryDirectory(prefix="quire-matrix-status-") as directory:
                root = pathlib.Path(directory)
                (root / "spec/functional").mkdir(parents=True)
                (root / "spec/functional/FR-100-test.md").write_text(
                    document, encoding="utf-8"
                )
                declared = live_criteria(root)
                self.assertEqual(
                    declared["FR-100"], ["FR-100-AC-2"], f"heading {heading!r}"
                )

    def test_retired_section_exclusion_is_position_independent(self) -> None:
        """TC-021. Trace: TC-021, NFR-004-AC-5.

        Regression for finding 7: the old `split(document, maxsplit=1)[0]`
        dropped everything after the *first* retired heading, so a live
        criterion authored later in the document (here, after a
        `## Dependencies` section that follows the `### Retired criteria`
        subsection) vanished from `declared` even though it was never
        retired. Only the retired subsection's own body may be excluded:
        `FR-101-AC-1`'s row lives inside that body and must disappear, while
        `FR-101-AC-3`'s row lives past the subsection's end and must survive.
        """
        document = """---
id: FR-101
---
# FR-101: test

## Acceptance Criteria

| FR-101-AC-2 | condition | Test |

### Retired criteria

`FR-101-AC-1` retired for reasons stated here.

| FR-101-AC-1 | condition | Test |

## Dependencies

A later section, unrelated to the retired subsection, that happens to name
another live criterion in a table row.

| FR-101-AC-3 | condition | Test |
"""
        with tempfile.TemporaryDirectory(prefix="quire-matrix-status-") as directory:
            root = pathlib.Path(directory)
            (root / "spec/functional").mkdir(parents=True)
            (root / "spec/functional/FR-101-test.md").write_text(
                document, encoding="utf-8"
            )
            declared = live_criteria(root)
            self.assertEqual(
                declared["FR-101"], ["FR-101-AC-2", "FR-101-AC-3"]
            )

