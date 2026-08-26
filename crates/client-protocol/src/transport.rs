use std::io::{self, Read, Write};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::affairs::M71TerminalDto;
use crate::error::ClientErrorDto;
use crate::value::{UnixMillis, WireText};

pub const MAX_FRAME_BYTES: usize = 1_048_576;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ActorIntentDto {
    Public,
    Authenticated { session_id: WireText },
}

impl std::fmt::Debug for ActorIntentDto {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => formatter
                .debug_struct("ActorIntentDto")
                .field("kind", &"public")
                .finish(),
            Self::Authenticated { .. } => formatter
                .debug_struct("ActorIntentDto")
                .field("kind", &"authenticated")
                .field("session_id", &"[REDACTED]")
                .finish(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientProvenanceDto {
    pub build: WireText,
    pub target: WireText,
    pub protocol: WireText,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmitAffairsGetDto {
    pub request_id: WireText,
    pub correlation_id: WireText,
    pub causation_id: Option<WireText>,
    pub idempotency_key: Option<WireText>,
    pub actor: ActorIntentDto,
    pub provenance: ClientProvenanceDto,
    pub payload_digest: WireText,
    pub procedure_id: WireText,
    pub as_of: Option<UnixMillis>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ViewerAuthorizationDto {
    PublicCapability {
        capability: WireText,
    },
    AuthenticatedOwner {
        tenant_id: WireText,
        user_id: WireText,
    },
    Operator {
        grant_id: WireText,
    },
}

impl std::fmt::Debug for ViewerAuthorizationDto {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PublicCapability { .. } => formatter
                .debug_struct("ViewerAuthorizationDto")
                .field("kind", &"public_capability")
                .field("capability", &"[REDACTED]")
                .finish(),
            Self::AuthenticatedOwner { .. } => formatter
                .debug_struct("ViewerAuthorizationDto")
                .field("kind", &"authenticated_owner")
                .field("tenant_id", &"[REDACTED]")
                .field("user_id", &"[REDACTED]")
                .finish(),
            Self::Operator { .. } => formatter
                .debug_struct("ViewerAuthorizationDto")
                .field("kind", &"operator")
                .field("grant_id", &"[REDACTED]")
                .finish(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClientIntentDto {
    SubmitAffairsGet {
        request: SubmitAffairsGetDto,
    },
    Lookup {
        command_id: WireText,
        viewer: ViewerAuthorizationDto,
    },
}

impl std::fmt::Debug for ClientIntentDto {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SubmitAffairsGet { .. } => formatter
                .debug_struct("ClientIntentDto")
                .field("kind", &"submit_affairs_get")
                .field("request", &"[REDACTED]")
                .finish(),
            Self::Lookup { .. } => formatter
                .debug_struct("ClientIntentDto")
                .field("kind", &"lookup")
                .field("command_id", &"[REDACTED]")
                .field("viewer", &"[REDACTED]")
                .finish(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClientResponseDto {
    Accepted {
        command_id: WireText,
        terminal: Box<M71TerminalDto>,
        public_capability: Option<WireText>,
    },
    Available {
        command_id: WireText,
        terminal: Box<M71TerminalDto>,
        redaction: RedactionDto,
    },
    Incomplete {
        command_id: WireText,
        retry_not_before: UnixMillis,
    },
    Unavailable,
    Error {
        error: ClientErrorDto,
    },
}

impl std::fmt::Debug for ClientResponseDto {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Accepted { .. } => formatter
                .debug_struct("ClientResponseDto")
                .field("kind", &"accepted")
                .field("command_id", &"[REDACTED]")
                .field("terminal", &"[REDACTED]")
                .field("public_capability", &"[REDACTED]")
                .finish(),
            Self::Available { redaction, .. } => formatter
                .debug_struct("ClientResponseDto")
                .field("kind", &"available")
                .field("command_id", &"[REDACTED]")
                .field("terminal", &"[REDACTED]")
                .field("redaction", redaction)
                .finish(),
            Self::Incomplete { .. } => formatter
                .debug_struct("ClientResponseDto")
                .field("kind", &"incomplete")
                .field("command_id", &"[REDACTED]")
                .field("retry_not_before", &"[REDACTED]")
                .finish(),
            Self::Unavailable => formatter
                .debug_struct("ClientResponseDto")
                .field("kind", &"unavailable")
                .finish(),
            Self::Error { error } => formatter
                .debug_struct("ClientResponseDto")
                .field("kind", &"error")
                .field("error", error)
                .finish(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RedactionDto {
    Public,
    AuthenticatedOwner,
    Operator,
}

pub fn write_frame<T: Serialize>(mut writer: impl Write, value: &T) -> io::Result<()> {
    let payload = serde_json::to_vec(value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if payload.len() > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "frame too large",
        ));
    }
    let length = u32::try_from(payload.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "frame too large"))?;
    writer.write_all(&length.to_be_bytes())?;
    writer.write_all(&payload)?;
    writer.flush()
}

pub fn read_frame<T: DeserializeOwned>(mut reader: impl Read) -> io::Result<T> {
    let mut length = [0_u8; 4];
    reader.read_exact(&mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid frame length",
        ));
    }
    let mut payload = vec![0_u8; length];
    reader.read_exact(&mut payload)?;
    serde_json::from_slice(&payload)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}
