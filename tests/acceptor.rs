use timeout_io::*;
use std::{ time::Duration, thread, net::{ TcpListener, TcpStream } };

#[test]
fn test_accept_ok() {
	let listener = TcpListener::bind("127.0.0.1:0").unwrap();
	
	let address = listener.local_addr().unwrap();
	thread::spawn(move || {
		thread::sleep(Duration::from_secs(4));
		TcpStream::connect(address).unwrap();
	});
	
	Acceptor::accept(&listener, Duration::from_secs(7)).unwrap();
}
#[test]
fn test_accept_timeout() {
	let listener = TcpListener::bind("127.0.0.1:0").unwrap();
	assert_eq!(
		Acceptor::accept(&listener, Duration::from_secs(4)).unwrap_err(),
		TimeoutIoError::TimedOut
	)
}