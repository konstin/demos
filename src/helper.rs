use std::path::{Path, PathBuf};

use hyper::Url;
use hyper::error::ParseError;



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