use std::error::Error;
use std::fs::File;
use std::io::Read;

use json;
use json::JsonValue;
use hyper::client::IntoUrl;

use Storage;
use FILE_EXTENSION;

pub struct Access<'a> {
    storage: Storage<'a>
}

impl<'a> Access<'a> {
    pub fn new(settings: Storage<'a>) -> Access<'a> {
        Access {storage: settings}
    }

    /// Retrieves a stored api response from the cache. Returns a boxed error if the url was invalid
    /// or when there was an error reading the cache file
    pub fn retrieve_from_cache<U: IntoUrl>(&self, url: U) -> Result<JsonValue, Box<Error>> {
        let path = self.storage.url_to_path(url, FILE_EXTENSION)?;
        let mut s = String::new();
        let mut file: File = File::open(path)?;
        file.read_to_string(&mut s)?;
        let json = json::from(s.as_str());
        Ok(json)
    }
}