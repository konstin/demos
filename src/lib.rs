#[macro_use] extern crate json;
extern crate hyper;
extern crate chrono;

use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::fs::{File, create_dir_all};

use chrono::*;
use json::JsonValue;
use hyper::Url;

#[cfg(test)]
mod test;

pub struct ExternalList {
    url: String,
    response: Option<JsonValue>,
}

impl ExternalList {
    pub fn new(url: String) -> ExternalList {
        ExternalList { url: url, response: None }
    }
}

impl ExternalList {
    fn download_url(&mut self) -> Option<JsonValue> {
        let client = hyper::Client::new();
        let mut res = client.get(&self.url).send().unwrap();
        assert_eq!(res.status, hyper::Ok);
        let mut json_string = String::new();
        res.read_to_string(&mut json_string).unwrap();
        Some(json::parse(&json_string).unwrap())
    }
}

impl Iterator for ExternalList {
    type Item = json::JsonValue;

    fn next(&mut self) -> Option<json::JsonValue> {
        if self.response.is_none() {
            self.response = self.download_url()
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
            self.response = self.download_url();
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

fn add_external_list(url: &str) {
    println!("Adding External List: {}", url)
}

pub fn url_to_path(url: &str) -> PathBuf {
    let cachefolder = "/home/konsti/cache-rust/";
    let mut url: Url = Url::from_str(url).unwrap();

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
    cachefile += ".json";

    Path::new(&cachefile).to_path_buf()
}

fn write_to_cache(url: &str, object: &JsonValue) {
    let filepath = url_to_path(url);
    println!("Writen to Cache: {}", filepath.display());

    create_dir_all(filepath.parent().unwrap()).unwrap();
    let mut file: File = File::create(filepath).unwrap();

    object.write_pretty(&mut file, 4).unwrap();
}

fn parse_entry(key: String, entry: &mut JsonValue, entry_def: &JsonValue) {
    let external_lists: Vec<JsonValue> = Vec::new();

    if entry_def["type"] == "array" {
        for mut i in entry.members_mut() {
            parse_entry(String::from(key.as_str()) + "[" + &i.to_string() + "]", &mut i, &entry_def["items"]);
        }
    } else if entry_def["type"] == "object" {
        if entry["type"] == "Feature" {
            return; // GeoJson is treated is a single value
        }
        // Extract the embedded object leaving its id
        parse_object(entry);
        *entry = JsonValue::String(entry["id"].to_string());
    } else if entry_def.members().any(|i| i == "entrydef") && entry_def["references"] == "externalList" {
        if !external_lists.iter().map(|i| i["url"].to_string()).any(|x| entry == &x) {
            add_external_list(&format!("{{\"url\": \"{}\", \"last_update\": null", entry));
        }
    }
}


fn parse_object(target: &mut JsonValue) {
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
            parse_entry(key.to_string(), &mut value, &spec_for_object[key]);
        }
    }

    write_to_cache(target["id"].as_str().unwrap(), &target)
}

pub fn parse_external_list(url_raw: String, last_sync: Option<String>) {
    let this_sync = Local::now().format("%Y-%m-%dT%H:%M:%S%Z").to_string();

    let limit: Option<usize> = None;
    let mut url: Url = Url::parse(&url_raw).unwrap();

    if let Some(last_sync_time) = last_sync {
        // Add the modified_since filter
        url.query_pairs_mut().append_pair("modified_since", &last_sync_time).finish();
    }

    let mut elist = ExternalList::new(url.to_string());

    /*if let Some(x) = limit {
        elist = elist.take(x);
    }*/

    let mut urls = Vec::new();
    for mut i in elist {
        parse_object(&mut i);
        urls.push(i["id"].to_string());
    }

    let mut old_urls = Vec::new();
    urls.append(&mut old_urls);


    let mut urls_as_json = JsonValue::new_array();
    for i in urls {
        urls_as_json.push(i).unwrap();
    }
    write_to_cache(&url_raw, &urls_as_json);

    // self.external_lists_lock.acquire()
    let mut external_lists = JsonValue::from("[]");
    for i in external_lists.members_mut() {
        if i["url"] == url_raw {
            i["last_update"] = JsonValue::String(this_sync);
            break;
        }
    };

    // save();
    // self.external_lists_lock.release()
}

/*
pub fn load_to_cache(entrypoint: &str) {

}
*/

