use crate::{ImporterError, PushProtocol, PushReceiverConfig, Result};

pub struct SftpPushServer;

impl SftpPushServer {
    pub async fn bind(config: PushReceiverConfig) -> Result<Self> {
        if config.protocol != PushProtocol::Sftp {
            return Err(ImporterError::UnsupportedProtocol);
        }
        Err(ImporterError::UnsupportedProtocol)
    }
}
