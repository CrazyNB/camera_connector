use std::path::PathBuf;

use rusqlite::{params, Connection, OptionalExtension, Row};

use super::{collect_rows, current_time_ms, sqlite_data_error, SqliteStore};
use crate::{
    ConnectedDevice, PushProtocol, ReceiverAccountConfig, ReceiverAuthMode, ReceiverRuntimePhase,
    ReceiverRuntimeStatus, Result, StoredReceiverAccount,
};

impl SqliteStore {
    pub fn upsert_receiver_account(
        &self,
        account: ReceiverAccountConfig,
    ) -> Result<StoredReceiverAccount> {
        let account = account.validated()?;
        let now = current_time_ms();
        self.with_connection(|connection| {
            let created_at_ms = connection
                .query_row(
                    "SELECT created_at_ms FROM receiver_accounts WHERE username = ?1",
                    params![&account.username],
                    |row| row.get::<_, i64>(0),
                )
                .optional()?
                .unwrap_or(now);
            connection.execute(
                "INSERT INTO receiver_accounts (
                    username, password_hash, device_name, enabled, created_at_ms, updated_at_ms
                 ) VALUES (?1, ?2, ?3, 1, ?4, ?5)
                 ON CONFLICT(username) DO UPDATE SET
                    password_hash = excluded.password_hash,
                    device_name = excluded.device_name,
                    enabled = 1,
                    updated_at_ms = excluded.updated_at_ms",
                params![
                    &account.username,
                    account.password_hash.as_deref(),
                    &account.device_name,
                    created_at_ms,
                    now,
                ],
            )?;
            receiver_account_by_username(connection, &account.username)?.ok_or_else(|| {
                rusqlite::Error::InvalidParameterName("receiver account not found".to_string())
            })
        })
    }

    pub fn remove_receiver_account(&self, username: &str) -> Result<bool> {
        self.with_connection(|connection| {
            let changed = connection.execute(
                "DELETE FROM receiver_accounts WHERE username = ?1",
                params![username],
            )?;
            if changed > 0 {
                connection.execute(
                    "UPDATE connected_devices SET username = NULL WHERE username = ?1",
                    params![username],
                )?;
            }
            Ok(changed > 0)
        })
    }

    pub fn receiver_accounts(&self) -> Result<Vec<StoredReceiverAccount>> {
        self.with_read_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT username, password_hash, device_name, enabled, created_at_ms, updated_at_ms
                 FROM receiver_accounts
                 WHERE enabled = 1
                 ORDER BY username ASC",
            )?;
            let rows = statement.query_map([], receiver_account_from_row)?;
            collect_rows(rows)
        })
    }

    pub fn record_connected_device(
        &self,
        remote_addr: &str,
        remote_port: Option<u16>,
        source_name: Option<&str>,
        username: Option<&str>,
    ) -> Result<()> {
        let now = current_time_ms();
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let mut devices = connected_devices_from_connection(&transaction)?;
            if let Some(device) = devices
                .iter_mut()
                .find(|device| device.remote_addr == remote_addr)
            {
                device.last_seen_at_ms = now;
                device.last_remote_port = remote_port;
                device.active_connections = device.active_connections.saturating_add(1);
                device.online = true;
                if let Some(source_name) = source_name {
                    device.source_name = Some(source_name.to_string());
                }
                if let Some(username) = username {
                    device.username = Some(username.to_string());
                }
            } else {
                devices.push(ConnectedDevice {
                    remote_addr: remote_addr.to_string(),
                    source_name: source_name.map(ToOwned::to_owned),
                    username: username.map(ToOwned::to_owned),
                    first_seen_at_ms: now,
                    last_seen_at_ms: now,
                    last_disconnected_at_ms: None,
                    last_remote_port: remote_port,
                    active_connections: 1,
                    online: true,
                });
            }
            replace_connected_devices(&transaction, &devices)?;
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn record_authenticated_device(
        &self,
        remote_addr: &str,
        source_name: Option<&str>,
        username: Option<&str>,
    ) -> Result<()> {
        let now = current_time_ms();
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let mut devices = connected_devices_from_connection(&transaction)?;
            let previous_for_username = username.and_then(|username| {
                devices
                    .iter()
                    .filter(|device| {
                        device.username.as_deref() == Some(username)
                            && device.remote_addr != remote_addr
                    })
                    .fold(None, |previous: Option<ConnectedDevice>, device| {
                        Some(match previous {
                            Some(previous)
                                if previous.first_seen_at_ms <= device.first_seen_at_ms =>
                            {
                                previous
                            }
                            _ => device.clone(),
                        })
                    })
            });

            if let Some(username) = username {
                devices.retain(|device| {
                    device.remote_addr == remote_addr
                        || device.username.as_deref() != Some(username)
                });
            }

            if let Some(device) = devices
                .iter_mut()
                .find(|device| device.remote_addr == remote_addr)
            {
                if let Some(previous) = previous_for_username {
                    device.first_seen_at_ms =
                        device.first_seen_at_ms.min(previous.first_seen_at_ms);
                    if device.source_name.is_none() {
                        device.source_name = previous.source_name;
                    }
                }
                device.last_seen_at_ms = now;
                if let Some(source_name) = source_name {
                    device.source_name = Some(source_name.to_string());
                }
                if let Some(username) = username {
                    device.username = Some(username.to_string());
                }
            }

            replace_connected_devices(&transaction, &devices)?;
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn record_disconnected_device(&self, remote_addr: &str) -> Result<()> {
        let now = current_time_ms();
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let mut devices = connected_devices_from_connection(&transaction)?;
            if let Some(device) = devices
                .iter_mut()
                .find(|device| device.remote_addr == remote_addr)
            {
                device.active_connections = device.active_connections.saturating_sub(1);
                device.online = device.active_connections > 0;
                device.last_seen_at_ms = now;
                if !device.online {
                    device.last_disconnected_at_ms = Some(now);
                }
            }
            replace_connected_devices(&transaction, &devices)?;
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn mark_all_connected_devices_offline(&self) -> Result<()> {
        let now = current_time_ms();
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let mut devices = connected_devices_from_connection(&transaction)?;
            for device in devices.iter_mut().filter(|device| device.online) {
                device.active_connections = 0;
                device.online = false;
                device.last_seen_at_ms = now;
                device.last_disconnected_at_ms = Some(now);
            }
            replace_connected_devices(&transaction, &devices)?;
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn connected_devices(&self) -> Result<Vec<ConnectedDevice>> {
        self.with_read_connection(|connection| {
            let mut devices = connected_devices_from_connection(connection)?;
            sort_connected_devices(&mut devices);
            Ok(devices)
        })
    }

    pub fn write_receiver_runtime_status(&self, status: &ReceiverRuntimeStatus) -> Result<()> {
        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO receiver_status (
                    key, phase, protocol, auth_mode, local_addr, output_dir, state_dir,
                    account_count, message, updated_at_ms
                 ) VALUES ('current', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(key) DO UPDATE SET
                    phase = excluded.phase,
                    protocol = excluded.protocol,
                    auth_mode = excluded.auth_mode,
                    local_addr = excluded.local_addr,
                    output_dir = excluded.output_dir,
                    state_dir = excluded.state_dir,
                    account_count = excluded.account_count,
                    message = excluded.message,
                    updated_at_ms = excluded.updated_at_ms",
                params![
                    receiver_runtime_phase_name(status.phase),
                    status.protocol.map(push_protocol_name),
                    receiver_auth_mode_name(status.auth_mode),
                    status.local_addr.map(|addr| addr.to_string()),
                    status
                        .output_dir
                        .as_ref()
                        .map(|path| path.to_string_lossy().into_owned()),
                    status
                        .state_dir
                        .as_ref()
                        .map(|path| path.to_string_lossy().into_owned()),
                    status.account_count as i64,
                    status.message.as_deref(),
                    current_time_ms(),
                ],
            )?;
            Ok(())
        })
    }

    pub fn read_receiver_runtime_status(&self) -> Result<Option<ReceiverRuntimeStatus>> {
        self.with_read_connection(|connection| {
            connection
                .query_row(
                    "SELECT phase, protocol, auth_mode, local_addr, output_dir, state_dir,
                            account_count, message
                     FROM receiver_status
                     WHERE key = 'current'",
                    [],
                    receiver_runtime_status_from_row,
                )
                .optional()
        })
    }
}

fn receiver_account_by_username(
    connection: &Connection,
    username: &str,
) -> std::result::Result<Option<StoredReceiverAccount>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT username, password_hash, device_name, enabled, created_at_ms, updated_at_ms
             FROM receiver_accounts
             WHERE username = ?1",
            params![username],
            receiver_account_from_row,
        )
        .optional()
}

