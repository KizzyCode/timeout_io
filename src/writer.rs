use super::{ IoError, Result, SliceQueue, ReadableSliceQueue, InstantExt, WaitForEvent };
use std::{ io::{ Write, ErrorKind as IoErrorKind }, time::{ Duration, Instant } };


/// A trait for writing with timeouts
pub trait Writer {
	/// Executes _one_ `write`-operation to write _as much bytes as possible_ from `data`
	///
	/// This is especially useful in packet-based contexts where `write`-operations are atomic (like
	/// in UDP)
	///
	/// _Note: This function catches all interal timeouts/interrupts and returns only if there was
	/// either one successful `write`-operation or the `timeout` was hit or a non-recoverable error
	/// occurred._
	///
	/// _Warning: This function makes `self` non-blocking. It's up to you to restore the previous
	/// state if necessary._
	///
	/// Parameters:
	///  - `data`: The data to write
	///  - `timeout`: The maximum time this function will wait for `self` to become writeable
	///
	/// Returns either __nothing__ or a corresponding `IoError`
	fn write_oneshot(&mut self, data: &mut SliceQueue<u8>, timeout: Duration) -> Result<()>;
	
	/// Writes all bytes in `data`
	///
	/// This is especially useful in stream-based contexts where partial-`write`-calls are common
	/// (like in TCP)
	///
	/// _Note: This function catches all interal timeouts/interrupts and returns only if either
	/// `data` has been filled completely or the `timeout` was hit or a non-recoverable error
	/// occurred._
	///
	/// _Warning: This function makes `self` non-blocking. It's up to you to restore the previous
	/// state if necessary._
	///
	/// Parameters:
	///  - `data`: The data to write
	///  - `timeout`: The maximum time this function will wait for `self` to become writeable
	///
	/// Returns either __nothing__ or a corresponding `IoError`
	fn write_exact(&mut self, data: &mut SliceQueue<u8>, timeout: Duration) -> Result<()>;
}
impl<T: Write + WaitForEvent> Writer for T {
	fn write_oneshot(&mut self, data: &mut SliceQueue<u8>, timeout: Duration) -> Result<()> {
		// Make the socket non-blocking
		try_err!(self.set_blocking_mode(false));
		
		// Immediately return if we should not read any bytes
		if data.is_empty() { return Ok(()) }
		
		// Wait for write-events and write data
		let timeout_point = Instant::now() + timeout;
		loop {
			try_err!(self.wait_until_writeable(timeout_point.remaining()));
			match self.write(data) {
				Ok(bytes_written) if bytes_written == 0 =>
					throw_err!(IoErrorKind::UnexpectedEof.into()),
				Ok(bytes_written) => {
					data.drop_n(bytes_written).unwrap();
					return Ok(())
				},
				Err(error) => {
					let error = IoError::from(error);
					if error.non_recoverable { throw_err!(error) }
				}
			}
		}
	}
	
	fn write_exact(&mut self, data: &mut SliceQueue<u8>, timeout: Duration) -> Result<()> {
		// Make the socket non-blocking
		try_err!(self.set_blocking_mode(false));
		
		// Compute timeout-point and loop until data is empty
		let timeout_point = Instant::now() + timeout;
		while !data.is_empty() {
			// Wait for write-event
			try_err!(self.wait_until_writeable(timeout_point.remaining()));
			
			// Write data
			match self.write(data) {
				// (Partial-)write
				Ok(bytes_written) => data.drop_n(bytes_written).unwrap(),
				// An error occurred
				Err(error) => {
					let error = IoError::from(error);
					if error.non_recoverable { throw_err!(error) }
				}
			}
		}
		Ok(())
	}
}