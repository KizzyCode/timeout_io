extern crate timeout_io;
use timeout_io::*;
use std::{ net::TcpListener, time::Duration };

#[test]
fn test_event_select_err() {
	let raw_fd = {
		let listener = TcpListener::bind("127.0.0.1:0").unwrap();
		listener.raw_fd()
	};
	let result = unsafe{ libselect::wait_for_event(
		raw_fd,
		Event::Read | Event::Error,
		Duration::from_secs(1).as_ms()
	) };
	assert!(result & Event::SyscallError)
}