fn receiver_account_from_row(
    row: &Row<'_>,
) -> std::result::Result<StoredReceiverAccount, rusqlite::Error> {
    Ok(StoredReceiverAccount {
        username: row.get(0)?,
        password_hash: row.get(1)?,
        device_name: row.get(2)?,
        enabled: row.get::<_, i64>(3)? != 0,
        created_at_ms: row.get(4)?,
        updated_at_ms: row.get(5)?,
    })
}

fn connected_devices_from_connection(
    connection: &Connection,
) -> std::result::Result<Vec<ConnectedDevice>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT remote_addr, source_name, username, first_seen_at_ms, last_seen_at_ms,
                last_disconnected_at_ms, last_remote_port, active_connections, online
         FROM connected_devices",
    )?;
    let rows = statement.query_map([], connected_device_from_row)?;
    collect_rows(rows)
}

fn connected_device_from_row(
    row: &Row<'_>,
) -> std::result::Result<ConnectedDevice, rusqlite::Error> {
    Ok(ConnectedDevice {
        remote_addr: row.get(0)?,
        source_name: row.get(1)?,
        username: row.get(2)?,
        first_seen_at_ms: row.get(3)?,
        last_seen_at_ms: row.get(4)?,
        last_disconnected_at_ms: row.get(5)?,
        last_remote_port: row.get::<_, Option<i64>>(6)?.map(|port| port as u16),
        active_connections: row.get::<_, i64>(7)? as u32,
        online: row.get::<_, i64>(8)? != 0,
    })
}

