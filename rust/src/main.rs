//! The stickler command line shell. The surface stands in the
//! library, and this shell only runs it.

fn main() {
    std::process::exit(stickler::cli::run());
}
