//! Exact adapter for the platform's admitted closed-object schema subset.
//! Unsupported JSON Schema semantics fail closed, never disappear in projection.
use super::McpError;
use serde_json::Value;
use ustc_campus_agent_core::invocation::{
    CanonicalArgumentValueV0, UnvalidatedArgumentValueV0, UnvalidatedSchemaNodeV0,
    UnvalidatedToolInputSchemaV0, ValidatedSchemaNodeV0, ValidatedToolInputSchemaV0,
};

pub(super) fn compile_schema(value: &Value) -> Result<ValidatedToolInputSchemaV0, McpError> {
    let expanded = expand_local_references(value, value, 0, &mut 512)?;
    let root = schema_node(&expanded, 0)?;
    if !matches!(root, UnvalidatedSchemaNodeV0::Object { .. }) {
        return Err(McpError::InvalidSchema);
    }
    ValidatedToolInputSchemaV0::try_from(UnvalidatedToolInputSchemaV0 {
        dialect: "tool-input-schema/v0".to_owned(),
        root,
    })
    .map_err(|_| McpError::InvalidSchema)
}

// Resolve only document-local definitions. No network references, cycles or validation-keyword siblings.
fn expand_local_references(
    value: &Value,
    root: &Value,
    depth: usize,
    remaining: &mut usize,
) -> Result<Value, McpError> {
    if depth > 8 || *remaining == 0 {
        return Err(McpError::InvalidSchema);
    }
    *remaining -= 1;
    let mut object = value.as_object().ok_or(McpError::InvalidSchema)?.clone();
    if let Some(reference) = object.remove("$ref") {
        let reference = reference.as_str().ok_or(McpError::InvalidSchema)?;
        if !(reference.starts_with("#/$defs/") || reference.starts_with("#/definitions/"))
            || object.keys().any(|key| {
                !matches!(
                    key.as_str(),
                    "title" | "description" | "default" | "examples" | "$comment"
                )
            })
        {
            return Err(McpError::InvalidSchema);
        }
        let target = root
            .pointer(&reference[1..])
            .ok_or(McpError::InvalidSchema)?;
        return expand_local_references(target, root, depth + 1, remaining);
    }
    // Definitions are inert until referenced; validation cannot depend on silently dropped keywords.
    for key in ["$defs", "definitions"] {
        if object
            .remove(key)
            .is_some_and(|definitions| !definitions.is_object())
        {
            return Err(McpError::InvalidSchema);
        }
    }
    if let Some(properties) = object.get_mut("properties") {
        let properties = properties.as_object_mut().ok_or(McpError::InvalidSchema)?;
        if properties.len() > 64 {
            return Err(McpError::InvalidSchema);
        }
        for schema in properties.values_mut() {
            *schema = expand_local_references(schema, root, depth + 1, remaining)?;
        }
    }
    if let Some(items) = object.get_mut("items") {
        *items = expand_local_references(items, root, depth + 1, remaining)?;
    }
    Ok(Value::Object(object))
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
            "title" | "description" | "$comment" => value.is_string(),
            "default" => true,
            "examples" => value.is_array(),
            "deprecated" | "readOnly" | "writeOnly" => value.is_boolean(),
            "$schema" => value.as_str() == Some("https://json-schema.org/draft/2020-12/schema"),
            "properties" | "required" | "additionalProperties" => kind == "object",
            "items" => kind == "array",
            "enum" => kind == "string",
            "minLength" | "maxLength" => kind == "string",
            "minimum" | "maximum" => matches!(kind, "integer" | "number"),
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
            let min_length = optional_bound(object, "minLength", Value::as_u64)?;
            let max_length = optional_bound(object, "maxLength", Value::as_u64)?;
            if min_length.is_some() || max_length.is_some() {
                UnvalidatedSchemaNodeV0::BoundedString {
                    enum_values,
                    min_length,
                    max_length,
                }
            } else {
                UnvalidatedSchemaNodeV0::String { enum_values }
            }
        }
        "integer" => {
            let minimum = optional_bound(object, "minimum", Value::as_i64)?;
            let maximum = optional_bound(object, "maximum", Value::as_i64)?;
            if minimum.is_some() || maximum.is_some() {
                UnvalidatedSchemaNodeV0::BoundedInteger { minimum, maximum }
            } else {
                UnvalidatedSchemaNodeV0::Integer
            }
        }
        "number" => {
            let minimum = optional_bound(object, "minimum", number_bound)?;
            let maximum = optional_bound(object, "maximum", number_bound)?;
            if minimum.is_some() || maximum.is_some() {
                UnvalidatedSchemaNodeV0::BoundedNumber { minimum, maximum }
            } else {
                UnvalidatedSchemaNodeV0::Number
            }
        }
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

fn optional_bound<T>(
    object: &serde_json::Map<String, Value>,
    key: &str,
    parse: impl FnOnce(&Value) -> Option<T>,
) -> Result<Option<T>, McpError> {
    object
        .get(key)
        .map(|value| parse(value).ok_or(McpError::InvalidSchema))
        .transpose()
}

fn number_bound(value: &Value) -> Option<f64> {
    let number = value.as_f64().filter(|number| number.is_finite())?;
    let integer = value
        .as_i64()
        .map(i128::from)
        .or_else(|| value.as_u64().map(i128::from));
    // An integer threshold must not silently shift through a lossy f64 conversion.
    // Keep thresholds inside the exact-integer binary64 range. Larger raw integer
    // tokens may already have rounded through serde_json::Value before this adapter.
    if number.abs() > 9_007_199_254_740_992.0 {
        return None;
    }
    if integer.is_some_and(|integer| number as i128 != integer) {
        return None;
    }
    Some(number)
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
        (_, Value::Number(number)) => {
            if let Some(integer) = number.as_i64() {
                schema.accepts_output_integer(i128::from(integer))
            } else if let Some(integer) = number.as_u64() {
                schema.accepts_output_integer(i128::from(integer))
            } else {
                number
                    .as_f64()
                    .is_some_and(|number| schema.accepts_output_number(number))
            }
        }
        (ValidatedSchemaNodeV0::Boolean, Value::Bool(_)) => true,
        (_, Value::String(value)) => schema.accepts_string_value(value),
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

#[cfg(test)]
mod compatibility_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn local_defs_and_annotations_preserve_validation() {
        let schema = compile_schema(&json!({"type":"object","additionalProperties":false,
            "$defs":{"Count":{"type":"integer","minimum":1,"maximum":3,"default":2}},
            "properties":{"count":{"$ref":"#/$defs/Count"}},"required":["count"]}))
        .expect("local definition");
        assert!(validate_arguments(&schema, &json!({"count":2})).is_ok());
        for value in [
            json!({"count":0}),
            json!({"count":4}),
            json!({"count":"2"}),
            json!({}),
        ] {
            assert!(validate_arguments(&schema, &value).is_err());
        }
    }
    #[test]
    fn remote_recursive_and_constraint_sibling_refs_reject() {
        for value in [
            json!({"$ref":"https://example.test/schema"}),
            json!({"$defs":{"Loop":{"$ref":"#/$defs/Loop"}},"$ref":"#/$defs/Loop"}),
            json!({"$defs":{"Count":{"type":"integer"}},"type":"object","additionalProperties":false,
                "properties":{"count":{"$ref":"#/$defs/Count","minimum":2}}}),
        ] {
            assert!(compile_schema(&value).is_err());
        }
    }
}
