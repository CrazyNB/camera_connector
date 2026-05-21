use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use russh::keys::{Algorithm, PrivateKey};
use russh::server::Server as _;
use tokio::net::TcpListener;

use crate::{ImporterError, PushProtocol, PushReceiverConfig, Result};

pub struct SftpPushServer {
    listener: TcpListener,
    ssh_config: Arc<russh::server::Config>,
}

impl SftpPushServer {
    pub async fn bind(config: PushReceiverConfig) -> Result<Self> {
        if config.protocol != PushProtocol::Sftp {
            return Err(ImporterError::UnsupportedProtocol);
        }
        config.validate_accounts()?;

        let listener = TcpListener::bind((config.bind_host.as_str(), config.port)).await?;
        let ssh_config = russh::server::Config {
            auth_rejection_time: Duration::from_millis(200),
            auth_rejection_time_initial: Some(Duration::from_millis(0)),
            keys: vec![PrivateKey::random(&mut rand::rng(), Algorithm::Ed25519)
                .map_err(|error| ImporterError::internal(error.to_string()))?],
            ..Default::default()
        };

        Ok(Self {
            listener,
            ssh_config: Arc::new(ssh_config),
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.listener
            .local_addr()
            .expect("bound SFTP listener should have a local address")
    }

    pub async fn run_until(self, shutdown: impl Future<Output = ()>) -> Result<()> {
        tokio::pin!(shutdown);
        let mut server = SshServer;
        let running = server.run_on_socket(self.ssh_config, &self.listener);
        let handle = running.handle();
        tokio::pin!(running);

        tokio::select! {
            result = &mut running => result.map_err(ImporterError::from),
            _ = &mut shutdown => {
                handle.shutdown("camera connector receiver stopped".to_string());
                running.await.map_err(ImporterError::from)
            }
        }
    }
}

#[derive(Clone)]
struct SshServer;

impl russh::server::Server for SshServer {
    type Handler = SshSession;

    fn new_client(&mut self, _: Option<SocketAddr>) -> Self::Handler {
        SshSession
    }
}

struct SshSession;

impl russh::server::Handler for SshSession {
    type Error = russh::Error;
}
