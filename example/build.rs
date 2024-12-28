use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

fn get_output_path() -> PathBuf {
	let build_type = env::var("PROFILE").unwrap();
	let path = Path::new("target").join(build_type);
	return PathBuf::from(path);
}

fn main() {
	// println!("cargo::rerun-if-changed=project.json");

	let mut out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

	let _ = out_dir.pop();
	let _ = out_dir.pop();

	let file = Path::new("project.json");

	let file_target = out_dir.join(file);

	println!("cargo:warn={} -> {}", file.display(), file_target.display());

	let src = fs::read(&file).unwrap_or_else(|error| {
		panic!("failed to read file: {}, reason: {}", file.display(), error)
	});

	fs::write(&file_target, &src).unwrap_or_else(|error| {
		panic!(
			"failed to write file: {}, reason: {}",
			file_target.display(),
			error
		)
	});
}
