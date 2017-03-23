use std::collections::VecDeque;
use std::error::Error;
use std::sync::mpsc::{Sender, channel};
use std::time::Duration;

use reqwest::Url;
use reqwest::IntoUrl;
use json::JsonValue;
use crossbeam;
use chrono::Local;

use server::Server;
use storage::Storage;
use external_list::ExternalList;

/// The type of the messages send from the worker to main thread
#[derive(Debug)]
pub enum Message {
    List(Url),
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
                   add_list: ListSender) {
        if entry_def["type"] == "array" {
            for mut i in entry.members_mut() {
                let key = key.to_string() + "[" + &i.to_string() + "]";
                self.parse_entry(key.as_str(), &mut i, &entry_def["items"], add_list.clone());
            }
        } else if entry_def["type"] == "object" {
            if entry["type"] == "Feature" {
                return; // GeoJSON is treated is a single value
            }
            // Extract the embedded object leaving its id
            self.parse_object(entry, add_list);
            *entry = JsonValue::String(entry["id"].to_string());
        } else if entry_def["references"] == "externalList" {
            add_list.send(Message::List(entry.to_string().into_url().unwrap())).unwrap();
        }
    }

    /// Determines the corresponding schema of an object, lets all it's attributes be parsed
    /// recursively and then writes the object to the cache
    fn parse_object(&self, target: &mut JsonValue, add_list: ListSender) {
        let let_binding = target["type"].to_string();
        let oparl_type = let_binding.split("/").last().unwrap();
        let spec_for_object = &self.get_schema()[oparl_type]["properties"];

        for (key, mut value) in target.entries_mut() {
            // Check if the key is defined in the specification
            if spec_for_object.entries().map(|(key, _)| key).any(|i| i == key) {
                self.parse_entry(key, &mut value, &spec_for_object[key], add_list.clone());
            }
        }

        self.write_to_cache(&target["id"]
                                 .as_str()
                                 .unwrap()
                                 .into_url()
                                 .unwrap(),
                            &target)
            .unwrap();
    }

    /// Downloads a whole external list and saves the results to the cache
    /// If `last_sync` is given, the filter modified_since will be appended to the url
    /// `add_list` allows adding external lists that were found when parsing this one
    fn parse_external_list(&self,
                           url: Url,
                           last_sync: Option<String>,
                           add_list: ListSender)
                           -> Result<(Url, Option<String>), Box<Error>> {
        // Take the time before the downloading as the data can change while obtaining pages
        let this_sync = Local::now().format("%Y-%m-%dT%H:%M:%S%Z").to_string();

        let limit: Option<usize> = None;
        let url_without_filters = url.into_url()?;
        let mut url_with_filters: Url = url_without_filters.clone();

        if let Some(last_sync_time) = last_sync {
            // Add the modified_since filter
            url_with_filters.query_pairs_mut()
                .append_pair("modified_since", &last_sync_time)
                .finish();
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

        // Get the the lists cached in the last run
        let mut urls_as_json = self.get(&url_with_filters).unwrap_or(JsonValue::new_array());

        if !urls_as_json.is_array() {
            return Err(From::from(format!("Invalid cache for {}", url_with_filters)));
        }

        for i in urls {
            urls_as_json.push(i)?;
        }
        self.write_to_cache(&url_with_filters, &urls_as_json)?;

        Ok((url_without_filters, Some(this_sync)))
    }

    /// Downloads and caches all external lists while adding those newly found in parallel. This
    /// function blocks until all threads have finished.
    /// The weird command order is due to the Mutex-locking which would otherwise dead-lock
    /// the child threads
    fn load_all_external_lists<T: Server>(&self,
                                          server: T,
                                          known: &Vec<(Url, Option<String>)>)
                                          -> Vec<(Url, Option<String>)> {
        let mut thread_handles = vec![];

        // Avoid doing same list more than once
        let mut done: Vec<Url> = vec![];

        // Keep track of how many threads there are so they function can exit all have finished
        let mut threadcounter: usize = 0;

        let mut queue = VecDeque::new();
        for i in known {
            queue.push_back(i.clone())
        }

        let (add_list, receive_list) = channel::<Message>();

        // Download the entrypoint which is the System object
        // This will set the first external list, which is the body list
        let mut system_object = server.get_json(server.get_entrypoint().clone()).unwrap();
        self.parse_object(&mut system_object, add_list.clone());

        crossbeam::scope(|scope| {
            loop {
                // Searches for new lists or exits when all workers finshed
                let (ref url, ref last_update): (Url, Option<String>) = {
                    if let Some(queued) = queue.pop_front() {
                        queued
                    } else {
                        match receive_list.recv_timeout(Duration::from_secs(10)) {
                            Ok(Message::List(url)) => (url, None),
                            Ok(Message::Done) => {
                                threadcounter -= 1;
                                if threadcounter == 0 { break } else { continue }
                            }
                            Err(_) => {
                                // Granted, this is no the optimal solution. But it's better than
                                // nothing at all
                                println!("No message from any worker. Have they hung up?");
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
                    let list_result = self.parse_external_list(url, last_update, add_list.clone());
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
                    new_cache_status.push(list);
                }
                Err(err) => println!("Failed: {}", err),
            }
        }

        new_cache_status
    }
}
