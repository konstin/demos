use chrono::*;

use std::path::Path;

#[test]
fn test_parse_object() {
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

    super::parse_object(&mut input);
    assert_eq!(input, expected_output);
}

#[test]
fn test_parse_external_list() {
    let eurl = "http://localhost:8080/oparl/v1.0/body/0/list/paper?a=b";
    let time = Local::now().format("%Y-%m-%dT%H:%M:%S%Z").to_string();
    super::parse_external_list(eurl.to_string(), Some(time));
}

#[test]
fn test_external_list_iterator() {
    let expected_ids = [
        "http://localhost:8080/oparl/v1.0/paper/1",
        "http://localhost:8080/oparl/v1.0/paper/2",
        "http://localhost:8080/oparl/v1.0/paper/3",
        "http://localhost:8080/oparl/v1.0/paper/4",
        "http://localhost:8080/oparl/v1.0/paper/5",
        "http://localhost:8080/oparl/v1.0/paper/6",
        "http://localhost:8080/oparl/v1.0/paper/7",
    ];

    let eurl = "http://localhost:8080/oparl/v1.0/body/0/list/paper";
    let list = super::ExternalList::new(eurl.to_string());
    let ids = list.map(|i| i["id"].to_owned()).collect::<Vec<_>>();
    assert_eq!(ids, expected_ids);
}

fn test_single_url_to_path(url: &str, path: &str) {
    assert_eq! (super::url_to_path(url), Path::new(path));
    assert_eq! (super::url_to_path(&(url.to_string() + "/")), Path::new(path));
}

#[test]
fn test_url_to_path() {
    test_single_url_to_path("https://example.tld:8080/oparl/v1.0/paper/1", "/home/konsti/cache-rust/https:example.tld:8080/oparl/v1.0/paper/1.json");
    test_single_url_to_path("https://example.tld/oparl/v1.0/paper/1", "/home/konsti/cache-rust/https:example.tld/oparl/v1.0/paper/1.json");
    test_single_url_to_path("https://example.tld", "/home/konsti/cache-rust/https:example.tld.json");
    test_single_url_to_path("https://example.tld/api?modified_until=2016-05-03T00%3A00%3A00%2B02%3A00", "/home/konsti/cache-rust/https:example.tld/api.json");
}
