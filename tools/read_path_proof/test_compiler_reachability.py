#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX
"""Adversarial controls compiled by the selected installed rustc, not AST mocks."""
import fcntl
import json
import os
from pathlib import Path
import subprocess
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

    def compile(self, name, source, *, thir=False):
        directory = self.output / name
        directory.mkdir(exist_ok=True)
        source_file = directory / "control.rs"
        source_file.write_text(source)
        command = [str(self.bin / "rustc"), str(source_file), "--crate-name", name,
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
                thir_command = [str(self.bin / "rustc"), str(source_file), "--crate-name", name,
                                "--crate-type", "lib", "--edition=2024", "-Cpanic=abort",
                                "-Zunpretty=thir-tree", "-Ztrim-diagnostic-paths=no", "-Zverbose-internals=yes"]
                proof.execute(["nice", "-n", "10", "ionice", "-c", "3"] + thir_command,
                              self.env, directory, directory / "compiler.thir")
        (directory / "invocation.json").write_text(json.dumps({
            "command": command, "lock_requested": requested, "lock_acquired": acquired,
        }, indent=2))
        return proof.analyze_artifacts([directory / (name + ".ll")],
            [directory / (name + ".mir")], [directory / (name + ".o")],
            self.llvm, self.env, "::reader_root")

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


if __name__ == "__main__":
    unittest.main()
