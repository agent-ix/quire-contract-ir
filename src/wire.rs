use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    conformance::{
        canonical_value, diagnostics_value, MAX_SEMANTIC_COLLECTION_ITEMS, MAX_SEMANTIC_DEPTH,
        MAX_SEMANTIC_NODES,
    },
    BooleanOperator, CanonicalProfile, CollectionType, ComparisonOperator, DeclarationEnvironment,
    Diagnostic, DiagnosticCode, EnumDeclaration, EnumVariantDeclaration, ExecutionPoint,
    Expression, ExpressionKind, FunctionParameter, IntegerDomain, IntegerType, NumericOperator,
    OverflowPolicy, PureFunctionDeclaration, QuantifierDomain, QuantifierKind, RationalType,
    RecordDeclaration, RecordFieldDeclaration, RecordLiteralField, RequirementRef, SourceSpan,
    StateObservation, SymbolName, TypeDeclaration, ValueDeclaration, ValueDeclarationKind,
    ValueType,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpressionInput {
    owner: RequirementRef,
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
        minimum: i64,
        maximum: i64,
        overflow: OverflowPolicy,
    },
    Rational {
        numerator_minimum: i64,
        numerator_maximum: i64,
        maximum_denominator: u64,
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
            } => IntegerType::new(domain, minimum, maximum, overflow).map(ValueType::integer),
            Self::Rational {
                numerator_minimum,
                numerator_maximum,
                maximum_denominator,
            } => RationalType::new(numerator_minimum, numerator_maximum, maximum_denominator)
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
        source: SourceSpan,
        variants: Vec<WireEnumVariant>,
    },
    Record {
        name: String,
        source: SourceSpan,
        fields: Vec<WireRecordField>,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireEnumVariant {
    name: String,
    source: SourceSpan,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireRecordField {
    name: String,
    value_type: WireValueType,
    source: SourceSpan,
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
                source,
                variants
                    .into_iter()
                    .map(|variant| {
                        Ok(EnumVariantDeclaration::new(
                            SymbolName::new(variant.name)?,
                            variant.source,
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
                source,
                fields
                    .into_iter()
                    .map(|field| {
                        Ok(RecordFieldDeclaration::new(
                            SymbolName::new(field.name)?,
                            field.value_type.validate()?,
                            field.source,
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
    source: SourceSpan,
}

impl WireValueDeclaration {
    fn validate(self) -> Result<ValueDeclaration, Diagnostic> {
        Ok(ValueDeclaration::new(
            SymbolName::new(self.name)?,
            self.kind,
            self.value_type.validate()?,
            self.source,
        ))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireFunctionParameter {
    name: String,
    value_type: WireValueType,
    source: SourceSpan,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireFunctionDeclaration {
    name: String,
    parameters: Vec<WireFunctionParameter>,
    result_type: WireValueType,
    source: SourceSpan,
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
                    parameter.source,
                ))
            })
            .collect::<Result<Vec<_>, Diagnostic>>()
            .map_err(|diagnostic| vec![diagnostic])?;
        PureFunctionDeclaration::new(
            one(SymbolName::new(self.name))?,
            parameters,
            one(self.result_type.validate())?,
            self.source,
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
        value: i64,
        value_type: WireValueType,
    },
    RationalLiteral {
        numerator: i64,
        denominator: i64,
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
        local_source: SourceSpan,
        predicate: Box<WireExpression>,
    },
}

#[derive(Deserialize)]
struct WireExpression {
    #[serde(flatten)]
    kind: WireExpressionKind,
    source: SourceSpan,
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
                ExpressionKind::IntegerLiteral { value, value_type }
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
                    numerator,
                    denominator,
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
                local_source,
                predicate: Box::new(predicate.validate()?),
            },
        };
        Ok(Expression::new(kind, self.source))
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

pub(crate) fn execute_expression(input: Value) -> Value {
    let request: ExpressionInput = match serde_json::from_value(input) {
        Ok(request) => request,
        Err(_) => return invalid(vec![wire_type_error("expression")]),
    };
    if let Err(diagnostic) = preflight(&request) {
        return invalid(vec![diagnostic]);
    }
    let types = match request
        .types
        .into_iter()
        .map(WireTypeDeclaration::validate)
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(types) => types,
        Err(diagnostics) => return invalid(diagnostics),
    };
    let values = match request
        .values
        .into_iter()
        .map(WireValueDeclaration::validate)
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(values) => values,
        Err(diagnostic) => return invalid(vec![diagnostic]),
    };
    let functions = match request
        .functions
        .into_iter()
        .map(WireFunctionDeclaration::validate)
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(functions) => functions,
        Err(diagnostics) => return invalid(diagnostics),
    };
    let environment = match DeclarationEnvironment::new(request.owner, types, values, functions) {
        Ok(environment) => environment,
        Err(diagnostics) => return invalid(diagnostics),
    };
    let expression = match request.expression.validate() {
        Ok(expression) => expression,
        Err(diagnostic) => return invalid(vec![diagnostic]),
    };
    let expected = match request.expected_type.validate() {
        Ok(expected) => expected,
        Err(diagnostic) => return invalid(vec![diagnostic]),
    };
    let typed = match environment.check_expression(
        &expression,
        &expected,
        &request.execution_point,
        request.clause_root,
    ) {
        Ok(typed) => typed,
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

fn preflight(request: &ExpressionInput) -> Result<(), Diagnostic> {
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
    Ok(())
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
