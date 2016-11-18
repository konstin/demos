#[macro_use] extern crate json;
extern crate hyper;
extern crate chrono;

use std::io::Read;
use std::path::{Path, PathBuf};
use std::fs::{File, create_dir_all};

use chrono::*;
use json::JsonValue;
use hyper::Url;
use hyper::client::IntoUrl;

#[cfg(test)]
mod test;

fn download_json<U: IntoUrl + Clone>(url: U) -> Option<JsonValue> {
    let client = hyper::Client::new();
    let mut res = client.get(url.clone()).send().unwrap();
    println!("{:?}", url.into_url());
    assert_eq! (res.status, hyper::Ok);
    let mut json_string = String::new();
    res.read_to_string(&mut json_string).unwrap();
    Some(json::parse(&json_string).unwrap())
}

#[derive(Debug)]
pub struct ExternalList {
    url: String,
    response: Option<JsonValue>,
}

impl ExternalList {
    pub fn new(url: String) -> ExternalList {
        ExternalList { url: url, response: None }
    }
}

impl Iterator for ExternalList {
    type Item = json::JsonValue;

    fn next(&mut self) -> Option<json::JsonValue> {
        if self.response.is_none() {
            self.response = download_json(&self.url)
        }

        let mut load_required = false;
        if let Some(ref mut response_ref) = self.response {
            if response_ref["data"].len() == 0 {
                if response_ref["links"].entries().any(|(x, _)| x == "next") {
                    self.url = response_ref["links"]["next"].to_string();
                    load_required = true;
                } else {
                    return None;
                }
            }
        }

        if load_required {
            self.response = download_json(&self.url)
        }

        // This unwrap can't fail as download_url will set response to some
        if let Some(ref mut response_ref) = self.response {
            if let JsonValue::Array(ref mut data) = response_ref["data"] {
                return Some(data.remove(0));
            } else {
                println!("Broken 1");
                return None;
            }
        } else {
            println!("Broken 2");
            return None;
        }
    }
}

#[derive(Debug)]
pub struct OParlCache {
    external_list_data: Vec<(String, Option<String>)>,
    external_list_worker: Vec<String>,
}

impl OParlCache {
    pub fn new() -> OParlCache {
        OParlCache { external_list_data: Vec::new(), external_list_worker: Vec::new() }
    }

    fn add_external_list(&mut self, url: String, last_update: Option<String>) {
        println!("Adding External List: {}", url);
        self.external_list_data.push((url, last_update));
    }

