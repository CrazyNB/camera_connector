use std::collections::BTreeSet;
use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

pub const LAN_PROJECT_SYNC_DISCOVERY_PORT: u16 = 48217;
const LAN_PROJECT_SYNC_DISCOVERY_PATH: &str = "/api/project-sync/discovery";
const DEFAULT_DISCOVERY_TIMEOUT_MS: u64 = 220;
const DISCOVERY_CONCURRENCY: usize = 32;

#[derive(Debug, Clone, Deserialize)]
pub struct LanProjectSnapshotDiscoveryRequest {
    #[serde(default)]
    pub candidate_base_urls: Vec<String>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LanProjectSnapshotSource {
    pub device_label: String,
    pub platform: String,
    pub project_name: String,
    pub snapshot_url: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct DiscoveryPayload {
    device_label: String,
    platform: String,
    project_name: String,
    #[serde(default)]
    snapshot_path: Option<String>,
    #[serde(default)]
    snapshot_url: Option<String>,
}

pub fn discover_lan_project_snapshot_sources(
    request: LanProjectSnapshotDiscoveryRequest,
) -> Result<Vec<LanProjectSnapshotSource>, String> {
    let base_urls = if request.candidate_base_urls.is_empty() {
        default_candidate_base_urls()
    } else {
        request.candidate_base_urls
    };
    if base_urls.is_empty() {
        return Ok(Vec::new());
    }

    let timeout = Duration::from_millis(request.timeout_ms.unwrap_or(DEFAULT_DISCOVERY_TIMEOUT_MS));
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|error| format!("LAN discovery client setup failed: {error}"))?;
    let mut sources = Vec::new();

    for chunk in base_urls.chunks(DISCOVERY_CONCURRENCY) {
        let handles: Vec<_> = chunk
            .iter()
            .cloned()
            .map(|base_url| {
                let client = client.clone();
                thread::spawn(move || discover_one(&client, base_url))
            })
            .collect();
        for handle in handles {
            if let Ok(Some(source)) = handle.join() {
                sources.push(source);
            }
        }
    }

    sources.sort_by(|left, right| {
        left.project_name
            .cmp(&right.project_name)
            .then_with(|| left.device_label.cmp(&right.device_label))
            .then_with(|| left.base_url.cmp(&right.base_url))
    });
    sources.dedup_by(|left, right| left.snapshot_url == right.snapshot_url);
    Ok(sources)
}

fn discover_one(
    client: &reqwest::blocking::Client,
    base_url: String,
) -> Option<LanProjectSnapshotSource> {
    let clean_base = base_url.trim_end_matches('/').to_string();
    let response = client
        .get(format!("{clean_base}{LAN_PROJECT_SYNC_DISCOVERY_PATH}"))
        .send()
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let payload: DiscoveryPayload = response.json().ok()?;
    let snapshot_url = payload
        .snapshot_url
        .filter(|value| value.starts_with("http://") || value.starts_with("https://"))
        .or_else(|| {
            payload
                .snapshot_path
                .map(|path| format!("{clean_base}/{}", path.trim_start_matches('/')))
        })?;
    Some(LanProjectSnapshotSource {
        device_label: payload.device_label,
        platform: payload.platform,
        project_name: payload.project_name,
        snapshot_url,
        base_url: clean_base,
    })
}

fn default_candidate_base_urls() -> Vec<String> {
    let mut addresses = local_ipv4_addresses();
    if addresses.is_empty() {
        if let Some(address) = primary_ipv4_address() {
            addresses.push(address);
        }
    }
    candidate_base_urls_for_ipv4_addresses(addresses)
}

fn candidate_base_urls_for_ipv4_addresses(
    addresses: impl IntoIterator<Item = Ipv4Addr>,
) -> Vec<String> {
    let mut hosts = BTreeSet::new();
    hosts.insert(Ipv4Addr::LOCALHOST);
    for address in addresses {
        if address.is_loopback() || address.is_unspecified() {
            continue;
        }
        let octets = address.octets();
        for host in 1..=254 {
            hosts.insert(Ipv4Addr::new(octets[0], octets[1], octets[2], host));
        }
    }
    hosts
        .into_iter()
        .map(|host| format!("http://{host}:{LAN_PROJECT_SYNC_DISCOVERY_PORT}"))
        .collect()
}

fn local_ipv4_addresses() -> Vec<Ipv4Addr> {
    local_ip_address::list_afinet_netifas()
        .map(|interfaces| {
            interfaces
                .into_iter()
                .filter_map(|(_, address)| match address {
                    IpAddr::V4(address) if !address.is_loopback() && !address.is_unspecified() => {
                        Some(address)
                    }
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn primary_ipv4_address() -> Option<Ipv4Addr> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    socket.connect((Ipv4Addr::new(8, 8, 8, 8), 80)).ok()?;
    match socket.local_addr().ok()?.ip() {
        std::net::IpAddr::V4(address) if !address.is_loopback() => Some(address),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    #[test]
    fn discovers_project_snapshot_source_from_candidate_base_url() {
        let url = serve_discovery_once(
            r#"{
              "device_label": "Android Field Kit",
              "platform": "android",
              "project_name": "Wedding Selects",
              "snapshot_path": "/api/s/token-1/project-snapshot"
            }"#,
        );

        let sources = discover_lan_project_snapshot_sources(LanProjectSnapshotDiscoveryRequest {
            candidate_base_urls: vec![url],
            timeout_ms: Some(1_000),
        })
        .expect("discovery should succeed");

        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].device_label, "Android Field Kit");
        assert_eq!(sources[0].platform, "android");
        assert_eq!(sources[0].project_name, "Wedding Selects");
        assert!(sources[0]
            .snapshot_url
            .ends_with("/api/s/token-1/project-snapshot"));
    }

    #[test]
    fn candidate_base_urls_cover_each_local_ipv4_subnet() {
        let urls = candidate_base_urls_for_ipv4_addresses([
            Ipv4Addr::new(192, 168, 1, 12),
            Ipv4Addr::new(10, 0, 3, 24),
        ]);

        assert!(urls.contains(&format!(
            "http://192.168.1.1:{LAN_PROJECT_SYNC_DISCOVERY_PORT}"
        )));
        assert!(urls.contains(&format!(
            "http://192.168.1.254:{LAN_PROJECT_SYNC_DISCOVERY_PORT}"
        )));
        assert!(urls.contains(&format!(
            "http://10.0.3.1:{LAN_PROJECT_SYNC_DISCOVERY_PORT}"
        )));
        assert!(urls.contains(&format!(
            "http://10.0.3.254:{LAN_PROJECT_SYNC_DISCOVERY_PORT}"
        )));
        assert!(urls.contains(&format!(
            "http://127.0.0.1:{LAN_PROJECT_SYNC_DISCOVERY_PORT}"
        )));
    }

    fn serve_discovery_once(body: &'static str) -> String {
        let listener =
            std::net::TcpListener::bind("127.0.0.1:0").expect("test HTTP listener should bind");
        let url = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("test HTTP request should arrive");
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request);
            let response = format!(
                "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .expect("test HTTP response should write");
        });
        url
    }
}
