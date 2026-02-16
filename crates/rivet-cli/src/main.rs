use std::ffi::OsString;

fn main() {
    let args: Vec<OsString> = std::env::args_os().collect();
    if let Err(err) = rivet_core::run(args) {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}
