use std::io::Read;
use std::path::{Path, PathBuf};
use std::fs::File;

use json;
use json::JsonValue;
use hyper::Url;
use hyper::client::IntoUrl;
use hyper::error::ParseError;

/// The definition of a cache for one OParl server
pub struct Storage<'a> {
    entrypoint: &'a str,
    schema: JsonValue,
    cache_dir: &'a str,
    cache_status_file: &'a str
}

impl<'a> Storage<'a> {
    pub fn new(entrypoint: &'a str, schema_dir: &'a str, cache_dir: &'a str,
               cache_status_file: &'a str) -> Storage<'a> {
        // Load the schema
        let mut schema = JsonValue::new_array();
        for i in Path::new(schema_dir).read_dir().unwrap() {
            let mut f: File = File::open(i.unwrap().path()).unwrap();
            let mut s = String::new();
            f.read_to_string(&mut s).unwrap();
            let x = json::parse(&s).unwrap();
            let y = x["title"].to_string();
            schema[y] = x;
        }

        assert_eq!(schema.len(), 12);
        Storage {
            entrypoint: entrypoint,
            schema: schema,
            cache_dir: cache_dir,
            cache_status_file: cache_status_file
        }
    }

    pub fn get_entrypoint(&self) -> &'a str {
        self.entrypoint
    }

    pub fn get_schema(&self) -> &JsonValue {
        &self.schema
    }

    pub fn get_cache_dir(&self) -> &'a str {
        self.cache_dir
    }

    pub fn get_cache_status_file(&self) -> &'a str {
        self.cache_status_file
    }

    /// Takes an `url` and returns the corresponding cache path
    /// <cachedir>/<scheme>[:<host>][:<port>][/<path>]<suffix>
    /// Returns an error if the given url is not a valid url
    pub fn url_to_path<U: IntoUrl>(&self, url: U, suffix: &str) -> Result<PathBuf, ParseError>{
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
