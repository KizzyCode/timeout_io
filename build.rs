extern crate cc;

fn main() {
	// Select the version according to the current platform
	let file = match true {
		_ if cfg!(unix) => "libselect/libselect_unix.c",
		_ if cfg!(windows) => "libselect/libselect_win.c",
		_ => panic!("Unsupported platform for libselect")
	};
	
	// Compile and link library
	cc::Build::new()
		.static_flag(true)
		.extra_warnings(true)
		.warnings_into_errors(true)
		.file(file).compile("select");
	println!("cargo:rustc-link-lib=static=select");
}