use serde::{Deserialize, Deserializer, Serialize, de};

use super::{
    CURRENT_CLIENT_PROTOCOL_MAJOR, ClientProtocolMajor, MINIMUM_CLIENT_PROTOCOL_MAJOR,
    SUPPORTED_CLIENT_PROTOCOL_MAJORS,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProtocolCompatibilityDto {
    UpgradeRequired {
        client_major: ClientProtocolMajor,
        minimum_client_major: ClientProtocolMajor,
        server_major: ClientProtocolMajor,
    },
    IncompatibleProtocol {
        client_major: Option<ClientProtocolMajor>,
        supported_majors: [ClientProtocolMajor; 1],
        server_major: ClientProtocolMajor,
    },
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum UncheckedProtocolCompatibilityDto {
    UpgradeRequired {
        client_major: ClientProtocolMajor,
        minimum_client_major: ClientProtocolMajor,
        server_major: ClientProtocolMajor,
    },
    IncompatibleProtocol {
        client_major: Option<ClientProtocolMajor>,
        supported_majors: [ClientProtocolMajor; 1],
        server_major: ClientProtocolMajor,
    },
}

impl ProtocolCompatibilityDto {
    pub fn try_upgrade_required(
        client_major: ClientProtocolMajor,
    ) -> Result<Self, ProtocolCompatibilityValidationError> {
        if client_major >= MINIMUM_CLIENT_PROTOCOL_MAJOR {
            return Err(ProtocolCompatibilityValidationError);
        }
        Ok(Self::UpgradeRequired {
            client_major,
            minimum_client_major: MINIMUM_CLIENT_PROTOCOL_MAJOR,
            server_major: CURRENT_CLIENT_PROTOCOL_MAJOR,
        })
    }

    pub fn try_incompatible_protocol(
        client_major: Option<ClientProtocolMajor>,
    ) -> Result<Self, ProtocolCompatibilityValidationError> {
        if client_major.is_some_and(|major| {
            major < MINIMUM_CLIENT_PROTOCOL_MAJOR
                || SUPPORTED_CLIENT_PROTOCOL_MAJORS.contains(&major)
        }) {
            return Err(ProtocolCompatibilityValidationError);
        }
        Ok(Self::IncompatibleProtocol {
            client_major,
            supported_majors: SUPPORTED_CLIENT_PROTOCOL_MAJORS,
            server_major: CURRENT_CLIENT_PROTOCOL_MAJOR,
        })
    }
}

impl<'de> Deserialize<'de> for ProtocolCompatibilityDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = UncheckedProtocolCompatibilityDto::deserialize(deserializer)?;
        match raw {
            UncheckedProtocolCompatibilityDto::UpgradeRequired {
                client_major,
                minimum_client_major,
                server_major,
            } => {
                let candidate =
                    Self::try_upgrade_required(client_major).map_err(de::Error::custom)?;
                if candidate
                    == (Self::UpgradeRequired {
                        client_major,
                        minimum_client_major,
                        server_major,
                    })
                {
                    Ok(candidate)
                } else {
                    Err(de::Error::custom("upgrade-required relation is invalid"))
                }
            }
            UncheckedProtocolCompatibilityDto::IncompatibleProtocol {
                client_major,
                supported_majors,
                server_major,
            } => {
                let candidate =
                    Self::try_incompatible_protocol(client_major).map_err(de::Error::custom)?;
                if candidate
                    == (Self::IncompatibleProtocol {
                        client_major,
                        supported_majors,
                        server_major,
                    })
                {
                    Ok(candidate)
                } else {
                    Err(de::Error::custom(
                        "incompatible-protocol relation is invalid",
                    ))
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtocolCompatibilityValidationError;

impl std::fmt::Display for ProtocolCompatibilityValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("protocol compatibility relation is invalid")
    }
}

impl std::error::Error for ProtocolCompatibilityValidationError {}

pub fn admit_protocol_major(
    client_major: Option<ClientProtocolMajor>,
) -> Result<(), ProtocolCompatibilityDto> {
    match client_major {
        Some(major) if major == CURRENT_CLIENT_PROTOCOL_MAJOR => Ok(()),
        Some(major) if major < MINIMUM_CLIENT_PROTOCOL_MAJOR => {
            Err(ProtocolCompatibilityDto::try_upgrade_required(major)
                .expect("older major has a canonical upgrade relation"))
        }
        other => Err(ProtocolCompatibilityDto::try_incompatible_protocol(other)
            .expect("unsupported major has a canonical incompatibility relation")),
    }
}
