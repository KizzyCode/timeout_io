use super::{ Result, DurationExt };
use std::{
	self, time::Duration,
	io::{ Error as StdIoError, ErrorKind as IoErrorKind },
	ops::{ BitOr, BitAnd },
	net::{ TcpListener, TcpStream, UdpSocket }
};


pub mod libselect {
	use std::os::raw::c_int;
	extern {
		pub fn wait_for_event(descriptor: u64, event_mask: u8, timeout_ms: u64) -> u8;
		pub fn get_errno() -> c_int;
	}
}
#[repr(u8)]
pub enum Event {
	Read  = 1 << 1, // const uint8_t EVENT_READ   = 1 << 1;
	Write = 1 << 2, // const uint8_t EVENT_WRITE  = 1 << 2;
	Error = 1 << 3, // const uint8_t EVENT_ERROR  = 1 << 3;
	SelectError = 1 << 7 // const uint8_t SELECT_ERROR = 1 << 7;
}
impl BitOr for Event {
	type Output = u8;
	fn bitor(self, rhs: Self) -> Self::Output {
		(self as u8) | (rhs as u8)
	}
}
impl BitAnd<Event> for u8 {
	type Output = bool;
	fn bitand(self, rhs: Event) -> Self::Output {
		self & (rhs as u8) != 0
	}
}


pub trait RawFd {
	fn raw_fd(&self) -> u64;
}
#[cfg(unix)]
impl<T: std::os::unix::io::AsRawFd> RawFd for T {
	fn raw_fd(&self) -> u64 { self.as_raw_fd() as u64 }
}
#[cfg(windows)]
impl<T: std::os::windows::io::AsRawSocket> RawFd for T {
	fn raw_fd(&self) -> u64 { self.as_raw_socket() as u64 }
}


pub trait SetBlockingMode {
	/// Makes IO-operations on `self` non-blocking
	///
	/// Returns either __nothing__ or a corresponding `IoError`
	fn make_nonblocking(&self) -> Result<()>;
	/// Makes IO-operations on `self` blocking
	///
	/// Returns either __nothing__ or a corresponding `IoError`
	fn make_blocking(&self) -> Result<()>;
}
impl SetBlockingMode for TcpListener {
	fn make_nonblocking(&self) -> Result<()> {
		Ok(try_err_from!(self.set_nonblocking(true)))
	}
	fn make_blocking(&self) -> Result<()> {
		Ok(try_err_from!(self.set_nonblocking(false)))
	}
}
impl SetBlockingMode for TcpStream {
	fn make_nonblocking(&self) -> Result<()> {
		Ok(try_err_from!(self.set_nonblocking(true)))
	}
	fn make_blocking(&self) -> Result<()> {
		Ok(try_err_from!(self.set_nonblocking(false)))
	}
}
impl SetBlockingMode for UdpSocket {
	fn make_nonblocking(&self) -> Result<()> {
		Ok(try_err_from!(self.set_nonblocking(true)))
	}
	fn make_blocking(&self) -> Result<()> {
		Ok(try_err_from!(self.set_nonblocking(false)))
	}
}


pub trait WaitForEvent {
	/// Waits until `self` is ready for reading or `timeout` was reached
	///
	/// Parameters:
	///  - `timeout`: The maximum time this function will wait for `self` to become readable
	///
	/// Returns either __nothing__ or a corresponding `IoError`
	fn wait_until_readable(&self, timeout: Duration) -> Result<()>;
	/// Waits until `self` is ready for writing or `timeout` was reached
	///
	/// Parameters:
	///  - `timeout`: The maximum time this function will wait for `self` to become writeable
	///
	/// Returns either __nothing__ or a corresponding `IoError`
	fn wait_until_writeable(&self, timeout: Duration) -> Result<()>;
}
impl<T: RawFd> WaitForEvent for T {
	fn wait_until_readable(&self, timeout: Duration) -> Result<()> {
		// Wait for event
		let result = unsafe{ libselect::wait_for_event(
			self.raw_fd(),
			Event::Read | Event::Error,
			timeout.as_ms()
		) };
		// Read result
		match result {
			r if r & Event::SelectError => throw_err!(StdIoError::from_raw_os_error(unsafe{ libselect::get_errno() }).into()),
			r if r & Event::Read => Ok(()),
			_ => throw_err!(IoErrorKind::TimedOut.into())
		}
	}
	fn wait_until_writeable(&self, timeout: Duration) -> Result<()> {
		// Wait for event
		let result = unsafe{ libselect::wait_for_event(
			self.raw_fd(),
			Event::Write | Event::Error,
			timeout.as_ms()
		) };
		// Read result
		match result {
			r if r & Event::SelectError => throw_err!(StdIoError::from_raw_os_error(unsafe{ libselect::get_errno() }).into()),
			r if r & Event::Write => Ok(()),
			_ => throw_err!(IoErrorKind::TimedOut.into())
		}
	}
}