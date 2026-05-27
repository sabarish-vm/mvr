use std::boxed::Box;
use std::panic;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;

pub(crate) fn panic_hook_fxn() {
    panic::set_hook(Box::new(|info| {
        eprintln!("Error: {}", info);
        exit(1);
    }));
}
