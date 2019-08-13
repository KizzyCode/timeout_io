use crate::TimeoutIoError;
use std::{ self, io, convert::TryInto, time::Duration };


/// Interface to `libselect`
mod libselect {
	use std::os::raw::c_int;
	extern "C" {
		pub static EVENT_READ:  u8;
		pub static EVENT_WRITE: u8;
		pub static EVENT_ERROR: u8;
		pub static INVALID_FD:  u64;
		
		pub fn wait_for_event(timeout_ms: u64, fds: *const u64, events: *mut u8) -> c_int;
		pub fn set_blocking_mode(descriptor: u64, blocking: u8) -> c_int;
	}
}


/// A wrapper-trait that unifies the `std::os::unix::io::AsRawFd` and
/// `std::os::windows::io::AsRawSocket` traits
pub trait RawFd {
	/// The underlying raw file descriptor
	fn raw_fd(&self) -> u64;
}
#[cfg(unix)]
impl<T> RawFd for T where T: std::os::unix::io::AsRawFd {
	fn raw_fd(&self) -> u64 { self.as_raw_fd() as u64 }
}
#[cfg(windows)]
impl<T> RawFd for T where T: std::os::windows::io::AsRawSocket {
	fn raw_fd(&self) -> u64 { self.as_raw_socket() as u64 }
}


/// A struct describing null or more IO-events
#[repr(transparent)] #[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct EventMask{ raw: u8 }
impl EventMask {
	/// Creates a new read/error event mask
	pub fn new_r() -> Self {
		Self{ raw: unsafe{ libselect::EVENT_READ | libselect::EVENT_ERROR } }
	}
	/// Creates a new write/error event mask
	pub fn new_w() -> Self {
		Self{ raw: unsafe{ libselect::EVENT_WRITE | libselect::EVENT_ERROR } }
	}
	/// Creates a new read/write/error event mask
	pub fn new_rw() -> Self {
		use self::libselect::{ EVENT_READ, EVENT_WRITE, EVENT_ERROR };
		Self{ raw: unsafe{ EVENT_READ | EVENT_WRITE | EVENT_ERROR } }
	}
	
	/// Checks if the mask contains read/write/error
	pub fn rwe(&self) -> (bool, bool, bool) {
		(
			self.raw & unsafe{ libselect::EVENT_READ } != 0,
			self.raw & unsafe{ libselect::EVENT_WRITE } != 0,
			self.raw & unsafe{ libselect::EVENT_ERROR } != 0
		)
	}
}


/// A set of multiple `(handle: event)`-pairs that allows you to call `select` on all pairs at the
/// same time
pub struct SelectSet<'a, T: RawFd> {
	handles: Vec<&'a T>,
	events: Vec<EventMask>
}
impl<'a, T: RawFd> SelectSet<'a, T> {
	/// Creates a new select set
	pub fn new() -> Self {
		Self{ handles: Vec::new(), events: Vec::new() }
	}
	
	/// Pushes a new `handle` and the according `event` mask wait for to the set
	pub fn push(&mut self, handle: &'a T, event: EventMask) {
		self.handles.push(handle);
		self.events.push(event);
	}
	
	/// Waits on all handles in the set until an event occurrs or `timeout` was reached. Returns
	/// only the `(handle, event_that_occurred)`-pairs for the handles where an event occurred.
	pub fn select(mut self, timeout: Duration) -> Result<Vec<(&'a T, EventMask)>, TimeoutIoError> {
		// Create raw event masks and raw FDs
		let mut fds: Vec<u64> = self.handles.iter().map(|h| h.raw_fd()).collect();
		fds.push(unsafe{ libselect::INVALID_FD });
		
		// Call libselect
		let result = unsafe{ libselect::wait_for_event(
			timeout.as_millis().try_into().expect("`timeout.as_millis()` > `u64`"),
			fds.as_ptr(), self.events.as_mut_ptr() as *mut u8
		) };
		if result != 0 { Err(io::Error::from_raw_os_error(result))? }
		
		// Yield the handles where an event occurred
		let yielded = self.handles.into_iter().zip(self.events)
			.filter(|(_, e)| e.rwe() != (false, false, false))
			.collect();
		Ok(yielded)
	}
}
/// Creates a new `SelectSet` for
macro_rules! select_set {
	($($handle:expr => $event:expr),*) => ({
		let mut select_set = $crate::SelectSet::new();
		$(select_set.push($handle, $event);)*
		select_set
	});
}


/// This trait defines an API to wait for an event
pub trait WaitForEvent {
	/// Waits until `event` occurs or `timeout` is exceeded and returns the event that occurred
	fn wait_for_event(&self, event: EventMask, timeout: Duration)
		-> Result<EventMask, TimeoutIoError>;
	
	/// Makes `self` blocking or non-blocking
	fn set_blocking_mode(&self, make_blocking: bool) -> Result<(), TimeoutIoError>;
}
impl<T: RawFd> WaitForEvent for T {
	fn wait_for_event(&self, event: EventMask, timeout: Duration)
		-> Result<EventMask, TimeoutIoError>
	{
		// Wait for `r | e`
		let events: Vec<(&Self, EventMask)> = select_set!(self => event).select(timeout)?;
		match events.first() {
			Some((_, event)) => Ok(*event),
			None => Err(TimeoutIoError::TimedOut)
		}
	}
	
	fn set_blocking_mode(&self, make_blocking: bool) -> Result<(), TimeoutIoError> {
		// Set the blocking mode
		let result = unsafe{ libselect::set_blocking_mode(
			self.raw_fd(),
			if make_blocking { 1 } else { 0 }
		) };
		
		// Check the result
		match result {
			0 => Ok(()),
			e => Err(io::Error::from_raw_os_error(e).into())
		}
	}
}