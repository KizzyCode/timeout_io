#[macro_use] extern crate tiny_future;
extern crate slice_queue;
extern crate io;

use tiny_future::{ Future, async };
use slice_queue::SliceQueue;
use io::*;
use std::{
	thread, io::{ Read, Write }, collections::hash_map::DefaultHasher, hash::Hasher,
	time::{ Duration, SystemTime, UNIX_EPOCH },
	net::{ TcpListener, TcpStream, Shutdown }
};


fn read_async(mut stream: impl 'static + Read + Send, to_read: usize) -> Future<Vec<u8>> {
	async(move |fut| {
		let mut buffer = vec![0u8; to_read];
		stream.read_exact(&mut buffer).unwrap();
		job_return!(fut, buffer);
	})
}

fn socket_pair() -> (TcpStream, TcpStream) {
	// Create listener
	let (listener, address): (Future<TcpStream>, _) = {
		let listener = TcpListener::bind("127.0.0.1:0").unwrap();
		let address = listener.local_addr().unwrap();
		
		(async(move |fut: Future<TcpStream>| {
			job_return!(fut, listener.accept().unwrap().0);
		}), address)
	};
	
	// Create and connect stream
	(TcpStream::connect(address).unwrap(), listener.get().unwrap())
}

fn rand(min_len: usize) -> SliceQueue<u8> {
	fn u64_be(value: u64) -> [u8; 8] {
		[(value >> 56) as u8, (value >> 48) as u8, (value >> 40) as u8, (value >> 32) as u8,
			(value >> 24) as u8, (value >> 16) as u8, (value >>  8) as u8, (value >>  0) as u8]
	}
	fn next(prev: &[u8]) -> [u8; 8] {
		let mut hasher = DefaultHasher::new();
		hasher.write(prev);
		u64_be(hasher.finish())
	}
	
	let mut slice_queue = SliceQueue::new();
	let mut prev = u64_be(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_ms());
	while slice_queue.len() < min_len {
		prev = next(&prev);
		slice_queue.push_from(&prev).unwrap();
	}
	slice_queue
}


#[test]
fn test_write_oneshot_ok() {
	let (mut s0, s1) = socket_pair();
	let fut = read_async(s1, 9);
	
	let mut data = rand(9);
	s0.write_oneshot(&mut data.clone(), Duration::from_secs(1)).unwrap();
	assert_eq!(fut.get().unwrap(), data.pop_n(9).unwrap());
}
#[test] #[ignore]
fn test_write_oneshot_err_broken_pipe() {
	let mut s0 = socket_pair().0;
	
	// Write some data to start the connection timeout
	s0.write(b"Testolope").unwrap();
	
	// Sleep until we can be sure that the timeout has been reached
	thread::sleep(Duration::from_secs(90));
	let mut data = rand(16 * 1024 * 1024);
	assert_eq!(
		s0.write_oneshot(&mut data, Duration::from_secs(1)).unwrap_err().kind.kind,
		IoErrorKind::BrokenPipe
	)
}
#[test]
fn test_write_oneshot_err() {
	let (mut s0, _s1) = socket_pair();
	s0.shutdown(Shutdown::Both).unwrap();
	
	let mut data = rand(16 * 1024 * 1024);
	assert_eq!(
		s0.write_oneshot(&mut data, Duration::from_secs(1)).unwrap_err().kind.kind,
		IoErrorKind::BrokenPipe
	)
}
#[test] #[ignore]
fn test_write_oneshot_timeout() {
	let (mut s0, _s1) = socket_pair();
	s0.set_nonblocking(true).unwrap();
	
	// Write until the connection buffer is apparently filled
	let mut data = rand(64 * 1024 * 1024);
	loop {
		if let Err(e) = s0.write(&data) {
			if e.kind() == IoErrorKind::WouldBlock { break }
				else { panic!(e) }
		}
	}
	
	// Try to write some data
	assert_eq!(
		s0.write_oneshot(&mut data, Duration::from_secs(1)).unwrap_err().kind.kind,
		IoErrorKind::TimedOut
	)
}


#[test]
fn test_write_exact_ok() {
	let (mut s0, s1) = socket_pair();
	
	let data = rand(64 * 1024 * 1024);
	let fut = read_async(s1, data.len());
	
	s0.write_exact(&mut data.clone(), Duration::from_secs(4)).unwrap();
	assert_eq!(fut.get().unwrap(), &data[..])
}
#[test]
fn test_write_exact_err() {
	let (mut s0, _s1) = socket_pair();
	s0.shutdown(Shutdown::Both).unwrap();
	
	let data = rand(64 * 1024 * 1024);
	assert_eq!(
		s0.write_exact(&mut data.clone(), Duration::from_secs(4)).unwrap_err().kind.kind,
		IoErrorKind::BrokenPipe
	)
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