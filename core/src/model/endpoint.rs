use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CameraEndpoint {
    pub host: String,
    pub port: u16,
    pub source: EndpointSource,
}

impl CameraEndpoint {
    pub fn new(host: impl Into<String>, port: u16, source: EndpointSource) -> Self {
        Self {
            host: host.into(),
            port,
            source,
        }
    }

    pub fn nikon_default(host: impl Into<String>, source: EndpointSource) -> Self {
        Self::new(host, 15740, source)
    }

    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndpointSource {
    Manual,
    LanScan,
    CameraApDefault,
    PreviousSuccessful,
}
