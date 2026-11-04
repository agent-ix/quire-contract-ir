// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original selected-model Port fixture shared by native and QSpec reader controls.

use super::{
    admitted_protocol_families, family_key, family_model_document, node_id, protocol_role_node,
    settle,
};
use serde_json::{json, Value};

pub fn role_port_fixture() -> (Value, Value, usize) {
    let mut package = admitted_protocol_families();
    let mut role = protocol_role_node();
    role["body"]["arguments"][0]["value"]["target"] = node_id(&family_key("1515"));
    let role_id = role["node_id"].clone();
    let position = package["semantic_graph"]["nodes"].as_array().unwrap().len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(role);
    let clause = package["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| node["semantic_form"] == "protocol_clause")
        .unwrap();
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]["members"] =
        json!([{"term":"reference", "target":role_id}]);
    let mut document = family_model_document();
    for (name, meaning) in [
        ("port", "port"),
        ("part", "part"),
        ("connection", "connection"),
    ] {
        document["constructs"].as_array_mut().unwrap().push(json!({"kind":{"module":"test/families", "name":name}, "construct":{"meaning":format!("quire.meaning.systems.{meaning}/v1")}}));
    }
    for (name, kind) in [
        ("PumpOutlet", "port"),
        ("PumpInlet", "port"),
        ("PortNamedPart", "part"),
        ("PortNamedConnection", "connection"),
    ] {
        document["types"].as_array_mut().unwrap().push(json!({"identity":format!("ix://test/families/{name}"), "displayName":name, "kind":{"module":"test/families", "name":kind}, "roles":[], "constraints":[], "extensions":[], "unknownPolicy":"reject"}));
    }
    let text = |value| json!({"term":"literal", "type":node_id(&family_key("a1a1")), "value_kind":"text", "value":value});
    package["semantic_graph"]["nodes"][position]["body"]["arguments"].as_array_mut().unwrap().push(json!({"term":"binding", "name":"model_member", "value":{"term":"aggregate", "members":[{"term":"binding", "name":"package", "value":text("test/families")}, {"term":"binding", "name":"node", "value":text("ix://test/families/PumpOutlet")}]}}));
    settle(&mut package);
    (package, document, position)
}
