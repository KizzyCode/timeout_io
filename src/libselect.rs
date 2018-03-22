use std;
use super::etrace::Error;
use super::IoError;



pub trait ToRawFd {
	fn get_raw_fd(&self) -> u64;
}
#[cfg(unix)]
impl<T: std::os::unix::io::AsRawFd> ToRawFd for T {
	fn get_raw_fd(&self) -> u64 { self.as_raw_fd() as u64 }
}
#[cfg(windows)]
impl<T: std::os::windows::io::AsRawSocket> ToRawFd for T {
	fn get_raw_fd(&self) -> u64 { self.as_raw_socket() as u64 }
}



mod c_impl {
	use std::os::raw::c_int;
	extern {
		pub static EVENT_READ: u8;
		pub static EVENT_WRITE: u8;
		pub static EVENT_ERROR: u8;
		pub static SELECT_ERROR: u8;
		
		pub fn wait_for_event(descriptor: u64, event: u8, timeout_ms: u64) -> u8;
		pub fn get_errno() -> c_int;
	}
}

// Wrapper
pub fn event_read<T: ToRawFd>(handle: &T, timeout: std::time::Duration) -> Result<bool, Error<IoError>> {
	// Create mask
	let (event_mask, result_mask) = (unsafe{ c_impl::EVENT_READ }, unsafe{ c_impl::EVENT_READ | c_impl::EVENT_ERROR });
	
	// Call `select` and check for error
	let result = unsafe{ c_impl::wait_for_event(
		handle.get_raw_fd(),
		event_mask,
		(timeout.as_secs() * 1000) + (timeout.subsec_nanos() as u64 / 1_000_000)
	) };
	if result & unsafe{ c_impl::SELECT_ERROR } != 0 { throw_err!(std::io::Error::from_raw_os_error(unsafe{ c_impl::get_errno() }).into()) }
	
	// Check mask
	Ok(result & result_mask != 0)
}

pub fn event_write<T: ToRawFd>(handle: &T, timeout: std::time::Duration) -> Result<bool, Error<IoError>> {
	// Create mask
	let (event_mask, result_mask) = (unsafe{ c_impl::EVENT_WRITE }, unsafe{ c_impl::EVENT_WRITE | c_impl::EVENT_ERROR });
	
	// Call `select` and check for error
	let result = unsafe{ c_impl::wait_for_event(
		handle.get_raw_fd(),
		event_mask,
		(timeout.as_secs() * 1000) + (timeout.subsec_nanos() as u64 / 1_000_000)
	) };
	if result & unsafe{ c_impl::SELECT_ERROR } != 0 { throw_err!(std::io::Error::from_raw_os_error(unsafe{ c_impl::get_errno() }).into()) }
	
	// Check mask
	Ok(result & result_mask != 0)
}



#[cfg(test)]
mod test {
	use std;
	
	#[test]
	fn test_accept_ready() {
		let listener = std::net::TcpListener::bind("localhost:0").expect("Failed to bind to address");
		listener.set_nonblocking(true).expect("Failed to make listener non-blocking");
		let address = listener.local_addr().expect("Failed to get socket-address");
		
		std::thread::spawn(move || {
			std::thread::sleep(std::time::Duration::from_secs(2));
			std::net::TcpStream::connect(address).expect("Failed to connect to listener");
		});
		
		assert!(super::event_read(&listener, std::time::Duration::from_secs(7)).unwrap());
	}
	
	#[test]
	fn test_accept_timeout() {
		let listener = std::net::TcpListener::bind("localhost:0").expect("Failed to bind to address");
		listener.set_nonblocking(true).expect("Failed to make listener non-blocking");
		
		assert_eq!(super::event_read(&listener, std::time::Duration::from_secs(7)).unwrap(), false)
	}
}
