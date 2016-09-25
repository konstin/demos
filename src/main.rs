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
use url::Url;

use std::fs::{File, read_dir, create_dir_all};
use std::io::prelude::*;
use std::str::FromStr;
use std::path::Path;
use std::sync::{Arc, Mutex, Condvar};
use std::thread;

//const ENTRYPOINT: &'static str = "http://localhost:8080/oparl/v1.0";
const ENTRYPOINT: &'static str = "http://ratsinformant.local/oparl/v1.0";
//const ENTRYPOINT: &'static str = "https://www.muenchen-transparent.de/oparl/v1.0";
const SCHEMA_DIR: &'static str = "schema";
const LIMIT: usize = 500;


fn check_oparl_object(schema: &json::JsonValue, object: &json::JsonValue) -> Result<(), String> {
    //let oparltype_regex = regex::Regex::new(r"^https://schema\.oparl\.org/1\.0/([:alpha:]+)$").unwrap();
    let oparltype = object["type"].as_str().unwrap();
    let typename = &oparltype[29..];
    //let typename = oparltype_regex.captures(oparltype).and_then(|x| x.at(1))
    //               .ok_or(format!("The type attribute of the object is not valid: {}", oparltype))?;
    let object_schema = &schema["typename"];
    for requirement in object_schema["required"].members().map(|i| i.as_str().unwrap()) {
        if !object.has_key(requirement) {
            return Err(format!("The object of the type oparl:{} does not provide the required attribute {}", typename, requirement).into());
        } else if !object[requirement].is_null() {
            return Err(format!("The object of the type oparl:{} does not provide a non-null value for the required attribute {}", typename, requirement).into());
        }
    }
    Ok(())
}

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

fn parse_single_object(schema: &json::JsonValue, object: &mut json::JsonValue, urls: &Arc<Mutex<Vec<String>>>) {
    check_oparl_object(schema, object).unwrap();
    for (_, value) in object.entries_mut() {
        if let Some(link) = value.take_string() {
            let mut urls2 = urls.lock().unwrap();
            if link.starts_with(ENTRYPOINT) && !urls2.contains(&link) {
                urls2.push(link);
            }
        }
    }
}

fn parse_response_json(schema: &json::JsonValue, json: &mut json::JsonValue, urls: &Arc<Mutex<Vec<String>>>) {
    if json.has_key("id") {
        parse_single_object(schema, json, urls);
    } else if json.has_key("data") {
        for mut object in json["data"].members_mut() {
            parse_single_object(schema, &mut object, urls);
        }
        if let Some(next) = json["links"]["next"].take_string() {
            let mut urls = urls.lock().unwrap();
            if next.starts_with(ENTRYPOINT) && !urls.contains(&next) {
                urls.push(next);
            }
        }
    } else {
        println!(" --- Returned JSON is invalid --- ");
    }
}

fn load_url_cached(client: &Client, url: &String) -> Result<String, String>{
    let x = Url::from_str(url).unwrap();
    let y = "cache/".to_string() + x.host_str().unwrap() + x.path() + ".json";
    let path = Path::new(&y);

    /*if let Ok(mut file) = File::open(path) {
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();
        return Ok(buffer);
    }*/

    let mut response = match client.get(url).send() {
        Ok(ok) => ok,
        Err(err) => { return Err(format!("The request failed: {}", err).into()); },
    };

    if response.status != hyper::Ok {
        return Err(format!("The request returned an error code: {}", response.url).into());
    }

    let mut json_string = String::new();
    response.read_to_string(&mut json_string).unwrap(); // Can this fail?

    /*create_dir_all(path.parent().unwrap()).unwrap();
    let mut file = File::create(path).unwrap();
    file.write_all(json_string.as_bytes()).unwrap();
    println!("Written to cache");*/

    Ok(json_string)
}

fn parse_one_url(my_url: &String, urls: &Arc<Mutex<Vec<String>>>, schema: &json::JsonValue, client: &Client) -> PreciseTime {
    let json_string = match load_url_cached(&client, &my_url) {
        Ok(ok) => ok,
        Err(err) => {
            println! ("{}", err);
            panic!();
        }
    };
    let time_b = PreciseTime::now();

    let mut json = match json::parse(json_string.as_str()) {
        Ok(ok) => ok,
        Err(err) => {
            println! ("Invalid JSON: {}", err);
            panic!();
        }
    };

    parse_response_json(&schema, &mut json, &urls);
    time_b
}

fn crawl(shared_iterator: Arc<Mutex<usize>>, urls: Arc<Mutex<Vec<String>>>, id: usize, pair: Arc<(Mutex<usize>, Condvar)>, threadcount: usize) {
    let schema = read_schema(SCHEMA_DIR).unwrap();

    let client = Client::new();

    let &(ref lock, ref cvar) = &*pair;

    loop {
        //let time_a = PreciseTime::now();

        let my_url;
        loop {
            {
                let urls_locked = urls.lock().unwrap();
                let mut iterator_locked = shared_iterator.lock().unwrap();
                if *iterator_locked < urls_locked.len() && *iterator_locked < LIMIT {
                    my_url = urls_locked[*iterator_locked].to_owned();
                    *iterator_locked += 1;
                    break
                }
            }
            let mut stalled_counter = lock.lock().unwrap();
            *stalled_counter += 1;
            if *stalled_counter == threadcount {
                // All threads are waiting, so we're done
                println!("DONE: {}", id);
                cvar.notify_all();
                return;
            }
            println!("COUNT: {} (id: {})", *stalled_counter, id);
            stalled_counter = cvar.wait(stalled_counter).unwrap();
            *stalled_counter -= 1;
        }

        println!("{}: {}", id, &my_url);
        let time_b = parse_one_url(&my_url, &urls, &schema, &client);
        cvar.notify_all();

        //let time_c = PreciseTime::now();
        //println! ("{} and {}", time_a.to(time_b).num_microseconds().unwrap(), time_b.to(time_c).num_microseconds().unwrap());
    }
}

fn main() {

    let start = PreciseTime::now();

    let entrypoint = String::from(ENTRYPOINT);
    let urls = Arc::new(Mutex::new(vec![entrypoint]));
    let iterator = Arc::new(Mutex::new(0));

    let pair = Arc::new((Mutex::new(0), Condvar::new()));

    let mut threads = Vec::new();
    let threadcount = 4;
    for id in 0..threadcount {
        let iterator = iterator.clone();
        let urls = urls.clone();
        let pair = pair.clone();
        let handle = thread::spawn(move || {
            crawl(iterator, urls, id, pair, threadcount);
        });
        threads.push(handle);
    }
    threads.into_iter().map(|i| i.join()).collect::<Vec<_>>();

    println!("TOTAL: {} milliseconds", start.to(PreciseTime::now()).num_milliseconds());
}
