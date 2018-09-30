extern crate gcc;

fn main() {
	// Compile libselect
	let mut gcc = gcc::Build::new();
	
	// Compile lib
	match true {
		_ if cfg!(unix) => gcc.file("libselect/libselect_unix.c").compile("select"),
		_ if cfg!(windows) => gcc.file("libselect/libselect_win.c").compile("select"),
		_ => panic!("Unsupported platform for libselect")
	}
	
	// Link lib
	println!("cargo:rustc-link-lib=static=select");
}