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
    fn write_to_cache(&self, url: &Url, object: &JsonValue) -> Result<(), Box<Error>>;
    /// Retrieves a cached object
    fn get<U: IntoUrl>(&self, url: U) -> Result<JsonValue, Box<Error>>;
}

impl<'a> Storage for FileStorage<'a> {
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

    /// Retrieves a stored cached object from the cache.
    ///
    /// Returns a boxed error if the url was invalid or if there was an error reading the cache file
    fn get<U: IntoUrl>(&self, url: U) -> Result<JsonValue, Box<Error>> {
        let path = self.url_to_path(&url.into_url()?, FILE_EXTENSION)?;
        let mut s = String::new();
        let mut file: File = File::open(path)?;
        file.read_to_string(&mut s)?;
        let json = json::from(s.as_str());
        Ok(json)
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