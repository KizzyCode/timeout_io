use super::{ IoError, Result, SliceQueue, ReadableSliceQueue, WriteableSliceQueue, InstantExt, WaitForEvent };
use std::{ io::Read, time::{ Duration, Instant }, io::ErrorKind as IoErrorKind };


/// A trait for reading with timeouts
pub trait Reader {
	/// Executes _one_ `read`-operation to read up to `buffer.len()`-bytes
	///
	/// This is especially useful in packet-based contexts where `read`-operations are atomic
	/// (like in UDP) or if you don't know the amount of bytes in advance
	///
	/// _Note: This function catches all interal timeouts/interrupts and returns only if there was
	/// either one successful `read`-operation or the `timeout` was hit or a non-recoverable error
	/// occurred._
	///
	/// __Warning: This function allocates `buffer.remaining()` bytes, so please ensure that you've
	/// set an acceptable limit.__
	///
	/// _Warning: This function makes `self` non-blocking. It's up to you to restore the previous
	/// state if necessary._
	///
	/// Parameters:
	///  - `buffer`: The buffer to write the data to
	///  - `timeout`: The maximum time this function will wait for `self` to become readable
	///
	/// Returns either __nothing__ or a corresponding `IoError`
	fn read_oneshot(&mut self, buffer: &mut SliceQueue<u8>, timeout: Duration) -> Result<()>;
	
	/// Reads until `buffer` has been filled completely
	///
	/// This is especially useful in stream-based contexts where partial-`read`-calls are common
	/// (like in TCP) and you want to read a well-known amount of bytes
	///
	/// _Note: This function catches all interal timeouts/interrupts and returns only if either
	/// `buffer` has been filled completely or the `timeout` was hit or a non-recoverable error
	/// occurred._
	///
	/// __Warning: The buffer is filled completely, so please ensure that you've set an acceptable
	/// limit.__
	///
	/// _Warning: This function makes `self` non-blocking. It's up to you to restore the previous
	/// state if necessary._
	///
	/// Parameters:
	///  - `buffer`: The buffer to fill with data
	///  - `timeout`: The maximum time this function will block
	///
	/// Returns either __nothing__ or a corresponding `IoError`
	fn read_exact(&mut self, buffer: &mut SliceQueue<u8>, timeout: Duration) -> Result<()>;
	
	/// Read until either `pattern` has been matched or `buffer` has been filled completely
	///
	/// _Note: This function catches all interal timeouts/interrupts and returns only if either
	/// `pattern` has been matched or `buffer` has been filled completely or the `timeout` was hit
	/// or a non-recoverable error occurred._
	///
	/// __Warning: The buffer may be filled completely, so please ensure that you've set an
	/// acceptable limit.__
	///
	/// _Warning: This function makes `self` non-blocking. It's up to you to restore the previous
	/// state if necessary._
	///
	/// Parameters:
	///  - `pattern`: The pattern up to which you want to read.
	///  - `buffer`: The buffer to write the data to
	///  - `timeout`: The maximum time this function will block
	///
	/// Returns either
	///  - `Ok(bytes_read)` if the pattern was found or
	///  - `Err(IOError(std::io::ErrorKind::NotFound))` if the buffer was filled completely without
	///    a match or
	///  - another corresponding `IoError`
	fn read_until(&mut self, pattern: &[u8], buffer: &mut SliceQueue<u8>, timeout: Duration) -> Result<()>;
}
impl<T: Read + WaitForEvent> Reader for T {
	fn read_oneshot(&mut self, buffer: &mut SliceQueue<u8>, timeout: Duration) -> Result<()> {
		// Make the socket non-blocking
		try_err!(self.set_blocking_mode(false));
		
		// Immediately return if we should not read any bytes
		if buffer.remaining() == 0 { return Ok(()) }
		
		// Wait for read-event and read data
		try_err!(self.wait_until_readable(timeout));
		loop {
			let remaining = buffer.remaining();
			match buffer.push_in_place(remaining, |buffer| self.read(buffer)) {
				Ok(bytes_read) => if bytes_read > 0 { return Ok(()) }
					else { throw_err!(IoErrorKind::UnexpectedEof.into()) },
				Err(error) => {
					let error = IoError::from(error);
					if error.non_recoverable { throw_err!(error) }
				}
			}
		}
	}
	
	fn read_exact(&mut self, buffer: &mut SliceQueue<u8>, timeout: Duration) -> Result<()> {
		// Make the socket non-blocking
		try_err!(self.set_blocking_mode(false));
		
		// Compute timeout-point and loop until buffer is filled completely
		let timeout_point = Instant::now() + timeout;
		while buffer.remaining() > 0 {
			// Wait for read-event
			try_err!(self.wait_until_readable(timeout_point.remaining()));
			
			// Read data
			let remaining = buffer.remaining();
			match buffer.push_in_place(remaining, |buffer| self.read(buffer)) {
				// (Partial-)read
				Ok(bytes_read) =>
					if bytes_read == 0 { throw_err!(IoErrorKind::UnexpectedEof.into()) },
				// An error occurred
				Err(error) => {
					let error = IoError::from(error);
					if error.non_recoverable { throw_err!(error) }
				}
			}
		}
		Ok(())
	}
	
	fn read_until(&mut self, pattern: &[u8], buffer: &mut SliceQueue<u8>, timeout: Duration) -> Result<()> {
		// Compute timeout-point
		let timeout_point = Instant::now() + timeout;
		
		// Compute timeout-point and loop until `data` has been filled
		let mut byte_buffer = SliceQueue::with_limit(1);
		while buffer.remaining() > 0 {
			// Read next byte
			{
				try_err!(Reader::read_exact(self, &mut byte_buffer, timeout_point.remaining()));
				buffer.push(byte_buffer.pop().unwrap()).unwrap();
			}
			// Check for pattern
			let filled = buffer.len();
			if filled >= pattern.len() && &buffer[filled - pattern.len() ..] == pattern { return Ok(()) }
		}
		throw_err!(IoErrorKind::NotFound.into())
	}
}