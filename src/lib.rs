//! OParl file cache
//!
//! Downloads the contents of an OParl API into a file based cache, allowing easy retrieval and
//! incremental cache updates
//!
//! Note that there is a CLI so you can `cargo run` this project. See `bin/main.rs` or the main
//! crate for more information
//!
//! # Usage
//!
//! FIXME
//!
//! # Implementation
//!
//! The cache directory contains a folder for each server, with each of these folder folder
//! containing a file with the last successfull update of an external list. All OParl entities are
//! stored in that folder, while the exact location is a reformatted version of the url.
//! For external lists only the ids of the elements are stored.
//!

//FIXME
//#![warn(missing_docs)]

#[macro_use]
extern crate json;
extern crate hyper;
extern crate chrono;
extern crate crossbeam;

mod file_storage;
mod storage;
mod constants;
mod server;
mod external_list;
mod cacher;

pub use file_storage::*;
pub use storage::*;
pub use constants::*;
pub use server::*;
pub use external_list::*;
pub use cacher::*;

#[cfg(test)]
mod test;


