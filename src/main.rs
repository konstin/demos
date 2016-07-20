extern crate json;
extern crate hyper;

use hyper::Client;
use std::fs::{File, read_dir};
use std::io::prelude::*;

//const ENTRYPOINT: &'static str = "http://localhost:8080/oparl/v1.0";
//const ENTRYPOINT: &'static str = "http://ratsinformant.local/oparl/v1.0";
const ENTRYPOINT: &'static str = "https://www.muenchen-transparent.de/oparl/v1.0";
const SCHEMA_DIR: &'static str = "schema";

fn read_schema(dir: &str) -> json::JsonValue {
    let mut schema = json::JsonValue::new_object();
    let paths = read_dir(dir).unwrap();

    for path in paths {
        let filename = path.unwrap().path();
        println!("Loading: {}", filename.display());

        let mut file = File::open(&filename).unwrap();
        let mut buffer = String::new();
        file.read_to_string(&mut buffer);

        let parsed = json::parse(buffer.as_str()).unwrap();
        let d = filename.file_stem().unwrap().to_str().unwrap();;
        schema[d] = parsed;
    };

    schema
}

#[test]
fn read_schema_test() {
    let schema = read_schema("schema");
    assert_eq!(schema.len(), 8);
}

fn parse_object(object: &mut json::JsonValue, urls: &mut Vec<String>) {
    for (_, value) in object.entries_mut() {
        if let Some(x) = value.take_string() {
            if x.starts_with(ENTRYPOINT) && !urls.contains(&x) {
                urls.push(x);
            }
        }
    }
}

fn main() {
    let schema = read_schema(SCHEMA_DIR);

    let client = Client::new();

    let entrypoint = String::from(ENTRYPOINT);
    let mut urls: Vec<String> = vec![entrypoint];

    let mut iterator = 0;
    while iterator < urls.len() {
        iterator += 1;
        println!("{}", &urls[iterator-1]);

        let mut response;
        match client.get(&urls[iterator-1]).send() {
            Ok(x) => response = x,
            Err(x) => {
                println!(" --- The request failed: {} --- ", x);
                continue;
            }
        }

        if response.status != hyper::Ok {
            println!(" --- The request returned an error code: {} ---", response.url);
            continue;
        }

        let mut json_string = String::new();
        response.read_to_string(&mut json_string);

        let mut json;
        match json::parse(json_string.as_str()) {
            Ok(x) => json = x,
            Err(x) => {
                println!(" --- Invalid JSON: {} --- ", x);
                continue;
            }
        }

        if json.entries().any(|(key, _)| key == "id") {
            parse_object(&mut json, &mut urls);
        } else if json.entries().any(|(key, _)| key == "data") {
            for mut object in json["data"].members_mut() {
                parse_object(&mut object, &mut urls);
            }
            if let Some(next) = json["links"]["next"].take_string() {
                urls.push(next);
            }
        } else {
            println!(" --- Invalid Response: {} --- ", response.url);
        }
    }
}
