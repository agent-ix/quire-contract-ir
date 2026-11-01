// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-070: model certificate members against QSpec's published schema.

use super::*;
use std::collections::BTreeSet;

const CERTIFICATE_DEFINITIONS: [&str; 7] = [
    "ProbabilityCertificate",
    "ZoneCertificate",
    "CertificateIdentity",
    "DbmBound",
    "ComponentReason",
    "RefinementPart",
    "ProbabilityProductKey",
];

fn certificate_properties(schema: &Value, names: &mut BTreeSet<String>) -> Result<(), String> {
    let object = schema
        .as_object()
        .ok_or("certificate schema node must be an object")?;
    if ![
        "$ref",
        "properties",
        "items",
        "oneOf",
        "anyOf",
        "allOf",
        "type",
        "const",
        "enum",
    ]
    .iter()
    .any(|keyword| object.contains_key(*keyword))
    {
        return Err("certificate schema node has no schema keyword".into());
    }
    if let Some(reference) = object.get("$ref") {
        reference
            .as_str()
            .ok_or("certificate schema $ref must be a string")?;
    }
    if let Some(properties) = object.get("properties") {
        let properties = properties
            .as_object()
            .ok_or("certificate schema properties must be an object")?;
        for (name, value) in properties {
            names.insert(name.clone());
            certificate_properties(value, names)?;
        }
    }
    if let Some(items) = object.get("items") {
        certificate_properties(items, names)?;
    }
    for alternative in ["oneOf", "anyOf", "allOf"] {
        if let Some(branches) = object.get(alternative) {
            for branch in branches
                .as_array()
                .ok_or_else(|| format!("certificate schema {alternative} must be an array"))?
            {
                certificate_properties(branch, names)?;
            }
        }
    }
    Ok(())
}

fn malformed_member_enum(schema: &Value) -> Result<Vec<String>, String> {
    let branches = schema["$defs"]["CertificateRejected"]["oneOf"]
        .as_array()
        .ok_or("CertificateRejected.oneOf is missing or malformed")?;
    let matches = branches
        .iter()
        .filter(|branch| {
            branch["properties"]["rule"]["const"] == "malformed"
                && branch["properties"]["at"]["properties"]["member"].is_object()
        })
        .collect::<Vec<_>>();
    let [branch] = matches.as_slice() else {
        return Err(format!(
            "CertificateRejected must have exactly one malformed member branch; found {}",
            matches.len()
        ));
    };
    let values = branch["properties"]["at"]["properties"]["member"]["enum"]
        .as_array()
        .ok_or("malformed member enum is missing or malformed")?;
    if values.is_empty() {
        return Err("malformed member enum is empty".into());
    }
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| "malformed member enum contains a non-string value".into())
        })
        .collect()
}

#[trace("FR-070-AC-1", "FR-070-AC-2", "FR-070-AC-3", "FR-070-AC-4")]
#[test]
fn model_certificate_members_match_qspec_schema() {
    let root = std::env::var(SPECIFICATION_DIR)
        .ok()
        .filter(|root| !root.is_empty())
        .unwrap_or_else(|| panic!("{SPECIFICATION_DIR} must name a quire-specification checkout"));
    let path = Path::new(&root).join("proposals/backend-provider-v1/schema.json");
    let schema = read_json(&path).unwrap_or_else(|error| panic!("{error}"));
    let definitions = schema["$defs"]
        .as_object()
        .expect("backend-provider schema must contain $defs");

    let mut properties = BTreeSet::new();
    for name in CERTIFICATE_DEFINITIONS {
        let definition = definitions
            .get(name)
            .unwrap_or_else(|| panic!("backend-provider schema lacks $defs.{name}"));
        certificate_properties(definition, &mut properties)
            .unwrap_or_else(|error| panic!("$defs.{name}: {error}"));
    }

    let published = malformed_member_enum(&schema).unwrap_or_else(|error| panic!("{error}"));
    let published_set = published.iter().cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        published_set.len(),
        published.len(),
        "duplicate schema member"
    );
    assert_eq!(
        published_set, properties,
        "certificate property closure differs"
    );

    let model = quire_contract_model::CertificateMember::ALL
        .iter()
        .map(|member| member.as_str())
        .collect::<Vec<_>>();
    let model_set = model
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(model_set.len(), model.len(), "duplicate model member");
    assert_eq!(model_set, published_set, "model member set differs");
    assert_eq!(model, published, "model member sequence differs");

    fn assert_derives<T: Clone + Copy + std::fmt::Debug + Eq + PartialEq>() {}
    assert_derives::<quire_contract_model::CertificateMember>();
}
