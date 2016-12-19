//! OParl file cache
//!
//! Downloads the contents of an OParl API into a file based cache, allowing easy retrieval and
//! incremental cache updates
//!
//! # Usage
//! Create an instance of the `OParlCache` struct with the url of the OParl entrypoint. Use the
//! `load_to_cache()` method to download the contents of the API. You can the retrieve objects using
//! `get_from_cache(id)`. Update by calling `load_to_cache()`. Note that embedded objects are
//! stripped out and replaced by their id, by which you can retrieve them.
//!
//! Note that there is a CLI so you can `cargo run` this project. See `bin/main.rs` for more details
//!
//! # Implementation
//! The cache folder contains a file called "cache_status.json" with an entry for each known OParl
//! server. An entry contains the external lists of the server with the date of last update of that
//! list. All OParl entities are stored in a file in the cache folder whose path is a reformatted
//! version of the url. For external lists only the ids of the elements are stored.
//!
//! # Examples
//!
//! ```rust
//! use oparl_cache::OParlCache;
//!
//! let cache = OParlCache::new(
//!     "http://localhost:8080/oparl/v1.0/",
//!     "/home/konsti/oparl/schema/",
//!     "/home/konsti/cache-rust/"
//! );
//! cache.load_to_cache();
//! ```

#[macro_use] extern crate json;
extern crate hyper;
extern crate chrono;

use std::io::Read;
use std::path::{Path, PathBuf};
use std::fs::{File, create_dir_all};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;

use chrono::{Local};
use json::JsonValue;
use hyper::Url;
use hyper::client::IntoUrl;

mod external_list;

use external_list::*;

#[cfg(test)]
mod test;

/// Abstracts the access to the cache for one oparl server
pub struct OParlCache<'a> {
    entrypoint: &'a str,
    schema: JsonValue,
    cache_dir: &'a str,
}

