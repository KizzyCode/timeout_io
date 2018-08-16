use super::{ IoError, Result, tiny_future::{ Future, async } };
use std::{
	time::Duration, str::FromStr,
	net::{ SocketAddr, ToSocketAddrs },
	io::ErrorKind as IoErrorKind
};


/// A trait for elements that contains a DNS-resolvable address
pub trait DnsResolvable {
	/// Resolves a domain-name or IP-address
	///
	/// __Warning: because `getaddrinfo` only provides a synchronously API, we have to resolve in a
	/// background-thread. This means the background-thread may outlive this call until the OS'
	/// `connect`-timeout is reached.__
	///
	/// _Important: If you want to resolve an address like "localhost" or "crates.io" you __must__
	/// include the target-port-number like this: "localhost:80" or "crates.io:443"_
	///
	/// Parameters:
	///  - `timeout`: The maximum time this function will wait until the address is resolved
	///
	/// Returns either __the resolved address__ or a corresponding `IoError`
	fn dns_resolve(&self, timeout: Duration) -> Result<SocketAddr>;
}
impl<T: ToString> DnsResolvable for T {
	fn dns_resolve(&self, timeout: Duration) -> Result<SocketAddr> {
		// Run resolver-job
		let address = self.to_string();
		let fut = async(move |fut: Future<Result<SocketAddr>>| {
			loop {
				// Check for timeout
				if !fut.is_waiting() { job_die!(fut) }
				
				// Resolve name
				match address.as_str().to_socket_addrs() {
					Ok(mut addresses) => if let Some(address) = addresses.next() { job_return!(fut, Ok(address)) }
						else { job_return!(fut, Err(new_err!(IoErrorKind::NotFound.into()))) },
					Err(error) => {
						let io_error = IoError::from(error);
						if io_error.non_recoverable { job_return!(fut, Err(new_err!(io_error))) }
					}
				};
			};
		});
		
		// Wait for result
		if let Ok(result) = fut.try_get_timeout(timeout) { result }
			else { throw_err!(IoErrorKind::TimedOut.into()) }
	}
}



/// A trait for elements that contain a parseable IP-address
pub trait IpParseable {
	/// Parses an IP-address
	///
	/// Returns either __the parsed address__ or a corresponding `IoError`
	fn parse_ip(&self) -> Result<SocketAddr>;
}
impl<T: AsRef<str>> IpParseable for T {
	fn parse_ip(&self) -> Result<SocketAddr> {
		if let Ok(address) = SocketAddr::from_str(self.as_ref()) { Ok(address) }
			else { throw_err!(IoErrorKind::InvalidInput.into()) }
	}
}



