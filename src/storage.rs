use std::convert::From;
use std::error::Error;
use std::fs::{File, create_dir_all};
use std::io::Read;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

use crossbeam;
use chrono::{Local};
use json;
use json::JsonValue;
use hyper::Url;
use hyper::client::IntoUrl;

use FileStorage;
use server::Server;
use external_list::ExternalList;

use super::FILE_EXTENSION;

pub trait Storage {
    /// Caches a servers contents or updates the cache
    fn cache<T: Server>(&self, server: T) -> Result<(), Box<Error>>;
}

impl<'a> FileStorage<'a> {
    /// Writes JSON to the path corresponding with the url. This will be an object and its id in the
    /// most cases
    fn write_to_cache(&self, url: &Url, object: &JsonValue) -> Result<(), Box<Error>> {
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
                      add_list: Sender<(Url, Option<String>)>) {
        if entry_def["type"] == "array" {
            for mut i in entry.members_mut() {
                let key = key.to_string() + "[" + &i.to_string() + "]";
                self.parse_entry(key.as_str(), &mut i, &entry_def["items"], add_list.clone());
            }
        } else if entry_def["type"] == "object" {
            if entry["type"] == "Feature" {
                return; // GeoJson is treated is a single value
            }
            // Extract the embedded object leaving its id
            self.parse_object(entry, add_list);
            *entry = JsonValue::String(entry["id"].to_string());
        } else if entry_def["references"] == "externalList" {
            add_list.send((entry.to_string().into_url().unwrap(), None)).unwrap();
        }
    }

    /// Determines the corresponding schema of an object, lets all it's attributes be parsed
    /// recursively and then writes the object to the cache
    fn parse_object(&self, target: &mut JsonValue, add_list: Sender<(Url, Option<String>)>) {
        let let_binding = target["type"].to_string();
        let oparl_type = let_binding.split("/").last().unwrap();
        let spec_for_object = &self.get_schema()[oparl_type]["properties"];

        for (key, mut value) in target.entries_mut() {
            // Check if the key is defined in the specification
            if spec_for_object.entries().map(|(key, _)| key).any(|i| i == key) {
                self.parse_entry(key, &mut value, &spec_for_object[key], add_list.clone());
            }
        }

        self.write_to_cache(&target["id"].as_str().unwrap().into_url().unwrap(), &target).unwrap();
    }

    /// Downloads a whole external list and saves the results to the cache
    /// If `last_sync` is given, the filter modified_since will be appended to the url
    /// `add_list` allows adding external lists that were found when parsing this one
    fn parse_external_list<U: IntoUrl>(&self, url: U, last_sync: Option<String>,
                                       add_list: Sender<(Url, Option<String>)>)
                                          -> Result<(Url, String), Box<Error>> {
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
                self.parse_object(&mut i, add_list.clone());
                urls.push(i["id"].to_string());
            }
        } else {
            for mut i in elist {
                self.parse_object(&mut i, add_list.clone());
                urls.push(i["id"].to_string());
            }
        }

        let mut old_urls = Vec::new();
        urls.append(&mut old_urls);

        // Get the urls that have already been retrieved when not using a modified_since
        let old_urls_filepath = self.url_to_path(&url_with_filters, FILE_EXTENSION)?;
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
        self.write_to_cache(&url_with_filters, &urls_as_json)?;

        Ok((url_with_filters, this_sync))
    }

    /// Download and cache all external lists while adding those newly found in a fully parallelized
    /// manner. This function blocks until all threads have finished.
    /// The weird command order is due to the Mutex-locking which would otherwise dead-lock
    /// the child threads
    pub fn load_all_external_lists<T: Server>(&self, server: T, known_external_lists: &Vec<(Url, Option<String>)>) {
        let mut results = vec![];
        let mut done = vec![];

        crossbeam::scope(|scope| {
            let (add_list, receive_list) = channel();

            for i in known_external_lists {
                add_list.send(i.clone()).unwrap();
            }

            // Download the entrypoint which is the System object
            // This will set the first external list, which is the body list
            let mut system_object = server.download_json(server.get_entrypoint().clone()).unwrap();
            {
                self.parse_object(&mut system_object, add_list.clone());
            }
            loop {
                let cur = match receive_list.recv() {
                    Ok(ok) => ok,
                    Err(_) => break,
                };

                println!("Got {}", cur.0);

                for i in done.iter() {
                    if *i == cur {
                        println!("known");
                        break;
                    }
                }
                done.push(cur.clone());

                let url = cur.0.clone();
                let last_update = cur.1.clone();
                let add_list = add_list.clone();

                let closure = move || -> Result<_, Box<Error + Send + Sync>> {
                    let list_result = self.parse_external_list(url, last_update, add_list);
                    let sendable_and_typed: Result<_, Box<Error + Send + Sync>>;
                    sendable_and_typed = list_result.map_err(|err| From::from(err.description()));
                    sendable_and_typed
                };
                results.push(scope.spawn(
                    closure
                ));
            }
        });

        for thread in results {
            match thread.join() {
                Ok(url) => println!("Success: {}", url.0),
                Err(err) => println!("Failed: {}", err),
            }
        }
    }
}

impl<'a> Storage for FileStorage<'a> {
    /// Loads the whole API to the cache or updates an existing cache
    /// This function does only do the loading saving and forwards the actual work
    fn cache<T: Server>(&self, server: T) -> Result<(), Box<Error>> {
        let cache_status_filepath = self.url_to_path(&server.get_entrypoint().clone(), "")?
            .join(self.get_cache_status_file());
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

        // Here the actual work is done
        self.load_all_external_lists(server, &known_lists);

        // Write the results back to the cache
        let mut cache_status_file: File = File::create(&cache_status_filepath)?;
        let mut cache_status_json = JsonValue::new_array();

        for i in known_lists {
            cache_status_json.push(object! {
                "url" => JsonValue::from(i.0.clone().into_string()),
                "last_sync" => JsonValue::from(i.1.clone())
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
}