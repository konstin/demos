use json;

use std::error::Error;

use json::JsonValue;
use reqwest::Url;
use reqwest::IntoUrl;

use server::Server;

/// Exposes the objects of an eternal list as iterator
/// The objects  will be returned in the order they were received from the server
/// (A stable sorting is demanded by the spec)
#[derive(Debug)]
pub struct ExternalList<'a, T: 'a + Server> {
    /// The url of the next list page, if any
    page_link: Option<Url>,
    /// The items of the current list page
    objects: Vec<JsonValue>,
    /// The server used for getting the objects
    server: &'a T,
}

impl<'a, T: 'a + Server> ExternalList<'a, T> {
    /// Constructs a new `ExternalList`
    pub fn new(url: Url, server: &'a T) -> ExternalList<'a, T> {
        ExternalList {
            page_link: Some(url),
            objects: vec![],
            server: server,
        }
    }
}

impl<'a, T: 'a + Server> Iterator for ExternalList<'a, T> {
    type Item = Result<json::JsonValue, Box<Error>>;

    fn next(&mut self) -> Option<Result<json::JsonValue, Box<Error>>> {
        // Case 1: There are still objects of the last page, so return them
        if self.objects.len() >= 1 {
            return Some(Ok(self.objects.remove(0)));
        }

        // The loop is used because there might be empty pages
        loop {
            let response = if let Some(ref url) = self.page_link {
                // Case 2: No items, but there is the url of the next page
                self.server.get_json(url.clone())
            } else {
                // Case 3: The list is finished (or errored)
                return None;
            };

            let mut response = match response {
                Ok(ok) => ok,
                Err(err) => {
                    // There's no way to recover from a failed request
                    self.page_link = None;
                    return Some(Err(err));
                }
            };

            // Update the next page link and the items, setting it to None or empty respectively
            // on failure
            self.page_link = response["links"]["next"].as_str().and_then(|x| x.into_url().ok());
            self.objects = match response["data"].take() {
                JsonValue::Array(items) => items,
                _ => vec![],
            };

            if self.objects.len() >= 1 {
                // Case 2.1: The page actually contains data
                return Some(Ok(self.objects.remove(0)));
            }
        }
    }
}
