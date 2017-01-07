use std::convert::From;
use std::error::Error;
use std::fs::{File, create_dir_all};
use std::io::Read;
use std::sync::{Arc, Mutex};

use crossbeam;
use chrono::{Local};
use json;
use json::JsonValue;
use hyper::Url;
use hyper::client::IntoUrl;

use external_list::{ExternalList, download_json};
use Storage;

use super::FILE_EXTENSION;

/// Loads the contents of an OParl to a cache
pub trait Cacher {
    /// Caches or updates one external list
    fn parse_external_list<U: IntoUrl + Copy>(&self, url: U, last_sync: Option<String>,
                                              external_list_adder: &Arc<Mutex<Vec<(String, Option<String>)>>>)
                                              -> Result<(), Box<Error>>;
    /// Caches a servers contents or updates the cache
    fn load_to_cache(&self) -> Result<(), Box<Error>>;
}

impl<'a> Storage<'a> {
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

    /// Writes JSON to the path corresponding with the url. This will be an object and its id in the
    /// most cases
    fn write_to_cache<U: IntoUrl>(&self, url: U, object: &JsonValue) -> Result<(), Box<Error>> {
        let filepath = self.url_to_path(url, FILE_EXTENSION)?;
        println!("Writen to Cache: {}", filepath.display());

        create_dir_all(filepath.parent().ok_or("Invalid cachepath for file")?)?;
        let mut file: File = File::create(filepath)?;

        object.write_pretty(&mut file, 4)?;
        Ok(())
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
            // TODO
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
        let spec_for_object = &self.get_schema()[oparl_type]["properties"];

        for (key, mut value) in target.entries_mut() {
            // Check if the key is defined in the specification
            if spec_for_object.entries().map(|(key, _)| key).any(|i| i == key) {
                self.parse_entry(key, &mut value, &spec_for_object[key], &external_list_adder);
            }
        }

        self.write_to_cache(target["id"].as_str().unwrap(), &target).unwrap();
    }

    /// Download and cache all external lists while adding those newly found in a fully parallelized
    /// manner. This function blocks until all threads have finished.
    /// The weird command order is due to the Mutex-locking which would otherwise dead-lock
    /// the child threads
    pub fn load_all_external_lists(&self, external_list_adder: &Arc<Mutex<Vec<(String, Option<String>)>>>) {
        let mut results = vec![];

        crossbeam::scope(|scope| {
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
                let external_list_adder = external_list_adder.clone();

                let closure = move || -> Result<String, Box<Error + Send + Sync>> {
                    let list_result = self.parse_external_list(&url, last_update, &external_list_adder);
                    let sendable_and_typed: Result<(), Box<Error + Send + Sync>> = list_result.map_err(|err| From::from(err.description()));
                    sendable_and_typed?;
                    Ok(url)
                };
                results.push(scope.spawn(
                    closure
                ));
                i += 1;
            }
        });

        for thread in results {
            match thread.join() {
                Ok(url) => println!("Success: {}", url),
                Err(err) => println!("Failed: {}", err),
            }
        }
    }
}

