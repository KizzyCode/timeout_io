extern crate timeout_io;
use timeout_io::*;
use std::{
	time::{ Duration, SystemTime, UNIX_EPOCH },
	net::{ SocketAddr, SocketAddrV4, SocketAddrV6, Ipv4Addr, Ipv6Addr }
};

#[test]
fn test_dns_resolve_ok() {
	"localhost:80".dns_resolve(Duration::from_secs(4)).unwrap();
}
#[test]
fn test_dns_resolve_invalid() {
	"domain.invalid:80".dns_resolve(Duration::from_secs(4)).unwrap_err();
}
#[test] #[ignore]
fn test_dns_resolve_timeout() {
	// Generate a new domain to avoid the cache
	let domain = format!("{}.invalid:80", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_ms());
	assert_eq!(
		domain.dns_resolve(Duration::from_secs(4)).unwrap_err().kind.kind,
		IoErrorKind::TimedOut
	)
}

#[test]
fn test_parse_ip_ok() {
	assert_eq!(
		"127.0.0.1:80".parse_ip().unwrap(),
		SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 80))
	);
	assert_eq!(
		"[2001:db8:0:8d3:0:8a2e:70:7344]:443".parse_ip().unwrap(),
		SocketAddr::V6(SocketAddrV6::new(
			Ipv6Addr::new(0x2001, 0x0db8, 0x0000, 0x08d3, 0x0000, 0x8a2e, 0x0070, 0x7344),
			443, 0, 0
		))
	)
}
#[test]
fn test_parse_ip_err() {
	assert_eq!("127.0.0.256:80".parse_ip().unwrap_err().kind.kind, IoErrorKind::InvalidInput);
}