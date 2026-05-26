use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use crate::{
    CameraConnectorService, ImporterError, PushProtocol, PushReceiverServer, ReceiverConfigRequest,
    Result, SqliteStore,
};

pub const RECEIVER_STATUS_FILENAME: &str = "receiver-status.json";

#[derive(Debug, Clone)]
pub struct CameraConnectorRuntime {
    service: CameraConnectorService,
    inner: Arc<Mutex<RuntimeInner>>,
}

#[derive(Debug)]
struct RuntimeInner {
    status: ReceiverRuntimeStatus,
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<()>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiverRuntimeStatus {
    pub phase: ReceiverRuntimePhase,
    pub protocol: Option<PushProtocol>,
    pub auth_mode: ReceiverAuthMode,
    pub local_addr: Option<SocketAddr>,
    pub output_dir: Option<PathBuf>,
    pub state_dir: Option<PathBuf>,
    pub account_count: usize,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReceiverRuntimePhase {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReceiverAuthMode {
    Anonymous,
    Accounts,
}

impl CameraConnectorRuntime {
    pub fn new(service: CameraConnectorService) -> Self {
        Self {
            service,
            inner: Arc::new(Mutex::new(RuntimeInner {
                status: stopped_status(),
                shutdown: None,
                task: None,
            })),
        }
    }

    pub fn status(&self) -> ReceiverRuntimeStatus {
        self.inner
            .lock()
            .expect("receiver runtime mutex should not be poisoned")
            .status
            .clone()
    }

    pub async fn start_receiver(
        &self,
        request: ReceiverConfigRequest,
    ) -> Result<ReceiverRuntimeStatus> {
        {
            let inner = self
                .inner
                .lock()
                .expect("receiver runtime mutex should not be poisoned");
            if matches!(
                inner.status.phase,
                ReceiverRuntimePhase::Starting
                    | ReceiverRuntimePhase::Running
                    | ReceiverRuntimePhase::Stopping
            ) {
                return Err(ImporterError::internal("receiver is already active"));
            }
        }
        let config = match self.service.receiver_config(request) {
            Ok(config) => config,
            Err(error) => {
                self.mark_failed(error.to_string());
                return Err(error);
            }
        };
        let protocol = config.protocol;
        let output_dir = config.output_dir.clone();
        let state_dir = config.state_dir.clone();
        let account_count = config.accounts.len();
        {
            let mut inner = self
                .inner
                .lock()
                .expect("receiver runtime mutex should not be poisoned");
            inner.status = ReceiverRuntimeStatus {
                phase: ReceiverRuntimePhase::Starting,
                protocol: Some(protocol),
                auth_mode: auth_mode(account_count),
                local_addr: None,
                output_dir: Some(output_dir.clone()),
                state_dir: Some(state_dir.clone()),
                account_count,
                message: None,
            };
            write_receiver_runtime_status(&state_dir, &inner.status)?;
        }

        let server = match PushReceiverServer::bind(config).await {
            Ok(server) => server,
            Err(error) => {
                self.set_status(ReceiverRuntimeStatus {
                    phase: ReceiverRuntimePhase::Failed,
                    protocol: Some(protocol),
                    auth_mode: auth_mode(account_count),
                    local_addr: None,
                    output_dir: Some(output_dir.clone()),
                    state_dir: Some(state_dir.clone()),
                    account_count,
                    message: Some(error.to_string()),
                });
                return Err(error);
            }
        };
        let local_addr = server.local_addr();
        let (shutdown, shutdown_rx) = oneshot::channel();
        let inner = Arc::clone(&self.inner);
        let task_output_dir = output_dir.clone();
        let task_state_dir = state_dir.clone();
        let task = tokio::spawn(async move {
            let result = server
                .run_until(async {
                    let _ = shutdown_rx.await;
                })
                .await;
            let mut inner = inner
                .lock()
                .expect("receiver runtime mutex should not be poisoned");
            inner.shutdown = None;
            inner.task = None;
            inner.status = match result {
                Ok(()) => stopped_status_for(task_output_dir.clone(), task_state_dir.clone()),
                Err(error) => ReceiverRuntimeStatus {
                    phase: ReceiverRuntimePhase::Failed,
                    protocol: Some(protocol),
                    auth_mode: auth_mode(account_count),
                    local_addr: None,
                    output_dir: Some(task_output_dir.clone()),
                    state_dir: Some(task_state_dir.clone()),
                    account_count,
                    message: Some(error.to_string()),
                },
            };
            let _ = write_receiver_runtime_status(&task_state_dir, &inner.status);
        });

        let status = ReceiverRuntimeStatus {
            phase: ReceiverRuntimePhase::Running,
            protocol: Some(protocol),
            auth_mode: auth_mode(account_count),
            local_addr: Some(local_addr),
            output_dir: Some(output_dir.clone()),
            state_dir: Some(state_dir.clone()),
            account_count,
            message: None,
        };
        let mut runtime = self
            .inner
            .lock()
            .expect("receiver runtime mutex should not be poisoned");
        runtime.shutdown = Some(shutdown);
        runtime.task = Some(task);
        runtime.status = status.clone();
        write_receiver_runtime_status(&state_dir, &status)?;
        Ok(status)
    }

    pub async fn stop_receiver(&self) -> Result<ReceiverRuntimeStatus> {
        let (shutdown, task) = {
            let mut inner = self
                .inner
                .lock()
                .expect("receiver runtime mutex should not be poisoned");
            if !matches!(inner.status.phase, ReceiverRuntimePhase::Running) {
                let output_dir = inner.status.output_dir.clone();
                let state_dir = inner.status.state_dir.clone();
                inner.status = output_dir
                    .zip(state_dir)
                    .map(|(output_dir, state_dir)| stopped_status_for(output_dir, state_dir))
                    .unwrap_or_else(stopped_status);
                inner.shutdown = None;
                inner.task = None;
                if let Some(state_dir) = inner.status.state_dir.as_deref() {
                    write_receiver_runtime_status(state_dir, &inner.status)?;
                }
                return Ok(inner.status.clone());
            }
            inner.status.phase = ReceiverRuntimePhase::Stopping;
            if let Some(state_dir) = inner.status.state_dir.as_deref() {
                write_receiver_runtime_status(state_dir, &inner.status)?;
            }
            (inner.shutdown.take(), inner.task.take())
        };

        if let Some(shutdown) = shutdown {
            let _ = shutdown.send(());
        }
        if let Some(task) = task {
            task.await
                .map_err(|error| ImporterError::internal(error.to_string()))?;
        }

        Ok(self.status())
    }

    fn mark_failed(&self, message: String) {
        self.set_status(ReceiverRuntimeStatus {
            phase: ReceiverRuntimePhase::Failed,
            protocol: None,
            auth_mode: ReceiverAuthMode::Anonymous,
            local_addr: None,
            output_dir: None,
            state_dir: None,
            account_count: 0,
            message: Some(message),
        });
    }

    fn set_status(&self, status: ReceiverRuntimeStatus) {
        if let Some(state_dir) = status.state_dir.as_deref() {
            let _ = write_receiver_runtime_status(state_dir, &status);
        }
        self.inner
            .lock()
            .expect("receiver runtime mutex should not be poisoned")
            .status = status;
    }
}

pub fn receiver_runtime_status_path(output_dir: impl AsRef<Path>) -> PathBuf {
    output_dir.as_ref().join(RECEIVER_STATUS_FILENAME)
}

pub fn read_receiver_runtime_status(
    output_dir: impl AsRef<Path>,
) -> Result<Option<ReceiverRuntimeStatus>> {
    Ok(SqliteStore::open_state_dir(output_dir.as_ref())?
        .read_receiver_runtime_status()?
        .map(observe_receiver_runtime_status))
}

pub fn write_receiver_runtime_status(
    output_dir: impl AsRef<Path>,
    status: &ReceiverRuntimeStatus,
) -> Result<()> {
    SqliteStore::open_state_dir(output_dir.as_ref())?.write_receiver_runtime_status(status)
}

fn stopped_status() -> ReceiverRuntimeStatus {
    ReceiverRuntimeStatus {
        phase: ReceiverRuntimePhase::Stopped,
        protocol: None,
        auth_mode: ReceiverAuthMode::Anonymous,
        local_addr: None,
        output_dir: None,
        state_dir: None,
        account_count: 0,
        message: None,
    }
}

fn stopped_status_for(output_dir: PathBuf, state_dir: PathBuf) -> ReceiverRuntimeStatus {
    ReceiverRuntimeStatus {
        phase: ReceiverRuntimePhase::Stopped,
        protocol: None,
        auth_mode: ReceiverAuthMode::Anonymous,
        local_addr: None,
        output_dir: Some(output_dir),
        state_dir: Some(state_dir),
        account_count: 0,
        message: None,
    }
}

fn observe_receiver_runtime_status(mut status: ReceiverRuntimeStatus) -> ReceiverRuntimeStatus {
    if matches!(status.phase, ReceiverRuntimePhase::Running)
        && !is_receiver_listener_alive(status.local_addr)
    {
        status.phase = ReceiverRuntimePhase::Stopped;
        status.local_addr = None;
        status.message = Some("receiver process is not listening".to_string());
    }
    status
}

fn is_receiver_listener_alive(local_addr: Option<SocketAddr>) -> bool {
    let Some(local_addr) = local_addr else {
        return false;
    };
    TcpStream::connect_timeout(&connectable_addr(local_addr), Duration::from_millis(150)).is_ok()
}

fn connectable_addr(local_addr: SocketAddr) -> SocketAddr {
    let ip = match local_addr.ip() {
        IpAddr::V4(addr) if addr.is_unspecified() => IpAddr::V4(Ipv4Addr::LOCALHOST),
        IpAddr::V6(addr) if addr.is_unspecified() => IpAddr::V6(Ipv6Addr::LOCALHOST),
        addr => addr,
    };
    SocketAddr::new(ip, local_addr.port())
}

fn auth_mode(account_count: usize) -> ReceiverAuthMode {
    if account_count == 0 {
        ReceiverAuthMode::Anonymous
    } else {
        ReceiverAuthMode::Accounts
    }
}
