#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![warn(unused_results, clippy::must_use_candidate)]

mod dag;
mod screen;
#[cfg(test)]
mod test;

pub use crate::dag::dag_to_text;
pub use crate::dag::ProcessingError;
