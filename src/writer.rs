use std;
use std::io::Write;

use super::etrace::Error;
use super::{ libselect, time_remaining, IoError, ReadableBuffer };



/// A trait for writing with timeouts
pub trait Writer {
	/// Executes _one_ `write`-operation to write _as much bytes as possible_ from
	/// `buffer.remaining()`
	///
	/// This is especially useful in packet-based contexts where `write`-operations are atomic
	/// (like in UDP)
	///
	/// This functions returns either `Ok(())` if __one__ `read`-call read some bytes or
	/// `Err(IOError(std::io::ErrorKind::TimedOut))` if `timeout` expired or
	/// `Err(IOError(...))` if another IO-error occurred
	///
	/// __Note: This function catches *all* interal timeouts/interrupts and returns only if we had
	/// _one_ successful `write`-operation or the passed timeout was hit or an error occurred__
	fn write_oneshot(&mut self, buffer: &mut ReadableBuffer<u8>, timeout: std::time::Duration) -> Result<(), Error<IoError>>;
	
	/// Writes all bytes in `buffer.remaining()`
	///
	/// This is especially useful in stream-based contexts where partial-`write`-calls are common
	/// (like in TCP)
	///
	/// This functions returns either `Ok(())` if the buffer was written completely or
	/// `Err(IOError(std::io::ErrorKind::TimedOut))` if `timeout` expired or
	/// `Err(IOError(...))` if another IO-error occurred
	///
	/// __Note: This function catches *all* interal timeouts/interrupts and returns only if
	/// `timeout` was hit or an error occurred__
	fn write_exact(&mut self, buffer: &mut ReadableBuffer<u8>, timeout: std::time::Duration) -> Result<(), Error<IoError>>;
}
impl<T> Writer for T where T: Write + libselect::ToRawFd {
	fn write_oneshot(&mut self, buffer: &mut ReadableBuffer<u8>, timeout: std::time::Duration) -> Result<(), Error<IoError>> {
		// Wait for write-event
		if !try_err!(libselect::event_write(self, timeout)) { throw_err!(std::io::ErrorKind::TimedOut.into()) }
		
		// Write data
		loop {
			match std::io::Write::write(self, buffer.remaining()) {
				// Successful write
				Ok(bytes_written) => {
					*buffer.pos() += bytes_written;
					return Ok(())
				},
				// An error occurred
				Err(error) => {
					let error = IoError::from(error);
					if error.is_fatal { throw_err!(error) }
				}
			}
		}
	}
	
	fn write_exact(&mut self, buffer: &mut ReadableBuffer<u8>, timeout: std::time::Duration) -> Result<(), Error<IoError>> {
		// Compute timeout-point
		let timeout_point = std::time::Instant::now() + timeout;
		
		// Loop until timeout
		while std::time::Instant::now() < timeout_point && !buffer.remaining().is_empty() {
			// Wait for write-event
			if !try_err!(libselect::event_write(self, time_remaining(timeout_point))) { throw_err!(std::io::ErrorKind::TimedOut.into()) }
			
			// Write data
			match std::io::Write::write(self, buffer.remaining()) {
				// (Partial-)write
				Ok(bytes_written) => *buffer.pos() += bytes_written,
				// An error occurred
				Err(error) => {
					let error = IoError::from(error);
					if error.is_fatal { throw_err!(error) }
				}
			}
		}
		
		// Check if we wrote all bytes or if an error occurred
		if !buffer.remaining().is_empty() { throw_err!(std::io::ErrorKind::TimedOut.into()) }
		Ok(())
	}
}