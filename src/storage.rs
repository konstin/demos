use std::error::Error;

use json::JsonValue;
use reqwest::Url;

/// Defines a storage for saving objects
///
/// An Implementation can be any kind of storage, be it a file storage, a database or even the ram
pub trait Storage {
    /// Caches a servers contents or updates the cache
    fn write_to_cache(&self, url: &Url, object: &JsonValue) -> Result<(), Box<Error>>;
    /// Retrieves a cached object
    fn get(&self, url: &Url) -> Result<JsonValue, Box<Error>>;
    /// Return the schema as one json dict
    /// TODO: Untangle schema and storage
    fn get_schema(&self) -> &JsonValue;
}
