#[macro_use] extern crate etrace;
#[macro_use] extern crate tiny_future;
extern crate slice_queue;

mod event;
mod reader;
mod writer;
mod acceptor;
mod resolver;


pub use slice_queue::SliceQueue;
pub use self::{
	event::{ RawFd, SetBlockingMode, WaitForEvent, Event, libselect },
	reader::Reader,
	writer::Writer,
	acceptor::Acceptor,
	resolver::{ DnsResolvable, IpParseable }
};
pub use std::io::ErrorKind as IoErrorKind;
use std::{ io::Error as StdIoError, time::{ Duration, Instant } };


#[derive(Debug, Clone)]
/// An IO-error-wrapper
pub struct IoError {
	pub kind: IoErrorKind,
	pub non_recoverable: bool
}
impl From<IoErrorKind> for IoError {
	fn from(kind: IoErrorKind) -> Self {
		match kind {
			IoErrorKind::Interrupted => IoError { kind: IoErrorKind::Interrupted, non_recoverable: false },
			IoErrorKind::TimedOut => IoError { kind: IoErrorKind::TimedOut, non_recoverable: false },
			other => Self{ kind: other, non_recoverable: true }
		}
	}
}
impl From<StdIoError> for IoError {
	fn from(error: StdIoError) -> Self {
		error.kind().into()
	}
}
/// Syntactic sugar for `std::result::Result<T, etrace::Error<IoError>>`
pub type Result<T> = std::result::Result<T, etrace::Error<IoError>>;


pub trait InstantExt {
	/// Computes the remaining time underflow-safe
	///
	/// Returns __the remaining time__
	fn remaining(self) -> Duration;
}
impl InstantExt for Instant {
	fn remaining(self) -> Duration {
		let now = Instant::now();
		if now > self { Duration::from_secs(0) }
			else { self - now }
	}
}
pub trait DurationExt {
	/// The duration in milliseconds
	///
	/// Returns __`self` as milliseconds__
	fn as_ms(&self) -> u64;
}
impl DurationExt for Duration {
	fn as_ms(&self) -> u64 {
		(self.as_secs() * 1000) + (self.subsec_nanos() as u64 / 1_000_000)
	}
}