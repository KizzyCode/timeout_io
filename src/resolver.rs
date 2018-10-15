use super::{ IoError, Result, InstantExt };
use std::{
	time::{ Duration, Instant },
	net::{ SocketAddr, ToSocketAddrs },
	sync::mpsc::{ self, RecvTimeoutError },
	io::ErrorKind as IoErrorKind, thread, str::FromStr
};


/// A trait for elements which contain a DNS-resolvable address
pub trait DnsResolvable {
	/// Resolves a domain-name or IP-address
	///
	/// __Warning: because `getaddrinfo` only provides a synchronous API, we have to resolve in a
	/// background thread. This means the background thread may outlive this call until the OS'
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
		enum Msg{ Result(Result<SocketAddr>), Ping }
		
		// Create address and channels
		let address = self.to_string();
		let (sender, receiver) = mpsc::channel();
		
		// Run resolver task
		thread::spawn(move || 'resolve_loop: loop {
			// Check for timeout
			if sender.send(Msg::Ping).is_err() { return }
			
			// Resolve name
			let to_send = match address.as_str().to_socket_addrs() {
				Ok(ref addresses) if (addresses as &ExactSizeIterator<Item = SocketAddr>).len() == 0 =>
					Err(new_err!(IoErrorKind::NotFound.into())),
				Ok(mut addresses) =>
					Ok(addresses.next().unwrap()),
				Err(error) => match IoError::from(error) {
					ref e if !e.non_recoverable => continue 'resolve_loop,
					e => Err(new_err!(e))
				}
			};
			
			// Send result
			let _ = sender.send(Msg::Result(to_send));
			return;
		});
		
		// Wait for result
		let timeout_point = Instant::now() + timeout;
		'receive_loop: loop {
			match receiver.recv_timeout(timeout_point.remaining()) {
				Ok(Msg::Result(result)) => return Ok(try_err!(result)),
				Ok(Msg::Ping) => continue 'receive_loop,
				Err(RecvTimeoutError::Timeout) => throw_err!(IoErrorKind::TimedOut.into()),
				Err(_) => panic!("Resolver thread crashed without result")
			}
		}
	}
}



/// A trait for elements which can be parsed to an IP-address
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