impl<'a> OParlCache<'a> {
    pub fn new(entrypoint: &'a str, schema_dir: &'a str, cache_dir: &'a str) -> OParlCache<'a> {
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
        OParlCache { entrypoint: entrypoint, schema: schema, cache_dir: cache_dir }
    }

    fn add_external_list(&self, url: String, last_update: Option<String>,
                         external_list_adder: &Arc<Mutex<Vec<(String, Option<String>)>>>) {
        let mut external_list_adder = external_list_adder.lock().unwrap();
        if external_list_adder.iter().all(|i| url != i.0) {
            println!("Adding External List: {}", url);
            external_list_adder.push((url, last_update));
        } else {
            println!("External List {} already known", url)
        }
    }

    /// Takes an `url` as string and returns the corresponding cache path
    /// <cachedir>/<scheme>[:<host>][:<port>][/<path>]<suffix>
    pub fn url_to_path<U: IntoUrl>(&self, url: U, suffix: &str) -> PathBuf {
        let mut url: Url = url.into_url().unwrap();

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

        Path::new(&cachefile).to_path_buf()
    }

    /// Writes JSON to the path corresponding with the url. This will be an object and its id in the
    /// most cases
    fn write_to_cache<U: IntoUrl>(&self, url: U, object: &JsonValue) {
        let filepath = self.url_to_path(url, ".json");
        println!("Writen to Cache: {}", filepath.display());

        create_dir_all(filepath.parent().unwrap()).unwrap();
        let mut file: File = File::create(filepath).unwrap();

        object.write_pretty(&mut file, 4).unwrap();
    }

    /// Parses the data of a single attribute of an object recursively and replaces embedded objects
    /// by the id. The embedded objects are them parsed by themselves
    fn parse_entry(&self, key: &str, entry: &mut JsonValue, entry_def: &JsonValue,
                   external_list_adder: &Arc<Mutex<Vec<(String, Option<String>)>>>) {
        if entry_def["type"] == "array" {
            for mut i in entry.members_mut() {
                let key = key.to_string() + "[" + &i.to_string() + "]";
                self.parse_entry(key.as_str(), &mut i, &entry_def["items"], &external_list_adder);
            }
        } else if entry_def["type"] == "object" {
            if entry["type"] == "Feature" {
                return; // GeoJson is treated is a single value
            }
            // Extract the embedded object leaving its id
            self.parse_object(entry, &external_list_adder);
            *entry = JsonValue::String(entry["id"].to_string());
            /*if external_list_adder.iter().all(|i| entry != &i.0) {
                self.add_external_list(entry.to_string(), None, external_list_adder);
            }*/
        } else if entry_def["references"] == "externalList" {
            if external_list_adder.lock().unwrap().iter().all(|i| entry != &i.0) {
                self.add_external_list(entry.to_string(), None, &external_list_adder);
            }
        }
    }

    /// Determines the corresponding schema of an object, lets all it's attributes be parsed
    /// recursively and then writes the object to the cache
    fn parse_object(&self, target: &mut JsonValue,
                    external_list_adder: &Arc<Mutex<Vec<(String, Option<String>)>>>) {
        let let_binding = target["type"].to_string();
        let oparl_type = let_binding.split("/").last().unwrap();
        let spec_for_object = &self.schema[oparl_type]["properties"];

        for (key, mut value) in target.entries_mut() {
            // Check if the key is defined in the specification
            if spec_for_object.entries().map(|(key, _)| key).any(|i| i == key) {
                self.parse_entry(key, &mut value, &spec_for_object[key], &external_list_adder);
            }
        }

        self.write_to_cache(target["id"].as_str().unwrap(), &target)
    }

    /// Downloads a whole external list and saves the results to the cache
    /// If `last_sync` is given, the filter modified_since will be appended to the url
    /// `external_list_adder` allows adding external lists that were found when parsing this one
    pub fn parse_external_list<U: IntoUrl + Copy>(&self, url: U, last_sync: Option<String>,
                                                  external_list_adder: &Arc<Mutex<Vec<(String, Option<String>)>>>) {
        // Taake the time before the downloading as the data can change while obtaining pages
        let this_sync = Local::now().format("%Y-%m-%dT%H:%M:%S%Z").to_string();

        let limit: Option<usize> = None;
        let mut url_with_filters: Url = url.into_url().unwrap();

        if let Some(last_sync_time) = last_sync {
            // Add the modified_since filter
            url_with_filters.query_pairs_mut().append_pair("modified_since", &last_sync_time).finish();
        }

        let elist = ExternalList::new(url_with_filters.to_string());

        let mut urls = Vec::new();

        if let Some(limeter) = limit {
            // Don't
            for mut i in elist.take(limeter) {
                self.parse_object(&mut i, &external_list_adder);
                urls.push(i["id"].to_string());
            }
        } else {
            for mut i in elist {
                self.parse_object(&mut i, &external_list_adder);
                urls.push(i["id"].to_string());
            }
        }

        let mut old_urls = Vec::new();
        urls.append(&mut old_urls);

        // Get the urls that have already been retrieved when not using a modified_since
        let old_urls_filepath = self.url_to_path(url, ".json");
        let mut urls_as_json = {
            if old_urls_filepath.exists() {
                let mut old_urls_file = File::open(&old_urls_filepath).unwrap();
                let mut read = String::new();
                old_urls_file.read_to_string(&mut read).unwrap();
                json::parse(&read).unwrap()
            } else {
                JsonValue::new_array()
            }
        };

        for i in urls {
            urls_as_json.push(i).unwrap();
        }
        self.write_to_cache(url, &urls_as_json);

        // self.external_lists_lock.acquire()
        for i in external_list_adder.lock().unwrap().iter_mut() {
            if i.0 == url.into_url().unwrap().as_str() {
                i.1 = Some(this_sync);
                break;
            }
        };

        // save();
        // self.external_lists_lock.release()
    }

    /// Loads the whole API to the cache or updates an existing cache
    pub fn load_to_cache(&self) {
        let entrypoint: Url = Url::from_str(self.entrypoint).unwrap();
        let cache_status_filepath = self.url_to_path(entrypoint.as_str(), "").join("cache-status.json");
        let external_list_adder: Arc<Mutex<Vec<(String, Option<String>)>>>;

        // Initialise external_list_adder
        if cache_status_filepath.exists() {
            // We have a cache, so let's load it
            println!("Cache found, updating...");
            let mut cache_status_file = File::open(&cache_status_filepath).unwrap();
            let mut read = String::new();
            cache_status_file.read_to_string(&mut read).unwrap();
            let known_external_lists = json::parse(&read).unwrap().members()
                .map(|i| (i["url"].to_string(), Some(i["last_sync"].to_string())))
                .collect::<Vec<(String, Option<String>)>>();

            println!("External lists found in cache: {}", known_external_lists.len());
            external_list_adder = Arc::new(Mutex::new(known_external_lists));
        } else {
            // We don't have a cache, so let's use an empty template
            println!("No cache found, initializing...");
            create_dir_all(cache_status_filepath.parent().unwrap()).unwrap();
            external_list_adder = Arc::new(Mutex::new(Vec::new()));
        }

        println!("\nLoaded from cache:");
        for i in external_list_adder.lock().unwrap().iter() {
            println!("{}: {}", i.1.clone().unwrap_or("None".to_string()), i.0);
        }
        println!("");

        // Download the entrypoint which is the System object
        // This will set the first external list, which is the body list
        let mut system_object = download_json(entrypoint).unwrap();
        self.parse_object(&mut system_object, &external_list_adder.clone());

        //let mut threads = Vec::new();

        // Download and cache all external lists while adding those newly found
        // The weird command order is due to the Mutex-locking
        let mut i = 0;
        loop {
            let url;
            let last_update;
            {
                let external_list_adder = external_list_adder.lock().unwrap();
                if i >= external_list_adder.len() {
                    break;
                }

                url = external_list_adder[i].0.clone();
                last_update = external_list_adder[i].1.clone();
            }
            //let thread = thread::spawn(|| {
                self.parse_external_list(&url, last_update, &external_list_adder.clone());
            //});

            //threads.push(thread);
            i += 1;
        }


        // Write the results back to the cache
        let mut cache_status_file: File = File::create(&cache_status_filepath).unwrap();
        let mut cache_status_json = JsonValue::new_array();

        {
            let external_list_adder = external_list_adder.lock().unwrap();

            for i in 0..external_list_adder.len() {
                cache_status_json.push(object! {
                    "url" => JsonValue::from(external_list_adder[i].0.clone()),
                    "last_sync" => JsonValue::from(external_list_adder[i].1.clone())
                }).unwrap();
            }
        }

        cache_status_json.write_pretty(&mut cache_status_file, 4).unwrap();
    }

    /// Retrieves a stored api response from the cache. Returns an io::Error if the was an error
    /// reading the cache file
    pub fn retrieve_from_cache<U: IntoUrl>(&self, url: U) -> Result<JsonValue, std::io::Error> {
        let path = self.url_to_path(url, ".json");
        let mut s = String::new();
        let mut file: File = File::open(path)?;
        file.read_to_string(&mut s)?;
        let json = json::from(s.as_str());
        Ok(json)
    }
}

