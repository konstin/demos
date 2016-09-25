#![feature(question_mark)]
#![feature(test)]

extern crate json;
extern crate hyper;
extern crate regex;
extern crate test;
extern crate time;
extern crate url;

use hyper::Client;
use time::PreciseTime;

use std::fs::{File, read_dir, create_dir_all};
use std::io::prelude::*;
use url::Url;
use std::str::FromStr;
use std::path::Path;

//const ENTRYPOINT: &'static str = "http://localhost:8080/oparl/v1.0";
const ENTRYPOINT: &'static str = "http://ratsinformant.local/oparl/v1.0";
//const ENTRYPOINT: &'static str = "https://www.muenchen-transparent.de/oparl/v1.0";
const SCHEMA_DIR: &'static str = "schema";


/*fn check_oparl_object(schema: &json::JsonValue, object: &json::JsonValue) -> Result<(), String> {
    let oparltype_regex = regex::Regex::new(r"^https://schema\.oparl\.org/1\.0/([:alpha:]+)$").unwrap();
    let oparltype = object["type"].as_str().unwrap();
    let typename = oparltype_regex.captures(oparltype).and_then(|x| x.at(1))
                   .ok_or(format!("The type attribute of the object is not valid: {}", oparltype))?;
    let object_schema = &schema["typename"];
    for requirement in object_schema["required"].members().map(|i| i.as_str().unwrap()) {
        if !object.has_key(requirement) {
            return Err(format!("The object of the type oparl:{} does not provide the required attribute {}", typename, requirement).into());
        } else if !object[requirement].is_null() {
            return Err(format!("The object of the type oparl:{} does not provide a non-null value for the required attribute {}", typename, requirement).into());
        }
    }
    Ok(())
}*/

fn read_schema(dir: &str) -> Result<json::JsonValue, String> {
    let mut schema = json::JsonValue::new_object();
    let paths = read_dir(dir).map_err(|err| format!("Could not open the directory '{}':  {}", dir, err).to_string())?;

    for path in paths {
        let filename = path.unwrap().path();

        let mut buffer = String::new();
        let mut file = File::open(&filename).map_err(|err| format!("Could not open file '{}': {}", filename.display(), err).to_string())?;
        file.read_to_string(&mut buffer).map_err(|err| format!("Could not read file '{}': {}", filename.display(), err).to_string())?;

        let parsed = json::parse(buffer.as_str()).map_err(|err| format!("File contains invalid json '{}': {}", filename.display(), err).to_string())?;
        let d = filename.file_stem().unwrap().to_str().unwrap();
        schema[d] = parsed;
    };

    Ok(schema)
}

#[test]
fn read_schema_test() {
    let schema = read_schema("schema").unwrap();
    assert_eq!(schema.len(), 12);
}

fn parse_single_object(schema: &json::JsonValue, object: &mut json::JsonValue, urls: &mut Vec<String>) {
    //check_oparl_object(schema, object);
    for (_, value) in object.entries_mut() {
        if let Some(x) = value.take_string() {
            if x.starts_with(ENTRYPOINT) && !urls.contains(&x) {
                urls.push(x);
            }
        }
    }
}

fn parse_response_json(schema: &json::JsonValue, json: &mut json::JsonValue, urls: &mut Vec<String>) {
    if json.has_key("id") {
        parse_single_object(schema, json, urls);
    } else if json.has_key("data") {
        for mut object in json["data"].members_mut() {
            parse_single_object(schema, &mut object, urls);
        }
        if let Some(next) = json["links"]["next"].take_string() {
            urls.push(next);
        }
    } else {
        println!(" --- Returned JSON is invalid --- ");
    }
}

fn load_url_cached(client: &Client, url: &String) -> Result<String, String>{
    let x = Url::from_str(url).unwrap();
    let y = "cache/".to_string() + x.host_str().unwrap() + x.path() + ".json";
    let path = Path::new(&y);

    if let Ok(mut file) = File::open(path) {
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();
        println!("Loaded from cache");
        return Ok(buffer);
    }

    let mut response = match client.get(url).send() {
        Ok(ok) => ok,
        Err(err) => { return Err(format!("The request failed: {}", err).into()); },
    };

    if response.status != hyper::Ok {
        return Err(format!("The request returned an error code: {}", response.url).into());
    }

    let mut json_string = String::new();
    response.read_to_string(&mut json_string).unwrap(); // Can this fail?

    create_dir_all(path.parent().unwrap()).unwrap();
    let mut file = File::create(path).unwrap();
    file.write_all(json_string.as_bytes()).unwrap();
    println!("Written to cache");

    Ok(json_string)
}

fn crawl(shared_iterator: &mut usize, urls: &mut Vec<String>, schema: &json::JsonValue) {
    let client = Client::new();

    loop {
        if *shared_iterator >= urls.len() || *shared_iterator > 500 {
            break
        }
        let my_url = urls[*shared_iterator].to_owned();
        *shared_iterator += 1;

        println!("{}", &my_url);

        let time_a = PreciseTime::now();
        let json_string = match load_url_cached(&client, &my_url) {
            Ok(ok) => ok,
            Err(err) => {
                println!("{}", err);
                continue
            }
        };
        let time_b = PreciseTime::now();

        let mut json = match json::parse(json_string.as_str()) {
            Ok(ok) => ok,
            Err(err) => {
                println!("Invalid JSON: {}", err);
                continue;
            }
        };

        parse_response_json(&schema, &mut json, urls);

        let time_c = PreciseTime::now();
        println!("{} and {}", time_a.to(time_b).num_milliseconds(), time_b.to(time_c).num_milliseconds());
    }
}

fn main() {
    let start = PreciseTime::now();

    let schema = match read_schema(SCHEMA_DIR) {
        Ok(ok) => ok,
        Err(err) => {
            println!("Failed to load the schema files");
            println!("Reason: {}", err);
            std::process::exit(1);
        }
    };
    println!("{} schema files loaded", schema.len());
    println!("{} milliseconds", start.to(PreciseTime::now()).num_milliseconds());

    let entrypoint = String::from(ENTRYPOINT);
    let mut urls: Vec<String> = vec![entrypoint];
    let mut iterator = 0;

    crawl(&mut iterator, &mut urls, &schema);

    println!("TOTAL: {} ", start.to(PreciseTime::now()).num_milliseconds());
}


