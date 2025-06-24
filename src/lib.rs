#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![warn(clippy::must_use_candidate)]
// #![warn(unused_results)]

mod dag;
mod screen;
#[cfg(test)]
mod test;

pub use crate::dag::ProcessingError;
pub use crate::dag::dag_to_text;
#[cfg(feature = "petgraph")]
pub use crate::dag::petgraph_dag_to_text;