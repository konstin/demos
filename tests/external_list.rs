extern crate oparl_cache;
extern crate reqwest;
extern crate chrono;
#[macro_use]
extern crate json;

mod common;

use reqwest::IntoUrl;

use oparl_cache::ExternalList;

use common::*;


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
