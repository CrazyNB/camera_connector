use std::future::Future;
use std::io::Write as _;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{timeout, Duration};

use crate::{
    append_transfer_record, mark_all_connected_devices_offline, record_device_authenticated,
    record_device_connected, record_device_disconnected, ImporterError, LocalFileSink,
    PushProtocol, PushReceiverConfig, ReceiverAccount, Result, TransferRecord, TransferStatus,
};

const DATA_TIMEOUT: Duration = Duration::from_secs(60);

pub struct FtpPushServer {
    listener: TcpListener,
    config: Arc<PushReceiverConfig>,
}

impl FtpPushServer {
    pub async fn bind(config: PushReceiverConfig) -> Result<Self> {
        if config.protocol != PushProtocol::Ftp {
            return Err(ImporterError::UnsupportedProtocol);
        }
        config.validate_accounts()?;
        mark_all_connected_devices_offline(&config.state_dir)?;

        let listener = TcpListener::bind((config.bind_host.as_str(), config.port)).await?;
        Ok(Self {
            listener,
            config: Arc::new(config),
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.listener
            .local_addr()
            .expect("bound FTP listener should have a local address")
    }

    pub async fn run(self) -> Result<()> {
        loop {
            let (stream, _) = self.listener.accept().await?;
            let config = Arc::clone(&self.config);
            tokio::spawn(async move {
                if let Err(error) = handle_control_connection(stream, config).await {
                    tracing::warn!(?error, "ftp control connection failed");
                }
            });
        }
    }

    pub async fn run_until(self, shutdown: impl Future<Output = ()>) -> Result<()> {
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                accept = self.listener.accept() => {
                    let (stream, _) = accept?;
                    let config = Arc::clone(&self.config);
                    tokio::spawn(async move {
                        if let Err(error) = handle_control_connection(stream, config).await {
                            tracing::warn!(?error, "ftp control connection failed");
                        }
                    });
                }
                _ = &mut shutdown => {
                    return Ok(());
                }
            }
        }
    }
}

struct ControlState {
    authenticated: bool,
    pending_user: Option<String>,
    authenticated_account: Option<ReceiverAccount>,
    cwd: String,
    passive_listener: Option<TcpListener>,
}

impl ControlState {
    fn new(config: &PushReceiverConfig) -> Self {
        Self {
            authenticated: config.accounts.is_empty()
                && config.username.is_none()
                && config.password.is_none(),
            pending_user: None,
            authenticated_account: None,
            cwd: "/".to_string(),
            passive_listener: None,
        }
    }
}

async fn handle_control_connection(
    stream: TcpStream,
    config: Arc<PushReceiverConfig>,
) -> Result<()> {
    let local_ip = stream.local_addr()?.ip();
    let peer_addr = stream.peer_addr().ok();
    let remote_addr = peer_addr.map(|addr| addr.ip().to_string());
    if let Some(remote_addr) = remote_addr.as_deref() {
        let source_name = config.resolved_source_name(Some(remote_addr));
        record_device_connected(
            &config.state_dir,
            remote_addr,
            peer_addr.map(|addr| addr.port()),
            source_name.as_deref(),
            None,
        )?;
    }
    let result: Result<()> = async {
        let mut reader = BufReader::new(stream);
        let mut state = ControlState::new(&config);

        reply(&mut reader, "220 Camera Connector FTP receiver ready").await?;

        loop {
            let mut line = String::new();
            let bytes = reader.read_line(&mut line).await?;
            if bytes == 0 {
                break Ok(());
            }

            let line = line.trim_end_matches(['\r', '\n']);
            let (command, argument) = parse_command(line);
            match command.as_str() {
                "USER" => {
                    handle_user(
                        &mut reader,
                        &config,
                        &mut state,
                        remote_addr.as_deref(),
                        argument,
                    )
                    .await?
                }
                "PASS" => {
                    handle_pass(
                        &mut reader,
                        &config,
                        &mut state,
                        remote_addr.as_deref(),
                        argument,
                    )
                    .await?
                }
                "SYST" => reply(&mut reader, "215 UNIX Type: L8").await?,
                "FEAT" => {
                    write_raw(
                        &mut reader,
                        "211-Features\r\n PASV\r\n EPSV\r\n UTF8\r\n211 End\r\n",
                    )
                    .await?
                }
                "PWD" | "XPWD" => reply(&mut reader, &format!("257 \"{}\"", state.cwd)).await?,
                "CWD" => {
                    state.cwd = normalize_cwd(&state.cwd, argument);
                    reply(&mut reader, "250 Directory changed").await?;
                }
                "CDUP" => {
                    state.cwd = parent_cwd(&state.cwd);
                    reply(&mut reader, "250 Directory changed").await?;
                }
                "MKD" | "XMKD" => {
                    let path = resolve_upload_path(&state.cwd, argument);
                    LocalFileSink::new(&config.output_dir).create_dir_all(&path)?;
                    reply(&mut reader, &format!("257 \"{path}\" created")).await?;
                }
                "TYPE" => reply(&mut reader, "200 Type set").await?,
                "OPTS" => reply(&mut reader, "200 Option accepted").await?,
                "NOOP" => reply(&mut reader, "200 OK").await?,
                "PASV" => enter_passive(&mut reader, &config, &mut state, local_ip, false).await?,
                "EPSV" => enter_passive(&mut reader, &config, &mut state, local_ip, true).await?,
                "PORT" | "EPRT" => {
                    reply(&mut reader, "502 Active mode is not supported; use PASV").await?
                }
                "LIST" | "NLST" => handle_empty_listing(&mut reader, &mut state).await?,
                "SIZE" | "MDTM" => reply(&mut reader, "550 File not found").await?,
                "STOR" => {
                    handle_stor(
                        &mut reader,
                        &config,
                        &mut state,
                        argument,
                        remote_addr.clone(),
                    )
                    .await?
                }
                "QUIT" => {
                    reply(&mut reader, "221 Goodbye").await?;
                    break Ok(());
                }
                _ => reply(&mut reader, "502 Command not implemented").await?,
            }
        }
    }
    .await;

    let disconnect_result = remote_addr
        .as_deref()
        .map(|remote_addr| record_device_disconnected(&config.state_dir, remote_addr))
        .unwrap_or(Ok(()));

    match (result, disconnect_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Ok(()), Err(error)) => Err(error),
        (Err(error), _) => Err(error),
    }
}

