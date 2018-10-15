extern crate cc;

fn main() {
	// Select the version according to the current platform
	let (file, flags) = match true {
		_ if cfg!(unix) => ("libselect/libselect_unix.c", ["-std=c99"].as_ref()),
		_ if cfg!(windows) => ("libselect/libselect_win.c", [""].as_ref()),
		_ => panic!("Unsupported platform for libselect")
	};
	
	// Configure builder
	let mut builder = cc::Build::new();
	builder.static_flag(true).extra_warnings(true).warnings_into_errors(true);
	
	// Add platform specific flags
	flags.iter().for_each(|flag| { builder.flag_if_supported(flag); });
	
	// Compile and link library
	builder.file(file).compile("select");
	println!("cargo:rustc-link-lib=static=select");
}