impl<'a> Cacher for Storage<'a> {
    /// Downloads a whole external list and saves the results to the cache
    /// If `last_sync` is given, the filter modified_since will be appended to the url
    /// `external_list_adder` allows adding external lists that were found when parsing this one
    fn parse_external_list<U: IntoUrl + Copy>(&self, url: U, last_sync: Option<String>,
                                              external_list_adder: &Arc<Mutex<Vec<(String, Option<String>)>>>)
                                              -> Result<(), Box<Error>> {
        // Taake the time before the downloading as the data can change while obtaining pages
        let this_sync = Local::now().format("%Y-%m-%dT%H:%M:%S%Z").to_string();

        let limit: Option<usize> = None;
        let mut url_with_filters: Url = url.into_url()?;

        if let Some(last_sync_time) = last_sync {
            // Add the modified_since filter
            url_with_filters.query_pairs_mut().append_pair("modified_since", &last_sync_time).finish();
        }

        let elist = ExternalList::new(Url::parse(url_with_filters.as_str())?);

        let mut urls = Vec::new();

        // TODO: use traits unstead of this weird type system hack
        if let Some(limeter) = limit {
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
        let old_urls_filepath = self.url_to_path(url, FILE_EXTENSION)?;
        let mut urls_as_json = {
            if old_urls_filepath.exists() {
                let mut old_urls_file = File::open(&old_urls_filepath)?;
                let mut read = String::new();
                old_urls_file.read_to_string(&mut read)?;
                json::parse(&read)?
            } else {
                JsonValue::new_array()
            }
        };

        for i in urls {
            urls_as_json.push(i)?;
        }
        self.write_to_cache(url, &urls_as_json)?;

        for i in external_list_adder.lock().unwrap().iter_mut() {
            if i.0 == url.into_url()?.as_str() {
                i.1 = Some(this_sync);
                break;
            }
        };

        Ok(())
    }

    /// Loads the whole API to the cache or updates an existing cache
    /// This function does only do the loading saving and forwards the actual work
    fn load_to_cache(&self) -> Result<(), Box<Error>> {
        let cache_status_filepath = self.url_to_path(self.get_entrypoint().clone(), "")?
            .join(self.get_cache_status_file());
        let external_list_adder: Vec<(String, Option<String>)>;

        if cache_status_filepath.exists() {
            // We have a cache, so let's load it
            println!("Cache found, updating...");
            let mut cache_status_file = File::open(&cache_status_filepath)?;
            let mut read = String::new();
            cache_status_file.read_to_string(&mut read)?;
            let known_external_lists = json::parse(&read)?.members()
                .map(|i| (i["url"].to_string(), Some(i["last_sync"].to_string())))
                .collect::<Vec<(String, Option<String>)>>();

            println!("External lists found in cache: {}", known_external_lists.len());
            external_list_adder = known_external_lists;
        } else {
            // We don't have a cache, so let's use an empty template
            println!("No cache found, initializing...");
            let err = "Could not create directory for the cache status file";
            create_dir_all(cache_status_filepath.parent().ok_or(err)?)?;
            external_list_adder = Vec::new();
        }

        println!("\nLoaded from cache:");
        for i in external_list_adder.iter() {
            println!("{}: {}", i.1.clone().unwrap_or("None".to_string()), i.0);
        }
        println!();

        let external_list_adder = Arc::new(Mutex::new(external_list_adder));

        // Download the entrypoint which is the System object
        // This will set the first external list, which is the body list
        let mut system_object = download_json(self.get_entrypoint().clone())?;
        self.parse_object(&mut system_object, &external_list_adder.clone());

        // Here the actual work is done
        self.load_all_external_lists(&external_list_adder);

        // Write the results back to the cache
        let mut cache_status_file: File = File::create(&cache_status_filepath)?;
        let mut cache_status_json = JsonValue::new_array();

        let external_list_adder = external_list_adder.lock().unwrap(); // TODO: Why does the questionmark cause a lifetime error here?

        for i in 0..external_list_adder.len() {
            cache_status_json.push(object! {
                "url" => JsonValue::from(external_list_adder[i].0.clone()),
                "last_sync" => JsonValue::from(external_list_adder[i].1.clone())
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
        let external_list_adder = Arc::new(Mutex::new(Vec::new()));
        storage().parse_object(&mut input, &external_list_adder);
        assert_eq!(input, expected_output);
        assert_eq!(*external_list_adder.lock().unwrap(), vec![]);
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
        let external_list_adder = Arc::new(Mutex::new(Vec::new()));
        storage().parse_object(&mut input, &external_list_adder);

        assert_eq!(input, expected_output);
        assert_eq!(*external_list_adder.lock().unwrap(), vec![
            ("http://localhost:8080/oparl/v1.0/body/0/list/organization".to_string(), None),
            ("http://localhost:8080/oparl/v1.0/body/0/list/person".to_string(), None),
            ("http://localhost:8080/oparl/v1.0/body/0/list/meeting".to_string(), None),
            ("http://localhost:8080/oparl/v1.0/body/0/list/paper".to_string(), None),
        ]);
    }
}