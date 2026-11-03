//! The codepoint scan command line.
//!
//! A thin shell over the library scan. It takes the tree to scan
//! as its one optional argument and scans the working directory
//! when no argument stands, prints the line the Python scan
//! script printed, and exits with status 0 when the tree is
//! clean and status 1 when a dash stands anywhere in it, as the
//! script did.

use std::path::PathBuf;

fn main() {
    let root = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let counts = stickler::scan::scan_tree(&root);
    println!("{}", stickler::scan::report_line(&counts));
    std::process::exit(if counts.is_clean() { 0 } else { 1 });
}
