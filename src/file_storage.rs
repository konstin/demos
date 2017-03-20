use std::io::Read;
use std::path::{Path, PathBuf};
use std::fs::{File, create_dir_all};
use std::error::Error;

use json;
use json::JsonValue;
use hyper::Url;
use hyper::client::IntoUrl;
use hyper::error::ParseError;

use constants::FILE_EXTENSION;
use cacher::Cacher;
use server::Server;

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

    /// Takes an `url` and returns the corresponding cache path in the form
    /// <cachedir>/<scheme>[:<host>][:<port>][/<path>]<suffix>
    ///
    /// Returns an error if the given url is not a valid url
    pub fn url_to_path(&self, url: &Url, suffix: &str) -> Result<PathBuf, ParseError> {
        // Remove the oparl filters
        // Those parameters shouldn't be parsed on anyway, but just in case we'll do this
        let url_binding: Url = url.clone();
        let query_without_filters = url_binding.query_pairs()
            .filter(|&(ref arg_name, _)| arg_name != "modified_until")
            .filter(|&(ref arg_name, _)| arg_name != "modified_since")
            .filter(|&(ref arg_name, _)| arg_name != "created_since")
            .filter(|&(ref arg_name, _)| arg_name != "created_until");

        let mut url_clone = url.clone();
        let url: &mut Url = url_clone.query_pairs_mut()
            .clear()
            .extend_pairs(query_without_filters)
            .finish();

        // Compute the path
        // Folder
        let mut cachefile = self.cache_dir.to_string().clone();
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
