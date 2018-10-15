use ::{ IoError, Result, InstantExt, WaitForEvent };
use ::std::{ io::Read, time::{ Duration, Instant }, io::ErrorKind as IoErrorKind };


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
	/// __Warning: This function makes `self` non-blocking. It's up to you to restore the previous
	/// state if necessary.__
	///
	/// Parameters:
	///  - `buffer`: The buffer to write the data to
	///  - `timeout`: The maximum time this function will wait for `self` to become readable
	///
	/// Returns either __the amount of bytes read__ or a corresponding `IoError`
	fn read(&mut self, buffer: &mut[u8], timeout: Duration) -> Result<usize>;
	
	/// Reads until `buffer` has been filled completely
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
	///
	/// Parameters:
	///  - `buffer`: The buffer to fill with data
	///  - `timeout`: The maximum time this function will block
	///
	/// Returns either __nothing__ or a corresponding `IoError`
	fn read_exact(&mut self, buffer: &mut[u8], timeout: Duration) -> Result<()>;
	
	/// Read until either `pattern` has been matched or `buffer` has been filled completely
	///
	/// _Note: This function catches all interal timeouts/interrupts and returns only if either
	/// `pattern` has been matched or `buffer` has been filled completely or the `timeout` was hit
	/// or a non-recoverable error occurred._
	///
	/// __Warning: This function makes `self` non-blocking. It's up to you to restore the previous
	/// state if necessary.__
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
	fn read_until(&mut self, pattern: &[u8], buffer: &mut[u8], timeout: Duration) -> Result<usize>;
}
impl<T: Read + WaitForEvent> Reader for T {
	fn read(&mut self, buffer: &mut[u8], timeout: Duration) -> Result<usize> {
		// Make the socket non-blocking
		try_err!(self.set_blocking_mode(false));
		
		// Immediately return if we should not read any bytes
		if buffer.len() == 0 { return Ok(0) }
		
		// Wait for read-event and read data
		try_err!(self.wait_until_readable(timeout));
		loop {
			match self.read(buffer) {
				Ok(bytes_read) => if bytes_read > 0 { return Ok(bytes_read) }
					else { throw_err!(IoErrorKind::UnexpectedEof.into()) },
				Err(error) => {
					let error = IoError::from(error);
					if error.non_recoverable { throw_err!(error) }
				}
			}
		}
	}
	
	fn read_exact(&mut self, buffer: &mut[u8], timeout: Duration) -> Result<()> {
		// Make the socket non-blocking
		try_err!(self.set_blocking_mode(false));
		
		// Compute timeout-point and loop until buffer is filled completely
		let timeout_point = Instant::now() + timeout;
		
		// Read loop
		let mut total_read = 0;
		while buffer.len() - total_read > 0 {
			// Wait for read-event and read data
			try_err!(self.wait_until_readable(timeout_point.remaining()));
			match self.read(&mut buffer[total_read..]) {
				Ok(bytes_read) => if bytes_read > 0 { total_read += bytes_read }
					else { throw_err!(IoErrorKind::UnexpectedEof.into()) },
				Err(error) => {
					let error = IoError::from(error);
					if error.non_recoverable { throw_err!(error) }
				}
			}
		}
		Ok(())
	}
	
	fn read_until(&mut self, pattern: &[u8], buffer: &mut[u8], timeout: Duration) -> Result<usize> {
		// Compute timeout-point
		let timeout_point = Instant::now() + timeout;
		
		// Compute timeout-point and loop until `data` has been filled
		let mut total_read = 0;
		while buffer.len() - total_read > 0 {
			// Read next byte
			try_err!(Reader::read_exact(self, &mut buffer[total_read .. total_read + 1], timeout_point.remaining()));
			total_read += 1;
			
			// Check for pattern
			if total_read >= pattern.len() && &buffer[total_read - pattern.len() .. total_read] == pattern {
				return Ok(total_read)
			}
		}
		throw_err!(IoErrorKind::NotFound.into())
	}
}