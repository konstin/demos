//! OParl file cache
//!
//! Downloads the contents of an OParl API into a file based cache, allowing easy retrieval and
//! incremental cache updates
//!
//! # Usage
//! Create an instance of the `OParlCache` struct with the url of the OParl entrypoint. Use the
//! `load_to_cache()` method to download the contents of the API. You can the retrieve objects using
//! `get_from_cache(id)`. Update by calling `load_to_cache()`. Note that embedded objects are
//! stripped out and replaced by their id, by which you can retrieve them.
//!
//! Note that there is a CLI so you can `cargo run` this project. See `bin/main.rs` for more details
//!
//! # Implementation
//! The cache folder contains a file called "cache_status.json" with an entry for each known OParl
//! server. An entry contains the external lists of the server with the date of last update of that
//! list. All OParl entities are stored in a file in the cache folder whose path is a reformatted
//! version of the url. For external lists only the ids of the elements are stored.
//!
//! # Examples
//!
//! ```rust
//! use oparl_cache::OParlCache;
//!
//! let cache = OParlCache::new(
//!     "http://localhost:8080/oparl/v1.0/",
//!     "/home/konsti/oparl/schema/",
//!     "/home/konsti/cache-rust/",
//!     oparl_cache::DEFAULT_CACHE_STATUS_FILE
//! );
//! cache.load_to_cache();
//! ```

#[macro_use]
extern crate json;
extern crate hyper;
extern crate chrono;
extern crate crossbeam;

mod external_list;
mod oparl_cache;

pub use oparl_cache::OParlCache;
pub use external_list::ExternalList;

#[cfg(test)]
mod test;

pub const DEFAULT_CACHE_STATUS_FILE: &'static str = "cache_status.json";
pub const FILE_EXTENSION: &'static str = ".json";



