extern crate oparl_cache;
extern crate reqwest;
extern crate chrono;

mod common;

use chrono::Local;
use reqwest::IntoUrl;

use std::sync::mpsc::channel;

use oparl_cache::ExternalList;
use oparl_cache::Cacher;

use common::storage;

#[test]
fn parse_external_list() {
    let url = "http://localhost:8080/oparl/v1.0/body/0/list/paper";
    let time = Local::now().format("%Y-%m-%dT%H:%M:%S%Z").to_string();

    let (add_list, receive_list) = channel();

    storage().parse_external_list(url.into_url().unwrap(), Some(time), add_list).unwrap();

    assert_eq!(receive_list.recv().is_err(), true);
    // TODO: Check returned value
}

#[test]
fn external_list() {
    let expected_ids = ["http://localhost:8080/oparl/v1.0/paper/1",
        "http://localhost:8080/oparl/v1.0/paper/2",
        "http://localhost:8080/oparl/v1.0/paper/3",
        "http://localhost:8080/oparl/v1.0/paper/4",
        "http://localhost:8080/oparl/v1.0/paper/5",
        "http://localhost:8080/oparl/v1.0/paper/6",
        "http://localhost:8080/oparl/v1.0/paper/7"];
    let eurl = "http://localhost:8080/oparl/v1.0/body/0/list/paper";
    let list = ExternalList::new(eurl.into_url().unwrap());
    let ids = list.map(|i| i["id"].to_owned()).collect::<Vec<_>>();
    assert_eq!(ids, expected_ids);
}
