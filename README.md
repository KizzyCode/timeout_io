[![License](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Travis CI](https://travis-ci.org/KizzyCode/timeout_io.svg?branch=master)](https://travis-ci.org/KizzyCode/timeout_io)

# About
This library provides a simple timeout-based API for IO-operations.

We provide the following features:
 - DNS-resolution (currently uses a background-thread)
 - TCP-accept (uses libselect)
 - TCP-read/read-until/write (uses libselect)
 - UDP-receive/send (uses libselect)

All functions are defined as traits, so that you can easily wrap your own IO-channels without breaking compatibility.

_Note: We currently do not provide a function for timeout-based `connect`-calls; use
`std::net::TcpStream::connect_timeout` for TCP-connections or build sth. using `io::libselect` (and feel free to commit
if you do so 😇)_

# Dependencies
My [`etrace`](https://crates.io/crates/etrace) crate for error handling and
[`tiny_future`](https://crates.io/crates/tiny_future) which is used during name resolution (which needs to be done in a
separate thread due to API limitations).

# Build Library and Documentation
To build the documentation, go into the projects root-directory and run `cargo doc --release`; to open the documentation
in your web-browser, run `cargo doc --open`.

To build the library, go into the projects root-directory and run `cargo build --release`; you can find the build in
target/release.