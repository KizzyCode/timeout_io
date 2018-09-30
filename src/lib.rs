//! # About
//! This library provides a simple timeout-based API for IO-operations.
//!
//! It provides the following features:
//!  - DNS-resolution (currently uses a background-thread)
//!  - TCP-accept (uses libselect)
//!  - TCP-read/read-until/write (uses libselect)
//!  - StdIOE-read/read-write/write (uses libselect)
//!  - UDP-receive/send (uses libselect)
//!
//! All functions are defined as traits, so that you can easily wrap your own IO-channels without breaking compatibility.
//!
//! _Note: We currently do not provide a function for timeout-based `connect`-calls; use
//! `std::net::TcpStream::connect_timeout` for TCP-connections or build sth. using `io::libselect` (and feel free to commit
//! if you do so 😇)_

#[macro_use] extern crate etrace;
#[macro_use] extern crate tiny_future;
extern crate slice_queue;

mod event;
mod reader;
mod writer;
mod acceptor;
mod resolver;


pub use slice_queue::{ SliceQueue, ReadableSliceQueue, WriteableSliceQueue };
pub use self::{
	event::{ RawFd, WaitForEvent, Event, libselect },
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


/// Extends `std::time::Instant`
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
/// Extends `std::time::Duration`
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