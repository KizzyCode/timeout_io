#[macro_use] extern crate etrace;
#[macro_use] extern crate future;

pub mod libselect;
pub mod reader;
pub mod writer;
pub mod acceptor;
pub mod resolver;
pub mod buffer;

pub use reader::Reader;
pub use writer::Writer;
pub use acceptor::Acceptor;
pub use resolver::{ DnsResolvable, IpParseable };
pub use buffer::{ ReadableBuffer, WriteableBuffer, BackedBuffer, MutableBackedBuffer, OwnedBuffer };


#[derive(Debug, Clone)]
/// An IO-error-wrapper
pub struct IoError {
	pub kind: std::io::ErrorKind,
	pub is_fatal: bool
}
impl From<std::io::ErrorKind> for IoError {
	fn from(kind: std::io::ErrorKind) -> Self {
		use std::io::ErrorKind;
		match kind {
			ErrorKind::Interrupted => IoError { kind: ErrorKind::Interrupted, is_fatal: false },
			ErrorKind::TimedOut => IoError { kind: ErrorKind::TimedOut, is_fatal: false },
			other => IoError { kind: other, is_fatal: true }
		}
	}
}
impl From<std::io::Error> for IoError {
	fn from(error: std::io::Error) -> Self {
		error.kind().into()
	}
}



/// Computes the remaining time underflow-safe
pub fn time_remaining(timeout_point: std::time::Instant) -> std::time::Duration {
	let now = std::time::Instant::now();
	if now > timeout_point { std::time::Duration::default() } else { timeout_point - now }
}