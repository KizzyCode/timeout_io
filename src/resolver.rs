use super::{ TimeoutIoError, InstantExt };
use std::{
	thread, str::FromStr,
	net::{ SocketAddr, ToSocketAddrs },
	time::{ Duration, Instant },
	sync::mpsc::{ self, RecvTimeoutError }
};


/// A trait for elements which contain a DNS-resolvable address
pub trait DnsResolvable {
	/// Tries to resolve a domain-name or IP-address until `timeout` is exceeded
	///
	/// _Info: If you want to resolve an address like "localhost" or "crates.io" you __must__
	/// include the port number like this: "localhost:80" or "crates.io:443"_
	///
	/// __Warning: because `getaddrinfo` only provides a synchronous API, we have to resolve in a
	/// background thread. This means the background thread may outlive this call until the OS'
	/// `connect`-timeout is reached.__
	fn dns_resolve(&self, timeout: Duration) -> Result<SocketAddr, TimeoutIoError>;
}
impl<T: ToString> DnsResolvable for T {
	fn dns_resolve(&self, timeout: Duration) -> Result<SocketAddr, TimeoutIoError> {
		// Create address and channels
		let address = self.to_string();
		let (sender, receiver) = mpsc::channel();
		
		// Run resolver task
		enum Msg{ Ping, Result(Result<SocketAddr, TimeoutIoError>) }
		thread::spawn(move || {
			let result = loop {
				// Check for timeout
				if sender.send(Msg::Ping).is_err() { return }
				
				// Resolve name
				match address.as_str().to_socket_addrs() {
					Ok(mut addresses) => break match addresses.next() {
						Some(address) => Ok(address),
						None => Err(TimeoutIoError::NotFound)
					},
					Err(error) => {
						let error = TimeoutIoError::from(error);
						if !error.should_retry() { break Err(error) }
					}
				};
			};
			let _ = sender.send(Msg::Result(result));
		});
		
		// Wait for result
		let deadline = Instant::now() + timeout;
		'receive_loop: loop {
			match receiver.recv_timeout(deadline.remaining()) {
				Ok(Msg::Ping) => continue 'receive_loop,
				Ok(Msg::Result(result)) => return result,
				Err(RecvTimeoutError::Timeout) => return Err(TimeoutIoError::TimedOut),
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
	fn parse_ip(&self) -> Result<SocketAddr, TimeoutIoError>;
}
impl<T: AsRef<str>> IpParseable for T {
	fn parse_ip(&self) -> Result<SocketAddr, TimeoutIoError> {
		match SocketAddr::from_str(self.as_ref()) {
			Ok(address) => Ok(address),
			Err(_) => Err(TimeoutIoError::InvalidInput)
		}
	}
}



