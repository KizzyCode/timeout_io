use super::{ IoError, Result, InstantExt, WaitForEvent };
use std::{
	io::Result as IoResult, time::{ Duration, Instant }, net::{ TcpListener, TcpStream }
};


/// A private trait wrapping the standard library's acceptors
#[doc(hidden)]
pub trait StdAcceptor<T> where Self: WaitForEvent {
	fn accept(&self) -> IoResult<T>;
}
impl StdAcceptor<TcpStream> for TcpListener {
	fn accept(&self) -> IoResult<TcpStream> {
		Ok(TcpListener::accept(self)?.0)
	}
}
#[cfg(unix)]
impl StdAcceptor<::std::os::unix::net::UnixStream> for ::std::os::unix::net::UnixListener {
	fn accept(&self) -> IoResult<::std::os::unix::net::UnixStream> {
		Ok(::std::os::unix::net::UnixListener::accept(self)?.0)
	}
}


/// A trait for accepting elements, e.g. a TCP-listener
pub trait Acceptor<T> {
	/// Accepts a type-`T`-connection
	///
	/// _Warning: This function makes `self` non-blocking. It's up to you to restore the previous
	/// state if necessary._
	///
	/// Parameters:
	///  - `timeout`: The time to wait for a connection
	///
	/// Returns either __the accepted connection__ or a corresponding `IoError`
	fn accept(&self, timeout: Duration) -> Result<T>;
}
impl<T, U> Acceptor<U> for T where T: StdAcceptor<U> {
	fn accept(&self, timeout: Duration) -> Result<U> {
		// Make the socket non-blocking
		try_err!(self.set_blocking_mode(false));
		
		// Compute timeout-point and try to accept once until the timeout occurred
		let timeout_point = Instant::now() + timeout;
		loop {
			// Wait for read-event
			try_err!(self.wait_until_readable(timeout_point.remaining()));
			
			// Accept connection
			match StdAcceptor::accept(self) {
				Ok(connection) => return Ok(connection),
				Err(error) => {
					let error = IoError::from(error);
					if error.non_recoverable { throw_err!(error) }
				}
			}
		}
	}
}