use std::collections::VecDeque;
use std::error::Error;
use std::sync::mpsc::{Sender, channel};
use std::time::Duration;
use std::io::stdout;

use reqwest::Url;
use reqwest::IntoUrl;
use json::JsonValue;
use crossbeam;
use chrono::Local;

use server::Server;
use storage::Storage;
use file_storage::{CacheStatus, UrlWithTimestamp};
use external_list::ExternalList;

/// The type of the messages send from the worker to main thread
#[derive(Debug)]
pub enum Message {
    /// Contains the url of an external list that was found in a worker thread
    List(Url),
    /// Tells the main thread that one worker is done
    Done,
}

type ListSender = Sender<Message>;

/// A Storage able to consume all data from a server
///
/// TODO: Refactor out helper functions
pub trait Cacher: Storage + Sync {
    /// Consumes all data from a server
    fn cache<T: Server>(&self, server: T) -> Result<(), Box<Error>>;

    /// Parses the data of a single attribute of an object recursively and replaces embedded objects
    /// by the id. The embedded objects are them parsed by themselves
    fn parse_entry(&self,
                   key: &str,
                   entry: &mut JsonValue,
                   entry_def: &JsonValue,
                   add_list: ListSender) -> Result<(), Box<Error>> {
        if entry_def["type"] == "array" {
            for mut i in entry.members_mut() {
                let key = key.to_string() + "[" + &i.to_string() + "]";
                self.parse_entry(key.as_str(), &mut i, &entry_def["items"], add_list.clone())?;
            }
        } else if entry_def["type"] == "object" {
            if entry["type"] == "Feature" {
                return Ok(()); // GeoJSON is treated is a single value
            }
            // Extract the embedded object leaving its id
            self.parse_object(entry, add_list)?;
            *entry = JsonValue::String(entry["id"].to_string());
        } else if entry_def["references"] == "externalList" {
            add_list.send(Message::List(entry.to_string().into_url()?)).unwrap();
        }

        Ok(())
    }

    /// Determines the corresponding schema of an object, lets all it's attributes be parsed
    /// recursively and then writes the object to the cache
    fn parse_object(&self, target: &mut JsonValue, add_list: ListSender) -> Result<(), Box<Error>> {
        let let_binding = target["type"].to_string();
        let oparl_type = let_binding.split("/").last().ok_or("Invalid type url")?;
        let spec_for_object = &self.get_schema()[oparl_type]["properties"];

        for (key, mut value) in target.entries_mut() {
            // Check if the key is defined in the specification
            if spec_for_object.has_key(key) {
                self.parse_entry(key, &mut value, &spec_for_object[key], add_list.clone())?;
            }
        }

        let id = target["id"].as_str().ok_or("The id has to be a String")?.into_url()?;
        self.write_to_cache(&id, &target)?;

        Ok(())
    }

    /// Downloads a whole external list and saves the results to the cache
    /// If `last_sync` is given, the filter modified_since will be appended to the url
    /// `add_list` allows adding external lists that were found when parsing this one
    fn parse_external_list<T: Server>(&self,
                                      url: Url,
                                      last_sync: Option<String>,
                                      server: &T,
                                      add_list: ListSender)
                                      -> Result<(Url, Option<String>), Box<Error>> {
        // Take the time before the downloading as the data can change while obtaining pages
        let this_sync = Local::now().format("%Y-%m-%dT%H:%M:%S%Z").to_string();

        let url_without_filters = url.into_url()?;
        let mut url_with_filters: Url = url_without_filters.clone();

        if let Some(ref last_sync_time) = last_sync {
            // Add the modified_since filter
            url_with_filters.query_pairs_mut()
                .append_pair("modified_since", &last_sync_time)
                .finish();
        }

        let list = ExternalList::new(Url::parse(url_with_filters.as_str())?, server);

        // A Vec is used instead of a Set as we want to preserve the ordering
        let mut urls: Vec<String> = Vec::new();

        for i in list {
            let mut i: JsonValue = i?;
            let result = self.parse_object(&mut i, add_list.clone());
            if let Err(err) = result {
                println!("Invalid object: {}", err);
                i.write_pretty(&mut stdout(), 4).unwrap();
                println!("Skipping the above object");
                continue;
            }
            let value = i["id"].to_string();
            if !urls.contains(&value) {
                urls.push(value);
            }
        }

        // Get the the lists cached in the last run
        let mut urls_as_json = if last_sync.is_some() {
            match self.get(&url_with_filters) {
                Ok(ok) => ok,
                Err(_) => {
                    println!("Warn: Trying to perform an incremental update on a list with an invalid cache");
                    JsonValue::new_array()
                }
            }
        } else {
            JsonValue::new_array()
        };

        if !urls_as_json.is_array() {
            return Err(From::from(format!("Invalid cache for {}", url_with_filters)));
        }

        for mut i in urls_as_json.members_mut() {
            let value = i.take_string().unwrap();
            if !urls.contains(&value) {
                urls.push(value);
            }
        }

        let urls_as_vec = urls.into_iter().map(|x| JsonValue::String(x)).collect();
        let urls_new_json = JsonValue::Array(urls_as_vec);

        self.write_to_cache(&url_with_filters, &urls_new_json)?;

        Ok((url_without_filters, Some(this_sync)))
    }

