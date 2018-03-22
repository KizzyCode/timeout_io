use std;
use std::io::Read;

use super::etrace::Error;
use super::{ libselect, time_remaining, IoError, WriteableBuffer, MutableBackedBuffer };



/// A trait for reading with timeouts
pub trait Reader {
	/// Executes _one_ `read`-operation to read _up to_ `buffer.remaining().len()`-bytes
	///
	/// This is especially useful in packet-based contexts where `read`-operations are atomic
	/// (like in UDP) or if you don't know the amount of bytes in advance
	///
	/// This functions returns either `Ok(())` if __one__ `read`-call read some bytes or
	/// `Err(IOError(std::io::ErrorKind::TimedOut))` if `timeout` expired or
	/// `Err(IOError(...))` if another IO-error occurred
	///
	/// __Note: This function catches *all* interal timeouts/interrupts and returns only if we had
	/// _one_ successful `read`-operation or `timeout` was hit or a fatal error occurred__
	fn read_oneshot(&mut self, buffer: &mut WriteableBuffer<u8>, timeout: std::time::Duration) -> Result<(), Error<IoError>>;
	
	/// Reads until the buffer is filled
	///
	/// This is especially useful in stream-based contexts where partial-`read`-calls are common
	/// (like in TCP) and you want to read a well-known amount of bytes
	///
	/// This functions returns either `Ok(())` if the buffer was filled completely or
	/// `Err(IOError(std::io::ErrorKind::TimedOut))` if `timeout` expired or
	/// `Err(IOError(...))` if another IO-error occurred
	///
	/// __Note: This function catches *all* interal timeouts/interrupts and returns only if
	/// `timeout` was hit or a fatal error occurred__
	fn read_exact(&mut self, buffer: &mut WriteableBuffer<u8>, timeout: std::time::Duration) -> Result<(), Error<IoError>>;
	
	/// Read until a pattern is matched or `buffer` has been filled completely
	///
	/// This functions returns either `Ok(())` if the pattern was found completely or
	/// `Err(IOError(std::io::ErrorKind::NotFound))` if the buffer was filled completely without a
	/// match or
	/// `Err(IOError(std::io::ErrorKind::TimedOut))` if `timeout` expired or
	/// `Err(IOError(...))` if another IO-error occurred
	///
	/// __Note: This function catches *all* interal timeouts/interrupts and returns only if
	/// `timeout` was hit or a fatal error occurred__
	fn read_until(&mut self, pattern: &[u8], buffer: &mut WriteableBuffer<u8>, timeout: std::time::Duration) -> Result<(), Error<IoError>>;
}
impl<T> Reader for T where T: Read + libselect::ToRawFd {
	fn read_oneshot(&mut self, buffer: &mut WriteableBuffer<u8>, timeout: std::time::Duration) -> Result<(), Error<IoError>> {
		// Wait for read-event
		if !try_err!(libselect::event_read(self, timeout)) { throw_err!(std::io::ErrorKind::TimedOut.into()) }
		
		// Read data
		loop {
			match std::io::Read::read(self, buffer.remaining_mut()) {
				// Successful read
				Ok(bytes_read) => {
					*buffer.pos_mut() += bytes_read;
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
	
	fn read_exact(&mut self, buffer: &mut WriteableBuffer<u8>, timeout: std::time::Duration) -> Result<(), Error<IoError>> {
		// Compute timeout-point
		let timeout_point = std::time::Instant::now() + timeout;
		
		// Loop until timeout
		while std::time::Instant::now() < timeout_point && !buffer.remaining().is_empty() {
			// Wait for read-event
			if !try_err!(libselect::event_read(self, time_remaining(timeout_point))) { throw_err!(std::io::ErrorKind::TimedOut.into()) }
			
			// Read data
			match std::io::Read::read(self, buffer.remaining_mut()) {
				// (Partial-)read
				Ok(bytes_read) => *buffer.pos_mut() += bytes_read,
				// An error occurred
				Err(error) => {
					let error = IoError::from(error);
					if error.is_fatal { throw_err!(error) }
				}
			}
		}
		
		// Check if we read all bytes or if an error occurred
		if !buffer.remaining().is_empty() { throw_err!(std::io::ErrorKind::TimedOut.into()) }
		Ok(())
	}
	
	fn read_until(&mut self, pattern: &[u8], buffer: &mut WriteableBuffer<u8>, timeout: std::time::Duration) -> Result<(), Error<IoError>> {
		// Compute timeout-point
		let timeout_point = std::time::Instant::now() + timeout;
		
		// Spin until `data` has been filled
		while !buffer.remaining().is_empty() {
			// Read next byte
			{
				let mut sub_buffer = MutableBackedBuffer::new(&mut buffer.remaining_mut()[.. 1]);
				try_err!(Reader::read_exact(self, &mut sub_buffer, time_remaining(timeout_point)));
			}
			// Check for pattern
			let filled = buffer.processed().len();
			if filled > pattern.len() && &buffer.processed()[filled - pattern.len() ..] == pattern { return Ok(()) }
		}
		throw_err!(std::io::ErrorKind::NotFound.into())
	}
}