use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use crate::{
    CameraConnectorService, FtpPushServer, ImporterError, PushProtocol, ReceiverConfigRequest,
    Result,
};

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
            let mut inner = self
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
            inner.status = ReceiverRuntimeStatus {
                phase: ReceiverRuntimePhase::Starting,
                protocol: Some(request.protocol),
                auth_mode: ReceiverAuthMode::Anonymous,
                local_addr: None,
                output_dir: Some(request.output_dir.clone()),
                account_count: 0,
                message: None,
            };
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
        let account_count = config.accounts.len();
        let server = match FtpPushServer::bind(config).await {
            Ok(server) => server,
            Err(error) => {
                self.set_status(ReceiverRuntimeStatus {
                    phase: ReceiverRuntimePhase::Failed,
                    protocol: Some(protocol),
                    auth_mode: auth_mode(account_count),
                    local_addr: None,
                    output_dir: Some(output_dir),
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
                Ok(()) => stopped_status(),
                Err(error) => ReceiverRuntimeStatus {
                    phase: ReceiverRuntimePhase::Failed,
                    protocol: Some(protocol),
                    auth_mode: auth_mode(account_count),
                    local_addr: None,
                    output_dir: Some(task_output_dir),
                    account_count,
                    message: Some(error.to_string()),
                },
            };
        });

        let status = ReceiverRuntimeStatus {
            phase: ReceiverRuntimePhase::Running,
            protocol: Some(protocol),
            auth_mode: auth_mode(account_count),
            local_addr: Some(local_addr),
            output_dir: Some(output_dir),
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
        Ok(status)
    }

    pub async fn stop_receiver(&self) -> Result<ReceiverRuntimeStatus> {
        let (shutdown, task) = {
            let mut inner = self
                .inner
                .lock()
                .expect("receiver runtime mutex should not be poisoned");
            if !matches!(inner.status.phase, ReceiverRuntimePhase::Running) {
                inner.status = stopped_status();
                inner.shutdown = None;
                inner.task = None;
                return Ok(inner.status.clone());
            }
            inner.status.phase = ReceiverRuntimePhase::Stopping;
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
            account_count: 0,
            message: Some(message),
        });
    }

    fn set_status(&self, status: ReceiverRuntimeStatus) {
        self.inner
            .lock()
            .expect("receiver runtime mutex should not be poisoned")
            .status = status;
    }
}

fn stopped_status() -> ReceiverRuntimeStatus {
    ReceiverRuntimeStatus {
        phase: ReceiverRuntimePhase::Stopped,
        protocol: None,
        auth_mode: ReceiverAuthMode::Anonymous,
        local_addr: None,
        output_dir: None,
        account_count: 0,
        message: None,
    }
}

fn auth_mode(account_count: usize) -> ReceiverAuthMode {
    if account_count == 0 {
        ReceiverAuthMode::Anonymous
    } else {
        ReceiverAuthMode::Accounts
    }
}
