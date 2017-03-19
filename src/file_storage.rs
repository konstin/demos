use std::io::Read;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::error::Error;

use json;
use json::JsonValue;
use hyper::Url;
use hyper::client::IntoUrl;
use hyper::error::ParseError;

use helper::url_to_path;
use constants::FILE_EXTENSION;

/// The mapping of an OParl server to a cache
#[derive(Clone)]
pub struct FileStorage<'a> {
    schema: JsonValue,
    cache_dir: &'a str,
    cache_status_file: &'a str
}

impl<'a> FileStorage<'a> {
    /// Creates a new `Storage`
    pub fn new(schema_dir: &'a str, cache_dir: &'a str,
               cache_status_file: &'a str) -> Result<FileStorage<'a>, Box<Error>> {
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

        assert_eq!(schema.len(), 12, "Expected 12 Schema files");
        Ok(FileStorage {
            schema: schema,
            cache_dir: cache_dir,
            cache_status_file: cache_status_file
        })
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
    pub fn url_to_path(&self, url: &Url, suffix: &str) -> Result<PathBuf, ParseError> {
        url_to_path(self.cache_dir.to_string(), url, suffix)
    }

    /// Retrieves a stored api response from the cache. Returns a boxed error if the url was invalid
    /// or when there was an error reading the cache file
    fn get<U: IntoUrl>(&self, url: U) -> Result<JsonValue, Box<Error>> {
        let path = self.url_to_path(&url.into_url()?, FILE_EXTENSION)?;
        let mut s = String::new();
        let mut file: File = File::open(path)?;
        file.read_to_string(&mut s)?;
        let json = json::from(s.as_str());
        Ok(json)
    }
}

