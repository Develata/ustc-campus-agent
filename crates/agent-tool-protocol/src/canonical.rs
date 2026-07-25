use sha2::{Digest as _, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

const SCHEMA_DOMAIN: &[u8] = b"tool-input-schema/v0\0";
const ARGUMENT_DOMAIN: &[u8] = b"tool-arguments/v0\0";
const MAX_DEPTH: usize = 8;
const MAX_NODES: usize = 256;
const MAX_OBJECT_MEMBERS: usize = 64;
const MAX_SCHEMA_BYTES: usize = 65_536;
const MAX_ARGUMENT_BYTES: usize = 65_536;

/// A validated lowercase SHA-256 identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sha256Digest(String);

impl Sha256Digest {
    /// Hash exact bytes into the canonical lowercase representation.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let digest = Sha256::digest(bytes);
        let mut rendered = String::with_capacity(71);
        rendered.push_str("sha256:");
        for byte in digest {
            use fmt::Write as _;
            let _ = write!(rendered, "{byte:02x}");
        }
        Self(rendered)
    }

    /// Parse an exact lowercase `sha256:<64 hex>` digest.
    pub fn parse(value: impl Into<String>) -> Result<Self, InvalidValue> {
        let value = value.into();
        let valid = value.strip_prefix("sha256:").is_some_and(|hex| {
            hex.len() == 64
                && hex
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        });
        if valid {
            Ok(Self(value))
        } else {
            Err(InvalidValue::Digest)
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Validation failure for an owned canonical value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidValue {
    Digest,
    Identity,
    Name,
}

impl fmt::Display for InvalidValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid canonical value: {self:?}")
    }
}

impl Error for InvalidValue {}

/// Source-order-preserving input-schema node.
#[derive(Debug, Clone, PartialEq)]
pub enum UnvalidatedSchemaNodeV0 {
    Object {
        properties: Vec<(String, UnvalidatedSchemaNodeV0)>,
        required: Vec<String>,
    },
    String {
        enum_values: Option<Vec<String>>,
    },
    Integer,
    Number,
    Boolean,
    Array {
        items: Box<UnvalidatedSchemaNodeV0>,
    },
}

/// Loader-produced schema with an exact dialect identity.
#[derive(Debug, Clone, PartialEq)]
pub struct UnvalidatedToolInputSchemaV0 {
    pub dialect: String,
    pub root: UnvalidatedSchemaNodeV0,
}

/// Canonically ordered, bounded schema node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatedSchemaNodeV0 {
    Object {
        properties: BTreeMap<String, ValidatedSchemaNodeV0>,
        required: BTreeSet<String>,
    },
    String {
        enum_values: Option<BTreeSet<String>>,
    },
    Integer,
    Number,
    Boolean,
    Array {
        items: Box<ValidatedSchemaNodeV0>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaConstructionError {
    SchemaDialectUnsupported,
    SchemaMalformed,
    SchemaLimitExceeded,
}

impl fmt::Display for SchemaConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "schema construction failed: {self:?}")
    }
}

impl Error for SchemaConstructionError {}

/// Validated immutable v0 tool-input schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedToolInputSchemaV0 {
    root: ValidatedSchemaNodeV0,
    canonical_bytes: Vec<u8>,
    digest: Sha256Digest,
}

impl ValidatedToolInputSchemaV0 {
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    #[must_use]
    pub const fn digest(&self) -> &Sha256Digest {
        &self.digest
    }

    #[must_use]
    pub const fn root(&self) -> &ValidatedSchemaNodeV0 {
        &self.root
    }
}

impl TryFrom<UnvalidatedToolInputSchemaV0> for ValidatedToolInputSchemaV0 {
    type Error = SchemaConstructionError;

    fn try_from(value: UnvalidatedToolInputSchemaV0) -> Result<Self, Self::Error> {
        if value.dialect != "tool-input-schema/v0" {
            return Err(SchemaConstructionError::SchemaDialectUnsupported);
        }
        if !matches!(value.root, UnvalidatedSchemaNodeV0::Object { .. }) {
            return Err(SchemaConstructionError::SchemaMalformed);
        }
        let mut nodes = 0;
        let root = validate_schema_node(value.root, 1, &mut nodes)?;
        let mut canonical_bytes = SCHEMA_DOMAIN.to_vec();
        encode_schema_node(&root, &mut canonical_bytes);
        if canonical_bytes.len() > MAX_SCHEMA_BYTES {
            return Err(SchemaConstructionError::SchemaLimitExceeded);
        }
        let digest = Sha256Digest::from_bytes(&canonical_bytes);
        Ok(Self {
            root,
            canonical_bytes,
            digest,
        })
    }
}

