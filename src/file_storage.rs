use std::io::Read;
use std::path::{Path, PathBuf};
use std::fs::{File, create_dir_all};
use std::error::Error;

use json;
use json::JsonValue;
use reqwest::Url;
use reqwest::IntoUrl;

use constants::FILE_EXTENSION;
use cacher::Cacher;
use server::Server;
use storage::Storage;

/// A Storage where every object becomes a file under a specified folder
#[derive(Clone)]
pub struct FileStorage<'a> {
    schema: JsonValue,
    cache_dir: &'a str,
    cache_status_file: &'a str
}

impl<'a> Storage for FileStorage<'a> {
    /// Writes JSON to the path corresponding with the url. This will be an object and its id in the
    /// most cases
    fn write_to_cache(&self, url: &Url, object: &JsonValue) -> Result<(), Box<Error>> {
        let filepath = self.url_to_path(url, FILE_EXTENSION);
        println!("Writen to Cache: {}", filepath.display());

        create_dir_all(filepath.parent().ok_or("Invalid cachepath for file")?)?;
        let mut file: File = File::create(filepath)?;

        object.write_pretty(&mut file, 4)?;
        Ok(())
    }

    /// Retrieves a stored cached object from the cache.
    ///
    /// Returns a boxed error if there was an error reading the cache file
    fn get(&self, url: &Url) -> Result<JsonValue, Box<Error>> {
        let path = self.url_to_path(&url, FILE_EXTENSION);
        let mut s = String::new();
        let mut file: File = File::open(path)?;
        file.read_to_string(&mut s)?;
        let json = json::parse(s.as_str())?;
        Ok(json)
    }

    /// Returns `schema`
    fn get_schema(&self) -> &JsonValue {
        &self.schema
    }
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
    pub fn url_to_path(&self, url: &Url, suffix: &str) -> PathBuf {
        // Remove the oparl filters
        // Those parameters shouldn't be on any object, but it's better to sanitize
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

        // Assemble the actual path
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

        Path::new(&cachefile).to_path_buf()
    }
}

impl<'a> Cacher for FileStorage<'a> {
    /// Loads the whole API to the cache or updates an existing cache
    /// This function does only do the loading saving and forwards the actual work
    fn cache<U: Server>(&self, server: U) -> Result<(), Box<Error>> {
        let cache_status_filepath = self.url_to_path(&server.get_entrypoint().clone(), "")
            .join(self.get_cache_status_file());
        println!("{}", &cache_status_filepath.display());
        let mut known_lists: Vec<(Url, Option<String>)>;

        if cache_status_filepath.exists() {
            // We have a cache, so let's load it
            println!("Cache found, updating...");
            let mut cache_status_file = File::open(&cache_status_filepath)?;
            let mut read = String::new();
            cache_status_file.read_to_string(&mut read)?;
            known_lists = vec![];
            for i in json::parse(&read)?.members() {
                known_lists.push((
                    i["url"].as_str().ok_or("invalid cache status file")?.into_url()?,
                    Some(i["last_sync"].to_string())
                ));
            }
            println!("External lists found in cache: {}", known_lists.len());
        } else {
            // We don't have a cache, so let's use an empty template
            println!("No cache found, initializing...");
            let err = "Could not create directory for the cache status file";
            create_dir_all(cache_status_filepath.parent().ok_or(err)?)?;
            known_lists = Vec::new();
        }

        println!("\nLoaded from cache:");
        for i in known_lists.iter() {
            println!("{}: {}", i.1.clone().unwrap_or("None".to_string()), i.0);
        }
        println!();

        // Write the results back to the cache
        let mut cache_status_file: File = File::create(&cache_status_filepath)?;
        let mut cache_status_json = JsonValue::new_array();

        // Here the actual work is done
        let mut new_cache_status = self.load_all_external_lists(server, &known_lists);
        for i in new_cache_status.drain(..) {
            cache_status_json.push(object! {
                "url" => JsonValue::from(i.0.to_string()),
                "last_sync" => JsonValue::from(i.1)
            })?;
        }

        cache_status_json.write_pretty(&mut cache_status_file, 4)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ::test::storage;
    use constants::FILE_EXTENSION;

    #[test]
    fn parse_object_extract_internal() {
        let mut input = object! {
            "id" => "http://localhost:8080/oparl/v1.0/paper/2",
            "type" => "https://schema.oparl.org/1.0/Paper",
            "auxiliaryFile" => array![
                object!{
                  "id" => "http://localhost:8080/oparl/v1.0/file/2",
                  "type" => "https://schema.oparl.org/1.0/File",
                  "accessUrl" => "http://localhost:8080/fileaccess/access/2",
                  "created" => "2016-05-02T19:53:08+02:00",
                  "modified" => "2016-05-02T19:53:08+02:00"
                }
            ],
            "created" => "2016-05-02T00:00:00+02:00",
            "modified" => "2016-05-02T00:00:00+02:00"
        };
        let expected_output = object! {
            "id" => "http://localhost:8080/oparl/v1.0/paper/2",
            "type" => "https://schema.oparl.org/1.0/Paper",
            "auxiliaryFile" => array! [
                "http://localhost:8080/oparl/v1.0/file/2"
            ],
            "created" => "2016-05-02T00:00:00+02:00",
            "modified" => "2016-05-02T00:00:00+02:00"
        };
        let (add_list, receive_list) = channel();
        storage().parse_object(&mut input, &add_list);
        assert_eq!(input, expected_output);
        assert_eq!(receive_list.try_recv().is_err(), true);
    }

    #[test]
    fn parse_object_find_external_list() {
        let mut input = object! {
            "id" => "http://localhost:8080/oparl/v1.0/body/0",
            "type" => "https://schema.oparl.org/1.0/Body",
            "legislativeTerm" => array! [
                object! {
                    "id" => "http://localhost:8080/oparl/v1.0/legislativeterm/0",
                    "type" => "https://schema.oparl.org/1.0/LegislativeTerm",
                    "name" => "Unbekannt"
                }
            ],
            "organization" => "http://localhost:8080/oparl/v1.0/body/0/list/organization",
            "person" => "http://localhost:8080/oparl/v1.0/body/0/list/person",
            "meeting" => "http://localhost:8080/oparl/v1.0/body/0/list/meeting",
            "paper" => "http://localhost:8080/oparl/v1.0/body/0/list/paper",
            "web" => "http://localhost:8080/",
            "created" => "2016-09-29T14:31:50+02:00",
            "modified" => "2016-09-29T14:42:52+02:00"
        };

        // Create a deep copy and replace the embedded object by its id
        let mut expected_output = json::parse(&input.dump()).unwrap();
        expected_output["legislativeTerm"][0] = expected_output["legislativeTerm"][0]["id"].take();
        let (add_list, receive_list) = channel();
        storage().parse_object(&mut input, &add_list);

        assert_eq!(input, expected_output);
        assert_eq!(receive_list.iter().collect(), vec![
            ("http://localhost:8080/oparl/v1.0/body/0/list/organization".to_string(), None),
            ("http://localhost:8080/oparl/v1.0/body/0/list/person".to_string(), None),
            ("http://localhost:8080/oparl/v1.0/body/0/list/meeting".to_string(), None),
            ("http://localhost:8080/oparl/v1.0/body/0/list/paper".to_string(), None),
        ]);
    }

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
