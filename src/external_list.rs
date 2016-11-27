use hyper;
use json;

use std::error::Error;
use std::io::Read;
use std::mem;
use std::convert::From;

use json::JsonValue;
use hyper::client::IntoUrl;

/// Helper function to download an object and return it as parsed json
pub fn download_json<U: IntoUrl + Clone>(url: U) -> Result<JsonValue, Box<Error>> {
    let client = hyper::Client::new();
    let mut res: hyper::client::Response = client.get(url.clone()).send()?;
    println!("Loaded: {:?}", url.into_url());
    let mut json_string = String::new();
    res.read_to_string(&mut json_string)?;
    Ok(json::parse(&json_string)?)
}

/// Exposes the objects of an eternal list as iterator
/// The objects  will be returned in the order they were received from the server
/// (A stable sorting is demanded by the spec)
#[derive(Debug)]
pub struct ExternalList {
    /// Points to the url of the current or, if the current page is exhausted, to the next one
    url: String,
    /// Before and after the download, this will be None.
    /// During a successfull download, this will be Some(Ok(JsonValue))
    /// If any request has failed for any reason, this will be Some(Err(_))
    response: Option<Result<JsonValue, Box<Error>>>,
}

impl ExternalList {
    pub fn new(url: String) -> ExternalList {
        ExternalList { url: url, response: None }
    }
}

impl Iterator for ExternalList {
    type Item = json::JsonValue;

    fn next<'a>(&mut self) -> Option<json::JsonValue> {
        let mut response_some: ::std::result::Result<_, _> = {
            match self.response {
                Some(ref mut remaining) => {
                    // Avoid moving self.response by assinging an intermediate value
                    let mut swap_partner = Err(From::from("asdf"));
                    mem::swap(remaining, &mut swap_partner);
                    swap_partner
                },
                None => {
                    download_json(&self.url)
                },
            }
        };

        let mut load_required = false;
        if let Ok(ref response) = response_some {
            if response["data"].len() == 0 {
                // Check wether this page is exhausted
                if response["links"].entries().any(|(x, _)| x == "next") {
                    self.url = response["links"]["next"].to_string();
                    load_required = true;
                } else {
                    return None; // List ended succesfully
                }
            }
        }

        if load_required {
            response_some = download_json(&self.url);
        }

        let return_value = match response_some {
            Ok(ref mut response) => {
                // Yield the objects in the order chosen by the server
                Some(response["data"].array_remove(0))
            },
            Err(ref e) => {
                println!("Downloading a list page failed. Aborting this list");
                println!("{:?}", e);
                None
            }
        };

        // self.response was replaced by an inermediate value, so move possibly updated value back
        // in place
        let mut x = Some(response_some);
        mem::swap(&mut self.response, &mut x);

        return_value
    }
}