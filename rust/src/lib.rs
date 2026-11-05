//! The stickler library. A strict reviewer of code and naming.
//!
//! The library carries the loaders, the checker, the seat readers,
//! the readings, and the renderers as pure functions. The binary
//! carries the command line surface over the library. The Python
//! package under src is the specification this port reproduces.

pub mod census;
pub mod changeset;
pub mod checker;
pub mod cli;
pub mod json;
pub mod pycompat;
pub mod rejudged;
pub mod render;
pub mod review;
pub mod scan;
pub mod sets;
pub mod verdicts;
