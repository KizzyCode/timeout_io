use timeout_io::*;
use std::{
	thread, time::Duration, io::Read,
	net::{ TcpListener, TcpStream, Shutdown },
	sync::mpsc::{ self, Receiver },
};


fn read_async(mut stream: impl 'static + Read + Send + RawFd, to_read: usize) -> Receiver<Vec<u8>> {
	let (sender, receiver) = mpsc::channel();
	thread::spawn(move || {
		// We need this for `read_exact`
		stream.set_blocking_mode(true).unwrap();
		
		// Block until we can fill `buf` completely
		let mut buf = vec![0u8; to_read];
		stream.read_exact(&mut buf).unwrap();
		sender.send(buf).unwrap();
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
	let (s0, s1) = (TcpStream::connect(address).unwrap(), listener.recv().unwrap());
	s0.set_blocking_mode(false).unwrap();
	s1.set_blocking_mode(false).unwrap();
	
	(s0, s1)
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
	let fut = read_async(s1, 9);
	
	let (data, mut pos) = (rand(9), 0);
	s0.try_write(&mut data.clone(), &mut pos, Duration::from_secs(1)).unwrap();
	assert_eq!(fut.recv().unwrap(), data);
}
#[test] #[ignore]
fn test_write_oneshot_err_broken_pipe() {
	let mut s0 = socket_pair().0;
	
	// Write some data to start the connection timeout
	s0.try_write(b"Testolope", &mut 0, Duration::from_secs(1)).unwrap();
	
	// Sleep until we can be sure that the timeout has been reached
	thread::sleep(Duration::from_secs(90));
	let (mut data, mut pos) = (rand(16 * 1024 * 1024), 0);
	assert_eq!(
		s0.try_write(&mut data, &mut pos, Duration::from_secs(1)).unwrap_err(),
		TimeoutIoError::ConnectionLost
	)
}
#[test]
fn test_write_oneshot_err() {
	let (mut s0, _s1) = socket_pair();
	s0.shutdown(Shutdown::Both).unwrap();
	
	let (mut data, mut pos) = (rand(16 * 1024 * 1024), 0);
	let err = s0.try_write(&mut data, &mut pos, Duration::from_secs(1)).unwrap_err();
	
	#[cfg(unix)]
	assert_eq!(err, TimeoutIoError::ConnectionLost);
	
	#[cfg(windows)]
	match err {
		TimeoutIoError::Other{ .. } => (),
		err => panic!("Invalid error returned: {:?}", err)
	}
}
#[test]
fn test_write_oneshot_timeout() {
	let (mut s0, _s1) = socket_pair();
	s0.set_nonblocking(true).unwrap();
	
	// Write until the connection buffer is apparently filled
	loop {
		let (mut data, mut pos) = (rand(64 * 1024 * 1024), 0);
		if let Err(e) = s0.try_write(&mut data, &mut pos, Duration::from_secs(1)) {
			if e == TimeoutIoError::TimedOut { break }
				else { panic!(e) }
		}
	}
	
	// Final test
	let (mut data, mut pos) = (rand(64 * 1024 * 1024), 0);
	assert_eq!(
		s0.try_write(&mut data, &mut pos, Duration::from_secs(1)).unwrap_err(),
		TimeoutIoError::TimedOut
	)
}


#[test]
fn test_write_exact_ok() {
	let (mut s0, s1) = socket_pair();
	
	let (data, mut pos) = (rand(64 * 1024 * 1024), 0);
	let fut = read_async(s1, data.len());
	
	s0.try_write_exact(
		&mut data.clone(), &mut pos,
		Duration::from_secs(4)
	).unwrap();
	assert_eq!(fut.recv().unwrap(), data)
}
#[test]
fn test_write_exact_err() {
	let (mut s0, _s1) = socket_pair();
	s0.shutdown(Shutdown::Both).unwrap();
	
	let (data, mut pos) = (rand(64 * 1024 * 1024), 0);
	let err = s0
		.try_write_exact(&mut data.clone(), &mut pos, Duration::from_secs(4))
		.unwrap_err();
	
	#[cfg(unix)]
	assert_eq!(err, TimeoutIoError::ConnectionLost);
	
	#[cfg(windows)]
		match err {
		TimeoutIoError::Other{ .. } => (),
		err => panic!("Invalid error returned: {:?}", err)
	}
}
#[test] #[ignore]
fn test_write_exact_timeout() {
	let (mut s0, _s1) = socket_pair();
	
	let (data, mut pos) = (rand(64 * 1024 * 1024), 0);
	assert_eq!(s0.try_write_exact(
		&mut data.clone(), &mut pos,
		Duration::from_secs(1)
	).unwrap_err(), TimeoutIoError::TimedOut)
}