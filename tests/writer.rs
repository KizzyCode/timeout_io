extern crate timeout_io;

use timeout_io::*;
use std::{
	thread, time::Duration, io::Read,
	sync::mpsc::{ self, Receiver }, net::{ TcpListener, TcpStream, Shutdown }
};


fn read_async(mut stream: impl 'static + Read + Send, to_read: usize) -> Receiver<Vec<u8>> {
	let (sender, receiver) = mpsc::channel();
	thread::spawn(move || {
		let mut buffer = vec![0u8; to_read];
		stream.read_exact(&mut buffer).unwrap();
		sender.send(buffer).unwrap();
	});
	receiver
}

fn socket_pair() -> (TcpStream, TcpStream) {
	// Create listener
	let (listener, address) = {
		// Create listener (to capture the address) and channels
		let listener = TcpListener::bind("127.0.0.1:0").unwrap();
		let address = listener.local_addr().unwrap();
		let (sender, receiver) = mpsc::channel();
		
		// Listen in background
		thread::spawn(move || sender.send(listener.accept().unwrap().0).unwrap());
		(receiver, address)
	};
	
	// Create and connect stream
	(TcpStream::connect(address).unwrap(), listener.recv().unwrap())
}

fn rand(len: usize) -> Vec<u8> {
	let block: &[u8] = include_bytes!("rand.dat");
	
	// Accumulate random data
	let mut slice_queue = Vec::new();
	while slice_queue.len() < len { slice_queue.extend_from_slice(block) }
	
	// Drop superflous bytes
	slice_queue.truncate(len);
	slice_queue
}


#[test]
fn test_write_oneshot_ok() {
	let (mut s0, s1) = socket_pair();
	let async = read_async(s1, 9);
	
	let data = rand(9);
	s0.write(&mut data.clone(), Duration::from_secs(1)).unwrap();
	assert_eq!(async.recv().unwrap(), data);
}
#[test] #[ignore]
fn test_write_oneshot_err_broken_pipe() {
	let mut s0 = socket_pair().0;
	
	// Write some data to start the connection timeout
	s0.write(b"Testolope", Duration::from_secs(1)).unwrap();
	
	// Sleep until we can be sure that the timeout has been reached
	thread::sleep(Duration::from_secs(90));
	let mut data = rand(16 * 1024 * 1024);
	assert_eq!(
		s0.write(&mut data, Duration::from_secs(1)).unwrap_err().kind.kind,
		IoErrorKind::BrokenPipe
	)
}
#[test]
fn test_write_oneshot_err() {
	let (mut s0, _s1) = socket_pair();
	s0.shutdown(Shutdown::Both).unwrap();
	
	let mut data = rand(16 * 1024 * 1024);
	
	assert_eq!(
		s0.write(&mut data, Duration::from_secs(1)).unwrap_err().kind.kind,
		match true {
			_ if cfg!(unix) => IoErrorKind::BrokenPipe,
			_ if cfg!(windows) => IoErrorKind::Other,
			_ => panic!("Unsupported platform")
		}
	);
}
#[test]
fn test_write_oneshot_timeout() {
	let (mut s0, _s1) = socket_pair();
	s0.set_nonblocking(true).unwrap();
	
	// Write until the connection buffer is apparently filled
	loop {
		let mut data = rand(64 * 1024 * 1024);
		if let Err(e) = s0.write(&mut data, Duration::from_secs(1)) {
			if e.kind.kind == IoErrorKind::TimedOut { break }
				else { panic!(e) }
		}
	}
	
	// Final test
	let mut data = rand(64 * 1024 * 1024);
	assert_eq!(
		s0.write(&mut data, Duration::from_secs(1)).unwrap_err().kind.kind,
		IoErrorKind::TimedOut
	)
}


#[test]
fn test_write_exact_ok() {
	let (mut s0, s1) = socket_pair();
	
	let data = rand(64 * 1024 * 1024);
	let async = read_async(s1, data.len());
	
	s0.write_exact(&mut data.clone(), Duration::from_secs(4)).unwrap();
	assert_eq!(async.recv().unwrap(), &data[..])
}
#[test]
fn test_write_exact_err() {
	let (mut s0, _s1) = socket_pair();
	s0.shutdown(Shutdown::Both).unwrap();
	
	let data = rand(64 * 1024 * 1024);
	
	assert_eq!(
		s0.write_exact(&mut data.clone(), Duration::from_secs(4)).unwrap_err().kind.kind,
		match true {
			_ if cfg!(unix) => IoErrorKind::BrokenPipe,
			_ if cfg!(windows) => IoErrorKind::Other,
			_ => panic!("Unsupported platform")
		}
	);
}
#[test] #[ignore]
fn test_write_exact_timeout() {
	let (mut s0, _s1) = socket_pair();
	
	let data = rand(64 * 1024 * 1024);
	assert_eq!(
		s0.write_exact(&mut data.clone(), Duration::from_secs(1)).unwrap_err().kind.kind,
		IoErrorKind::TimedOut
	)
}