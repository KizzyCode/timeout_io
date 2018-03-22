use std;

use super::etrace::Error;
use super::future::{ self, Future };
use super::IoError;



/// A trait for elements that contains a DNS-resolvable address
pub trait DnsResolvable {
	/// Resolves a domain-name or IP-address
	///
	/// __Warning: because `getaddrinfo` only provides a synchronously API, we have to resolve in a
	/// background-thread. This means the background-thread may outlive this call until the OS'
	/// `connect`-timeout is reached.__
	fn resolve_address(&self, timeout: std::time::Duration) -> Result<std::net::SocketAddr, Error<IoError>>;
}
impl<T> DnsResolvable for T where T: ToString {
	fn resolve_address(&self, timeout: std::time::Duration) -> Result<std::net::SocketAddr, Error<IoError>> {
		// Run resolver-job
		let address = self.to_string();
		let fut = future::async(move |fut: Future<Result<std::net::SocketAddr, Error<IoError>>>| {
			loop {
				// Check for timeout
				if !fut.is_waiting() { job_die!(fut) }
				
				// Resolve name
				match std::net::ToSocketAddrs::to_socket_addrs(address.as_str()) {
					Ok(mut addresses) => if let Some(address) = addresses.next() { job_return!(fut, Ok(address)) }
						else { job_return!(fut, Err(new_err!(std::io::ErrorKind::NotFound.into()))) },
					Err(error) => {
						let io_error = IoError::from(error);
						if io_error.is_fatal { job_return!(fut, Err(new_err!(io_error))) }
					}
				};
			};
		});
		// Wait for result
		if let Ok(result) = fut.try_get_timeout(timeout) { result }
			else { throw_err!(std::io::ErrorKind::TimedOut.into()) }
	}
}



/// A trait for elements that contain a parseable IP-address
pub trait IpParseable {
	/// Parses an IP-address
	fn parse_address(&self) -> Result<std::net::SocketAddr, Error<IoError>>;
}
impl<T> IpParseable for T where T: ToString {
	fn parse_address(&self) -> Result<std::net::SocketAddr, Error<IoError>> {
		use std::str::FromStr;
		
		if let Ok(address) = std::net::SocketAddr::from_str(self.to_string().as_str()) { Ok(address) }
			else { throw_err!(std::io::ErrorKind::InvalidInput.into()) }
	}
}