async fn handle_user(
    reader: &mut BufReader<TcpStream>,
    config: &PushReceiverConfig,
    state: &mut ControlState,
    remote_addr: Option<&str>,
    username: &str,
) -> Result<()> {
    if !config.accounts.is_empty() {
        let Some(account) = config.account_for_username(username).cloned() else {
            reply(reader, "530 Invalid username").await?;
            return Ok(());
        };

        state.pending_user = Some(username.to_string());
        if account.password.is_some() {
            state.authenticated_account = Some(account);
            reply(reader, "331 Password required").await
        } else {
            state.authenticated = true;
            state.authenticated_account = Some(account.clone());
            if let Some(remote_addr) = remote_addr {
                record_device_authenticated(
                    &config.state_dir,
                    remote_addr,
                    Some(&account.device_name),
                    Some(&account.username),
                )?;
            }
            reply(reader, "230 Login successful").await
        }
    } else {
        if let Some(expected) = &config.username {
            if username != expected {
                reply(reader, "530 Invalid username").await?;
                return Ok(());
            }
        }

        state.pending_user = Some(username.to_string());
        if config.password.is_some() {
            reply(reader, "331 Password required").await
        } else {
            state.authenticated = true;
            reply(reader, "230 Login successful").await
        }
    }
}

async fn handle_pass(
    reader: &mut BufReader<TcpStream>,
    config: &PushReceiverConfig,
    state: &mut ControlState,
    remote_addr: Option<&str>,
    password: &str,
) -> Result<()> {
    if !config.accounts.is_empty() {
        let Some(account) = state.authenticated_account.clone() else {
            reply(reader, "530 Authentication failed").await?;
            return Ok(());
        };
        let password_ok = account
            .password
            .as_ref()
            .map(|configured| configured.verify(password))
            .transpose()?
            .unwrap_or(true);

        if state.pending_user.as_deref() == Some(&account.username) && password_ok {
            state.authenticated = true;
            if let Some(remote_addr) = remote_addr {
                record_device_authenticated(
                    &config.state_dir,
                    remote_addr,
                    Some(&account.device_name),
                    Some(&account.username),
                )?;
            }
            reply(reader, "230 Login successful").await
        } else {
            state.authenticated = false;
            state.authenticated_account = None;
            reply(reader, "530 Authentication failed").await
        }
    } else {
        let password_ok = config
            .password
            .as_deref()
            .map(|expected| expected == password)
            .unwrap_or(true);

        if state.pending_user.is_some() && password_ok {
            state.authenticated = true;
            reply(reader, "230 Login successful").await
        } else {
            reply(reader, "530 Authentication failed").await
        }
    }
}

async fn enter_passive(
    reader: &mut BufReader<TcpStream>,
    config: &PushReceiverConfig,
    state: &mut ControlState,
    local_ip: IpAddr,
    extended: bool,
) -> Result<()> {
    if !state.authenticated {
        reply(reader, "530 Login required").await?;
        return Ok(());
    }

    let listener = TcpListener::bind((config.bind_host.as_str(), 0)).await?;
    let port = listener.local_addr()?.port();
    state.passive_listener = Some(listener);

    if extended {
        reply(
            reader,
            &format!("229 Entering Extended Passive Mode (|||{port}|)"),
        )
        .await
    } else {
        let advertised_ip = advertised_ip(config, local_ip);
        let p1 = port / 256;
        let p2 = port % 256;
        reply(
            reader,
            &format!(
                "227 Entering Passive Mode ({},{},{},{},{p1},{p2})",
                advertised_ip.octets()[0],
                advertised_ip.octets()[1],
                advertised_ip.octets()[2],
                advertised_ip.octets()[3]
            ),
        )
        .await
    }
}