fn validate_schema_node(
    node: UnvalidatedSchemaNodeV0,
    depth: usize,
    nodes: &mut usize,
) -> Result<ValidatedSchemaNodeV0, SchemaConstructionError> {
    if depth > MAX_DEPTH {
        return Err(SchemaConstructionError::SchemaLimitExceeded);
    }
    *nodes += 1;
    if *nodes > MAX_NODES {
        return Err(SchemaConstructionError::SchemaLimitExceeded);
    }
    match node {
        UnvalidatedSchemaNodeV0::Object {
            properties,
            required,
        } => {
            if properties.len() > MAX_OBJECT_MEMBERS {
                return Err(SchemaConstructionError::SchemaLimitExceeded);
            }
            let mut validated = BTreeMap::new();
            for (name, child) in properties {
                if !is_valid_tool_name(&name) || validated.contains_key(&name) {
                    return Err(SchemaConstructionError::SchemaMalformed);
                }
                validated.insert(name, validate_schema_node(child, depth + 1, nodes)?);
            }
            let mut required_set = BTreeSet::new();
            for name in required {
                if !is_valid_tool_name(&name)
                    || !validated.contains_key(&name)
                    || !required_set.insert(name)
                {
                    return Err(SchemaConstructionError::SchemaMalformed);
                }
            }
            Ok(ValidatedSchemaNodeV0::Object {
                properties: validated,
                required: required_set,
            })
        }
        UnvalidatedSchemaNodeV0::String { enum_values } => {
            let enum_values = match enum_values {
                None => None,
                Some(values) => {
                    if values.is_empty() || values.len() > 64 {
                        return Err(SchemaConstructionError::SchemaLimitExceeded);
                    }
                    let mut unique = BTreeSet::new();
                    for value in values {
                        if value.is_empty() || value.len() > 256 {
                            return Err(SchemaConstructionError::SchemaLimitExceeded);
                        }
                        if !unique.insert(value) {
                            return Err(SchemaConstructionError::SchemaMalformed);
                        }
                    }
                    Some(unique)
                }
            };
            Ok(ValidatedSchemaNodeV0::String { enum_values })
        }
        UnvalidatedSchemaNodeV0::Integer => Ok(ValidatedSchemaNodeV0::Integer),
        UnvalidatedSchemaNodeV0::Number => Ok(ValidatedSchemaNodeV0::Number),
        UnvalidatedSchemaNodeV0::Boolean => Ok(ValidatedSchemaNodeV0::Boolean),
        UnvalidatedSchemaNodeV0::Array { items } => Ok(ValidatedSchemaNodeV0::Array {
            items: Box::new(validate_schema_node(*items, depth + 1, nodes)?),
        }),
    }
}

fn encode_schema_node(node: &ValidatedSchemaNodeV0, output: &mut Vec<u8>) {
    match node {
        ValidatedSchemaNodeV0::Object {
            properties,
            required,
        } => {
            output.push(0x01);
            encode_count(properties.len(), output);
            for (name, child) in properties {
                encode_string(name, output);
                encode_schema_node(child, output);
            }
            encode_count(required.len(), output);
            for name in required {
                encode_string(name, output);
            }
        }
        ValidatedSchemaNodeV0::String { enum_values } => {
            output.push(0x02);
            match enum_values {
                None => output.push(0),
                Some(values) => {
                    output.push(1);
                    encode_count(values.len(), output);
                    for value in values {
                        encode_string(value, output);
                    }
                }
            }
        }
        ValidatedSchemaNodeV0::Integer => output.push(0x03),
        ValidatedSchemaNodeV0::Number => output.push(0x04),
        ValidatedSchemaNodeV0::Boolean => output.push(0x05),
        ValidatedSchemaNodeV0::Array { items } => {
            output.push(0x06);
            encode_schema_node(items, output);
        }
    }
}

/// Source-order-preserving parsed argument value.
#[derive(Debug, Clone, PartialEq)]
pub enum UnvalidatedArgumentValueV0 {
    Null,
    Boolean(bool),
    Integer(String),
    Number(String),
    String(String),
    Array(Vec<UnvalidatedArgumentValueV0>),
    Object(Vec<(String, UnvalidatedArgumentValueV0)>),
}

/// Canonical argument tree; finite numbers use normalized binary64 bits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalArgumentNodeV0 {
    Null,
    Boolean(bool),
    Integer(i64),
    Number(u64),
    String(String),
    Array(Vec<CanonicalArgumentNodeV0>),
    Object(BTreeMap<String, CanonicalArgumentNodeV0>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgumentConstructionError {
    ArgumentDuplicateKey,
    ArgumentInvalidName,
    ArgumentNumberOutOfRange,
    ArgumentLimitExceeded,
}

impl fmt::Display for ArgumentConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "argument construction failed: {self:?}")
    }
}

impl Error for ArgumentConstructionError {}

/// Bounded canonical argument value and its exact digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalArgumentValueV0 {
    root: CanonicalArgumentNodeV0,
    canonical_bytes: Vec<u8>,
    digest: Sha256Digest,
}

impl CanonicalArgumentValueV0 {
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    #[must_use]
    pub const fn digest(&self) -> &Sha256Digest {
        &self.digest
    }

    #[must_use]
    pub const fn root(&self) -> &CanonicalArgumentNodeV0 {
        &self.root
    }
}

impl TryFrom<UnvalidatedArgumentValueV0> for CanonicalArgumentValueV0 {
    type Error = ArgumentConstructionError;