    /// Takes an `url` as string and returns the corresponding cache path
    /// <cachedir>/<scheme>[:<host>][:<port>][/<path>].json
    pub fn url_to_path<U: IntoUrl>(&self, url: U, suffix: &str) -> PathBuf {
        let cachefolder = "/home/konsti/cache-rust/";
        let mut url: Url = url.into_url().unwrap();

        // Remove the oparl filters
        // Those parameters shouldn't be parsed on anyway, but just in case we'll do this
        let url_binding: Url = url.clone();
        let query_without_filters = url_binding.query_pairs()
            .filter(|&(ref x, _)| x != "modified_until")
            .filter(|&(ref x, _)| x != "modified_since")
            .filter(|&(ref x, _)| x != "created_since")
            .filter(|&(ref x, _)| x != "created_until");

        let url: &mut Url = url.query_pairs_mut()
            .clear()
            .extend_pairs(query_without_filters)
            .finish();

        // Compute the path
        // Folder
        let mut cachefile = cachefolder.to_string();
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

    fn write_to_cache<U: IntoUrl>(&self, url: U, object: &JsonValue) {
        let filepath = self.url_to_path(url, ".json");
        println!("Writen to Cache: {}", filepath.display());

        create_dir_all(filepath.parent().unwrap()).unwrap();
        let mut file: File = File::create(filepath).unwrap();

        object.write_pretty(&mut file, 4).unwrap();
    }

    fn parse_entry(&mut self, key: &str, entry: &mut JsonValue, entry_def: &JsonValue) {
        if entry_def["type"] == "array" {
            for mut i in entry.members_mut() {
                let key = key.to_string() + "[" + &i.to_string() + "]";
                self.parse_entry(key.as_str(), &mut i, &entry_def["items"]);
            }
        } else if entry_def["type"] == "object" {
            if entry["type"] == "Feature" {
                return; // GeoJson is treated is a single value
            }
            // Extract the embedded object leaving its id
            self.parse_object(entry);
            *entry = JsonValue::String(entry["id"].to_string());
        } else if entry_def["references"] == "externalList" {
            if self.external_list_data.iter().all(|i| entry != &i.0) {
                self.add_external_list(entry.to_string(), None);
            }
        }
    }

    fn parse_object(&mut self, target: &mut JsonValue) {
        // Load the schema
        let mut schema = JsonValue::new_array();
        for i in Path::new("/home/konsti/oparl/schema/").read_dir().unwrap() {
            let mut f: File = File::open(i.unwrap().path()).unwrap();
            let mut s = String::new();
            f.read_to_string(&mut s).unwrap();
            let x = json::parse(&s).unwrap();
            let y = x["title"].to_string();
            schema[y] = x;
        }

        assert_eq!(schema.len(), 12);

        let let_binding = target["type"].to_string();
        let oparl_type = let_binding.split("/").last().unwrap();
        let spec_for_object = &schema[oparl_type]["properties"];

        for (key, mut value) in target.entries_mut() {
            // Check if the key is defined in the specification
            if spec_for_object.entries().map(|(key, _)| key).any(|i| i == key) {
                self.parse_entry(key, &mut value, &spec_for_object[key]);
            }
        }

        self.write_to_cache(target["id"].as_str().unwrap(), &target)
    }

    pub fn parse_external_list<U: IntoUrl + Copy>(&mut self, url: U, last_sync: Option<String>) {
        let this_sync = Local::now().format("%Y-%m-%dT%H:%M:%S%Z").to_string();

        let limit: Option<usize> = None;
        let mut url_only_new: Url = url.into_url().unwrap();

        if let Some(last_sync_time) = last_sync {
            // Add the modified_since filter
            url_only_new.query_pairs_mut().append_pair("modified_since", &last_sync_time).finish();
        }

        let elist = ExternalList::new(url_only_new.to_string());

        let mut urls = Vec::new();

        if let Some(limeter) = limit {
            for mut i in elist.take(limeter) {
                self.parse_object(&mut i);
                urls.push(i["id"].to_string());
            }
        } else {
            for mut i in elist {
                self.parse_object(&mut i);
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
        for i in self.external_list_data.iter_mut() {
            if i.0 == url.into_url().unwrap().as_str() {
                i.1 = Some(this_sync);
                break;
            }
        };

        // save();
        // self.external_lists_lock.release()
    }

    pub fn load_to_cache<U: IntoUrl>(&mut self, entrypoint: U) {
        let entrypoint: Url = entrypoint.into_url().unwrap();
        let cache_status_filepath = self.url_to_path(entrypoint.as_str(), "").join("cache-status.json");

        if cache_status_filepath.exists() {
            // We have a cache, so let's load it
            println!("Cache found, updating...");
            let mut cache_status_file = File::open(&cache_status_filepath).unwrap();
            let mut read = String::new();
            cache_status_file.read_to_string(&mut read).unwrap();
            self.external_list_data = json::parse(&read).unwrap().members()
                .map(|i| (i["url"].to_string(), Some(i["last_sync"].to_string())))
                .collect();

            println!("External lists found in cache: {}", self.external_list_data.len())
        } else {
            // We don't have a cache, so let's use an empty template
            println!("No cache found, initializing...");
            create_dir_all(cache_status_filepath.parent().unwrap()).unwrap();
            self.external_list_data = Vec::new();
        }

        println!("\nLoaded from cache:");
        for i in self.external_list_data.iter() {
            println!("{}: {}", i.1.clone().unwrap_or("None".to_string()), i.0);
        }
        println!("");

        // Download the entrypoint which is the System object
        // This will set the first external list, which is the body list
        let mut system_object = download_json(entrypoint).unwrap();
        self.parse_object(&mut system_object);

        // Download and cache all external lists while adding those newly found
        let mut i = 0;
        while i < self.external_list_data.len() {
            let ref x = self.external_list_data[i].0.clone();
            let y = self.external_list_data[i].1.clone();
            self.parse_external_list(x, y);
            i += 1;
        }

        // Write the results back to the cache
        let mut cache_status_file: File = File::create(&cache_status_filepath).unwrap();
        let mut cache_status_json = JsonValue::new_array();
        for i in 0..self.external_list_data.len() {
            cache_status_json.push(JsonValue::new_object());
            cache_status_json[i]["url"] = JsonValue::from(self.external_list_data[i].0.clone());
            cache_status_json[i]["last_sync"] = JsonValue::from(self.external_list_data[i].1.clone());
        }
        cache_status_json.write(&mut cache_status_file).unwrap();
    }
}

