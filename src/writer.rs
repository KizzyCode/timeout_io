use crate::{ TimeoutIoError, InstantExt, WaitForEvent, EventMask };
use std::{
	io::Write,
	time::{ Duration, Instant }
};


/// A trait for writing with timeouts
pub trait Writer {
	/// Executes _one_ `write`-operation to write _as much bytes as possible_ from `data[*pos..]`
	/// and adjusts `pos` accordingly
	///
	/// This is especially useful in packet-based contexts where `write`-operations are atomic (like
	/// in UDP)
	///
	/// _Note: This function catches all interal timeouts/interrupts and returns only if there was
	/// either one successful `write`-operation or the `timeout` was hit or a non-recoverable error
	/// occurred._
	///
	/// __Warning: `self` must non-blocking or the function won't work as expected__
	fn try_write(&mut self, data: &[u8], pos: &mut usize, timeout: Duration)
		-> Result<(), TimeoutIoError>;
	
	/// Reads until `buf[*pos..]` has been written completely and adjusts `pos` _on every successful
	/// `write`-call_ (so that you can continue seamlessly on `TimedOut`-errors etc.)
	///
	/// This is especially useful in stream-based contexts where partial-`write`-calls are common
	/// (like in TCP)
	///
	/// _Note: This function catches all interal timeouts/interrupts and returns only if either
	/// `data` has been filled completely or the `timeout` was hit or a non-recoverable error
	/// occurred._
	///
	/// __Warning: `self` must non-blocking or the function won't work as expected__
	fn try_write_exact(&mut self, data: &[u8], pos: &mut usize, timeout: Duration)
		-> Result<(), TimeoutIoError>;
}
impl<T: Write + WaitForEvent> Writer for T {
	fn try_write(&mut self, data: &[u8], pos: &mut usize, timeout: Duration)
		-> Result<(), TimeoutIoError>
	{
		// Compute the deadline
		let deadline = Instant::now() + timeout;
		
		// Wait for write-events and write data
		if *pos >= data.len() { return Ok(()) }
		loop {
			self.wait_for_event(EventMask::new_w(),deadline.remaining())?;
			match self.write(data) {
				Ok(0) => return Err(TimeoutIoError::UnexpectedEof),
				Ok(written) => {
					*pos += written;
					return Ok(())
				},
				Err(error) => {
					let error = TimeoutIoError::from(error);
					if !error.should_retry() { return Err(error) }
				}
			}
		}
	}
	fn try_write_exact(&mut self, data: &[u8], pos: &mut usize, timeout: Duration)
		-> Result<(), TimeoutIoError>
	{
		// Compute the deadline
		let deadline = Instant::now() + timeout;
		
		// Loop until `data` has been written
		while *pos < data.len() {
			// Wait for write-event
			self.wait_for_event(EventMask::new_w(), deadline.remaining())?;
			
			// Write data
			match self.write(&data[*pos..]) {
				Ok(0) => return Err(TimeoutIoError::UnexpectedEof),
				Ok(written) => *pos += written,
				Err(error) => {
					let error = TimeoutIoError::from(error);
					if !error.should_retry() { return Err(error) }
				}
			}
		}
		Ok(())
	}
}