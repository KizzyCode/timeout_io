use ::{ Result, DurationExt };
use ::std::{ self, time::Duration, io::{ Error as IoError, ErrorKind as IoErrorKind } };


/// Interface to `libselect`
mod libselect {
	use std::os::raw::c_int;
	extern {
		pub static EVENT_READ:  u8;
		pub static EVENT_WRITE: u8;
		pub static EVENT_ERROR: u8;
		pub static INVALID_FD:  u64;
		
		pub fn wait_for_event(timeout_ms: u64, fds: *const u64, events: *mut u8) -> c_int;
		pub fn set_blocking_mode(descriptor: u64, blocking: u8) -> c_int;
	}
}
/// A struct describing null or more IO-events
#[derive(Copy, Clone, Default)]
pub struct Event {
	raw: u8
}
impl Event {
	#[doc(hidden)]
	/// Sets the event to a `libselect` raw value
	///
	/// __Warning: This function panics on an invalid `raw` value.__
	///
	/// Parameters:
	///  - `raw`: The `libselect` raw event value
	///
	/// Returns _a mutable reference to `self`_ to allow chaining
	pub fn set_raw(&mut self, raw: u8) -> &mut Self {
		if raw & !(Self::r() | Self::w() | Self::e()) != 0 { panic!("Invalid raw event {:08b}", raw) }
		self.raw = raw;
		self
	}
	#[doc(hidden)]
	/// The event's `libselect` raw value
	pub fn raw(&self) -> u8 {
		self.raw
	}
	
	/// Adds the read-event to `self`
	///
	/// Returns _a mutable reference to `self`_ to allow chaining
	pub fn add_r(&mut self) -> &mut Self {
		self.raw |= Self::r();
		self
	}
	/// Adds the write-event to `self`
	///
	/// Returns _a mutable reference to `self`_ to allow chaining
	pub fn add_w(&mut self) -> &mut Self {
		self.raw |= Self::w();
		self
	}
	/// Adds the error-event to `self`
	///
	/// Returns _a mutable reference to `self`_ to allow chaining
	pub fn add_e(&mut self) -> &mut Self {
		self.raw |= Self::e();
		self
	}
	
	/// Checks if `self` contains a read event
	pub fn is_r(&self) -> bool {
		self.raw & Self::r() != 0
	}
	/// Checks if `self` contains a write event
	pub fn is_w(&self) -> bool {
		self.raw & Self::w() != 0
	}
	/// Checks if `self` contains a error event
	pub fn is_e(&self) -> bool {
		self.raw & Self::e() != 0
	}
	
	fn r() -> u8 {
		unsafe{ libselect::EVENT_READ }
	}
	fn w() -> u8 {
		unsafe{ libselect::EVENT_WRITE }
	}
	fn e() -> u8 {
		unsafe{ libselect::EVENT_ERROR }
	}
}


/// A wrapper-trait that unifies the `std::os::unix::io::AsRawFd` and
/// `std::os::windows::io::AsRawSocket` traits
pub trait RawFd {
	/// The underlying raw file descriptor
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


/// Waits on multiple handles until an event occurrs or `timeout` was reached
///
/// Parameters:
///  - `handles`: A list of `(handle, event)`-pairs that contains the handles and the corresponding
///    events to wait for. If a matching event occurrs on a handle, the corresponding event struct
///    will be modified to reflect the event that occurred.
///  - `timeout`: The maximum amount of time this function will wait for an event
///
/// Returns either __nothing__ or a corresponding `IoError`
pub fn wait_multiple<'a>(mut handles: impl AsMut<[(&'a RawFd, &'a mut Event)]> + 'a, timeout: Duration) -> Result<()> {
	// Extract raw FDs and events
	let (mut fds, mut events): (Vec<u64>, Vec<u8>) = (Vec::new(), Vec::new());
	for (handle, event) in handles.as_mut() {
		fds.push(handle.raw_fd());
		events.push(event.raw());
	}
	fds.push(unsafe{ libselect::INVALID_FD });
	
	// Call libselect
	let result = unsafe{ libselect::wait_for_event(
		timeout.as_ms(), fds.as_ptr(), events.as_mut_ptr()
	) };
	if result != 0 { throw_err!(IoError::from_raw_os_error(result).into()) }
	
	// Copy events
	for i in 0..handles.as_mut().len() { handles.as_mut()[i].1.set_raw(events[i]); }
	Ok(())
}


/// This trait defines an API to wait for an event
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
	
	/// Makes `self` blocking or non-blocking
	///
	/// Parameters:
	///  - `blocking`: Set to `true` if you want to make the socket blocking, `false` otherwise
	///
	/// Returns either __nothing__ or a corresponding `IoError`
	fn set_blocking_mode(&self, blocking: bool) -> Result<()>;
}
impl<T: RawFd> WaitForEvent for T {
	fn wait_until_readable(&self, timeout: Duration) -> Result<()> {
		let mut event: Event = *Event::default().add_r().add_e();
		try_err!(wait_multiple([(self as &RawFd, &mut event)], timeout));
		
		if !(event.is_r() | event.is_e()) { throw_err!(IoErrorKind::TimedOut.into()) }
			else { Ok(()) }
	}
	fn wait_until_writeable(&self, timeout: Duration) -> Result<()> {
		let mut event: Event = *Event::default().add_w().add_e();
		try_err!(wait_multiple([(self as &RawFd, &mut event)], timeout));
		
		if !(event.is_w() | event.is_e()) { throw_err!(IoErrorKind::TimedOut.into()) }
			else { Ok(()) }
	}
	fn set_blocking_mode(&self, blocking: bool) -> Result<()> {
		let result = unsafe{ libselect::set_blocking_mode(
			self.raw_fd(), if blocking { 1 } else { 0 }
		) };
		if result != 0 { throw_err!(IoError::from_raw_os_error(result).into()) }
			else { Ok(()) }
	}
}
