use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

use crate::{CameraEndpoint, EndpointSource, ImporterError, Result};

pub async fn scan_subnet_for_ptp(
    cidr: &str,
    port: u16,
    timeout_duration: Duration,
    concurrency: usize,
) -> Result<Vec<CameraEndpoint>> {
    let hosts = hosts_in_ipv4_cidr(cidr)?;
    let concurrency = concurrency.max(1);
    let mut found = Vec::new();

    for chunk in hosts.chunks(concurrency) {
        let mut tasks = Vec::with_capacity(chunk.len());
        for host in chunk {
            let host = *host;
            tasks.push(tokio::spawn(async move {
                if probe(host, port, timeout_duration).await {
                    Some(CameraEndpoint::new(
                        host.to_string(),
                        port,
                        EndpointSource::LanScan,
                    ))
                } else {
                    None
                }
            }));
        }

        for task in tasks {
            if let Ok(Some(endpoint)) = task.await {
                found.push(endpoint);
            }
        }
    }

    Ok(found)
}

pub fn hosts_in_ipv4_cidr(cidr: &str) -> Result<Vec<Ipv4Addr>> {
    let (base, prefix) = cidr
        .split_once('/')
        .ok_or_else(|| ImporterError::internal("cidr must include prefix length"))?;
    let base: Ipv4Addr = base
        .parse()
        .map_err(|_| ImporterError::internal("invalid ipv4 address"))?;
    let prefix: u32 = prefix
        .parse()
        .map_err(|_| ImporterError::internal("invalid cidr prefix"))?;
    if prefix > 32 {
        return Err(ImporterError::internal("cidr prefix must be <= 32"));
    }

    let base_u32 = u32::from(base);
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };
    let network = base_u32 & mask;
    let host_count = 1_u64 << (32 - prefix);
    if host_count > 4096 {
        return Err(ImporterError::internal(
            "refusing to expand more than 4096 hosts",
        ));
    }

    let mut hosts = Vec::new();
    for offset in 0..host_count {
        let candidate = Ipv4Addr::from(network.wrapping_add(offset as u32));
        if prefix <= 30 && (offset == 0 || offset == host_count - 1) {
            continue;
        }
        hosts.push(candidate);
    }
    Ok(hosts)
}

async fn probe(host: Ipv4Addr, port: u16, timeout_duration: Duration) -> bool {
    let addr = SocketAddr::new(IpAddr::V4(host), port);
    timeout(timeout_duration, TcpStream::connect(addr))
        .await
        .is_ok_and(|result| result.is_ok())
}
