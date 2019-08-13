use timeout_io::*;
use std::{
	time::Duration, thread, io::Write, sync::mpsc,
	net::{ TcpListener, TcpStream }
};


fn write_delayed(mut stream: impl 'static + Write + Send + RawFd, data: &'static [u8],
	delay: Duration)
{
	thread::spawn(move || {
		// We need this for `write_all`
		stream.set_blocking_mode(true).unwrap();
		
		// Write the data
		thread::sleep(delay);
		stream.write_all(data).unwrap();
	});
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


#[test]
fn test_read_oneshot_ok() {
	let (mut s0, s1) = socket_pair();
	write_delayed(
		s1.try_clone().unwrap(), b"Testolope",
		Duration::from_secs(4)
	);
	
	let (mut buf, mut pos) = ([0u8; 4096], 0);
	s0.try_read(&mut buf, &mut pos, Duration::from_secs(7)).unwrap();
	assert_eq!(&buf[..pos], b"Testolope");
}
#[test]
fn test_read_oneshot_err() {
	let mut s0 = socket_pair().0;
	let (mut buf, mut pos) = ([0u8; 4096], 0);
	assert_eq!(
		s0.try_read(&mut buf, &mut pos, Duration::from_secs(4)).unwrap_err(),
		TimeoutIoError::UnexpectedEof
	)
}
#[test]
fn test_read_oneshot_timeout() {
	let (mut s0, _s1) = socket_pair();
	let (mut buf, mut pos) = ([0u8; 4096], 0);
	assert_eq!(
		s0.try_read(&mut buf, &mut pos, Duration::from_secs(4)).unwrap_err(),
		TimeoutIoError::TimedOut
	)
}


#[test]
fn test_read_exact_ok() {
	let (mut s0, s1) = socket_pair();
	write_delayed(
		s1.try_clone().unwrap(), b"Test",
		Duration::from_secs(1)
	);
	write_delayed(
		s1.try_clone().unwrap(), b"olope",
		Duration::from_secs(4)
	);
	
	let (mut buf, mut pos) = ([0u8; 9], 0);
	s0.try_read_exact(&mut buf, &mut pos, Duration::from_secs(7)).unwrap();
	assert_eq!(&buf, b"Testolope");
}
#[test]
fn test_read_exact_err() {
	let (mut s0, s1) = socket_pair();
	write_delayed(s1, b"Test", Duration::from_secs(4));
	
	let (mut buf, mut pos) = ([0u8; 9], 0);
	assert_eq!(
		s0.try_read_exact(&mut buf, &mut pos, Duration::from_secs(7)).unwrap_err(),
		TimeoutIoError::UnexpectedEof
	)
}
#[test]
fn test_read_exact_timeout() {
	let (mut s0, s1) = socket_pair();
	write_delayed(
		s1.try_clone().unwrap(), b"Test",
		Duration::from_secs(4)
	);
	
	let (mut buf, mut pos) = ([0u8; 9], 0);
	assert_eq!(
		s0.try_read_exact(&mut buf, &mut pos, Duration::from_secs(7)).unwrap_err(),
		TimeoutIoError::TimedOut
	)
}


#[test]
fn test_read_until_ok() {
	let (mut s0, s1) = socket_pair();
	write_delayed(
		s1.try_clone().unwrap(), b"Test",
		Duration::from_secs(1)
	);
	write_delayed(
		s1.try_clone().unwrap(), b"o",
		Duration::from_secs(3)
	);
	write_delayed(
		s1.try_clone().unwrap(), b"lope\n!",
		Duration::from_secs(5)
	);
	
	let (mut buf, mut pos) = ([0u8; 4096], 0);
	assert!(s0.try_read_until(
		&mut buf, &mut pos, b"\n",
		Duration::from_secs(7)
	).unwrap());
	assert_eq!(&buf[..pos], b"Testolope\n");
}
#[test]
fn test_read_until_not_found() {
	let (mut s0, s1) = socket_pair();
	write_delayed(
		s1.try_clone().unwrap(), b"Testolope",
		Duration::from_secs(1)
	);
	write_delayed(s1, b"!", Duration::from_secs(4));
	
	let (mut buf, mut pos) = ([0u8; 10], 0);
	assert!(!s0.try_read_until(
		&mut buf, &mut pos,
		b"\n", Duration::from_secs(7))
	.unwrap())
}
#[test]
fn test_read_until_err() {
	let (mut s0, s1) = socket_pair();
	write_delayed(
		s1.try_clone().unwrap(), b"Testolope",
		Duration::from_secs(1)
	);
	write_delayed(s1, b"!", Duration::from_secs(4));
	
	let (mut buf, mut pos) = ([0u8; 4096], 0);
	assert_eq!(s0.try_read_until(
		&mut buf, &mut pos,
		b"\n", Duration::from_secs(7)
	).unwrap_err(), TimeoutIoError::UnexpectedEof)
}
#[test]
fn test_read_until_timeout() {
	let (mut s0, s1) = socket_pair();
	write_delayed(
		s1.try_clone().unwrap(), b"Testolope",
		Duration::from_secs(1)
	);
	write_delayed(
		s1.try_clone().unwrap(), b"!",
		Duration::from_secs(4)
	);
	
	let (mut buf, mut pos) = ([0u8; 4096], 0);
	assert_eq!(s0.try_read_until(
		&mut buf, &mut pos,
		b"\n", Duration::from_secs(7)
	).unwrap_err(), TimeoutIoError::TimedOut)
}