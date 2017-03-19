use std::path::{Path, PathBuf};

use hyper::Url;
use hyper::error::ParseError;

/// Takes an `url` and returns the corresponding cache path in the form
/// <cachedir>/<scheme>[:<host>][:<port>][/<path>]<suffix>
///
/// Returns an error if the given url is not a valid url
pub fn url_to_path(cachedir: String, url: &Url, suffix: &str) -> Result<PathBuf, ParseError> {
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
    let mut cachefile = cachedir.clone();
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

#[cfg(test)]
mod test {
    use ::test::storage;
    use super::*;
    use constants::FILE_EXTENSION;

    fn for_one(url: &str, query_string: &str, path: &str) {
        let x = ((url.to_string() + query_string).as_str(), ".json").into_url().unwrap();
        assert_eq! (url_to_path("/home/konsti/oparl/schema/", &x, FILE_EXTENSION).unwrap(), Path::new(path));
        let y = ((url.to_string() + "/" + query_string).as_str(), ".json").into_url().unwrap();
        assert_eq! (url_to_path("/home/konsti/oparl/schema/", &y, FILE_EXTENSION).unwrap(), Path::new(path));
    }

    #[test]
    fn test_url_to_path() {
        let cache_status_file = "/tmp/cache-rust/http:localhost:8080/oparl/v1.0/cache-status.json";
        assert_eq! (url_to_path("/home/konsti/oparl/schema/", &"http://localhost:8080/oparl/v1.0".into_url().unwrap(), "").unwrap().join("cache-status.json"), Path::new(cache_status_file));
        for_one("https://example.tld:8080/oparl/v1.0/paper/1", "", "/tmp/cache-rust/https:example.tld:8080/oparl/v1.0/paper/1.json");
        for_one("https://example.tld/oparl/v1.0/paper/1", "", "/tmp/cache-rust/https:example.tld/oparl/v1.0/paper/1.json");
        for_one("https://example.tld/oparl/v1.0", "", "/tmp/cache-rust/https:example.tld/oparl/v1.0.json");
        for_one("https://example.tld", "", "/tmp/cache-rust/https:example.tld.json");
        for_one("https://example.tld/api", "?modified_until=2016-05-03T00%3A00%3A00%2B02%3A00", "/tmp/cache-rust/https:example.tld/api.json");
    }
}