    /// Downloads and caches all external lists while adding those newly found in parallel. This
    /// function blocks until all threads have finished.
    /// The weird command order is due to the Mutex-locking which would otherwise dead-lock
    /// the child threads
    fn load_all_external_lists<T: Server>(&self,
                                          server: &T,
                                          known: &CacheStatus)
                                          -> CacheStatus {
        let mut thread_handles = vec![];

        // Avoid doing same list more than once
        let mut done: Vec<Url> = vec![];

        // Keep track of how many threads there are so the function can exit when all have finished
        let mut threadcounter: usize = 0;

        let mut queue: VecDeque<UrlWithTimestamp> = VecDeque::new();
        for i in known {
            queue.push_back(i.clone());
        }

        let (add_list, receive_list) = channel::<Message>();

        // Download the entrypoint which is the System object
        // This will set the first external list, which is the body list
        let system_object = server.get_json(server.get_entrypoint().clone());
        let result = system_object.and_then(|mut x| self.parse_object(&mut x, add_list.clone()));

        if let Err(err) = result {
            println!("Failed to parse the System object: {}", err);
            println!("Aborting");
            return vec![];
        };

        for i in receive_list.try_iter() {
            if let Message::List(url) = i {
                queue.push_back(UrlWithTimestamp {url: url, last_sync: None});
            }
        }

        if queue.is_empty() {
            println!("Warn: No external lists found");
            return vec![];
        }

        crossbeam::scope(|scope| {
            loop {
                // Searches for new lists or exits when all workers finshed
                let UrlWithTimestamp {url, last_sync: last_update} = {
                    if let Some(queued) = queue.pop_front() {
                        queued
                    } else {
                        match receive_list.recv_timeout(Duration::from_secs(10)) {
                            Ok(Message::List(url)) => UrlWithTimestamp {url: url, last_sync: None},
                            Ok(Message::Done) => {
                                threadcounter -= 1;
                                if threadcounter == 0 { break } else { continue }
                            }
                            Err(_) => {
                                // Granted, this is no the optimal solution. But it's better than
                                // nothing at all
                                println!("No message from any worker after 10 seconds ...");
                                continue;
                            }
                        }
                    }
                };

                println!("List Found: {}", url);

                if done.contains(&url) {
                    println!("List is known, skipping");
                    continue;
                }

                done.push(url.clone());

                let add_list = add_list.clone();
                let url = url.clone();
                let last_update = last_update.clone();

                let closure = move || {
                    let list_result =
                        self.parse_external_list(url, last_update, server, add_list.clone());
                    add_list.send(Message::Done).unwrap();

                    let sendable_and_typed: Result<_, Box<Error + Send + Sync>>;
                    sendable_and_typed = list_result.map_err(|err| From::from(err.description()));
                    sendable_and_typed
                };
                thread_handles.push(scope.spawn(closure));
                threadcounter += 1;
            }
        });

        let mut new_cache_status = vec![];

        for thread in thread_handles {
            match thread.join() {
                Ok(list) => {
                    println!("Success: {}", &list.0);
                    new_cache_status.push(UrlWithTimestamp {url: list.0, last_sync: list.1});
                }
                Err(err) => println!("Failed: {}", err),
            }
        }

        new_cache_status
    }
}
