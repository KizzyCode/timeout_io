use crate::{ TimeoutIoError, InstantExt, WaitForEvent, EventMask };
use std::{ io::Read, time::{ Duration, Instant } };


/// A trait for reading with timeouts
pub trait Reader {
	/// Executes _one_ `read`-operation to read up to `buffer.len()`-bytes and returns the amount
	/// of bytes read
	///
	/// This is especially useful in packet-based contexts where `read`-operations are atomic
	/// (like in UDP) or if you don't know the amount of bytes in advance
	///
	/// _Note: This function catches all interal timeouts/interrupts and returns only if there was
	/// either one successful `read`-operation or the `timeout` was hit or a non-recoverable error
	/// occurred._
	///
	/// __Warning: This function makes `self` non-blocking. It's up to you to restore the previous
	/// state if necessary.__
	fn read(&mut self, buffer: &mut[u8], timeout: Duration) -> Result<usize, TimeoutIoError>;
	
	/// Reads until `buffer` is filled completely
	///
	/// This is especially useful in stream-based contexts where partial-`read`-calls are common
	/// (like in TCP) and you want to read a well-known amount of bytes
	///
	/// _Note: This function catches all interal timeouts/interrupts and returns only if either
	/// `buffer` has been filled completely or the `timeout` was hit or a non-recoverable error
	/// occurred._
	///
	/// __Warning: This function makes `self` non-blocking. It's up to you to restore the previous
	/// state if necessary.__
	fn read_exact(&mut self, buffer: &mut[u8], timeout: Duration) -> Result<(), TimeoutIoError>;
	
	/// Reads until either `pattern` is matched or `buffer` is filled completely. Returns either
	/// `Some(bytes_read)` if the pattern has been matched or `None` if `buffer` has been filled
	/// completely without matching the pattern.
	///
	/// _Note: This function catches all interal timeouts/interrupts and returns only if either
	/// `pattern` has been matched or `buffer` has been filled completely or the `timeout` was hit
	/// or a non-recoverable error occurred._
	///
	/// __Warning: This function makes `self` non-blocking. It's up to you to restore the previous
	/// state if necessary.__
	fn read_until(&mut self, buffer: &mut[u8], pattern: &[u8], timeout: Duration)
		-> Result<Option<usize>, TimeoutIoError>;
}
impl<T: Read + WaitForEvent> Reader for T {
	fn read(&mut self, buffer: &mut[u8], timeout: Duration) -> Result<usize, TimeoutIoError> {
		// Make the socket non-blocking
		self.set_blocking_mode(false)?;
		
		// Immediately return if we should not read any bytes
		if buffer.len() == 0 { return Ok(0) }
		
		// Wait for read-event and read data
		self.wait_for_event(EventMask::new_r(), timeout)?;
		loop {
			match self.read(buffer) {
				Ok(0) => return Err(TimeoutIoError::UnexpectedEof),
				Ok(bytes_read) => return Ok(bytes_read),
				Err(error) => {
					let error = TimeoutIoError::from(error);
					if !error.should_retry() { return Err(error) }
				}
			}
		}
	}
	
	fn read_exact(&mut self, mut buffer: &mut[u8], timeout: Duration) -> Result<(), TimeoutIoError>
	{
		// Make the socket non-blocking
		self.set_blocking_mode(false)?;
		
		// Compute timeout-point and loop until buffer is filled completely
		let timeout_point = Instant::now() + timeout;
		while !buffer.is_empty() {
			// Wait for read-event and read data
			self.wait_for_event(EventMask::new_r(), timeout_point.remaining())?;
			match self.read(buffer) {
				Ok(0) => return Err(TimeoutIoError::UnexpectedEof),
				Ok(bytes_read) => buffer = &mut buffer[bytes_read..],
				Err(error) => {
					let error = TimeoutIoError::from(error);
					if !error.should_retry() { return Err(error) }
				}
			}
		}
		Ok(())
	}
	
	fn read_until(&mut self, buffer: &mut[u8], pattern: &[u8], timeout: Duration)
		-> Result<Option<usize>, TimeoutIoError>
	{
		// Compute timeout-point
		let timeout_point = Instant::now() + timeout;
		
		// Compute timeout-point and loop until `data` has been filled
		let mut pos = 0;
		while pos < buffer.len() {
			// Read next byte
			Reader::read_exact(
				self, &mut buffer[pos .. pos + 1],
				timeout_point.remaining()
			)?;
			pos += 1;
			
			// Check for pattern
			if pos >= pattern.len() && &buffer[pos - pattern.len() .. pos] == pattern {
				return Ok(Some(pos))
			}
		}
		Ok(None)
	}
}