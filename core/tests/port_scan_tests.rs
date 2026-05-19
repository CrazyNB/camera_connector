use nikon_importer_core::scanner::hosts_in_ipv4_cidr;

#[test]
fn expands_single_host_cidr() {
    let hosts = hosts_in_ipv4_cidr("127.0.0.1/32").expect("cidr should parse");

    assert_eq!(hosts.len(), 1);
    assert_eq!(hosts[0].to_string(), "127.0.0.1");
}

#[test]
fn skips_network_and_broadcast_for_24() {
    let hosts = hosts_in_ipv4_cidr("192.168.1.0/24").expect("cidr should parse");

    assert_eq!(hosts.len(), 254);
    assert_eq!(hosts[0].to_string(), "192.168.1.1");
    assert_eq!(hosts[253].to_string(), "192.168.1.254");
}

#[test]
fn rejects_large_ranges() {
    let result = hosts_in_ipv4_cidr("10.0.0.0/8");

    assert!(result.is_err());
}
