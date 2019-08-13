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
//! All functions are defined as traits, so that you can easily wrap your own IO-channels without
//! breaking compatibility.
//!
//! _Note: We currently do not provide a function for timeout-based `connect`-calls; use
//! `std::net::TcpStream::connect_timeout` for TCP-connections or build sth. using `io::libselect`
//! (and feel free to commit if you do so 😇)_


// Mods
mod event;
mod reader;
mod writer;
mod acceptor;
mod resolver;


// Create re-exports
pub use crate::{
	acceptor::Acceptor, reader::Reader, writer::Writer,
	event::{ RawFd, EventMask, SelectSet, WaitForEvent },
	resolver::{ DnsResolvable, IpParseable }
};
use std::{
	error::Error,
	fmt::{ self, Display, Formatter },
	time::{ Duration, Instant },
	io::{
		self,
		ErrorKind::{
			Interrupted, TimedOut, WouldBlock, UnexpectedEof,
			BrokenPipe, ConnectionAborted, ConnectionReset
		}
	}
};


/// An IO-error-wrapper
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TimeoutIoError {
	InterruptedSyscall,
	TimedOut,
	UnexpectedEof,
	ConnectionLost,
	NotFound,
	InvalidInput,
	Other{ desc: String }
}
impl TimeoutIoError {
	pub fn should_retry(&self) -> bool {
		match self {
			TimeoutIoError::InterruptedSyscall | TimeoutIoError::TimedOut => true,
			_ => false
		}
	}
}
impl Display for TimeoutIoError {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}
impl Error for TimeoutIoError {}
impl From<io::Error> for TimeoutIoError {
	fn from(error: io::Error) -> Self {
		match error.kind() {
			Interrupted => TimeoutIoError::InterruptedSyscall,
			TimedOut | WouldBlock => TimeoutIoError::TimedOut,
			UnexpectedEof => TimeoutIoError::UnexpectedEof,
			BrokenPipe | ConnectionAborted | ConnectionReset => TimeoutIoError::ConnectionLost,
			_ => TimeoutIoError::Other{ desc: format!("{:#?}", error) }
		}
	}
}


/// Extends `std::time::Instant`
pub trait InstantExt {
	/// Computes the remaining time underflow-safe
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
	fn as_ms(&self) -> u64;
}
impl DurationExt for Duration {
	fn as_ms(&self) -> u64 {
		(self.as_secs() * 1000) + (self.subsec_nanos() as u64 / 1_000_000)
	}
}