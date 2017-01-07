use std::io::Read;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::error::Error;

use json;
use json::JsonValue;
use hyper::Url;
use hyper::client::IntoUrl;
use hyper::error::ParseError;

/// The mapping of an OParl server to a cache
#[derive(Clone)]
pub struct Storage<'a> {
    entrypoint: Url,
    schema: JsonValue,
    cache_dir: &'a str,
    cache_status_file: &'a str
}

impl<'a> Storage<'a> {
    /// Creates a new `Storage`
    pub fn new<U: IntoUrl>(entrypoint: U, schema_dir: &'a str, cache_dir: &'a str,
               cache_status_file: &'a str) -> Result<Storage<'a>, Box<Error>> {
        // Load the schema
        let mut schema = JsonValue::new_array();
        for i in Path::new(schema_dir).read_dir()? {
            let mut f: File = File::open(i?.path())?;
            let mut s = String::new();
            f.read_to_string(&mut s)?;
            let x = json::parse(&s)?;
            let y = x["title"].to_string();
            schema[y] = x;
        }

        assert_eq!(schema.len(), 12);
        Ok(Storage {
            entrypoint: entrypoint.into_url()?,
            schema: schema,
            cache_dir: cache_dir,
            cache_status_file: cache_status_file
        })
    }

    /// Returns `entrypoint`
    pub fn get_entrypoint(&self) -> &Url {
        &self.entrypoint
    }

    /// Returns `schema`
    pub fn get_schema(&self) -> &JsonValue {
        &self.schema
    }

    /// Returns `cache_dir`
    pub fn get_cache_dir(&self) -> &'a str {
        self.cache_dir
    }

    /// Returns `cache_status_file`
    pub fn get_cache_status_file(&self) -> &'a str {
        self.cache_status_file
    }

    /// Takes an `url` and returns the corresponding cache path
    /// <cachedir>/<scheme>[:<host>][:<port>][/<path>]<suffix>
    /// Returns an error if the given url is not a valid url
    pub fn url_to_path<U: IntoUrl>(&self, url: U, suffix: &str) -> Result<PathBuf, ParseError> {
        let mut url: Url = url.into_url()?;

        // Remove the oparl filters
        // Those parameters shouldn't be parsed on anyway, but just in case we'll do this
        let url_binding: Url = url.clone();
        let query_without_filters = url_binding.query_pairs()
            .filter(|&(ref arg_name, _)| arg_name != "modified_until")
            .filter(|&(ref arg_name, _)| arg_name != "modified_since")
            .filter(|&(ref arg_name, _)| arg_name != "created_since")
            .filter(|&(ref arg_name, _)| arg_name != "created_until");

        let url: &mut Url = url.query_pairs_mut()
            .clear()
            .extend_pairs(query_without_filters)
            .finish();

        // Compute the path
        // Folder
        let mut cachefile = self.cache_dir.to_string();
        // Schema and host
        cachefile += url.scheme();

        // Host
        if let Some(host) = url.host_str() {
            cachefile += ":";
            cachefile += host;
        }

        // Port
        if let Some(port) = url.port() {
            cachefile += ":";
            cachefile += &port.to_string();
        }

        // Path
        let mut path = url.path().to_string();
        if path.ends_with("/") {
            path.pop(); // We have a file here, not a folder, dear url creators
        };
        cachefile += &path;

        // Query
        if let Some(query) = url.query() {
            if query != "" {
                cachefile += "?";
                cachefile += query;
            }
        }

        // File extension
        cachefile += suffix;

        Ok(Path::new(&cachefile).to_path_buf())
    }
}

#[cfg(test)]
mod test {
    use ::test::storage;
    use super::*;

    fn single_url_to_path(url: &str, query_string: &str, path: &str) {
        assert_eq! (storage().url_to_path((url.to_string() + query_string).as_str(), ".json").unwrap(), Path::new(path));
        assert_eq! (storage().url_to_path((url.to_string() + "/" + query_string).as_str(), ".json").unwrap(), Path::new(path));
    }

    #[test]
    fn url_to_path() {
        let cache_status_file = "/home/konsti/cache-rust/http:localhost:8080/oparl/v1.0/cache-status.json";
        assert_eq! (storage().url_to_path("http://localhost:8080/oparl/v1.0", "").unwrap().join("cache-status.json"), Path::new(cache_status_file));
        single_url_to_path("https://example.tld:8080/oparl/v1.0/paper/1", "", "/home/konsti/cache-rust/https:example.tld:8080/oparl/v1.0/paper/1.json");
        single_url_to_path("https://example.tld/oparl/v1.0/paper/1", "", "/home/konsti/cache-rust/https:example.tld/oparl/v1.0/paper/1.json");
        single_url_to_path("https://example.tld/oparl/v1.0", "", "/home/konsti/cache-rust/https:example.tld/oparl/v1.0.json");
        single_url_to_path("https://example.tld", "", "/home/konsti/cache-rust/https:example.tld.json");
        single_url_to_path("https://example.tld/api", "?modified_until=2016-05-03T00%3A00%3A00%2B02%3A00", "/home/konsti/cache-rust/https:example.tld/api.json");
    }
}