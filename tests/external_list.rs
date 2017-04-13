extern crate oparl_cache;
extern crate reqwest;
extern crate chrono;
#[macro_use]
extern crate json;

mod common;

use reqwest::IntoUrl;
use reqwest::Url;

use std::sync::mpsc::channel;

use oparl_cache::ExternalList;
use oparl_cache::Cacher;

use common::*;

fn test_parse_external_list(with_modified: bool) {
    let base_url = "http://localhost:8080/oparl/v1.0".into_url().unwrap();
    let mut server = mocking_server(base_url);

    let url = "http://localhost:8080/oparl/v1.0/body/0/list/paper";
    let time = "1969-07-21T02:56:00+00:00".to_string();

    let url_with_time1;
    let url_with_time2;

    if with_modified {
        url_with_time1 = Url::parse_with_params(url, &[("modified_since", &time)]).unwrap();
        url_with_time2 = Url::parse_with_params(url, &[("id", "3"), ("modified_since", &time)]).unwrap();
    } else {
        url_with_time1 = Url::parse(url).unwrap();
        url_with_time2 = Url::parse_with_params(url, &[("id", "3")]).unwrap();
    }

    let next = if with_modified {
        "http://localhost:8080/oparl/v1.0/body/0/list/paper?id=3&modified_since=1969-07-21T02%3A56%3A00%2B00%3A00"
    } else {
        "http://localhost:8080/oparl/v1.0/body/0/list/paper?id=3"
    };

    let page1 = object!{
        "data" => array![],
        "links" => object!{
            "next" => next
        }
    };

    let page2 = object!{
        "data" => array![],
        "links" => object!{}
    };

    server.add_response(url_with_time1, page1);
    server.add_response(url_with_time2, page2);

    let (add_list, receive_list) = channel();
    let storage = storage();

    let modified = if with_modified { Some(time) } else { None };

    let returned = storage.parse_external_list(url.into_url().unwrap(), modified, &server, add_list).unwrap();

    cleanup(&storage);

    assert_eq!(returned.0, url.into_url().unwrap());
    assert_eq!(receive_list.recv().is_err(), true);
}

#[test]
fn run_test_parse_external_list() {
    test_parse_external_list(false);
    test_parse_external_list(true);
}

/// Check whether iterating over a simple external list with an empty returns the right object ids
#[test]
fn external_list() {
    let expected_ids = ["http://oparl.tld/0",
                        "http://oparl.tld/1",
                        "http://oparl.tld/2",
                        "http://oparl.tld/3"];
    let url = "http://oparl.tld/".into_url().unwrap();
    let server = mocking_server(url.clone())
        .with_response("http://oparl.tld/list", object!{
            "data" => array![
                object!{"id" => "http://oparl.tld/0"},
                object!{"id" => "http://oparl.tld/1"}
             ],
            "links" => object!{
                "next" => "http://oparl.tld/list?page=2"
            }
        }).with_response("http://oparl.tld/list?page=2", object!{
            // Empty pages are allowed by the spec
            "data" => array![],
            "links" => object!{
                "next" => "http://oparl.tld/list?page=2"
            }
        }).with_response("http://oparl.tld/list?page=2", object!{
            "data" => array![
                object!{"id" => "http://oparl.tld/2"},
                object!{"id" => "http://oparl.tld/3"}
             ],
            "links" => object!{
                // Last page
            }
        });

    let list = ExternalList::new("http://oparl.tld/list".into_url().unwrap(), &server);

    let ids = list.map(|i| i.unwrap()["id"].to_owned()).collect::<Vec<_>>();
    assert_eq!(ids, expected_ids);
}
