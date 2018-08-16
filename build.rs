extern crate gcc;

fn main() {
	// Compile libselect
	let mut gcc = gcc::Build::new();
	
	// Set OS-flag
	let os = if cfg!(unix) { "PLATFORM_UNIX" }
		else if cfg!(windows) { "PLATFORM_WINDOWS" }
		else { panic!("Unsupported platform for libselect") };
	gcc.define(os, None);
	
	// Compile lib
	gcc.file("libselect/libselect.c").compile("select");
	
	// Link lib
	println!("cargo:rustc-link-lib=static=select");
}