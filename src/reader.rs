use crate::{ TimeoutIoError, InstantExt, WaitForEvent, EventMask };
use std::{
	io::Read,
	time::{ Duration, Instant }
};


/// A trait for reading with timeouts
pub trait Reader {
	/// Executes _one_ `read`-operation to read _as much bytes as possible_ into `buf[*pos..]` and
	/// adjusts `pos` accordingly
	///
	/// This is especially useful in packet-based contexts where `read`-operations are atomic
	/// (like in UDP) or if you don't know the amount of bytes in advance
	///
	/// _Note: This function catches all internal timeouts/interrupts and returns only if there was
	/// either one successful `read`-operation or the `timeout` was hit or a non-recoverable error
	/// occurred._
	///
	/// __Warning: `self` must non-blocking or the function won't work as expected__
	fn try_read(&mut self, buf: &mut[u8], pos: &mut usize, timeout: Duration)
		-> Result<(), TimeoutIoError>;
	
	/// Reads until `buf[*pos..]` is filled completely and adjusts `pos` _on every successful
	/// `read`-call_ (so that you can continue seamlessly on `TimedOut`-errors etc.)
	///
	/// This is especially useful in stream-based contexts where partial-`read`-calls are common
	/// (like in TCP) and you want to read a well-known amount of bytes
	///
	/// _Note: This function catches all internal timeouts/interrupts and returns only if either
	/// `buf` has been filled completely or the `timeout` was exceeded or a non-recoverable error
	/// occurred._
	///
	/// __Warning: `self` must non-blocking or the function won't work as expected__
	fn try_read_exact(&mut self, buf: &mut[u8], pos: &mut usize, timeout: Duration)
		-> Result<(), TimeoutIoError>;
	
	/// Reads until either `pat` is matched or `buf` is filled completely and adjusts `pos`
	/// accordingly. Returns `true` if `pat` was matched and `false` otherwise.
	///
	/// _Note: While the reading is continued at `*pos`, `pat` is matched against the entire `buf`_
	///
	/// _Note: This function catches all interal timeouts/interrupts and returns only if either
	/// `pattern` has been matched or `buffer` has been filled completely or the `timeout` was hit
	/// or a non-recoverable error occurred._
	///
	/// __Warning: `self` must non-blocking or the function won't work as expected__
	fn try_read_until(&mut self, buf: &mut[u8], pos: &mut usize, pat: &[u8], timeout: Duration)
		-> Result<bool, TimeoutIoError>;
}
impl<T: Read + WaitForEvent> Reader for T {
	fn try_read(&mut self, buf: &mut[u8], pos: &mut usize, timeout: Duration)
		-> Result<(), TimeoutIoError>
	{
		// Loop until we have *one* successful read
		if *pos >= buf.len() { return Ok(()) }
		loop {
			// Wait for read-event and read data
			self.wait_for_event(EventMask::new_r(), timeout)?;
			match self.read(&mut buf[*pos..]) {
				Ok(0) => return Err(TimeoutIoError::UnexpectedEof),
				Ok(read) => {
					*pos += read;
					return Ok(())
				},
				Err(error) => {
					let error = TimeoutIoError::from(error);
					if !error.should_retry() { return Err(error) }
				}
			}
		}
	}
	fn try_read_exact(&mut self, buf: &mut[u8], pos: &mut usize, timeout: Duration)
		-> Result<(), TimeoutIoError>
	{
		// Compute the deadline
		let deadline = Instant::now() + timeout;
		
		// Loop until buffer is filled completely
		while *pos < buf.len() {
			// Wait for read-event and read data
			self.wait_for_event(EventMask::new_r(), deadline.remaining())?;
			match self.read(&mut buf[*pos..]) {
				Ok(0) => return Err(TimeoutIoError::UnexpectedEof),
				Ok(read) => *pos += read,
				Err(error) => {
					let error = TimeoutIoError::from(error);
					if !error.should_retry() { return Err(error) }
				}
			}
		}
		Ok(())
	}
	fn try_read_until(&mut self, buf: &mut[u8], pos: &mut usize, pat: &[u8], timeout: Duration)
		-> Result<bool, TimeoutIoError>
	{
		// Compute deadline
		let deadline = Instant::now() + timeout;
		
		// Loop until `data` has been filled
		while *pos < buf.len() {
			// Read next byte
			let next = *pos + 1;
			self.try_read_exact(&mut buf[..next], pos, deadline.remaining())?;
			
			// Check for pattern
			if *pos >= pat.len() && &buf[*pos - pat.len() .. *pos] == pat {
				return Ok(true)
			}
		}
		Ok(false)
	}
}