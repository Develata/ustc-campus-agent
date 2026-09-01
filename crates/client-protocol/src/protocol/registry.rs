use serde::{Deserialize, Deserializer, Serialize, de};

use crate::value::WireText;

use super::{
    CLIENT_PROTOCOL_SCHEMA, CLIENT_RESULT_SCHEMA, CURRENT_CLIENT_PROTOCOL_MAJOR,
    ClientProtocolMajor, MINIMUM_CLIENT_PROTOCOL_MAJOR, OPERATION_REGISTRY_REVISION,
    SUPPORTED_CLIENT_PROTOCOL_MAJORS,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationIdDto {
    #[serde(rename = "server.info")]
    ServerInfo,
    #[serde(rename = "capability.list")]
    CapabilityList,
    #[serde(rename = "affairs.get")]
    AffairsGet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientAdapterDto {
    Web,
    Cli,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationSchemaDto {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "server-info/v1")]
    ServerInfoV1,
    #[serde(rename = "capability-list/v1")]
    CapabilityListV1,
    #[serde(rename = "affairs-get-query/v1")]
    AffairsGetQueryV1,
    #[serde(rename = "client-response/v1")]
    ClientResponseV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicPermissionClassDto {
    PublicRead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationEffectClassDto {
    Read,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethodDto {
    Get,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HttpRouteDto {
    #[serde(rename = "/api/v1/server/info")]
    ServerInfo,
    #[serde(rename = "/api/v1/client/capabilities")]
    CapabilityList,
    #[serde(rename = "/api/v1/affairs/{procedure_id}?as_of=<unix-ms>")]
    AffairsGet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OperationDescriptorDto {
    operation_id: OperationIdDto,
    protocol_major: ClientProtocolMajor,
    request_schema: OperationSchemaDto,
    result_schema: OperationSchemaDto,
    permission_class: PublicPermissionClassDto,
    effect_class: OperationEffectClassDto,
    method: HttpMethodDto,
    route: HttpRouteDto,
    requires_protocol_major: bool,
    adapters: [ClientAdapterDto; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UncheckedOperationDescriptorDto {
    operation_id: OperationIdDto,
    protocol_major: ClientProtocolMajor,
    request_schema: OperationSchemaDto,
    result_schema: OperationSchemaDto,
    permission_class: PublicPermissionClassDto,
    effect_class: OperationEffectClassDto,
    method: HttpMethodDto,
    route: HttpRouteDto,
    requires_protocol_major: bool,
    adapters: [ClientAdapterDto; 2],
}

impl OperationDescriptorDto {
    fn canonical(operation_id: OperationIdDto) -> Self {
        let (request_schema, result_schema, route, requires_protocol_major) = match operation_id {
            OperationIdDto::ServerInfo => (
                OperationSchemaDto::None,
                OperationSchemaDto::ServerInfoV1,
                HttpRouteDto::ServerInfo,
                false,
            ),
            OperationIdDto::CapabilityList => (
                OperationSchemaDto::None,
                OperationSchemaDto::CapabilityListV1,
                HttpRouteDto::CapabilityList,
                true,
            ),
            OperationIdDto::AffairsGet => (
                OperationSchemaDto::AffairsGetQueryV1,
                OperationSchemaDto::ClientResponseV1,
                HttpRouteDto::AffairsGet,
                true,
            ),
        };
        Self {
            operation_id,
            protocol_major: CURRENT_CLIENT_PROTOCOL_MAJOR,
            request_schema,
            result_schema,
            permission_class: PublicPermissionClassDto::PublicRead,
            effect_class: OperationEffectClassDto::Read,
            method: HttpMethodDto::Get,
            route,
            requires_protocol_major,
            adapters: [ClientAdapterDto::Web, ClientAdapterDto::Cli],
        }
    }

    #[must_use]
    pub const fn operation_id(&self) -> OperationIdDto {
        self.operation_id
    }

    #[must_use]
    pub const fn protocol_major(&self) -> ClientProtocolMajor {
        self.protocol_major
    }

    #[must_use]
    pub const fn adapters(&self) -> &[ClientAdapterDto; 2] {
        &self.adapters
    }

    #[must_use]
    pub const fn requires_protocol_major(&self) -> bool {
        self.requires_protocol_major
    }
}

impl<'de> Deserialize<'de> for OperationDescriptorDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = UncheckedOperationDescriptorDto::deserialize(deserializer)?;
        let candidate = Self {
            operation_id: raw.operation_id,
            protocol_major: raw.protocol_major,
            request_schema: raw.request_schema,
            result_schema: raw.result_schema,
            permission_class: raw.permission_class,
            effect_class: raw.effect_class,
            method: raw.method,
            route: raw.route,
            requires_protocol_major: raw.requires_protocol_major,
            adapters: raw.adapters,
        };
        if candidate == Self::canonical(candidate.operation_id) {
            Ok(candidate)
        } else {
            Err(de::Error::custom("operation descriptor is not canonical"))
        }
    }
}

fn canonical_operations() -> [OperationDescriptorDto; 3] {
    [
        OperationDescriptorDto::canonical(OperationIdDto::ServerInfo),
        OperationDescriptorDto::canonical(OperationIdDto::CapabilityList),
        OperationDescriptorDto::canonical(OperationIdDto::AffairsGet),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CapabilityListDto {
    registry_revision: WireText,
    protocol_major: ClientProtocolMajor,
    operations: [OperationDescriptorDto; 3],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UncheckedCapabilityListDto {
    registry_revision: WireText,
    protocol_major: ClientProtocolMajor,
    operations: [OperationDescriptorDto; 3],
}

impl CapabilityListDto {
    #[must_use]
    pub fn affairs_first() -> Self {
        Self {
            registry_revision: static_text(OPERATION_REGISTRY_REVISION),
            protocol_major: CURRENT_CLIENT_PROTOCOL_MAJOR,
            operations: canonical_operations(),
        }
    }

    #[must_use]
    pub fn registry_revision(&self) -> &WireText {
        &self.registry_revision
    }

    #[must_use]
    pub const fn protocol_major(&self) -> ClientProtocolMajor {
        self.protocol_major
    }

    #[must_use]
    pub const fn operations(&self) -> &[OperationDescriptorDto; 3] {
        &self.operations
    }

    pub fn operations_for(
        &self,
        adapter: ClientAdapterDto,
    ) -> impl Iterator<Item = &OperationDescriptorDto> {
        self.operations
            .iter()
            .filter(move |descriptor| descriptor.adapters.contains(&adapter))
    }
}

impl<'de> Deserialize<'de> for CapabilityListDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = UncheckedCapabilityListDto::deserialize(deserializer)?;
        let candidate = Self {
            registry_revision: raw.registry_revision,
            protocol_major: raw.protocol_major,
            operations: raw.operations,
        };
        if candidate == Self::affairs_first() {
            Ok(candidate)
        } else {
            Err(de::Error::custom("capability list is not canonical"))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerInfoDto {
    protocol_schema: WireText,
    protocol_major: ClientProtocolMajor,
    supported_protocol_majors: [ClientProtocolMajor; 1],
    minimum_client_protocol_major: ClientProtocolMajor,
    result_schema: WireText,
    server_build: WireText,
    capabilities_route: HttpRouteDto,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UncheckedServerInfoDto {
    protocol_schema: WireText,
    protocol_major: ClientProtocolMajor,
    supported_protocol_majors: [ClientProtocolMajor; 1],
    minimum_client_protocol_major: ClientProtocolMajor,
    result_schema: WireText,
    server_build: WireText,
    capabilities_route: HttpRouteDto,
}

impl ServerInfoDto {
    #[must_use]
    pub fn new(server_build: WireText) -> Self {
        Self {
            protocol_schema: static_text(CLIENT_PROTOCOL_SCHEMA),
            protocol_major: CURRENT_CLIENT_PROTOCOL_MAJOR,
            supported_protocol_majors: SUPPORTED_CLIENT_PROTOCOL_MAJORS,
            minimum_client_protocol_major: MINIMUM_CLIENT_PROTOCOL_MAJOR,
            result_schema: static_text(CLIENT_RESULT_SCHEMA),
            server_build,
            capabilities_route: HttpRouteDto::CapabilityList,
        }
    }

    #[must_use]
    pub const fn protocol_major(&self) -> ClientProtocolMajor {
        self.protocol_major
    }

    #[must_use]
    pub const fn supported_protocol_majors(&self) -> &[ClientProtocolMajor; 1] {
        &self.supported_protocol_majors
    }

    #[must_use]
    pub const fn minimum_client_protocol_major(&self) -> ClientProtocolMajor {
        self.minimum_client_protocol_major
    }

    #[must_use]
    pub fn server_build(&self) -> &WireText {
        &self.server_build
    }
}

impl<'de> Deserialize<'de> for ServerInfoDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = UncheckedServerInfoDto::deserialize(deserializer)?;
        let candidate = Self {
            protocol_schema: raw.protocol_schema,
            protocol_major: raw.protocol_major,
            supported_protocol_majors: raw.supported_protocol_majors,
            minimum_client_protocol_major: raw.minimum_client_protocol_major,
            result_schema: raw.result_schema,
            server_build: raw.server_build,
            capabilities_route: raw.capabilities_route,
        };
        if candidate == Self::new(candidate.server_build.clone()) {
            Ok(candidate)
        } else {
            Err(de::Error::custom("server info is not canonical"))
        }
    }
}

fn static_text(value: &'static str) -> WireText {
    WireText::parse(value).expect("static protocol text is valid")
}
