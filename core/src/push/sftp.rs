use std::collections::HashMap;
use std::fs;
use std::future::Future;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use russh::keys::ssh_key::LineEnding;
use russh::keys::{Algorithm, PrivateKey};
use russh::server::Server as _;
use russh::server::{Auth, Msg, Session};
use russh::{Channel, ChannelId};
use russh_sftp::protocol::{FileAttributes, Handle, OpenFlags, Status, StatusCode, Version};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::{
    append_transfer_record, mark_all_connected_devices_offline, record_device_authenticated,
    record_device_connected, record_device_disconnected, ImporterError, LocalFileSink,
    LocalFileUpload, PushProtocol, PushReceiverConfig, ReceiverAccount, Result, TransferRecord,
    TransferStatus,
};

const SFTP_HOST_KEY_FILENAME: &str = "sftp-host-key";

pub struct SftpPushServer {
    listener: TcpListener,
    ssh_config: Arc<russh::server::Config>,
    receiver_config: Arc<PushReceiverConfig>,
}

impl SftpPushServer {
    pub async fn bind(config: PushReceiverConfig) -> Result<Self> {
        if config.protocol != PushProtocol::Sftp {
            return Err(ImporterError::UnsupportedProtocol);
        }
        config.validate_accounts()?;
        mark_all_connected_devices_offline(&config.state_dir)?;

        let listener = TcpListener::bind((config.bind_host.as_str(), config.port)).await?;
        let ssh_config = russh::server::Config {
            auth_rejection_time: Duration::from_millis(200),
            auth_rejection_time_initial: Some(Duration::from_millis(0)),
            keys: vec![load_or_create_host_key(&config.state_dir)?],
            ..Default::default()
        };

        Ok(Self {
            listener,
            ssh_config: Arc::new(ssh_config),
            receiver_config: Arc::new(config),
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.listener
            .local_addr()
            .expect("bound SFTP listener should have a local address")
    }

    pub async fn run_until(self, shutdown: impl Future<Output = ()>) -> Result<()> {
        tokio::pin!(shutdown);
        let mut server = SshServer {
            config: Arc::clone(&self.receiver_config),
        };
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
struct SshServer {
    config: Arc<PushReceiverConfig>,
}

impl russh::server::Server for SshServer {
    type Handler = SshSession;

    fn new_client(&mut self, peer_addr: Option<SocketAddr>) -> Self::Handler {
        if let Some(peer_addr) = peer_addr {
            let remote_addr = peer_addr.ip().to_string();
            let source_name = self.config.resolved_source_name(Some(&remote_addr));
            if let Err(error) = record_device_connected(
                &self.config.state_dir,
                &remote_addr,
                Some(peer_addr.port()),
                source_name.as_deref(),
                None,
            ) {
                tracing::warn!(?error, "failed to record SFTP device connection");
            }
        }
        SshSession::new(Arc::clone(&self.config), peer_addr)
    }
}

struct SshSession {
    config: Arc<PushReceiverConfig>,
    peer_addr: Option<SocketAddr>,
    authenticated_account: Option<ReceiverAccount>,
    authenticated_username: Option<String>,
    clients: Arc<Mutex<HashMap<ChannelId, Channel<Msg>>>>,
}

impl SshSession {
    fn new(config: Arc<PushReceiverConfig>, peer_addr: Option<SocketAddr>) -> Self {
        Self {
            config,
            peer_addr,
            authenticated_account: None,
            authenticated_username: None,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn take_channel(&self, channel_id: ChannelId) -> Option<Channel<Msg>> {
        self.clients.lock().await.remove(&channel_id)
    }

    fn remote_addr(&self) -> Option<String> {
        self.peer_addr.map(|addr| addr.ip().to_string())
    }

    fn source_name(&self) -> Option<String> {
        self.authenticated_account
            .as_ref()
            .map(|account| account.device_name.clone())
            .or_else(|| {
                self.config
                    .resolved_source_name(self.remote_addr().as_deref())
            })
    }
}

impl russh::server::Handler for SshSession {
    type Error = russh::Error;

    async fn auth_password(
        &mut self,
        user: &str,
        password: &str,
    ) -> std::result::Result<Auth, Self::Error> {
        if !self.config.accounts.is_empty() {
            let Some(account) = self.config.account_for_username(user).cloned() else {
                return Ok(Auth::reject());
            };
            let accepted = account
                .password
                .as_ref()
                .map(|stored| stored.verify(password).unwrap_or(false))
                .unwrap_or(true);
            if accepted {
                if let Some(remote_addr) = self.remote_addr() {
                    if let Err(error) = record_device_authenticated(
                        &self.config.state_dir,
                        &remote_addr,
                        Some(&account.device_name),
                        Some(&account.username),
                    ) {
                        tracing::warn!(?error, "failed to record SFTP device authentication");
                    }
                }
                self.authenticated_username = Some(account.username.clone());
                self.authenticated_account = Some(account);
                return Ok(Auth::Accept);
            }
            return Ok(Auth::reject());
        }

        if let Some(expected) = &self.config.username {
            if user != expected {
                return Ok(Auth::reject());
            }
        }
        if let Some(expected) = &self.config.password {
            if password != expected {
                return Ok(Auth::reject());
            }
        }
        if let Some(remote_addr) = self.remote_addr() {
            let source_name = self.source_name();
            if let Err(error) = record_device_authenticated(
                &self.config.state_dir,
                &remote_addr,
                source_name.as_deref(),
                Some(user),
            ) {
                tracing::warn!(?error, "failed to record SFTP device authentication");
            }
        }
        self.authenticated_username = Some(user.to_string());
        Ok(Auth::Accept)
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        _: &mut Session,
    ) -> std::result::Result<bool, Self::Error> {
        self.clients.lock().await.insert(channel.id(), channel);
        Ok(true)
    }

    async fn subsystem_request(
        &mut self,
        channel_id: ChannelId,
        name: &str,
        session: &mut Session,
    ) -> std::result::Result<(), Self::Error> {
        let Some(channel) = self.take_channel(channel_id).await else {
            session.channel_failure(channel_id)?;
            return Ok(());
        };

        if name != "sftp" {
            session.channel_failure(channel_id)?;
            return Ok(());
        }

        session.channel_success(channel_id)?;
        let handler = SftpSession::new(
            Arc::clone(&self.config),
            self.remote_addr(),
            self.source_name(),
            self.authenticated_username.clone(),
        );
        russh_sftp::server::run(channel.into_stream(), handler).await;
        Ok(())
    }
}

impl Drop for SshSession {
    fn drop(&mut self) {
        if let Some(remote_addr) = self.remote_addr() {
            if let Err(error) = record_device_disconnected(&self.config.state_dir, remote_addr) {
                tracing::warn!(?error, "failed to record SFTP device disconnection");
            }
        }
    }
}

struct PendingUpload {
    transfer_id: String,
    original_path: String,
    started_at_ms: i64,
    upload: LocalFileUpload,
}

struct SftpSession {
    config: Arc<PushReceiverConfig>,
    remote_addr: Option<String>,
    source_name: Option<String>,
    username: Option<String>,
    next_handle: u64,
    uploads: HashMap<String, PendingUpload>,
}

impl SftpSession {
    fn new(
        config: Arc<PushReceiverConfig>,
        remote_addr: Option<String>,
        source_name: Option<String>,
        username: Option<String>,
    ) -> Self {
        Self {
            config,
            remote_addr,
            source_name,
            username,
            next_handle: 0,
            uploads: HashMap::new(),
        }
    }
}

impl russh_sftp::server::Handler for SftpSession {
    type Error = StatusCode;

    fn unimplemented(&self) -> Self::Error {
        StatusCode::OpUnsupported
    }

    async fn init(
        &mut self,
        _: u32,
        _: HashMap<String, String>,
    ) -> std::result::Result<Version, Self::Error> {
        Ok(Version::new())
    }

    async fn open(
        &mut self,
        id: u32,
        filename: String,
        pflags: OpenFlags,
        _: FileAttributes,
    ) -> std::result::Result<Handle, Self::Error> {
        if !pflags.contains(OpenFlags::WRITE) && !pflags.contains(OpenFlags::CREATE) {
            return Err(StatusCode::PermissionDenied);
        }

        self.next_handle += 1;
        let handle = format!("upload-{}", self.next_handle);
        let started_at_ms = current_time_ms();
        let transfer_id = format!("sftp:{started_at_ms}:{filename}");
        let upload = LocalFileSink::new(&self.config.output_dir)
            .begin_write(&transfer_id, &filename)
            .map_err(|_| StatusCode::Failure)?;
        self.uploads.insert(
            handle.clone(),
            PendingUpload {
                transfer_id,
                original_path: filename,
                started_at_ms,
                upload,
            },
        );
        Ok(Handle { id, handle })
    }

    async fn write(
        &mut self,
        id: u32,
        handle: String,
        offset: u64,
        data: Vec<u8>,
    ) -> std::result::Result<Status, Self::Error> {
        let upload = self
            .uploads
            .get_mut(&handle)
            .ok_or(StatusCode::NoSuchFile)?;
        upload
            .upload
            .write_at(offset, &data)
            .map_err(|_| StatusCode::Failure)?;
        Ok(ok_status(id))
    }

    async fn close(&mut self, id: u32, handle: String) -> std::result::Result<Status, Self::Error> {
        let upload = self.uploads.remove(&handle).ok_or(StatusCode::NoSuchFile)?;
        let progress = upload.upload.finish().map_err(|_| StatusCode::Failure)?;
        append_transfer_record(
            &self.config.state_dir,
            &TransferRecord {
                transfer_id: upload.transfer_id,
                protocol: "sftp".to_string(),
                status: TransferStatus::Completed,
                original_path: upload.original_path,
                final_filename: progress.filename,
                final_path: progress.output_path,
                final_location: progress.output_location,
                size_bytes: progress.bytes_written,
                username: self.username.clone(),
                remote_addr: self.remote_addr.clone(),
                source_name: self.source_name.clone(),
                started_at_ms: upload.started_at_ms,
                completed_at_ms: Some(current_time_ms()),
                error: None,
            },
        )
        .map_err(|_| StatusCode::Failure)?;
        Ok(ok_status(id))
    }
}

fn ok_status(id: u32) -> Status {
    Status {
        id,
        status_code: StatusCode::Ok,
        error_message: "Ok".to_string(),
        language_tag: "en-US".to_string(),
    }
}

fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

pub fn sftp_host_key_path(state_dir: impl AsRef<Path>) -> PathBuf {
    state_dir.as_ref().join(SFTP_HOST_KEY_FILENAME)
}

fn load_or_create_host_key(state_dir: &Path) -> Result<PrivateKey> {
    fs::create_dir_all(state_dir)?;
    let path = sftp_host_key_path(state_dir);
    if path.exists() {
        return PrivateKey::read_openssh_file(&path)
            .map_err(|error| ImporterError::internal(error.to_string()));
    }

    let key = PrivateKey::random(&mut rand::rng(), Algorithm::Ed25519)
        .map_err(|error| ImporterError::internal(error.to_string()))?;
    key.write_openssh_file(&path, LineEnding::LF)
        .map_err(|error| ImporterError::internal(error.to_string()))?;
    Ok(key)
}