fn replace_connected_devices(
    connection: &Connection,
    devices: &[ConnectedDevice],
) -> std::result::Result<(), rusqlite::Error> {
    connection.execute("DELETE FROM connected_devices", [])?;
    for device in devices {
        connection.execute(
            "INSERT INTO connected_devices (
                remote_addr, source_name, username, first_seen_at_ms, last_seen_at_ms,
                last_disconnected_at_ms, last_remote_port, active_connections, online
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &device.remote_addr,
                device.source_name.as_deref(),
                device.username.as_deref(),
                device.first_seen_at_ms,
                device.last_seen_at_ms,
                device.last_disconnected_at_ms,
                device.last_remote_port.map(|port| port as i64),
                device.active_connections as i64,
                if device.online { 1_i64 } else { 0_i64 },
            ],
        )?;
    }
    Ok(())
}

fn sort_connected_devices(devices: &mut [ConnectedDevice]) {
    devices.sort_by(|left, right| {
        right
            .online
            .cmp(&left.online)
            .then_with(|| right.last_seen_at_ms.cmp(&left.last_seen_at_ms))
            .then_with(|| left.remote_addr.cmp(&right.remote_addr))
    });
}

fn receiver_runtime_status_from_row(
    row: &Row<'_>,
) -> std::result::Result<ReceiverRuntimeStatus, rusqlite::Error> {
    let phase = row.get::<_, String>(0)?;
    let protocol = row.get::<_, Option<String>>(1)?;
    let auth_mode = row.get::<_, String>(2)?;
    let local_addr = row.get::<_, Option<String>>(3)?;
    let output_dir = row.get::<_, Option<String>>(4)?;
    let state_dir = row.get::<_, Option<String>>(5)?;
    Ok(ReceiverRuntimeStatus {
        phase: receiver_runtime_phase_from_name(&phase)?,
        protocol: protocol
            .as_deref()
            .map(push_protocol_from_name)
            .transpose()?,
        auth_mode: receiver_auth_mode_from_name(&auth_mode)?,
        local_addr: local_addr
            .as_deref()
            .map(|value| value.parse().map_err(sqlite_data_error))
            .transpose()?,
        output_dir: output_dir.map(PathBuf::from),
        state_dir: state_dir.map(PathBuf::from),
        account_count: row.get::<_, i64>(6)? as usize,
        message: row.get(7)?,
    })
}

fn receiver_runtime_phase_name(phase: ReceiverRuntimePhase) -> &'static str {
    match phase {
        ReceiverRuntimePhase::Stopped => "stopped",
        ReceiverRuntimePhase::Starting => "starting",
        ReceiverRuntimePhase::Running => "running",
        ReceiverRuntimePhase::Stopping => "stopping",
        ReceiverRuntimePhase::Failed => "failed",
    }
}

fn receiver_runtime_phase_from_name(
    phase: &str,
) -> std::result::Result<ReceiverRuntimePhase, rusqlite::Error> {
    match phase {
        "stopped" => Ok(ReceiverRuntimePhase::Stopped),
        "starting" => Ok(ReceiverRuntimePhase::Starting),
        "running" => Ok(ReceiverRuntimePhase::Running),
        "stopping" => Ok(ReceiverRuntimePhase::Stopping),
        "failed" => Ok(ReceiverRuntimePhase::Failed),
        value => Err(sqlite_data_error(format!(
            "invalid receiver runtime phase: {value}"
        ))),
    }
}

fn receiver_auth_mode_name(auth_mode: ReceiverAuthMode) -> &'static str {
    match auth_mode {
        ReceiverAuthMode::Anonymous => "anonymous",
        ReceiverAuthMode::Accounts => "accounts",
    }
}

fn receiver_auth_mode_from_name(
    auth_mode: &str,
) -> std::result::Result<ReceiverAuthMode, rusqlite::Error> {
    match auth_mode {
        "anonymous" => Ok(ReceiverAuthMode::Anonymous),
        "accounts" => Ok(ReceiverAuthMode::Accounts),
        value => Err(sqlite_data_error(format!(
            "invalid receiver auth mode: {value}"
        ))),
    }
}

fn push_protocol_name(protocol: PushProtocol) -> &'static str {
    match protocol {
        PushProtocol::Ftp => "ftp",
        PushProtocol::Sftp => "sftp",
    }
}

fn push_protocol_from_name(protocol: &str) -> std::result::Result<PushProtocol, rusqlite::Error> {
    match protocol {
        "ftp" => Ok(PushProtocol::Ftp),
        "sftp" => Ok(PushProtocol::Sftp),
        value => Err(sqlite_data_error(format!("invalid push protocol: {value}"))),
    }
}
