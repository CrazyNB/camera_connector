use std::future::Future;
use std::net::SocketAddr;

use super::{FtpPushServer, PushProtocol, PushReceiverConfig};
use crate::{ImporterError, Result};

pub enum PushReceiverServer {
    Ftp(FtpPushServer),
}

impl PushReceiverServer {
    pub async fn bind(config: PushReceiverConfig) -> Result<Self> {
        match config.protocol {
            PushProtocol::Ftp => FtpPushServer::bind(config).await.map(Self::Ftp),
            PushProtocol::Sftp => Err(ImporterError::UnsupportedProtocol),
        }
    }

    pub fn local_addr(&self) -> SocketAddr {
        match self {
            Self::Ftp(server) => server.local_addr(),
        }
    }

    pub async fn run_until(self, shutdown: impl Future<Output = ()>) -> Result<()> {
        match self {
            Self::Ftp(server) => server.run_until(shutdown).await,
        }
    }
}
