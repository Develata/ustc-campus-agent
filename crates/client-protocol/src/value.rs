use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::fmt;

/// Checked bounded wire text. It rejects control characters and unbounded input.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WireText(String);

impl WireText {
    pub const MAX_BYTES: usize = 4096;

    pub fn parse(value: impl Into<String>) -> Result<Self, WireValueError> {
        let value = value.into();
        if value.is_empty() {
            return Err(WireValueError::Empty);
        }
        if value.len() > Self::MAX_BYTES {
            return Err(WireValueError::TooLong);
        }
        if value.chars().any(char::is_control) {
            return Err(WireValueError::ControlCharacter);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn fallback() -> Self {
        Self("m10_error".to_owned())
    }
}

impl fmt::Debug for WireText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("WireText")
            .field(&"<redacted>")
            .finish()
    }
}

impl Serialize for WireText {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for WireText {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireValueError {
    Empty,
    TooLong,
    ControlCharacter,
}

impl fmt::Display for WireValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "wire text is empty",
            Self::TooLong => "wire text exceeds its bound",
            Self::ControlCharacter => "wire text contains a control character",
        })
    }
}

impl std::error::Error for WireValueError {}

/// Unix timestamp in milliseconds. Kept numeric across every client projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnixMillis(i64);

impl UnixMillis {
    #[must_use]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }
}
