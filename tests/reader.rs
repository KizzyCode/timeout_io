#[macro_use] extern crate tiny_future;
extern crate slice_queue;
extern crate timeout_io;

use tiny_future::{ Future, async };
use slice_queue::SliceQueue;
use timeout_io::*;
use std::{
	time::Duration, thread, io::Write,
	net::{ TcpListener, TcpStream }
};


fn write_delayed(mut stream: impl 'static + Write + Send, data: &'static [u8], delay: Duration) {
	thread::spawn(move || {
		thread::sleep(delay);
		stream.write_all(data).unwrap();
	});
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


#[test]
fn test_read_oneshot_ok() {
	let (mut s0, s1) = socket_pair();
	write_delayed(s1.try_clone().unwrap(), b"Testolope", Duration::from_secs(4));
	
	let mut buffer = SliceQueue::with_limit(4096);
	s0.read_oneshot(&mut buffer, Duration::from_secs(7)).unwrap();
	assert_eq!(&buffer[..], b"Testolope");
}
#[test]
fn test_read_oneshot_err() {
	let mut s0 = socket_pair().0;
	let mut buffer = SliceQueue::with_limit(4096);
	assert_eq!(
		s0.read_oneshot(&mut buffer, Duration::from_secs(4)).unwrap_err().kind.kind,
		IoErrorKind::UnexpectedEof
	)
}
#[test]
fn test_read_oneshot_timeout() {
	let (mut s0, _s1) = socket_pair();
	let mut buffer = SliceQueue::with_limit(4096);
	assert_eq!(
		s0.read_oneshot(&mut buffer, Duration::from_secs(4)).unwrap_err().kind.kind,
		IoErrorKind::TimedOut
	)
}

#[test]
fn test_read_exact_ok() {
	let (mut s0, s1) = socket_pair();
	write_delayed(s1.try_clone().unwrap(), b"Test", Duration::from_secs(1));
	write_delayed(s1.try_clone().unwrap(), b"olope", Duration::from_secs(4));
	
	let mut buffer = SliceQueue::with_limit(9);
	s0.read_exact(&mut buffer, Duration::from_secs(7)).unwrap();
	assert_eq!(&buffer[..], b"Testolope");
}
#[test]
fn test_read_exact_err() {
	let (mut s0, s1) = socket_pair();
	write_delayed(s1, b"Test", Duration::from_secs(4));
	
	let mut buffer = SliceQueue::with_limit(9);
	assert_eq!(
		s0.read_exact(&mut buffer, Duration::from_secs(7)).unwrap_err().kind.kind,
		IoErrorKind::UnexpectedEof
	)
}
#[test]
fn test_read_exact_timeout() {
	let (mut s0, s1) = socket_pair();
	write_delayed(s1.try_clone().unwrap(), b"Test", Duration::from_secs(4));
	
	let mut buffer = SliceQueue::with_limit(9);
	assert_eq!(
		s0.read_exact(&mut buffer, Duration::from_secs(7)).unwrap_err().kind.kind,
		IoErrorKind::TimedOut
	)
}

#[test]
fn test_read_until_ok() {
	let (mut s0, s1) = socket_pair();
	write_delayed(s1.try_clone().unwrap(), b"Test", Duration::from_secs(1));
	write_delayed(s1.try_clone().unwrap(), b"o", Duration::from_secs(3));
	write_delayed(s1.try_clone().unwrap(), b"lope\n!", Duration::from_secs(5));
	
	let mut buffer = SliceQueue::with_limit(4096);
	s0.read_until(b"\n", &mut buffer, Duration::from_secs(7)).unwrap();
	assert_eq!(&buffer[..], b"Testolope\n");
}
#[test]
fn test_read_until_not_found() {
	let (mut s0, s1) = socket_pair();
	write_delayed(s1.try_clone().unwrap(), b"Testolope", Duration::from_secs(1));
	write_delayed(s1, b"!", Duration::from_secs(4));
	
	let mut buffer = SliceQueue::with_limit(10);
	assert_eq!(
		s0.read_until(b"\n", &mut buffer, Duration::from_secs(7)).unwrap_err().kind.kind,
		IoErrorKind::NotFound
	)
}
#[test]
fn test_read_until_err() {
	let (mut s0, s1) = socket_pair();
	write_delayed(s1.try_clone().unwrap(), b"Testolope", Duration::from_secs(1));
	write_delayed(s1, b"!", Duration::from_secs(4));
	
	let mut buffer = SliceQueue::with_limit(4096);
	assert_eq!(
		s0.read_until(b"\n", &mut buffer, Duration::from_secs(7)).unwrap_err().kind.kind,
		IoErrorKind::UnexpectedEof
	)
}
#[test]
fn test_read_until_timeout() {
	let (mut s0, s1) = socket_pair();
	write_delayed(s1.try_clone().unwrap(), b"Testolope", Duration::from_secs(1));
	write_delayed(s1.try_clone().unwrap(), b"!", Duration::from_secs(4));
	
	let mut buffer = SliceQueue::with_limit(4096);
	assert_eq!(
		s0.read_until(b"\n", &mut buffer, Duration::from_secs(7)).unwrap_err().kind.kind,
		IoErrorKind::TimedOut
	)
}