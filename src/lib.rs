//! OParl file cache
//!
//! Downloads the contents of an OParl API into a file based cache, allowing easy retrieval and
//! incremental cache updates
//!
//! Note that there is a cli so you can `cargo run` this project. See `bin/main.rs` or the main
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
extern crate reqwest;
extern crate chrono;
extern crate crossbeam;

pub mod file_storage;
pub mod cacher;

mod storage;
mod server;
mod external_list;

pub use file_storage::FileStorage;
pub use storage::Storage;
pub use server::{Server, CommonServer};
pub use external_list::ExternalList;
pub use cacher::{Cacher};



