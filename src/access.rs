use std::error::Error;
use std::fs::File;
use std::io::Read;

use json;
use json::JsonValue;
use hyper::client::IntoUrl;

use Storage;
use FILE_EXTENSION;

/// Allows accessing the cache of an OParl server
pub trait Access {
    fn get<U: IntoUrl>(&self, url: U) -> Result<JsonValue, Box<Error>>;
}

impl<'a> Access for Storage<'a> {
    /// Retrieves a stored api response from the cache. Returns a boxed error if the url was invalid
    /// or when there was an error reading the cache file
    fn get<U: IntoUrl>(&self, url: U) -> Result<JsonValue, Box<Error>> {
        let path = self.url_to_path(url, FILE_EXTENSION)?;
        let mut s = String::new();
        let mut file: File = File::open(path)?;
        file.read_to_string(&mut s)?;
        let json = json::from(s.as_str());
        Ok(json)
    }
}