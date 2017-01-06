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
//! First, load the server's contents to the cache with an instance of `Storage`. It describes which
//! server should be stored in which directory. As it implements the `Cacher` trait, you can
//! call `load_to_cache` on it:
//!
//! ```rust,no_run
//! use oparl_cache::{Storage, Cacher, Access};
//!
//! let storage = Storage::new(
//!     "https://oparl.example.org/v1",
//!     "/home/username/oparl/schema/",
//!     "/home/username/.cache/oparl",
//!     oparl_cache::DEFAULT_CACHE_STATUS_FILE
//! );
//! storage.load_to_cache().unwrap();
//! ```
//!
//! Now the whole OParl API has been loaded to the cache! Now use the `Access` trait to retrieve
//! objects:
//!
//! ```rust,no_run
//! # use oparl_cache::{Storage, Cacher, Access};
//! #
//! # let storage = Storage::new(
//! #     "https://oparl.example.org/v1",
//! #     "/home/username/oparl/schema/",
//! #     "/home/username/.cache/oparl",
//! #     oparl_cache::DEFAULT_CACHE_STATUS_FILE
//! # );
//! storage.get("https://oparl.example.org/v1/body/0/person/42_douglas_adams");
//! ```
//!
//! If you want an incremental cache update, just do `cacher.load_to_cache()` again.
//!
//! # Implementation
//!
//! The cache directory contains a folder for each server, with each of these folder folder
//! containing a file with the last successfull update of an external list. All OParl entities are
//! stored in that folder, while the exact location is a reformatted version of the url.
//! For external lists only the ids of the elements are stored.
//!

//#![warn(missing_docs)]

#[macro_use]
extern crate json;
extern crate hyper;
extern crate chrono;
extern crate crossbeam;

mod external_list;
mod storage;
mod access;
mod cacher;
mod constants;

pub use external_list::ExternalList;
pub use storage::Storage;
pub use access::Access;
pub use cacher::Cacher;
pub use constants::*;

#[cfg(test)]
mod test;


