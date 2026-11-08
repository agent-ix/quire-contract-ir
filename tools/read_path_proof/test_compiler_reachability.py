#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX
"""Adversarial controls compiled by the selected installed rustc, not AST mocks."""
import fcntl
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest

import compiler_reachability as proof


class CompilerReachabilityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.bin = Path(os.environ["STATIC_PROOF_TOOLCHAIN_BIN"]).resolve(strict=True)
        cls.llvm = Path(os.environ["STATIC_PROOF_LLVM_BIN"]).resolve(strict=True)
        cls.output = Path(os.environ["STATIC_PROOF_TEST_OUT"]).resolve()
        cls.output.mkdir(parents=True, exist_ok=True)
        cls.env = proof.safe_environment(cls.bin)
        proof.lower_process_priority()

    def compile(self, name, source, *, thir=False, run_test=False, relative_source=False, analysis_options=None):
        directory = self.output / name
        directory.mkdir(exist_ok=True)
        source_file = directory / "control.rs"
        source_file.write_text(source)
        command = [str(self.bin / "rustc"), source_file.name if relative_source else str(source_file), "--crate-name", name,
                   "--crate-type", "lib", "--edition=2024", "--emit=llvm-ir,obj,mir",
                   "--out-dir", str(directory), "-Csymbol-mangling-version=v0",
                   "-Cdebuginfo=2", "-Copt-level=0", "-Ccodegen-units=1",
                   "-Cpanic=abort", "-Zmir-include-spans=yes", "-Zprint-mono-items=yes"]
        requested = proof.time.time()
        with open("/tmp/titan-cargo.lock", "a") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            acquired = proof.time.time()
            proof.execute(["nice", "-n", "10", "ionice", "-c", "3"] + command,
                          self.env, directory, directory / "compiler.log")
            if thir:
                thir_command = [str(self.bin / "rustc"), source_file.name if relative_source else str(source_file), "--crate-name", name,
                                "--crate-type", "lib", "--edition=2024", "-Cpanic=abort",
                                "-Zunpretty=thir-tree", "-Ztrim-diagnostic-paths=no", "-Zverbose-internals=yes"]
                proof.execute(["nice", "-n", "10", "ionice", "-c", "3"] + thir_command,
                              self.env, directory, directory / "compiler.thir")
            if run_test:
                executable = directory / "runtime-control"
                runtime_command = [str(self.bin / "rustc"), source_file.name if relative_source else str(source_file), "--crate-name", name,
                                   "--edition=2024", "--test", "-Cdebuginfo=0", "-o", str(executable)]
                proof.execute(["nice", "-n", "10", "ionice", "-c", "3"] + runtime_command,
                              self.env, directory, directory / "runtime-compiler.log")
                test_name = run_test if isinstance(run_test, str) else "valid_sequential_alias"
                observed = proof.execute([str(executable), "--exact", test_name, "--nocapture"],
                                         self.env, directory, directory / "runtime-control.log")
                if not re.search(r"test result: ok\. 1 passed; 0 failed;", observed):
                    raise proof.EvidenceError("runtime control did not execute exactly one value-asserting test")
        (directory / "invocation.json").write_text(json.dumps({
            "command": command, "lock_requested": requested, "lock_acquired": acquired,
        }, indent=2))
        with open("/tmp/titan-cargo.lock", "a") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            return proof.analyze_artifacts([directory / (name + ".ll")],
                [directory / (name + ".mir")], [directory / (name + ".o")],
                self.llvm, self.env, "::reader_root", source_root=directory, analysis_options=analysis_options)

    def test_monomorphic_callback_closure_drop_operator_and_finite_vtables(self):
        result = self.compile("finite", """
trait Read { fn read(&self,x:u8)->u8; }
struct A; struct B;
impl Read for A { fn read(&self,x:u8)->u8 {a(x)} }
impl Read for B { fn read(&self,x:u8)->u8 {b(x)} }
#[inline(never)] fn a(x:u8)->u8 {x}
#[inline(never)] fn b(x:u8)->u8 {x}
#[inline(never)] fn generic<T:Copy>(x:T)->T {x}
struct Guard;
impl Drop for Guard { fn drop(&mut self) { a(0); } }
struct Word(u8);
impl core::ops::Add for Word { type Output=Word; fn add(self,rhs:Self)->Self {Word(self.0 ^ rhs.0)} }
pub fn reader_root(x:u8)->u8 {
 let _guard=Guard;
 let target:&dyn Read=if x&1==0 {&A}else{&B};
 let callback:fn(u8)->u8=if x>5 {a}else{b};
 let closure=|value|generic(value);
 (Word(callback(x)) + Word(target.read(closure(x)))).0
}
pub fn unreachable_legacy(x:u8)->u8 { legacy(x) }
#[inline(never)] fn legacy(x:u8)->u8 {x}
""")
        paths = {item["def_path"] for item in result["functions"]}
        self.assertTrue(any("generic::<u8>" in path for path in paths), paths)
        self.assertFalse(any("legacy" in path for path in paths), paths)
        kinds = {edge["kind"] for edge in result["edges"]}
        self.assertTrue({"call", "drop", "operator", "closure", "vtable"} <= kinds, kinds)
        names = {item["instance"]: item["def_path"] for item in result["functions"]}
        virtual = {names[edge["callee_instance"]] for edge in result["edges"] if edge["kind"] == "vtable"}
        self.assertEqual(len(virtual), 2, virtual)
        self.assertTrue(any("finite::A" in item for item in virtual), virtual)
        self.assertTrue(any("finite::B" in item for item in virtual), virtual)
        for edge in result["edges"]:
            self.assertIsNotNone(edge["source_span"], edge)

    def test_opaque_callback_is_not_guessed_from_compatible_functions(self):
        result = self.compile("opaque", """
pub fn reader_root(callback:fn(u8)->u8,x:u8)->u8 {callback(x)}
pub fn compatible(x:u8)->u8 {x}
""")
        self.assertFalse(result["result"]["static_ac513_proven"])
        self.assertTrue(any("opaque root argument" in item["reason"] for item in result["unresolved"]), result)
        self.assertFalse(any("compatible" in item["def_path"] for item in result["functions"]))

    def test_external_callback_body_missing_fails_closed(self):
        result = self.compile("external", """
unsafe extern "C" { fn unknown_native(x:u8)->u8; }
pub fn reader_root(x:u8)->u8 { unsafe {unknown_native(x)} }
""")
        self.assertFalse(result["result"]["static_ac513_proven"])
        self.assertTrue(any("external body unavailable" in item["reason"] for item in result["unresolved"]), result)

    def test_complete_local_scalar_closure_has_real_body_and_edge_evidence(self):
        result = self.compile("scalar", """
#[inline(never)] fn leaf(x:u8)->u8 {x ^ 7}
pub fn reader_root(x:u8)->u8 {leaf(x)}
""")
        self.assertEqual(result["unresolved"], [])
        self.assertTrue(result["result"]["pointer_flow_fixed_point"])
        self.assertEqual(len(result["edges"]), 1)
        self.assertTrue(all(item["source_span"] and item["mir_artifact"] for item in result["functions"]))

    def test_reachable_forbidden_map_is_not_hidden_by_monomorphization(self):
        result = self.compile("forbidden", """
pub fn reader_root()->std::collections::BTreeMap<u8,u8> {std::collections::BTreeMap::new()}
""")
        self.assertFalse(result["result"]["static_ac513_proven"])
        self.assertTrue(result["violations"] or any("forbidden-type" in item["reason"] for item in result["unresolved"]), result)

    def test_actual_unstable_sort_implementation_is_not_misclassified(self):
        result = self.compile("unstable", """
pub fn reader_root(values:&mut [u8]) {values.sort_unstable();}
""")
        self.assertEqual(result["violations"], [])
        self.assertTrue(any("core::slice::sort::unstable::" in item["def_path"] for item in result["functions"]))

    def test_actual_stable_sort_remains_a_forbidden_api(self):
        result = self.compile("stable", """
pub fn reader_root(values:&mut [u8]) {values.sort();}
""")
        self.assertTrue(any(item["reason"] == "sort other than sort_unstable" for item in result["violations"]), result)

    def test_borrowed_caller_evidence_map_lookup_is_not_construction(self):
        result = self.compile("borrowed_map", """
pub fn reader_root(map:&std::collections::BTreeMap<u8,u8>,key:&u8)->Option<u8> {
 map.get(key).copied()
}
""")
        self.assertEqual(result["violations"], [])
        self.assertTrue(any("BTreeMap" in item["def_path"] and "::get" in item["def_path"]
                            for item in result["functions"]))

    def test_aliased_default_map_constructor_is_still_construction(self):
        result = self.compile("aliased_map", """
type Evidence=std::collections::BTreeMap<u8,u8>;
pub fn reader_root()->Evidence {Evidence::default()}
""")
        self.assertTrue(any(item["reason"] == "tree/hash/serde map construction" for item in result["violations"]), result)

    def test_same_named_local_type_is_not_the_forbidden_library_type(self):
        result = self.compile("same_named", """
mod benign { #[derive(Default)] pub struct BTreeMap; }
type Alias=benign::BTreeMap;
pub fn reader_root()->Alias {Alias::default()}
""")
        self.assertEqual(result["violations"], [])
        self.assertTrue(any("same_named::benign::BTreeMap" in item["def_path"] for item in result["functions"]))

    def test_thir_resolves_aliases_and_distinguishes_local_same_named_module(self):
        self.compile("qualified_types", """
mod benign { pub enum Value { Null } }
mod serde_json { pub enum Value { Null } }
type Alias=benign::Value;
pub fn reader_root()->Alias {Alias::Null}
pub fn local_same_name()->serde_json::Value {serde_json::Value::Null}
""", thir=True)
        evidence = proof.thir_aggregate_evidence(self.output / "qualified_types/compiler.thir")
        self.assertEqual({item["qualified_adt"] for item in evidence},
                         {"qualified_types::benign::Value", "qualified_types::serde_json::Value"})
        self.assertTrue(all(item["source_span"] and item["owner_def_id"] for item in evidence))
        self.assertTrue(all(item["violation"] is None for item in evidence))

    def test_unknown_raw_store_can_alias_a_known_function_cell(self):
        result = self.compile("raw_alias", """
fn initial(x:u8)->u8 {x ^ 1}
fn other(x:u8)->u8 {x ^ 128}
static mut TARGET:fn(u8)->u8=initial;
pub unsafe fn reader_root(target:*mut fn(u8)->u8,x:u8)->u8 {
 unsafe { *target=other; TARGET(x) }
}
#[test]
fn valid_sequential_alias() {
 assert_eq!(unsafe {reader_root(&raw mut TARGET,3)},131);
}
""", run_test=True)
        self.assertFalse(result["result"]["static_ac513_proven"])
        self.assertTrue(any("unknown-address store" in item["reason"] for item in result["unresolved"]), result)

    def test_callee_return_delta_preserves_both_selected_targets(self):
        result = self.compile("returned_callback", """
#[inline(never)] fn a(x:u8)->u8 {x ^ 1}
#[inline(never)] fn b(x:u8)->u8 {x ^ 128}
#[inline(never)] fn select(flag:bool)->fn(u8)->u8 {if flag {a} else {b}}
pub fn reader_root(flag:bool,x:u8)->u8 {select(flag)(x)}
""")
        self.assertTrue(result["result"]["pointer_flow_fixed_point"], result["result"])
        targets = {item["def_path"] for item in result["functions"]}
        self.assertTrue({"returned_callback::a", "returned_callback::b"} <= targets, targets)

    def test_object_local_pointer_iteration_converges_without_unrelated_targets(self):
        result = self.compile("pointer_iteration", """
#[inline(never)] fn a(x:u8)->u8 {x ^ 1}
#[inline(never)] fn b(x:u8)->u8 {x ^ 128}
#[inline(never)] fn unrelated(x:u8)->u8 {x ^ 42}
static CALLBACKS:[fn(u8)->u8;2]=[a,b];
pub static UNRELATED:[fn(u8)->u8;1]=[unrelated];
pub fn reader_root(count:usize,x:u8)->u8 {
 if count>CALLBACKS.len() {return x;}
 let mut result=x;
 let mut cursor=CALLBACKS.as_ptr();
 let end=cursor.wrapping_add(count);
 while cursor!=end {
  let callback=unsafe {*cursor};
  result=callback(result);
  cursor=cursor.wrapping_add(1);
 }
 result
}
#[test] fn valid_pointer_iteration() {
 assert_eq!(reader_root(0,3),3);
 assert_eq!(reader_root(1,3),2);
 assert_eq!(reader_root(2,3),130);
}
""", run_test="valid_pointer_iteration")
        self.assertTrue(result["result"]["pointer_flow_fixed_point"], result["result"])
        self.assertGreater(result["result"]["widened_object_offsets"], 0)
        targets = {item["def_path"] for item in result["functions"]}
        self.assertTrue({"pointer_iteration::a", "pointer_iteration::b"} <= targets, targets)
        self.assertNotIn("pointer_iteration::unrelated", targets)

    # Trace: TC-472, FR-038-AC-513
    def test_widening_keeps_a_callback_beyond_the_exact_offset_window(self):
        callbacks = ",".join(["first"] * 32 + ["last"])
        result = self.compile("wide_callback_tail", """
#[inline(never)] fn first(x:u8)->u8 {x ^ 1}
#[inline(never)] fn last(x:u8)->u8 {x ^ 128}
#[inline(never)] fn unrelated(x:u8)->u8 {x ^ 42}
pub static UNRELATED:fn(u8)->u8=unrelated;
static CALLBACKS:[fn(u8)->u8;33]=[CALLBACK_ENTRIES];
pub fn reader_root(count:usize,x:u8)->u8 {
 if count>CALLBACKS.len() {return x;}
 let mut result=x;
 let mut cursor=CALLBACKS.as_ptr();
 let end=cursor.wrapping_add(count);
 while cursor!=end {
  result=unsafe {(*cursor)(result)};
  cursor=cursor.wrapping_add(1);
 }
 result
}
#[test] fn tail_value() {assert_eq!(reader_root(33,3),131);}
""".replace("CALLBACK_ENTRIES", callbacks), run_test="tail_value")
        self.assertTrue(result["result"]["pointer_flow_fixed_point"], result["result"])
        self.assertGreater(result["result"]["widened_object_offsets"], 0)
        paths = {item["def_path"] for item in result["functions"]}
        self.assertIn("wide_callback_tail::last", paths)
        self.assertNotIn("wide_callback_tail::unrelated", paths)

    # Trace: TC-472, FR-038-AC-513
    def test_nested_generic_and_relative_source_have_compiler_mir_correspondence(self):
        result = self.compile("nested_generic", """
#[inline(never)] fn generic<T>(value:T)->T {value}
pub fn reader_root(x:u8)->Option<Option<u8>> {generic(Some(Some(x)))}
""", relative_source=True)
        generic = [item for item in result["functions"] if "nested_generic::generic::<" in item["def_path"]]
        self.assertEqual(len(generic), 1, result["functions"])
        self.assertIsNotNone(generic[0]["mir_artifact"], generic)
        self.assertEqual(generic[0]["mir_resolution"]["status"], "unique_source_body")

    # Trace: TC-472, FR-038-AC-513
    def test_same_leaf_in_one_source_is_not_assigned_an_arbitrary_mir_body(self):
        result = self.compile("ambiguous_leaf", """
mod a {#[inline(never)] pub fn leaf(x:u8)->u8 {x ^ 1}}
mod b {#[inline(never)] pub fn leaf(x:u8)->u8 {x ^ 128}}
pub fn reader_root(flag:bool,x:u8)->u8 {if flag {a::leaf(x)} else {b::leaf(x)}}
""")
        leaves = [item for item in result["functions"] if item["def_path"].endswith("::leaf")]
        self.assertEqual(len(leaves), 2)
        self.assertTrue(all(item["mir_artifact"] is None for item in leaves), leaves)
        self.assertTrue(all(item["mir_resolution"]["status"] == "ambiguous_source_body" for item in leaves))
        self.assertFalse(result["result"]["static_ac513_proven"])

    # Trace: TC-472, FR-038-AC-513
    def test_visit_limit_preserves_pending_work_and_names_the_exhausted_bound(self):
        result = self.compile("visit_limit", """
#[inline(never)] fn leaf(x:u8)->u8 {x ^ 7}
pub fn reader_root(x:u8)->u8 {leaf(x)}
""", analysis_options={"max_visits": 1})
        self.assertEqual(result["result"]["termination"], "visit_bound")
        self.assertEqual(result["result"]["dirty_function_visits"], 1)
        self.assertGreater(result["result"]["pending_dirty_functions"], 0)
        self.assertFalse(result["result"]["pointer_flow_fixed_point"])
        self.assertTrue(any("visit_bound" in item["reason"] for item in result["unresolved"]))

    # Trace: TC-472, FR-038-AC-513
    def test_memory_limit_preserves_unknown_pending_root(self):
        result = self.compile("memory_limit", """
pub fn reader_root(x:u8)->u8 {x ^ 7}
""", analysis_options={"max_resident_bytes": 1})
        self.assertEqual(result["result"]["termination"], "memory_bound")
        self.assertEqual(result["result"]["dirty_function_visits"], 0)
        self.assertEqual(result["result"]["pending_dirty_functions"], 1)
        self.assertFalse(result["result"]["pointer_flow_fixed_point"])
        self.assertTrue(any("memory_bound" in item["reason"] for item in result["unresolved"]))

    # Trace: TC-472, FR-038-AC-513
    def test_compiler_memory_copy_preserves_callback_targets(self):
        result = self.compile("copied_callbacks", """
#[inline(never)] fn initial(x:u8)->u8 {x ^ 1}
#[inline(never)] fn copied(x:u8)->u8 {x ^ 128}
#[inline(never)] fn unrelated(x:u8)->u8 {x ^ 42}
pub static UNRELATED:fn(u8)->u8=unrelated;
pub fn reader_root(x:u8)->u8 {
 let source:[fn(u8)->u8;1]=[copied];
 let mut destination:[fn(u8)->u8;1]=[initial];
 unsafe {core::ptr::copy_nonoverlapping(source.as_ptr(),destination.as_mut_ptr(),1);}
 destination[0](x)
}
#[test] fn copied_value() {assert_eq!(reader_root(7),135);}
""", run_test="copied_value")
        self.assertTrue(result["result"]["pointer_flow_fixed_point"], result["result"])
        paths = {item["def_path"] for item in result["functions"]}
        self.assertIn("copied_callbacks::copied", paths)
        self.assertNotIn("copied_callbacks::unrelated", paths)
        self.assertFalse(any("memory intrinsic effects not represented" in item["reason"]
                             and "llvm.memcpy" in item["reason"] for item in result["unresolved"]))

    # Trace: TC-472, FR-038-AC-513
    def test_dynamic_opaque_copy_remains_unresolved(self):
        result = self.compile("opaque_copy", """
#[inline(never)] fn initial(x:u8)->u8 {x ^ 1}
pub unsafe fn reader_root(source:*const fn(u8)->u8,count:usize,x:u8)->u8 {
 let mut destination:[fn(u8)->u8;1]=[initial];
 unsafe {core::ptr::copy_nonoverlapping(source,destination.as_mut_ptr(),count);}
 destination[0](x)
}
""")
        self.assertFalse(result["result"]["static_ac513_proven"])
        self.assertTrue(any("memory copy extent" in item["reason"] or "memory copy addresses" in item["reason"]
                            for item in result["unresolved"]), result)

    def test_recursive_memory_delta_reaches_the_new_global_target(self):
        result = self.compile("recursive_memory", """
#[inline(never)] fn initial(x:u8)->u8 {x ^ 1}
#[inline(never)] fn changed(x:u8)->u8 {x ^ 128}
static mut CELL:fn(u8)->u8=initial;
#[inline(never)] unsafe fn fill(slot:*mut fn(u8)->u8,depth:u8) {
 if depth==0 {unsafe {*slot=changed;}} else {unsafe {fill(slot,depth-1);}}
}
pub fn reader_root(depth:u8,x:u8)->u8 {
 unsafe {fill(&raw mut CELL,depth); CELL(x)}
}
#[test] fn valid_recursive_memory() {assert_eq!(reader_root(5,7),135);}
""", run_test="valid_recursive_memory")
        self.assertTrue(result["result"]["pointer_flow_fixed_point"], result["result"])
        self.assertTrue(any(item["def_path"] == "recursive_memory::changed" for item in result["functions"]))


