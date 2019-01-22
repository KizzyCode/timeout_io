use crate::{ TimeoutIoError, InstantExt, WaitForEvent, EventMask };
use std::{ io::Write, time::{ Duration, Instant } };


/// A trait for writing with timeouts
pub trait Writer {
	/// Executes _one_ `write`-operation to write _as much bytes as possible_ from `data` and
	/// returns the amount of bytes written
	///
	/// This is especially useful in packet-based contexts where `write`-operations are atomic (like
	/// in UDP)
	///
	/// _Note: This function catches all interal timeouts/interrupts and returns only if there was
	/// either one successful `write`-operation or the `timeout` was hit or a non-recoverable error
	/// occurred._
	///
	/// __Warning: This function makes `self` non-blocking. It's up to you to restore the previous
	/// state if necessary.__
	fn write(&mut self, data: &[u8], timeout: Duration) -> Result<usize, TimeoutIoError>;
	
	/// Writes all bytes in `data`
	///
	/// This is especially useful in stream-based contexts where partial-`write`-calls are common
	/// (like in TCP)
	///
	/// _Note: This function catches all interal timeouts/interrupts and returns only if either
	/// `data` has been filled completely or the `timeout` was hit or a non-recoverable error
	/// occurred._
	///
	/// __Warning: This function makes `self` non-blocking. It's up to you to restore the previous
	/// state if necessary.__
	fn write_exact(&mut self, data: &[u8], timeout: Duration) -> Result<(), TimeoutIoError>;
}
impl<T: Write + WaitForEvent> Writer for T {
	fn write(&mut self, data: &[u8], timeout: Duration) -> Result<usize, TimeoutIoError> {
		// Make the socket non-blocking
		self.set_blocking_mode(false)?;
		
		// Immediately return if we should not write any bytes
		if data.is_empty() { return Ok(0) }
		
		// Wait for write-events and write data
		let timeout_point = Instant::now() + timeout;
		loop {
			self.wait_for_event(EventMask::new_w(),timeout_point.remaining())?;
			match self.write(data) {
				Ok(0) => return Err(TimeoutIoError::UnexpectedEof),
				Ok(bytes_written) => return Ok(bytes_written),
				Err(error) => {
					let error = TimeoutIoError::from(error);
					if !error.should_retry() { return Err(error) }
				}
			}
		}
	}
	
	fn write_exact(&mut self, mut data: &[u8], timeout: Duration) -> Result<(), TimeoutIoError> {
		// Make the socket non-blocking
		self.set_blocking_mode(false)?;
		
		// Compute timeout-point and loop until data is empty
		let timeout_point = Instant::now() + timeout;
		while !data.is_empty() {
			// Wait for write-event
			self.wait_for_event(EventMask::new_w(), timeout_point.remaining())?;
			
			// Write data
			match self.write(data) {
				Ok(0) => return Err(TimeoutIoError::UnexpectedEof),
				Ok(bytes_written) => data = &data[bytes_written..],
				Err(error) => {
					let error = TimeoutIoError::from(error);
					if !error.should_retry() { return Err(error) }
				}
			}
		}
		Ok(())
	}
}