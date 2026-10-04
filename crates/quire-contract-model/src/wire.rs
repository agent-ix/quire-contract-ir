use serde::Deserialize;
use serde_json::{json, Value};

use crate::decimal::IntegerString;
use crate::expression::{INTEGER_BOUNDS_PATH, RATIONAL_BOUNDS_PATH};
use crate::{
    conformance::{canonical_value, diagnostics_value},
    identity::{WireRequirementRef, WireSourceSpan},
    limits::{MAX_SEMANTIC_COLLECTION_ITEMS, MAX_SEMANTIC_DEPTH, MAX_SEMANTIC_NODES},
    BooleanOperator, CanonicalProfile, CollectionType, ComparisonOperator, DeclarationEnvironment,
    Diagnostic, DiagnosticCode, EnumDeclaration, EnumVariantDeclaration, ExecutionPoint,
    Expression, ExpressionKind, FunctionParameter, IntegerDomain, IntegerType, NumericOperator,
    OverflowPolicy, PureFunctionDeclaration, QuantifierDomain, QuantifierKind, RationalType,
    RecordDeclaration, RecordFieldDeclaration, RecordLiteralField, RequirementRef,
    StateObservation, SymbolName, TypeDeclaration, ValueDeclaration, ValueDeclarationKind,
    ValueType,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpressionInput {
    owner: WireRequirementRef,
    #[serde(default)]
    types: Vec<WireTypeDeclaration>,
    #[serde(default)]
    values: Vec<WireValueDeclaration>,
    #[serde(default)]
    functions: Vec<WireFunctionDeclaration>,
    expression: WireExpression,
    expected_type: WireValueType,
    execution_point: ExecutionPoint,
    clause_root: bool,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum WireValueType {
    Boolean,
    Integer {
        domain: IntegerDomain,
        minimum: IntegerString,
        maximum: IntegerString,
        overflow: OverflowPolicy,
    },
    Rational {
        numerator_minimum: IntegerString,
        numerator_maximum: IntegerString,
        maximum_denominator: IntegerString,
    },
    Text,
    Enum {
        name: String,
    },
    Record {
        name: String,
    },
    Option {
        value: Box<WireValueType>,
    },
    Collection {
        element: Box<WireValueType>,
        maximum_items: u64,
    },
}

impl WireValueType {
    fn validate(self) -> Result<ValueType, Diagnostic> {
        match self {
            Self::Boolean => Ok(ValueType::Boolean),
            Self::Integer {
                domain,
                minimum,
                maximum,
                overflow,
            } => IntegerType::new(
                domain,
                minimum.to_i64(INTEGER_BOUNDS_PATH)?,
                maximum.to_i64(INTEGER_BOUNDS_PATH)?,
                overflow,
            )
            .map(ValueType::integer),
            Self::Rational {
                numerator_minimum,
                numerator_maximum,
                maximum_denominator,
            } => RationalType::new(
                numerator_minimum.to_i64(RATIONAL_BOUNDS_PATH)?,
                numerator_maximum.to_i64(RATIONAL_BOUNDS_PATH)?,
                maximum_denominator.to_u64(RATIONAL_BOUNDS_PATH)?,
            )
            .map(ValueType::rational),
            Self::Text => Ok(ValueType::Text),
            Self::Enum { name } => Ok(ValueType::Enum {
                name: SymbolName::new(name)?,
            }),
            Self::Record { name } => Ok(ValueType::Record {
                name: SymbolName::new(name)?,
            }),
            Self::Option { value } => Ok(ValueType::option(value.validate()?)),
            Self::Collection {
                element,
                maximum_items,
            } => {
                let maximum_items = u32::try_from(maximum_items).map_err(|_| {
                    Diagnostic::error(
                        DiagnosticCode::InvalidNumericBounds,
                        "collection maximum exceeds unsigned 32-bit range",
                        "type.collection.maximum_items",
                    )
                })?;
                CollectionType::new(element.validate()?, maximum_items).map(ValueType::collection)
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum WireTypeDeclaration {
    Enum {
        name: String,
        source: WireSourceSpan,
        variants: Vec<WireEnumVariant>,
    },
    Record {
        name: String,
        source: WireSourceSpan,
        fields: Vec<WireRecordField>,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireEnumVariant {
    name: String,
    source: WireSourceSpan,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireRecordField {
    name: String,
    value_type: WireValueType,
    source: WireSourceSpan,
}

impl WireTypeDeclaration {
    fn validate(self) -> Result<TypeDeclaration, Vec<Diagnostic>> {
        match self {
            Self::Enum {
                name,
                source,
                variants,
            } => EnumDeclaration::new(
                one(SymbolName::new(name))?,
                one(source.validate())?,
                variants
                    .into_iter()
                    .map(|variant| {
                        Ok(EnumVariantDeclaration::new(
                            SymbolName::new(variant.name)?,
                            variant.source.validate()?,
                        ))
                    })
                    .collect::<Result<Vec<_>, Diagnostic>>()
                    .map_err(|diagnostic| vec![diagnostic])?,
            )
            .map(|declaration| TypeDeclaration::Enum { declaration }),
            Self::Record {
                name,
                source,
                fields,
            } => RecordDeclaration::new(
                one(SymbolName::new(name))?,
                one(source.validate())?,
                fields
                    .into_iter()
                    .map(|field| {
                        Ok(RecordFieldDeclaration::new(
                            SymbolName::new(field.name)?,
                            field.value_type.validate()?,
                            field.source.validate()?,
                        ))
                    })
                    .collect::<Result<Vec<_>, Diagnostic>>()
                    .map_err(|diagnostic| vec![diagnostic])?,
            )
            .map(|declaration| TypeDeclaration::Record { declaration }),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireValueDeclaration {
    name: String,
    kind: ValueDeclarationKind,
    value_type: WireValueType,
    source: WireSourceSpan,
}

impl WireValueDeclaration {
    fn validate(self) -> Result<ValueDeclaration, Diagnostic> {
        Ok(ValueDeclaration::new(
            SymbolName::new(self.name)?,
            self.kind,
            self.value_type.validate()?,
            self.source.validate()?,
        ))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireFunctionParameter {
    name: String,
    value_type: WireValueType,
    source: WireSourceSpan,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireFunctionDeclaration {
    name: String,
    parameters: Vec<WireFunctionParameter>,
    result_type: WireValueType,
    source: WireSourceSpan,
}

impl WireFunctionDeclaration {
    fn validate(self) -> Result<PureFunctionDeclaration, Vec<Diagnostic>> {
        let parameters = self
            .parameters
            .into_iter()
            .map(|parameter| {
                Ok(FunctionParameter::new(
                    SymbolName::new(parameter.name)?,
                    parameter.value_type.validate()?,
                    parameter.source.validate()?,
                ))
            })
            .collect::<Result<Vec<_>, Diagnostic>>()
            .map_err(|diagnostic| vec![diagnostic])?;
        PureFunctionDeclaration::new(
            one(SymbolName::new(self.name))?,
            parameters,
            one(self.result_type.validate())?,
            one(self.source.validate())?,
        )
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireRecordLiteralField {
    name: String,
    value: WireExpression,
}

#[derive(Deserialize)]
#[serde(tag = "node", rename_all = "snake_case")]
enum WireExpressionKind {
    BooleanLiteral {
        value: bool,
    },
    IntegerLiteral {
        value: IntegerString,
        value_type: WireValueType,
    },
    RationalLiteral {
        numerator: IntegerString,
        denominator: IntegerString,
        value_type: WireValueType,
    },
    TextLiteral {
        value: String,
    },
    EnumLiteral {
        enumeration: String,
        variant: String,
    },
    OptionNone {
        value_type: WireValueType,
    },
    OptionSome {
        value_type: WireValueType,
        value: Box<WireExpression>,
    },
    RecordLiteral {
        record: String,
        fields: Vec<WireRecordLiteralField>,
    },
    CollectionLiteral {
        value_type: WireValueType,
        items: Vec<WireExpression>,
    },
    ValueReference {
        name: String,
        observation: StateObservation,
    },
    LocalReference {
        name: String,
    },
    FieldAccess {
        base: Box<WireExpression>,
        field: String,
    },
    IsPresent {
        option: Box<WireExpression>,
    },
    Unwrap {
        option: Box<WireExpression>,
    },
    Length {
        collection: Box<WireExpression>,
    },
    Index {
        collection: Box<WireExpression>,
        index: Box<WireExpression>,
    },
    Call {
        function: String,
        arguments: Vec<WireExpression>,
    },
    Numeric {
        operator: NumericOperator,
        left: Box<WireExpression>,
        right: Box<WireExpression>,
    },
    NumericNegate {
        operand: Box<WireExpression>,
    },
    Compare {
        operator: ComparisonOperator,
        left: Box<WireExpression>,
        right: Box<WireExpression>,
    },
    BooleanNot {
        operand: Box<WireExpression>,
    },
    Boolean {
        operator: BooleanOperator,
        left: Box<WireExpression>,
        right: Box<WireExpression>,
    },
    Quantifier {
        quantifier: QuantifierKind,
        domain: QuantifierDomain,
        collection: Box<WireExpression>,
        local: String,
        local_source: WireSourceSpan,
        predicate: Box<WireExpression>,
    },
}

#[derive(Deserialize)]
struct WireExpression {
    #[serde(flatten)]
    kind: WireExpressionKind,
    source: WireSourceSpan,
}

impl WireExpression {
    fn validate(self) -> Result<Expression, Diagnostic> {
        let kind = match self.kind {
            WireExpressionKind::BooleanLiteral { value } => {
                ExpressionKind::BooleanLiteral { value }
            }
            WireExpressionKind::IntegerLiteral { value, value_type } => {
                let ValueType::Integer { value: value_type } = value_type.validate()? else {
                    return Err(wire_type_error("expression.integer_literal.value_type"));
                };
                ExpressionKind::IntegerLiteral {
                    value: value.to_i64("expression.integer_literal.value")?,
                    value_type,
                }
            }
            WireExpressionKind::RationalLiteral {
                numerator,
                denominator,
                value_type,
            } => {
                let ValueType::Rational { value: value_type } = value_type.validate()? else {
                    return Err(wire_type_error("expression.rational_literal.value_type"));
                };
                ExpressionKind::RationalLiteral {
                    numerator: numerator.to_i64("expression.rational_literal.numerator")?,
                    denominator: denominator.to_i64("expression.rational_literal.denominator")?,
                    value_type,
                }
            }
            WireExpressionKind::TextLiteral { value } => ExpressionKind::TextLiteral { value },
            WireExpressionKind::EnumLiteral {
                enumeration,
                variant,
            } => ExpressionKind::EnumLiteral {
                enumeration: SymbolName::new(enumeration)?,
                variant: SymbolName::new(variant)?,
            },
            WireExpressionKind::OptionNone { value_type } => ExpressionKind::OptionNone {
                value_type: value_type.validate()?,
            },
            WireExpressionKind::OptionSome { value_type, value } => ExpressionKind::OptionSome {
                value_type: value_type.validate()?,
                value: Box::new(value.validate()?),
            },
            WireExpressionKind::RecordLiteral { record, fields } => ExpressionKind::RecordLiteral {
                record: SymbolName::new(record)?,
                fields: fields
                    .into_iter()
                    .map(|field| {
                        Ok(RecordLiteralField::new(
                            SymbolName::new(field.name)?,
                            field.value.validate()?,
                        ))
                    })
                    .collect::<Result<Vec<_>, Diagnostic>>()?,
            },
            WireExpressionKind::CollectionLiteral { value_type, items } => {
                let ValueType::Collection { value: value_type } = value_type.validate()? else {
                    return Err(wire_type_error("expression.collection_literal.value_type"));
                };
                ExpressionKind::CollectionLiteral {
                    value_type,
                    items: items
                        .into_iter()
                        .map(Self::validate)
                        .collect::<Result<Vec<_>, _>>()?,
                }
            }
            WireExpressionKind::ValueReference { name, observation } => {
                ExpressionKind::ValueReference {
                    name: SymbolName::new(name)?,
                    observation,
                }
            }
            WireExpressionKind::LocalReference { name } => ExpressionKind::LocalReference {
                name: SymbolName::new(name)?,
            },
            WireExpressionKind::FieldAccess { base, field } => ExpressionKind::FieldAccess {
                base: Box::new(base.validate()?),
                field: SymbolName::new(field)?,
            },
            WireExpressionKind::IsPresent { option } => ExpressionKind::IsPresent {
                option: Box::new(option.validate()?),
            },
            WireExpressionKind::Unwrap { option } => ExpressionKind::Unwrap {
                option: Box::new(option.validate()?),
            },
            WireExpressionKind::Length { collection } => ExpressionKind::Length {
                collection: Box::new(collection.validate()?),
            },
            WireExpressionKind::Index { collection, index } => ExpressionKind::Index {
                collection: Box::new(collection.validate()?),
                index: Box::new(index.validate()?),
            },
            WireExpressionKind::Call {
                function,
                arguments,
            } => ExpressionKind::Call {
                function: SymbolName::new(function)?,
                arguments: arguments
                    .into_iter()
                    .map(Self::validate)
                    .collect::<Result<Vec<_>, _>>()?,
            },
            WireExpressionKind::Numeric {
                operator,
                left,
                right,
            } => ExpressionKind::Numeric {
                operator,
                left: Box::new(left.validate()?),
                right: Box::new(right.validate()?),
            },
            WireExpressionKind::NumericNegate { operand } => ExpressionKind::NumericNegate {
                operand: Box::new(operand.validate()?),
            },
            WireExpressionKind::Compare {
                operator,
                left,
                right,
            } => ExpressionKind::Compare {
                operator,
                left: Box::new(left.validate()?),
                right: Box::new(right.validate()?),
            },
            WireExpressionKind::BooleanNot { operand } => ExpressionKind::BooleanNot {
                operand: Box::new(operand.validate()?),
            },
            WireExpressionKind::Boolean {
                operator,
                left,
                right,
            } => ExpressionKind::Boolean {
                operator,
                left: Box::new(left.validate()?),
                right: Box::new(right.validate()?),
            },
            WireExpressionKind::Quantifier {
                quantifier,
                domain,
                collection,
                local,
                local_source,
                predicate,
            } => ExpressionKind::Quantifier {
                quantifier,
                domain,
                collection: Box::new(collection.validate()?),
                local: SymbolName::new(local)?,
                local_source: local_source.validate()?,
                predicate: Box::new(predicate.validate()?),
            },
        };
        Ok(Expression::new(kind, self.source.validate()?))
    }

    fn children<'a>(
        &'a self,
        output: &mut Vec<(&'a WireExpression, u32)>,
        depth: u32,
    ) -> Result<(), Diagnostic> {
        match &self.kind {
            WireExpressionKind::OptionSome { value, .. } => output.push((value, depth)),
            WireExpressionKind::RecordLiteral { fields, .. } => {
                add_collection(fields.len(), "expression.record.fields")?;
                output.extend(fields.iter().map(|field| (&field.value, depth)));
            }
            WireExpressionKind::CollectionLiteral { items, .. } => {
                add_collection(items.len(), "expression.collection.items")?;
                output.extend(items.iter().map(|item| (item, depth)));
            }
            WireExpressionKind::Call { arguments, .. } => {
                add_collection(arguments.len(), "expression.call.arguments")?;
                output.extend(arguments.iter().map(|argument| (argument, depth)));
            }
            WireExpressionKind::FieldAccess { base, .. }
            | WireExpressionKind::IsPresent { option: base }
            | WireExpressionKind::Unwrap { option: base }
            | WireExpressionKind::Length { collection: base }
            | WireExpressionKind::NumericNegate { operand: base }
            | WireExpressionKind::BooleanNot { operand: base } => output.push((base, depth)),
            WireExpressionKind::Index { collection, index }
            | WireExpressionKind::Numeric {
                left: collection,
                right: index,
                ..
            }
            | WireExpressionKind::Compare {
                left: collection,
                right: index,
                ..
            }
            | WireExpressionKind::Boolean {
                left: collection,
                right: index,
                ..
            } => {
                output.push((collection, depth));
                output.push((index, depth));
            }
            WireExpressionKind::Quantifier {
                collection,
                predicate,
                ..
            } => {
                output.push((collection, depth));
                output.push((predicate, depth));
            }
            _ => {}
        }
        Ok(())
    }

    fn value_types<'a>(&'a self, output: &mut Vec<(&'a WireValueType, u32)>) {
        let value_type = match &self.kind {
            WireExpressionKind::IntegerLiteral { value_type, .. }
            | WireExpressionKind::RationalLiteral { value_type, .. }
            | WireExpressionKind::OptionNone { value_type }
            | WireExpressionKind::OptionSome { value_type, .. }
            | WireExpressionKind::CollectionLiteral { value_type, .. } => Some(value_type),
            _ => None,
        };
        if let Some(value_type) = value_type {
            output.push((value_type, 1));
        }
    }
}

pub(crate) struct CheckedExpression {
    pub environment: DeclarationEnvironment,
    pub expression: crate::TypedExpression,
    pub semantic_nodes: u32,
}

pub(crate) fn check_expression_input(
    input: Value,
    binding: Option<(&RequirementRef, &ExecutionPoint)>,
) -> Result<CheckedExpression, Vec<Diagnostic>> {
    let request: ExpressionInput =
        serde_json::from_value(input).map_err(|_| vec![wire_type_error("expression")])?;
    let semantic_nodes = one(preflight(&request))?;
    let request_owner = one(request.owner.validate())?;
    if let Some((owner, anchor)) = binding {
        if &request_owner != owner {
            return Err(vec![Diagnostic::error(
                DiagnosticCode::MalformedReference,
                "expression declaration owner differs from the bound clause owner",
                "expression.owner",
            )]);
        }
        if &request.execution_point != anchor {
            return Err(vec![Diagnostic::error(
                DiagnosticCode::IncompatibleClauseAnchor,
                "expression execution point differs from the bound clause anchor",
                "expression.execution_point",
            )]);
        }
        if !request.clause_root || !matches!(&request.expected_type, WireValueType::Boolean) {
            return Err(vec![Diagnostic::error(
                DiagnosticCode::NonBooleanClauseRoot,
                "a bound executable expression requires a Boolean clause root",
                "expression.clause_root",
            )]);
        }
    }
    let types = request
        .types
        .into_iter()
        .map(WireTypeDeclaration::validate)
        .collect::<Result<Vec<_>, _>>()?;
    let values = one(request
        .values
        .into_iter()
        .map(WireValueDeclaration::validate)
        .collect::<Result<Vec<_>, _>>())?;
    let functions = request
        .functions
        .into_iter()
        .map(WireFunctionDeclaration::validate)
        .collect::<Result<Vec<_>, _>>()?;
    let environment = DeclarationEnvironment::new(request_owner, types, values, functions)?;
    let expression = one(request.expression.validate())?;
    let expected = one(request.expected_type.validate())?;
    let expression = environment.check_expression(
        &expression,
        &expected,
        &request.execution_point,
        request.clause_root,
    )?;
    Ok(CheckedExpression {
        environment,
        expression,
        semantic_nodes,
    })
}

pub(crate) fn execute_expression(input: Value) -> Value {
    // A `document_json` input hands the decoder the exact text, so a number in
    // one of the eight integer members reaches the member's own type instead
    // of being refused by a schema first.
    let input = match input.get("document_json").and_then(Value::as_str) {
        Some(document) => match parse_document(document) {
            Ok(value) => value,
            Err(diagnostic) => return invalid(vec![diagnostic]),
        },
        None => input,
    };
    let CheckedExpression {
        environment,
        expression: typed,
        ..
    } = match check_expression_input(input, None) {
        Ok(checked) => checked,
        Err(diagnostics) => return invalid(diagnostics),
    };
    let declaration = match environment.canonical_declaration(CanonicalProfile::V1) {
        Ok(output) => output,
        Err(diagnostic) => return invalid(vec![diagnostic]),
    };
    let expression = match typed.canonical_expression(CanonicalProfile::V1) {
        Ok(output) => output,
        Err(diagnostic) => return invalid(vec![diagnostic]),
    };
    let canonical = match [
        canonical_value("declaration", declaration),
        canonical_value("expression", expression),
    ]
    .into_iter()
    .collect::<Result<Vec<_>, _>>()
    {
        Ok(canonical) => canonical,
        Err(diagnostic) => return invalid(vec![diagnostic]),
    };
    json!({
        "valid": true,
        "diagnostics": [],
        "canonical": canonical,
        "dependencies": typed.dependencies(),
    })
}

/// The JSON of a `document_json` input, refused as `invalid_wire_format` when
/// it nests past the wire depth limit or is not JSON.
fn parse_document(document: &str) -> Result<Value, Diagnostic> {
    if crate::limits::json_nesting_exceeds(document.as_bytes(), crate::MAX_WIRE_JSON_DEPTH) {
        return Err(Diagnostic::error(
            DiagnosticCode::InvalidWireFormat,
            "JSON nesting exceeds decode limit",
            "document.nesting",
        ));
    }
    crate::identity::parse_json_stack_safe(document).map_err(|error| {
        Diagnostic::error(
            DiagnosticCode::InvalidWireFormat,
            error.to_string(),
            "document",
        )
    })
}

fn invalid(diagnostics: Vec<Diagnostic>) -> Value {
    json!({
        "valid": false,
        "diagnostics": diagnostics_value(&diagnostics),
        "canonical": [],
        "dependencies": [],
    })
}

fn wire_type_error(path: &'static str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::InvalidWireFormat,
        "expression wire value has an invalid shape",
        path,
    )
}

fn one<T>(result: Result<T, Diagnostic>) -> Result<T, Vec<Diagnostic>> {
    result.map_err(|diagnostic| vec![diagnostic])
}

fn preflight(request: &ExpressionInput) -> Result<u32, Diagnostic> {
    if [
        request.types.len(),
        request.values.len(),
        request.functions.len(),
    ]
    .into_iter()
    .any(|length| length > MAX_SEMANTIC_COLLECTION_ITEMS as usize)
    {
        return Err(too_large("declarations"));
    }
    let mut nodes = 0_u32;
    let mut types = vec![(&request.expected_type, 1_u32)];
    for declaration in &request.types {
        match declaration {
            WireTypeDeclaration::Enum { variants, .. } => {
                add_collection(variants.len(), "types.enum.variants")?;
                add_nodes(&mut nodes, variants.len() as u32 + 1, "types")?;
            }
            WireTypeDeclaration::Record { fields, .. } => {
                add_collection(fields.len(), "types.record.fields")?;
                add_nodes(&mut nodes, fields.len() as u32 + 1, "types")?;
                types.extend(fields.iter().map(|field| (&field.value_type, 1_u32)));
            }
        }
    }
    for value in &request.values {
        add_nodes(&mut nodes, 1, "values")?;
        types.push((&value.value_type, 1));
    }
    for function in &request.functions {
        add_collection(function.parameters.len(), "functions.parameters")?;
        add_nodes(
            &mut nodes,
            function.parameters.len() as u32 + 1,
            "functions",
        )?;
        types.push((&function.result_type, 1));
        types.extend(
            function
                .parameters
                .iter()
                .map(|parameter| (&parameter.value_type, 1)),
        );
    }
    let mut expressions = vec![(&request.expression, 1_u32)];
    while let Some((expression, depth)) = expressions.pop() {
        if depth > MAX_SEMANTIC_DEPTH {
            return Err(too_large("expression.depth"));
        }
        add_nodes(&mut nodes, 1, "expression.nodes")?;
        expression.value_types(&mut types);
        expression.children(&mut expressions, depth + 1)?;
    }
    while let Some((value_type, depth)) = types.pop() {
        if depth > MAX_SEMANTIC_DEPTH {
            return Err(too_large("type.depth"));
        }
        add_nodes(&mut nodes, 1, "type.nodes")?;
        match value_type {
            WireValueType::Option { value } => types.push((value, depth + 1)),
            WireValueType::Collection { element, .. } => types.push((element, depth + 1)),
            _ => {}
        }
    }
    Ok(nodes)
}

fn add_collection(length: usize, path: &'static str) -> Result<(), Diagnostic> {
    if length > MAX_SEMANTIC_COLLECTION_ITEMS as usize {
        Err(too_large(path))
    } else {
        Ok(())
    }
}

fn add_nodes(nodes: &mut u32, additional: u32, path: &'static str) -> Result<(), Diagnostic> {
    *nodes = nodes.saturating_add(additional);
    if *nodes > MAX_SEMANTIC_NODES {
        Err(too_large(path))
    } else {
        Ok(())
    }
}

fn too_large(path: &'static str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::SemanticInputTooLarge,
        "semantic input exceeds a fixed validation limit",
        path,
    )
}

#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;
    use serde_json::json;

    use super::*;

    const WIDEST_INTEGER: &str = "9223372036854775807";
    const NARROWEST_INTEGER: &str = "-9223372036854775808";

    fn span() -> Value {
        json!({
            "start": {"source": {"document": "doc", "revision": 1}, "line": 1, "column": 1,
                      "byte_offset": 0},
            "end": {"source": {"document": "doc", "revision": 1}, "line": 1, "column": 2,
                    "byte_offset": 1},
        })
    }

    fn integer_type() -> Value {
        json!({"kind": "integer", "domain": "signed", "minimum": NARROWEST_INTEGER,
               "maximum": WIDEST_INTEGER, "overflow": "reject"})
    }

    fn rational_type() -> Value {
        json!({"kind": "rational", "numerator_minimum": NARROWEST_INTEGER,
               "numerator_maximum": WIDEST_INTEGER, "maximum_denominator": WIDEST_INTEGER})
    }

    fn request(expression: Value, expected_type: Value) -> Value {
        json!({
            "owner": {"package": "agent-ix/pkg", "requirement": "REQ_a", "revision": 1},
            "types": [], "values": [], "functions": [],
            "expression": expression,
            "expected_type": expected_type,
            "execution_point": {"kind": "pre", "operation": "check"},
            "clause_root": false,
        })
    }

    fn integer_literal(value: &str) -> Value {
        json!({"node": "integer_literal", "value": value, "value_type": integer_type(),
               "source": span()})
    }

    fn rational_literal(numerator: &str, denominator: &str) -> Value {
        json!({"node": "rational_literal", "numerator": numerator, "denominator": denominator,
               "value_type": rational_type(), "source": span()})
    }

    /// The eight members, each with a request that is valid until the member
    /// is replaced, and the JSON pointer of the member.
    fn members() -> Vec<(&'static str, Value, &'static str)> {
        let rational_expected = || request(rational_literal("-3", "4"), rational_type());
        let integer_expected = || request(integer_literal("0"), integer_type());
        vec![
            (
                "IntegerType.minimum",
                integer_expected(),
                "/expected_type/minimum",
            ),
            (
                "IntegerType.maximum",
                integer_expected(),
                "/expected_type/maximum",
            ),
            (
                "RationalType.numerator_minimum",
                rational_expected(),
                "/expected_type/numerator_minimum",
            ),
            (
                "RationalType.numerator_maximum",
                rational_expected(),
                "/expected_type/numerator_maximum",
            ),
            (
                "RationalType.maximum_denominator",
                rational_expected(),
                "/expected_type/maximum_denominator",
            ),
            (
                "IntegerLiteral.value",
                integer_expected(),
                "/expression/value",
            ),
            (
                "RationalLiteral.numerator",
                rational_expected(),
                "/expression/numerator",
            ),
            (
                "RationalLiteral.denominator",
                rational_expected(),
                "/expression/denominator",
            ),
        ]
    }

    fn with_member(base: &Value, pointer: &str, member: Value) -> Value {
        let mut changed = base.clone();
        *changed.pointer_mut(pointer).expect("the member exists") = member;
        changed
    }

    fn refusal(input: Value) -> Diagnostic {
        let mut diagnostics = match check_expression_input(input, None) {
            Ok(_) => panic!("the input must be refused"),
            Err(diagnostics) => diagnostics,
        };
        assert_eq!(diagnostics.len(), 1);
        diagnostics.remove(0)
    }

    const REVISION_BOUND: u64 = 9_007_199_254_740_992;

    /// The expression operation's output for `input`: validity, then the code
    /// and path of its one diagnostic when refused.
    fn operation_refusal(input: Value) -> (String, String) {
        let output = execute_expression(input);
        assert_eq!(output["valid"], false);
        let diagnostics = output["diagnostics"].as_array().expect("diagnostics");
        assert_eq!(diagnostics.len(), 1);
        (
            diagnostics[0]["code"].as_str().expect("code").to_owned(),
            diagnostics[0]["path"].as_str().expect("path").to_owned(),
        )
    }

    /// A revision or byte offset above 2^53 (and a zero revision) in the
    /// owner or in any span member reaches the expression operation as the
    /// constructor's registered code and path, not `invalid_wire_format`;
    /// 2^53 itself is admitted.
    ///
    /// Tracing: TC-015, FR-011-AC-3, FR-012-AC-6.
    #[trace("TC-015", "FR-011-AC-3", "FR-012-AC-6")]
    #[test]
    fn tc_015_the_expression_operation_refuses_a_revision_or_offset_above_two_to_the_53_with_its_registered_code(
    ) {
        let base = request(integer_literal("0"), integer_type());
        let refused =
            |pointer: &str, member: Value| operation_refusal(with_member(&base, pointer, member));
        let admitted = |pointer: &str, member: Value| {
            let output = execute_expression(with_member(&base, pointer, member));
            assert_eq!(output["valid"], true, "{pointer}: {output}");
        };
        admitted("/owner/revision", json!(REVISION_BOUND));
        admitted("/expression/source/end/byte_offset", json!(REVISION_BOUND));
        let mut both_ends = base.clone();
        for end in ["start", "end"] {
            both_ends["expression"]["source"][end]["source"]["revision"] = json!(REVISION_BOUND);
        }
        assert_eq!(execute_expression(both_ends)["valid"], true);
        assert_eq!(
            refused("/owner/revision", json!(REVISION_BOUND + 1)),
            (
                "invalid_requirement_revision".to_owned(),
                "reference.revision".to_owned()
            )
        );
        assert_eq!(
            refused("/owner/revision", json!(0)),
            (
                "invalid_requirement_revision".to_owned(),
                "reference.revision".to_owned()
            )
        );
        assert_eq!(
            refused(
                "/expression/source/end/byte_offset",
                json!(REVISION_BOUND + 1)
            ),
            ("invalid_source_span".to_owned(), "source_span".to_owned())
        );
        assert_eq!(
            refused(
                "/expression/source/start/source/revision",
                json!(REVISION_BOUND + 1)
            ),
            (
                "invalid_source_revision".to_owned(),
                "source.revision".to_owned()
            )
        );
        // Every span member of the request goes through the same mapping.
        let mut declared = base.clone();
        declared["values"] = json!([{"name": "x", "kind": "input", "value_type": integer_type(),
                                     "source": span()}]);
        declared["values"][0]["source"]["end"]["byte_offset"] = json!(REVISION_BOUND + 1);
        assert_eq!(
            operation_refusal(declared),
            ("invalid_source_span".to_owned(), "source_span".to_owned())
        );
    }

    /// The binding path of the expression decoder (an executable projection's
    /// binding) maps the same registered codes, ahead of the owner and anchor
    /// comparisons; 2^53 is admitted.
    ///
    /// Tracing: TC-015, FR-011-AC-3, FR-012-AC-6.
    #[trace("TC-015", "FR-011-AC-3", "FR-012-AC-6")]
    #[test]
    fn tc_015_a_binding_expression_refuses_a_revision_or_offset_above_two_to_the_53_with_its_registered_code(
    ) {
        let owner = RequirementRef::parse("agent-ix/pkg", "REQ_a", 1).unwrap();
        let anchor = ExecutionPoint::Pre {
            operation: crate::AnchorName::new("check").unwrap(),
        };
        let mut base = request(
            json!({"node": "boolean_literal", "value": true, "source": span()}),
            json!({"kind": "boolean"}),
        );
        base["clause_root"] = json!(true);
        let decode = |pointer: &str, member: u64| {
            check_expression_input(
                with_member(&base, pointer, json!(member)),
                Some((&owner, &anchor)),
            )
        };
        let code_and_path = |result: Result<CheckedExpression, Vec<Diagnostic>>| {
            let diagnostics = result.err().expect("the input must be refused");
            assert_eq!(diagnostics.len(), 1);
            (diagnostics[0].code, diagnostics[0].path.clone())
        };
        assert!(decode("/owner/revision", 1).is_ok());
        assert!(decode("/expression/source/end/byte_offset", REVISION_BOUND).is_ok());
        assert_eq!(
            code_and_path(decode("/owner/revision", REVISION_BOUND + 1)),
            (
                DiagnosticCode::InvalidRequirementRevision,
                "reference.revision".to_owned()
            )
        );
        assert_eq!(
            code_and_path(decode(
                "/expression/source/end/byte_offset",
                REVISION_BOUND + 1
            )),
            (DiagnosticCode::InvalidSourceSpan, "source_span".to_owned())
        );
    }

    /// Tracing: TC-016, FR-013-AC-5.
    #[trace("TC-016", "FR-013-AC-5")]
    #[test]
    fn tc_016_the_widest_strings_decode_and_serialize_back_to_the_same_strings() {
        let integer = check_expression_input(request(integer_literal("0"), integer_type()), None)
            .expect("integer request decodes");
        let value_type = serde_json::to_value(integer.expression.value_type()).unwrap();
        assert_eq!(value_type["value"]["minimum"], NARROWEST_INTEGER);
        assert_eq!(value_type["value"]["maximum"], WIDEST_INTEGER);
        let tree = serde_json::to_value(integer.expression.expression()).unwrap();
        assert_eq!(tree["kind"]["value"], "0");

        let rational =
            check_expression_input(request(rational_literal("-3", "4"), rational_type()), None)
                .expect("rational request decodes");
        let value_type = serde_json::to_value(rational.expression.value_type()).unwrap();
        assert_eq!(value_type["value"]["numerator_minimum"], NARROWEST_INTEGER);
        assert_eq!(value_type["value"]["numerator_maximum"], WIDEST_INTEGER);
        assert_eq!(value_type["value"]["maximum_denominator"], WIDEST_INTEGER);
        let tree = serde_json::to_value(rational.expression.expression()).unwrap();
        assert_eq!(tree["kind"]["numerator"], "-3");
        assert_eq!(tree["kind"]["denominator"], "4");
    }

    /// Tracing: TC-016, FR-013-AC-5.
    #[trace("TC-016", "FR-013-AC-5")]
    #[test]
    fn tc_016_each_member_refuses_a_number_or_an_out_of_grammar_string_as_invalid_wire_format() {
        for (name, base, pointer) in members() {
            check_expression_input(base.clone(), None)
                .unwrap_or_else(|_| panic!("{name}: the base request decodes"));
            let numbers = [
                json!(0),
                json!(1),
                json!(1.0),
                json!(9_223_372_036_854_775_807_i64),
                serde_json::from_str::<Value>("100000000000000000001").unwrap(),
            ];
            let strings = ["", "+1", "01", "-0", "1.0", "1e3", " 1"].map(Value::from);
            for member in numbers.into_iter().chain(strings) {
                let diagnostic = refusal(with_member(&base, pointer, member.clone()));
                assert_eq!(
                    diagnostic.code,
                    DiagnosticCode::InvalidWireFormat,
                    "{name} = {member}"
                );
                assert_eq!(diagnostic.path, "expression", "{name} = {member}");
            }
        }
    }

    /// Tracing: TC-016, FR-013-AC-5.
    #[trace("TC-016", "FR-013-AC-5")]
    #[test]
    fn tc_016_a_grammar_valid_string_out_of_range_is_invalid_numeric_bounds() {
        let (_, integer, minimum) = members().remove(0);
        let diagnostic = refusal(with_member(&integer, minimum, json!("9223372036854775808")));
        assert_eq!(diagnostic.code, DiagnosticCode::InvalidNumericBounds);
        let (_, rational, numerator) = members().remove(2);
        let diagnostic = refusal(with_member(
            &rational,
            numerator,
            json!("-9223372036854775809"),
        ));
        assert_eq!(diagnostic.code, DiagnosticCode::InvalidNumericBounds);
        let (_, rational, denominator) = members().remove(4);
        for member in ["0", "-1", "9223372036854775808", "18446744073709551616"] {
            let diagnostic = refusal(with_member(&rational, denominator, json!(member)));
            assert_eq!(
                diagnostic.code,
                DiagnosticCode::InvalidNumericBounds,
                "maximum_denominator = {member}"
            );
        }
        let (_, literal, value) = members().remove(5);
        let diagnostic = refusal(with_member(&literal, value, json!("9223372036854775808")));
        assert_eq!(diagnostic.code, DiagnosticCode::InvalidNumericBounds);
        assert_eq!(diagnostic.path, "expression.integer_literal.value");
    }

    /// Tracing: TC-016, FR-013-AC-5.
    #[trace("TC-016", "FR-013-AC-5")]
    #[test]
    fn tc_016_a_document_input_reaches_the_decoder_as_the_text_it_is() {
        let number = with_member(
            &request(integer_literal("0"), integer_type()),
            "/expected_type/minimum",
            json!(0),
        );
        let result = execute_expression(json!({"document_json": number.to_string()}));
        assert_eq!(result["valid"], false);
        assert_eq!(result["diagnostics"][0]["code"], "invalid_wire_format");
        assert_eq!(result["diagnostics"][0]["path"], "expression");
        let malformed = execute_expression(json!({"document_json": "{"}));
        assert_eq!(malformed["diagnostics"][0]["code"], "invalid_wire_format");
        assert_eq!(malformed["diagnostics"][0]["path"], "document");
    }
}
