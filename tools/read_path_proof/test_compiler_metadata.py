#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX
"""TC-472 / FR-038-AC-513: actual selected-compiler identity controls.

This is a bounded metadata feasibility check, never production closure credit.
Every invocation and supplied-metadata cohort remains in the owned output root.
"""
import fcntl
import json
import os
from pathlib import Path
import resource
import subprocess
import tempfile
import time
import unittest


class CompilerMetadataTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.bin = Path(os.environ["STATIC_PROOF_TOOLCHAIN_BIN"]).resolve(strict=True)
        cls.output = Path(os.environ["STATIC_PROOF_METADATA_OUT"]).resolve()
        owned = Path(__file__).resolve().parents[2] / "target"
        if not cls.output.is_relative_to(owned):
            raise ValueError("metadata controls output must stay inside the owning tool target")
        cls.output.mkdir(parents=True, exist_ok=True)
        cls.env = {key: os.environ[key] for key in ("HOME", "PATH", "TMPDIR") if key in os.environ}
        cls.env.update({"RUSTC_BOOTSTRAP": "1", "CARGO_BUILD_JOBS": "2", "CARGO_INCREMENTAL": "0"})
        cls.env["LD_LIBRARY_PATH"] = str(cls.bin.parent / "lib") + ":" + str(
            cls.bin.parent / "lib/rustlib/x86_64-unknown-linux-gnu/lib")
        cls.driver = cls.output / "compiler-metadata"
        cls.invoke("driver-build", [str(cls.bin / "rustc"), str(Path(__file__).with_name("compiler_metadata.rs")),
                   "--edition=2021", "-Cprefer-dynamic=yes", "-Cdebuginfo=0", "-Copt-level=0", "-Ccodegen-units=2", "-o", str(cls.driver)])
        cls.version = cls.invoke("selected-rustc", [str(cls.bin / "rustc"), "-vV"])[1]
        cls.cfg = cls.invoke("selected-cfg", [str(cls.bin / "rustc"), "--print=cfg"])[1]

    @classmethod
    def invoke(cls, label, command, *, env=None, expected=0):
        directory = cls.output / label
        directory.mkdir(exist_ok=True)
        invocation_env = env or cls.env
        requested = time.time()
        with open("/tmp/titan-cargo.lock", "a") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            acquired = time.time()
            # All compiler children share the selected thermal priority. Bounds
            # cover frontend metadata decoding too, not merely our callback.
            def limits():
                resource.setrlimit(resource.RLIMIT_AS, (2 * 1024**3, 2 * 1024**3))
                resource.setrlimit(resource.RLIMIT_FSIZE, (8 * 1024**2, 8 * 1024**2))
            with tempfile.TemporaryFile() as captured:
                result = subprocess.run(["nice", "-n", "10", "ionice", "-c", "3"] + command,
                                        env=invocation_env, stdout=captured, stderr=subprocess.STDOUT,
                                        timeout=120, preexec_fn=limits)
                captured.seek(0)
                output = captured.read(8 * 1024**2 + 1).decode("utf-8", errors="strict")
        (directory / "compiler.log").write_text(output)
        (directory / "invocation.json").write_text(json.dumps({
            "command": command, "exit": result.returncode, "lock_requested": requested,
            "lock_acquired": acquired, "timeout_seconds": 120, "address_space_limit": 2 * 1024**3,
            "output_limit": 8 * 1024**2, "environment": invocation_env,
            "purpose": "selected compiler metadata feasibility, not production reachability",
        }, indent=2) + "\n")
        if result.returncode != expected:
            raise AssertionError(f"{label}: exit {result.returncode}, expected {expected}; {output[-4000:]}")
        if len(output.encode()) > 8 * 1024**2:
            raise AssertionError("compiler output cap exceeded")
        return result.returncode, output

    def inspect(self, name, source, *, roots, externs=(), extra_env=None, expected=0, cohort=None):
        directory = self.output / name
        directory.mkdir(exist_ok=True)
        source_path = directory / "input.rs"
        source_path.write_text(source)
        command = [str(self.driver), str(source_path), "--crate-name", name, "--crate-type", "lib",
                   "--edition=2021", "--sysroot", str(self.bin.parent), "--emit=metadata",
                   "--out-dir", str(directory)]
        for crate, artifact in externs:
            path = Path(artifact).resolve(strict=True)
            command += ["--extern", crate + "=" + str(path), "-L", "dependency=" + str(path.parent)]
        env = dict(self.env)
        env["AC513_METADATA_ROOTS"] = "\n".join(roots)
        env.update(extra_env or {})
        _, output = self.invoke(name, command, env=env, expected=expected)
        records = [json.loads(line) for line in output.splitlines()]
        self.assertEqual(records[-1]["kind"], "summary")
        self.assertFalse(records[-1]["closure_proven"])
        if expected == 0:
            self.assertTrue((directory / ("lib" + name + ".rmeta")).is_file(), "successful exporter preserves requested metadata emission")
        (directory / "cohort.json").write_text(json.dumps({
            "rustc": self.version, "cfg": self.cfg, "supplied_metadata": list(externs),
            "cohort": cohort or "tiny selected-compiler control, not production root",
            "driver_scope": records[-1]["scope"],
        }, indent=2) + "\n")
        return records

    # Trace: TC-472 / FR-038-AC-513
    def test_same_leaf_calls_keep_distinct_compiler_def_ids(self):
        records = self.inspect("distinct", """
mod first { pub fn same() {} }
mod second { pub fn same() {} }
pub fn reader_root() { first::same(); second::same(); }
""", roots=("reader_root",))
        calls = [r for r in records if r["kind"] == "call" and r["caller_def_path"] == "reader_root"]
        self.assertEqual(len(calls), 2)
        self.assertEqual({r["resolved_def_path"] for r in calls}, {"first::same", "second::same"})
        self.assertEqual(len({r["resolved_def_id"] for r in calls}), 2)
        self.assertTrue(records[-1]["complete"])

    # Trace: TC-472 / FR-038-AC-513
    def test_allocator_trait_resolves_actual_implementation_and_mir_status(self):
        records = self.inspect("allocator", """
pub fn allocated(cap: usize) -> Vec<u8> { Vec::with_capacity(cap) }
pub unsafe fn system(layout: std::alloc::Layout) -> *mut u8 {
    // SAFETY: the caller supplies a valid Layout; the pointer is returned untouched.
    unsafe { std::alloc::GlobalAlloc::alloc(&std::alloc::System, layout) }
}
extern "C" { fn unknown_native(); }
pub unsafe fn foreign() {
    // Compiled as metadata only; this deliberately unknown native call is never executed.
    unsafe { unknown_native(); }
}
""", roots=("allocated", "system", "foreign"))
        calls = {r["caller_def_path"]: r for r in records if r["kind"] == "call"}
        allocator = calls["system"]
        self.assertNotEqual(allocator["declaration_def_id"], allocator["resolved_def_id"])
        self.assertIn("std::alloc::GlobalAlloc::alloc", allocator["declaration_def_path"])
        self.assertIn("std::sys::alloc::unix", allocator["resolved_def_path"])
        self.assertIn("library/std/src/sys/alloc/unix.rs", allocator["source"])
        self.assertTrue(allocator["symbol"])
        self.assertEqual(calls["allocated"]["is_mir_available"], "true")
        self.assertEqual(calls["allocated"]["body_exported"], "true")
        self.assertTrue(records[-1]["complete"])
        self.assertEqual(calls["foreign"]["is_mir_available"], "false")
        self.assertEqual(calls["foreign"]["body_exported"], "false")
        self.assertIn("absent", calls["foreign"]["mir_status"])

    # Trace: TC-472 / FR-038-AC-513
    def test_shadowed_aggregate_names_are_compiler_qualified(self):
        records = self.inspect("aggregates", """
mod first { pub enum Value { Null } }
mod second { pub enum Value { Null } }
pub fn one() -> first::Value { first::Value::Null }
pub fn two() -> second::Value { second::Value::Null }
pub mod legacy { pub enum Value { Null } pub fn old() -> Value { Value::Null } }
""", roots=("one", "two"))
        adts = [r for r in records if r["kind"] in ("adt", "adt_constant")]
        self.assertEqual({r["def_path"] for r in adts}, {"first::Value", "second::Value"})
        self.assertEqual(len({r["def_id"] for r in adts}), 2)
        self.assertTrue(all(r["phase"] == "built_local" for r in adts))

    # Trace: TC-472 / FR-038-AC-513
    def test_target_bound_retains_partial_records_and_reports_incomplete(self):
        records = self.inspect("bounded", """
mod first { pub fn call() {} }
mod second { pub fn call() {} }
pub fn root() { first::call(); second::call(); }
""", roots=("root",), extra_env={"AC513_METADATA_MAX_TARGETS": "1"}, expected=2)
        self.assertFalse(records[-1]["complete"])
        self.assertIn("TargetBound", records[-1]["failure"])
        self.assertEqual(len([r for r in records if r["kind"] == "call"]), 1)

    # Trace: TC-472 / FR-038-AC-513
    def test_actual_supplied_sdk_metadata_keeps_absent_mir_open(self):
        sdk = os.environ["STATIC_PROOF_SDK_META"]
        model = os.environ["STATIC_PROOF_MODEL_META"]
        cohort = os.environ["STATIC_PROOF_METADATA_COHORT"]
        records = self.inspect("actual_metadata", """
extern crate quire_exact;
extern crate quire_contract_model;
pub fn reserved(text: &str, r: &mut dyn quire_exact::Reservation)
 -> Result<quire_exact::Integer, quire_exact::IntegerParseError> {
    quire_exact::Integer::parse_reserved(text, r)
}
pub fn one() -> quire_exact::Integer { quire_exact::Integer::one() }
pub fn reader(bytes: &[u8], limits: quire_contract_model::CheckedPackageReadLimits,
 evidence: &quire_contract_model::CheckedPackageEvidence,
 budget: &mut quire_contract_model::NativeJoinBudget)
 -> quire_contract_model::CheckedPackageV2BudgetedReadResult {
    quire_contract_model::CheckedPackageV2::read_with_budget(bytes, limits, evidence, budget)
}
""", roots=("reserved", "one", "reader"), externs=(("quire_exact", sdk), ("quire_contract_model", model)), cohort=cohort)
        calls = {r["caller_def_path"]: r for r in records if r["kind"] == "call"}
        self.assertEqual(calls["reserved"]["is_mir_available"], "true")
        self.assertTrue(any(r["kind"] == "adt" and "quire_exact" in r["def_path"] for r in records))
        self.assertEqual(calls["one"]["is_mir_available"], "false")
        self.assertEqual(calls["reader"]["is_mir_available"], "false")
        self.assertIn("absent", calls["reader"]["mir_status"])
        self.assertFalse(records[-1]["closure_proven"])


if __name__ == "__main__":
    unittest.main()
