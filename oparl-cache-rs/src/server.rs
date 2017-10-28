use json;
use json::JsonValue;
use std::error::Error;
use reqwest;
use reqwest::Url;

use std::io::Read;

/// Defines an oparl server
///
/// Send and Sync are required because the server will be used from multiple workers
pub trait Server: Send + Sync {
    /// Returns a freshly loaded object as json
    fn get_json(&self, url: Url) -> Result<JsonValue, Box<Error>>;
    /// Returns the entrypoint for the server
    fn get_entrypoint(&self) -> Url;
}

/// A OParl server that is defined by its entrypoint url
pub struct CommonServer {
    entrypoint: Url,
}

impl CommonServer {
    /// Creates a new CommonServer
    pub fn new(entrypoint: Url) -> CommonServer {
        CommonServer { entrypoint: entrypoint }
    }
}

impl Server for CommonServer {
    /// Downloads an object and returns its parsed json
    fn get_json(&self, url: Url) -> Result<JsonValue, Box<Error>> {
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

    fn get_entrypoint(&self) -> Url {
        self.entrypoint.clone()
    }
}