    fn try_from(value: UnvalidatedArgumentValueV0) -> Result<Self, Self::Error> {
        let mut nodes = 0;
        let root = validate_argument_node(value, 1, &mut nodes)?;
        let mut canonical_bytes = ARGUMENT_DOMAIN.to_vec();
        encode_argument_node(&root, &mut canonical_bytes);
        if canonical_bytes.len() > MAX_ARGUMENT_BYTES {
            return Err(ArgumentConstructionError::ArgumentLimitExceeded);
        }
        let digest = Sha256Digest::from_bytes(&canonical_bytes);
        Ok(Self {
            root,
            canonical_bytes,
            digest,
        })
    }
}

fn validate_argument_node(
    node: UnvalidatedArgumentValueV0,
    depth: usize,
    nodes: &mut usize,
) -> Result<CanonicalArgumentNodeV0, ArgumentConstructionError> {
    if depth > MAX_DEPTH {
        return Err(ArgumentConstructionError::ArgumentLimitExceeded);
    }
    *nodes += 1;
    if *nodes > MAX_NODES {
        return Err(ArgumentConstructionError::ArgumentLimitExceeded);
    }
    match node {
        UnvalidatedArgumentValueV0::Null => Ok(CanonicalArgumentNodeV0::Null),
        UnvalidatedArgumentValueV0::Boolean(value) => Ok(CanonicalArgumentNodeV0::Boolean(value)),
        UnvalidatedArgumentValueV0::Integer(token) => token
            .parse::<i64>()
            .map(CanonicalArgumentNodeV0::Integer)
            .map_err(|_| ArgumentConstructionError::ArgumentNumberOutOfRange),
        UnvalidatedArgumentValueV0::Number(token) => {
            let number = token
                .parse::<f64>()
                .map_err(|_| ArgumentConstructionError::ArgumentNumberOutOfRange)?;
            if !number.is_finite() {
                return Err(ArgumentConstructionError::ArgumentNumberOutOfRange);
            }
            let bits = if number == 0.0 { 0 } else { number.to_bits() };
            Ok(CanonicalArgumentNodeV0::Number(bits))
        }
        UnvalidatedArgumentValueV0::String(value) => {
            if value.len() > 4_096 {
                Err(ArgumentConstructionError::ArgumentLimitExceeded)
            } else {
                Ok(CanonicalArgumentNodeV0::String(value))
            }
        }
        UnvalidatedArgumentValueV0::Array(values) => {
            if values.len() > 256 {
                return Err(ArgumentConstructionError::ArgumentLimitExceeded);
            }
            values
                .into_iter()
                .map(|value| validate_argument_node(value, depth + 1, nodes))
                .collect::<Result<Vec<_>, _>>()
                .map(CanonicalArgumentNodeV0::Array)
        }
        UnvalidatedArgumentValueV0::Object(members) => {
            if members.len() > MAX_OBJECT_MEMBERS {
                return Err(ArgumentConstructionError::ArgumentLimitExceeded);
            }
            let mut object = BTreeMap::new();
            for (name, value) in members {
                if !is_valid_tool_name(&name) {
                    return Err(ArgumentConstructionError::ArgumentInvalidName);
                }
                if object.contains_key(&name) {
                    return Err(ArgumentConstructionError::ArgumentDuplicateKey);
                }
                object.insert(name, validate_argument_node(value, depth + 1, nodes)?);
            }
            Ok(CanonicalArgumentNodeV0::Object(object))
        }
    }
}

fn encode_argument_node(node: &CanonicalArgumentNodeV0, output: &mut Vec<u8>) {
    match node {
        CanonicalArgumentNodeV0::Null => output.push(0x00),
        CanonicalArgumentNodeV0::Boolean(value) => {
            output.push(0x01);
            output.push(u8::from(*value));
        }
        CanonicalArgumentNodeV0::Integer(value) => {
            output.push(0x02);
            output.extend_from_slice(&value.to_be_bytes());
        }
        CanonicalArgumentNodeV0::Number(bits) => {
            output.push(0x03);
            output.extend_from_slice(&bits.to_be_bytes());
        }
        CanonicalArgumentNodeV0::String(value) => {
            output.push(0x04);
            encode_string(value, output);
        }
        CanonicalArgumentNodeV0::Array(values) => {
            output.push(0x05);
            encode_count(values.len(), output);
            for value in values {
                encode_argument_node(value, output);
            }
        }
        CanonicalArgumentNodeV0::Object(members) => {
            output.push(0x06);
            encode_count(members.len(), output);
            for (name, value) in members {
                encode_string(name, output);
                encode_argument_node(value, output);
            }
        }
    }
}

fn encode_count(count: usize, output: &mut Vec<u8>) {
    output.extend_from_slice(&(count as u64).to_be_bytes());
}

fn encode_string(value: &str, output: &mut Vec<u8>) {
    encode_count(value.len(), output);
    output.extend_from_slice(value.as_bytes());
}

/// Shared v0 grammar for model-visible tool/property names.
#[must_use]
pub fn is_valid_tool_name(value: &str) -> bool {
    if value.is_empty() || value.len() > 64 {
        return false;
    }
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
}
