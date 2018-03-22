use std;

use super::etrace::Error;
use super::{ libselect, time_remaining, IoError };



/// A trait for accepting elements, e.g. a TCP-listener
pub trait Acceptor<T> {
	/// Accepts a type-`T`-connection
	fn accept(&self, timeout: std::time::Duration) -> Result<T, Error<IoError>>;
}
impl Acceptor<std::net::TcpStream> for std::net::TcpListener {
	fn accept(&self, timeout: std::time::Duration) -> Result<std::net::TcpStream, Error<IoError>> {
		// Compute timeout-point
		let timeout_point = std::time::Instant::now() + timeout;
		
		// Try to accept once until the timeout occurred
		while std::time::Instant::now() < timeout_point {
			// Wait for read-event
			if !try_err!(libselect::event_read(self, time_remaining(timeout_point))) { throw_err!(std::io::ErrorKind::TimedOut.into()) }
			
			// Accept connection
			match self.accept() {
				// Accepted connection
				Ok(connection) => return Ok(connection.0),
				// An error occurred
				Err(error) => {
					let error = IoError::from(error);
					if error.is_fatal { throw_err!(error) }
				}
			}
		}
		throw_err!(std::io::ErrorKind::TimedOut.into())
	}
}