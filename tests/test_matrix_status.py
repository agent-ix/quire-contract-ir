"""Tests for fail-closed matrix status classification."""

import contextlib
import io
import pathlib
import tempfile
import unittest

from scripts.validate_matrix_status import (
    cited_criteria,
    executable_tests,
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
        """TC-021. Trace: TC-021, NFR-004-AC-5."""
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
            (root / "tests").mkdir()
            (root / "spec/test-matrix.md").write_text(matrix, encoding="utf-8")
            (root / "spec/contract-test-matrix.md").write_text("", encoding="utf-8")
            (root / "spec/program/PGM-01-governance.md").write_text(
                policy, encoding="utf-8"
            )

            stderr = io.StringIO()
            with contextlib.redirect_stderr(stderr):
                self.assertEqual(main(["--root", str(root)]), 1)
            self.assertIn(
                "TC-021 is complete but has no executable test", stderr.getvalue()
            )

            (root / "tests/matrix.rs").write_text(
                "#[test]\nfn tc_021_matrix_status() {}\n", encoding="utf-8"
            )
            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                self.assertEqual(main(["--root", str(root)]), 0)
            self.assertIn("declared test symbol", stdout.getvalue())

    def test_rust_test_symbols_accept_either_attribute_order(self) -> None:
        """TC-021. Trace: TC-021, NFR-004-AC-5."""
        source = """
/// Tracing: TC-044
#[test]
#[trace("TC-044", "FR-035-AC-1")]
fn tc_044_test_before_trace() {}

#[trace("TC-047", "FR-038-AC-1")]
#[test]
fn tc_047_trace_before_test() {}

#[test]
fn tc_048_plain() {}

#[trace("TC-049", "FR-038-AC-6")]
fn tc_049_not_a_test() {}
"""
        with tempfile.TemporaryDirectory(prefix="quire-matrix-status-") as directory:
            root = pathlib.Path(directory)
            (root / "tests").mkdir()
            (root / "tests/traced.rs").write_text(source, encoding="utf-8")
            self.assertEqual(executable_tests(root), {"TC-044", "TC-047", "TC-048"})

    def test_rejects_rows_that_omit_a_live_acceptance_criterion(self) -> None:
        """TC-021. Trace: TC-021, NFR-004-AC-5."""
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
        by_method = """
| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| FR-099 | inspection | TC-001 | ✅ covered |
"""
        self.assertEqual(validate_criterion_citations([by_method], declared), [])

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