async fn handle_empty_listing(
    reader: &mut BufReader<TcpStream>,
    state: &mut ControlState,
) -> Result<()> {
    let Some(listener) = state.passive_listener.take() else {
        reply(reader, "425 Use PASV first").await?;
        return Ok(());
    };

    reply(reader, "150 Opening data connection").await?;
    let (mut data, _) = timeout(DATA_TIMEOUT, listener.accept()).await??;
    data.shutdown().await?;
    reply(reader, "226 Transfer complete").await
}

async fn handle_stor(
    reader: &mut BufReader<TcpStream>,
    config: &PushReceiverConfig,
    state: &mut ControlState,
    argument: &str,
    remote_addr: Option<String>,
) -> Result<()> {
    if !state.authenticated {
        reply(reader, "530 Login required").await?;
        return Ok(());
    }

    let Some(listener) = state.passive_listener.take() else {
        reply(reader, "425 Use PASV first").await?;
        return Ok(());
    };

    let upload_path = resolve_upload_path(&state.cwd, argument);
    let started_at_ms = current_time_ms();
    let transfer_id = format!("ftp:{started_at_ms}:{upload_path}");
    reply(reader, "150 Opening data connection").await?;
    let (mut data, _) = timeout(DATA_TIMEOUT, listener.accept()).await??;
    let mut upload =
        LocalFileSink::new(&config.output_dir).begin_write(&transfer_id, &upload_path)?;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let bytes_read = timeout(DATA_TIMEOUT, data.read(&mut buffer)).await??;
        if bytes_read == 0 {
            break;
        }
        upload.write_all(&buffer[..bytes_read])?;
    }
    let progress = upload.finish()?;
    let final_path = progress
        .output_path
        .clone()
        .ok_or_else(|| ImporterError::internal("completed transfer missing output path"))?;
    let source_name = state
        .authenticated_account
        .as_ref()
        .map(|account| account.device_name.clone())
        .or_else(|| config.resolved_source_name(remote_addr.as_deref()));
    let username = state
        .authenticated_account
        .as_ref()
        .map(|account| account.username.clone())
        .or_else(|| {
            state
                .authenticated
                .then(|| state.pending_user.clone())
                .flatten()
        });
    let record = TransferRecord {
        transfer_id,
        protocol: "ftp".to_string(),
        status: TransferStatus::Completed,
        original_path: upload_path,
        final_filename: progress.filename,
        final_path: Some(final_path),
        final_location: progress.output_location,
        size_bytes: progress.bytes_written,
        username,
        remote_addr,
        source_name,
        started_at_ms,
        completed_at_ms: Some(current_time_ms()),
        error: None,
    };
    append_transfer_record(&config.state_dir, &record)?;
    config.record_storage_transfer(&record)?;
    reply(reader, "226 Transfer complete").await
}

async fn reply(reader: &mut BufReader<TcpStream>, message: &str) -> Result<()> {
    write_raw(reader, &format!("{message}\r\n")).await
}

async fn write_raw(reader: &mut BufReader<TcpStream>, message: &str) -> Result<()> {
    reader.get_mut().write_all(message.as_bytes()).await?;
    Ok(())
}

fn parse_command(line: &str) -> (String, &str) {
    let Some((command, argument)) = line.split_once(' ') else {
        return (line.to_ascii_uppercase(), "");
    };
    (command.to_ascii_uppercase(), argument.trim())
}

fn normalize_cwd(cwd: &str, argument: &str) -> String {
    let raw = if argument.starts_with('/') {
        argument.to_string()
    } else if cwd == "/" {
        format!("/{argument}")
    } else {
        format!("{cwd}/{argument}")
    };

    let parts: Vec<&str> = raw
        .split('/')
        .filter(|part| !part.is_empty() && *part != "." && *part != "..")
        .collect();

    if parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", parts.join("/"))
    }
}

fn parent_cwd(cwd: &str) -> String {
    let mut parts: Vec<&str> = cwd.split('/').filter(|part| !part.is_empty()).collect();
    parts.pop();
    if parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", parts.join("/"))
    }
}

fn resolve_upload_path(cwd: &str, argument: &str) -> String {
    let raw = normalize_cwd(cwd, argument);
    raw.trim_start_matches('/').to_string()
}

fn advertised_ip(config: &PushReceiverConfig, local_ip: IpAddr) -> Ipv4Addr {
    config
        .advertised_host
        .as_deref()
        .and_then(|host| host.parse().ok())
        .or_else(|| match local_ip {
            IpAddr::V4(ip) if !ip.is_unspecified() => Some(ip),
            _ => None,
        })
        .unwrap_or(Ipv4Addr::LOCALHOST)
}

fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}