class ArtifactGuardsTests(unittest.TestCase):
    def test_resolved_target_rejects_a_real_symlink_into_another_target(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source, tool, other = root / "source", root / "tool", root / "other"
            source.mkdir()
            tool.mkdir()
            other.mkdir()
            sentinel = other / "preserved"
            sentinel.write_bytes(b"other lane cache")
            (source / "target").symlink_to(other, target_is_directory=True)
            with self.assertRaisesRegex(proof.EvidenceError, "escapes"):
                proof.owned_target(source / "target/proof", source, tool)
            self.assertEqual(sentinel.read_bytes(), b"other lane cache")
            self.assertEqual(proof.owned_target(tool / "target/proof", source, tool), tool / "target/proof")

    def test_existing_report_is_never_overwritten_even_on_cli_error(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            report = proof.reserve_report(root / "report")
            saved = report / "compiler-reachability.json"
            saved.write_bytes(b"previous immutable evidence\n")
            result = subprocess.run([sys.executable, str(Path(proof.__file__).resolve()),
                                     "--source-root", str(root), "--out-dir", str(report),
                                     "--toolchain-bin", "/usr/bin", "--llvm-bin", "/usr/bin"],
                                    capture_output=True, text=True, check=False)
            self.assertEqual(result.returncode, 2)
            self.assertFalse(json.loads(result.stderr)["report_written"])
            self.assertEqual(saved.read_bytes(), b"previous immutable evidence\n")


if __name__ == "__main__":
    unittest.main()
