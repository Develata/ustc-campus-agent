//! Exact adapter for the platform's admitted closed-object schema subset.
//! Unsupported JSON Schema semantics fail closed, never disappear in projection.
use super::McpError;
use serde_json::Value;
use ustc_campus_agent_core::invocation::{
    CanonicalArgumentValueV0, UnvalidatedArgumentValueV0, UnvalidatedSchemaNodeV0,
    UnvalidatedToolInputSchemaV0, ValidatedSchemaNodeV0, ValidatedToolInputSchemaV0,
};

pub(super) fn compile_schema(value: &Value) -> Result<ValidatedToolInputSchemaV0, McpError> {
    let root = schema_node(value, 0)?;
    if !matches!(root, UnvalidatedSchemaNodeV0::Object { .. }) {
        return Err(McpError::InvalidSchema);
    }
    ValidatedToolInputSchemaV0::try_from(UnvalidatedToolInputSchemaV0 {
        dialect: "tool-input-schema/v0".to_owned(),
        root,
    })
    .map_err(|_| McpError::InvalidSchema)
}

fn schema_node(value: &Value, depth: usize) -> Result<UnvalidatedSchemaNodeV0, McpError> {
    if depth > 8 {
        return Err(McpError::InvalidSchema);
    }
    let object = value.as_object().ok_or(McpError::InvalidSchema)?;
    let kind = object
        .get("type")
        .and_then(Value::as_str)
        .ok_or(McpError::InvalidSchema)?;
    for (key, value) in object {
        let admitted = match key.as_str() {
            "type" => true,
            "title" | "description" => value.is_string(),
            "$schema" => value.as_str() == Some("https://json-schema.org/draft/2020-12/schema"),
            "properties" | "required" | "additionalProperties" => kind == "object",
            "items" => kind == "array",
            "enum" => kind == "string",
            _ => false,
        };
        if !admitted {
            return Err(McpError::InvalidSchema);
        }
    }
    Ok(match kind {
        "object" => {
            if object.get("additionalProperties") != Some(&Value::Bool(false)) {
                return Err(McpError::InvalidSchema);
            }
            let properties = match object.get("properties") {
                None => Vec::new(),
                Some(Value::Object(properties)) if properties.len() <= 64 => properties
                    .iter()
                    .map(|(key, value)| Ok((key.clone(), schema_node(value, depth + 1)?)))
                    .collect::<Result<Vec<_>, McpError>>()?,
                _ => return Err(McpError::InvalidSchema),
            };
            let required = match object.get("required") {
                None => Vec::new(),
                Some(Value::Array(required)) => required
                    .iter()
                    .map(|v| v.as_str().map(str::to_owned).ok_or(McpError::InvalidSchema))
                    .collect::<Result<Vec<_>, _>>()?,
                _ => return Err(McpError::InvalidSchema),
            };
            UnvalidatedSchemaNodeV0::Object {
                properties,
                required,
            }
        }
        "string" => {
            let enum_values = match object.get("enum") {
                None => None,
                Some(Value::Array(values)) => Some(
                    values
                        .iter()
                        .map(|v| v.as_str().map(str::to_owned).ok_or(McpError::InvalidSchema))
                        .collect::<Result<Vec<_>, _>>()?,
                ),
                _ => return Err(McpError::InvalidSchema),
            };
            UnvalidatedSchemaNodeV0::String { enum_values }
        }
        "integer" => UnvalidatedSchemaNodeV0::Integer,
        "number" => UnvalidatedSchemaNodeV0::Number,
        "boolean" => UnvalidatedSchemaNodeV0::Boolean,
        "array" => UnvalidatedSchemaNodeV0::Array {
            items: Box::new(schema_node(
                object.get("items").ok_or(McpError::InvalidSchema)?,
                depth + 1,
            )?),
        },
        _ => return Err(McpError::InvalidSchema),
    })
}

pub(super) fn validate_arguments(
    schema: &ValidatedToolInputSchemaV0,
    value: &Value,
) -> Result<(), McpError> {
    let canonical = CanonicalArgumentValueV0::try_from(argument_node(value, 0, false)?)
        .map_err(|_| McpError::InvalidArguments)?;
    if schema.accepts(&canonical) {
        Ok(())
    } else {
        Err(McpError::InvalidArguments)
    }
}

fn argument_node(
    value: &Value,
    depth: usize,
    output: bool,
) -> Result<UnvalidatedArgumentValueV0, McpError> {
    if depth > 8 {
        return Err(McpError::InvalidArguments);
    }
    Ok(match value {
        Value::Null => UnvalidatedArgumentValueV0::Null,
        Value::Bool(value) => UnvalidatedArgumentValueV0::Boolean(*value),
        Value::String(value) => UnvalidatedArgumentValueV0::String(value.clone()),
        Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                UnvalidatedArgumentValueV0::Integer(value.to_string())
            } else if value.is_f64() || (output && value.is_u64()) {
                UnvalidatedArgumentValueV0::Number(value.to_string())
            } else {
                return Err(McpError::InvalidArguments);
            }
        }
        Value::Array(values) if values.len() <= 256 => UnvalidatedArgumentValueV0::Array(
            values
                .iter()
                .map(|v| argument_node(v, depth + 1, output))
                .collect::<Result<_, _>>()?,
        ),
        Value::Object(values) if values.len() <= 64 => UnvalidatedArgumentValueV0::Object(
            values
                .iter()
                .map(|(k, v)| Ok((k.clone(), argument_node(v, depth + 1, output)?)))
                .collect::<Result<_, McpError>>()?,
        ),
        _ => return Err(McpError::InvalidArguments),
    })
}

/// Output data has JSON Schema numeric membership; input authority retains exact
/// platform Integer/Number tags. Reuse the compiler and canonical value bounds.
pub(super) fn validate_output(schema: &Value, value: &Value) -> Result<(), McpError> {
    let schema = compile_schema(schema)?;
    CanonicalArgumentValueV0::try_from(argument_node(value, 0, true)?)
        .map_err(|_| McpError::InvalidSchema)?;
    if output_matches(schema.root(), value) {
        Ok(())
    } else {
        Err(McpError::InvalidSchema)
    }
}

fn output_matches(schema: &ValidatedSchemaNodeV0, value: &Value) -> bool {
    match (schema, value) {
        (ValidatedSchemaNodeV0::Number, Value::Number(number)) => {
            number.as_f64().is_some_and(f64::is_finite)
        }
        (ValidatedSchemaNodeV0::Integer, Value::Number(number)) => {
            number.is_i64()
                || number.is_u64()
                || number
                    .as_f64()
                    .is_some_and(|n| n.is_finite() && n.fract() == 0.0)
        }
        (ValidatedSchemaNodeV0::Boolean, Value::Bool(_)) => true,
        (ValidatedSchemaNodeV0::String { enum_values }, Value::String(value)) => enum_values
            .as_ref()
            .is_none_or(|choices| choices.contains(value)),
        (ValidatedSchemaNodeV0::Array { items }, Value::Array(values)) => {
            values.iter().all(|value| output_matches(items, value))
        }
        (
            ValidatedSchemaNodeV0::Object {
                properties,
                required,
            },
            Value::Object(values),
        ) => {
            required.iter().all(|name| values.contains_key(name))
                && values.iter().all(|(name, value)| {
                    properties
                        .get(name)
                        .is_some_and(|schema| output_matches(schema, value))
                })
        }
        _ => false,
    }
}
