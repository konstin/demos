use json;

use std::error::Error;
use std::mem;
use std::convert::From;

use json::JsonValue;
use reqwest::Url;
use reqwest::IntoUrl;

use server::Server;

/// Exposes the objects of an eternal list as iterator
/// The objects  will be returned in the order they were received from the server
/// (A stable sorting is demanded by the spec)
#[derive(Debug)]
pub struct ExternalList<'a, T: 'a + Server> {
    /// Points to the url of the current or, if the current page is exhausted, to the next one
    url: Url,
    /// Before and after the download, this will be None.
    /// During a successfull download, this will be Some(Ok(JsonValue))
    /// If any request has failed for any reason, this will be Some(Err(_))
    response: Option<Result<JsonValue, Box<Error>>>,
    /// The server used for getting the objects
    server: &'a T,
}

impl<'a, T: 'a + Server> ExternalList<'a, T> {
    /// Constructs a new `ExternalList`
    pub fn new(url: Url, server: &'a T) -> ExternalList<'a, T> {
        ExternalList {
            url: url,
            response: None,
            server: server,
        }
    }
}

impl<'a, T: 'a + Server> Iterator for ExternalList<'a, T> {
    type Item = json::JsonValue;

    fn next(&mut self) -> Option<json::JsonValue> {
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
                None => self.server.get_json(self.url.clone()),
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
            response_some = self.server.get_json(self.url.clone());
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
