use reqwest;
use json;

use std::error::Error;
use std::io::Read;
use std::mem;
use std::convert::From;

use json::JsonValue;
use reqwest::Url;
use reqwest::IntoUrl;

/// FIXME: Remove me
/// Helper function to download an object and return it as parsed json
pub fn get_json(url: Url) -> Result<JsonValue, Box<Error>> {
    println!("Loading: {:?}", &url);
    let mut reponse = reqwest::get(url)?;
    if !reponse.status().is_success() {
        return Err(From::from(format!("Bad status code returned for request: {}",
                                      reponse.status())));
    }

    let mut json_string = String::new();
    reponse.read_to_string(&mut json_string)?;
    Ok(json::parse(&json_string)?)
}

/// Exposes the objects of an eternal list as iterator
/// The objects  will be returned in the order they were received from the server
/// (A stable sorting is demanded by the spec)
#[derive(Debug)]
pub struct ExternalList {
    /// Points to the url of the current or, if the current page is exhausted, to the next one
    url: Url,
    /// Before and after the download, this will be None.
    /// During a successfull download, this will be Some(Ok(JsonValue))
    /// If any request has failed for any reason, this will be Some(Err(_))
    response: Option<Result<JsonValue, Box<Error>>>,
}

impl ExternalList {
    /// Constructs a new `ExternalList`
    pub fn new(url: Url) -> ExternalList {
        ExternalList {
            url: url,
            response: None,
        }
    }
}

impl Iterator for ExternalList {
    type Item = json::JsonValue;

    fn next<'a>(&mut self) -> Option<json::JsonValue> {
        // To avoid having multiple mutable borrows of self, a possibly existing version of
        // self.response is swapped out againt an Err at the beginning and re-emplaced at the end
        // of the function call.

        let mut response_some: ::std::result::Result<_, _> = {
            match self.response {
                Some(ref mut remaining) => {
                    // Avoid moving self.response by assinging an intermediate value
                    let mut swap_partner = Err(From::from("placeholder"));
                    mem::swap(remaining, &mut swap_partner);
                    swap_partner
                }
                None => get_json(self.url.clone()),
            }
        };

        let mut load_required = false;
        if let Ok(ref response) = response_some {
            if response["data"].len() == 0 {
                // Check wether this page is exhausted
                if response["links"].entries().any(|(x, _)| x == "next") {
                    self.url = response["links"]["next"]
                        .as_str()
                        .unwrap()
                        .into_url()
                        .unwrap();
                    load_required = true;
                } else {
                    return None; // List ended succesfully
                }
            }
        }

        if load_required {
            response_some = get_json(self.url.clone());
        }

        let return_value = match response_some {
            Ok(ref mut response) => {
                // Yield the objects in the order chosen by the server
                Some(response["data"].array_remove(0))
            }
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

