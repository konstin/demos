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
//! Currently the only available storage is the FileStorage, which stores the data in a user-defined
//! folder:
//!
//! ```rust,ignore
//! use oparl_cache::{CommonServer, FileStorage, Cacher, IntoUrl};
//! use oparl_cache::file_storage::CACHE_STATUS_FILE;
//! use std::path::Path;
//!
//! let server = CommonServer::new("https://example.com/entrypoint".into_url().unwrap());
//! let storage = FileStorage::new(Path::new("path/to/oparl/schema"),
//!                                Path::new("path/to/cachedir").to_owned()).unwrap();
//! storage.cache(server).unwrap();
//! ```

#![warn(missing_docs)]

#[macro_use]
extern crate json;
extern crate reqwest;
extern crate chrono;
extern crate crossbeam;
extern crate serde;
extern crate serde_json;

/// Contains the FileStorage struct with some associated constants
pub mod file_storage;
/// Contains the Cacher trait with a Message type
pub mod cacher;

mod storage;
mod server;
mod external_list;

pub use file_storage::FileStorage;
pub use storage::Storage;
pub use server::{Server, CommonServer};
pub use external_list::ExternalList;
pub use cacher::{Cacher};

/// Reexported from reqwest
pub use reqwest::IntoUrl;
/// Reexported from reqwest
pub use reqwest::Url;




