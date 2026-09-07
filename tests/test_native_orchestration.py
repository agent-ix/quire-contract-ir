"""Exercise GNU Make scheduling/error propagation, not substitute domain verdicts."""

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
# Each native command is a required lane. The doubles below test orchestration
# only; the real tools still execute in make ci and own all domain results.
LANES = (
    ("cargo", "fmt"),
    ("cargo", "clippy"),
    ("python", "-m", "unittest"),
    ("cargo", "test", "--", "--include-ignored"),
    ("cargo", "run"),
    ("cargo", "build", "--quiet"),
    ("python", "scripts/generate_conformance_corpus.py", "--check"),
    ("quire", "validate"),
    ("quire", "coverage", "--scope", ".", "--strict"),
    ("python", "scripts/validate_matrix_status.py"),
    ("rustup", "run", "1.75.0"),
    ("cargo", "deny", "check", "licenses"),
    ("bash", "scripts/check_unsafe_comments.sh"),
    ("python", "scripts/check_shared_pins.py"),
    ("quire", "coverage", "--scope", ".", "--json"),
    ("python", "scripts/assurance_chain.py"),
)


def exercise(makefile: str, failing: tuple[str, ...] = (), target: str = "ci"):
    with tempfile.TemporaryDirectory(prefix="quire-native-orchestration-") as name:
        root = Path(name)
        (root / "Makefile").write_text(makefile, encoding="utf-8")
        binary = root / "bin"
        binary.mkdir()
        # Actual subprocess exits are observed by GNU Make. There is no verdict
        # on stdout/stderr and no assurance/evidence output is invented.
        double = f"#!{sys.executable}\n" + '''
import json, os, sys
from pathlib import Path
invocation = [Path(sys.argv[0]).name, *sys.argv[1:]]
with open(os.environ["INVOCATIONS"], "a", encoding="utf-8") as stream:
    stream.write(json.dumps(invocation) + "\\n")
failure = json.loads(os.environ["FAIL_LANE"])
sys.exit(7 if failure and invocation[:len(failure)] == failure else 0)
'''
        for tool in ("cargo", "python", "quire", "quoin", "rustup", "bash"):
            path = binary / tool
            path.write_text(double, encoding="utf-8")
            path.chmod(0o755)
        log = root / "invocations.jsonl"
        environment = {
            **os.environ,
            "PATH": f"{binary}{os.pathsep}{os.environ['PATH']}",
            "INVOCATIONS": str(log),
            "FAIL_LANE": json.dumps(failing),
            "MAKEFLAGS": "",
            "MFLAGS": "",
            "MAKEOVERRIDES": "",
        }
        result = subprocess.run(
            ["make", "--no-print-directory", target, "REVISION=test-candidate",
             f"CARGO={binary / 'cargo'}", f"PYTHON={binary / 'python'}",
             f"QUIRE={binary / 'quire'}", f"QUOIN={binary / 'quoin'}",
             f"ASSURANCE_PYTHON={binary / 'python'}"],
            cwd=root, env=environment, capture_output=True, check=False,
        )
        calls = [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []
        return result.returncode, calls


def missing_lanes(calls):
    return [lane for lane in LANES if not any(tuple(call[:len(lane)]) == lane for call in calls)]


class NativeOrchestrationTests(unittest.TestCase):
    def test_every_gate_is_invoked_and_every_native_failure_propagates(self) -> None:
        """TC-021. Trace: NFR-004-AC-6."""
        makefile = (ROOT / "Makefile").read_text()
        for target in ("ci", "release-check"):
            status, calls = exercise(makefile, target=target)
            self.assertEqual(status, 0)
            self.assertEqual(missing_lanes(calls), [])
        for lane in LANES:
            with self.subTest(lane=lane):
                status, calls = exercise(makefile, lane)
                self.assertTrue(any(tuple(call[:len(lane)]) == lane for call in calls))
                self.assertNotEqual(status, 0)

    def test_negative_make_controls_are_detected(self) -> None:
        """TC-021. Trace: NFR-004-AC-6."""
        makefile = (ROOT / "Makefile").read_text()
        # Suppressed native failure, including global and recipe-local forms.
        for mutated in (
            ".IGNORE:\n" + makefile,
            makefile.replace("\t$(CARGO) fmt --all -- --check", "\t-$(CARGO) fmt --all -- --check"),
            makefile.replace("\t$(CARGO) fmt --all -- --check", "\t$(CARGO) fmt --all -- --check || true"),
        ):
            status, _ = exercise(mutated, ("cargo", "fmt"))
            self.assertEqual(status, 0, "control must reproduce swallowed failure")
        # Removing the corpus-reproduction dependency must create a missing lane.
        mutated = makefile.replace("test corpus corpus-repro spec", "test corpus spec")
        self.assertNotEqual(mutated, makefile)
        status, calls = exercise(mutated)
        self.assertEqual(status, 0)
        self.assertIn(("python", "scripts/generate_conformance_corpus.py", "--check"), missing_lanes(calls))

    def test_registry_commands_match_the_native_interfaces(self) -> None:
        """TC-021. Trace: NFR-004-AC-6."""
        registry = (ROOT / "spec/evidence/suites.md").read_text()
        makefile = (ROOT / "Makefile").read_text()
        suite = next(line for line in registry.splitlines() if line.startswith("| SUITE-001 |"))
        self.assertTrue(suite.endswith("| Integration |"))
        self.assertIn("--kind Integration", makefile)
        self.assertNotIn("planning/**/*.md", registry)
        chain = next(line for line in registry.splitlines() if line.startswith("| SUITE-004 |"))
        self.assertIn("--conformance target/assurance/conformance.jsonl", chain)
        self.assertIn("--quire-export target/assurance/quire-static-export.json", chain)
        self.assertIn("not release qualification", (ROOT / ".github/workflows/ci.yml").read_text())
