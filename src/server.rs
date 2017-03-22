use json;
use json::JsonValue;
use std::error::Error;
use hyper::client::IntoUrl;
use hyper::Url;
use hyper;

use std::io::Read;

/// Defines an oparl server
pub trait Server {
    fn get_object<T: IntoUrl>(&self, url: T) -> Result<JsonValue, Box<Error>>;
    fn get_entrypoint(&self) -> Url;
    /// Helper function to download an object and return it as parsed json
    fn download_json(&self, url: Url) -> Result<JsonValue, Box<Error>>;
}

/// A OParl server that is defined by its entrypoint url
pub struct CommonServer {
    entrypoint: Url
}

impl CommonServer {
    pub fn new(entrypoint: Url) -> CommonServer {
        CommonServer { entrypoint: entrypoint }
    }
}

impl Server for CommonServer {
    fn get_object<T: IntoUrl>(&self, url: T) -> Result<JsonValue, Box<Error>> {
        let url = url.into_url()?;
        let client = hyper::Client::new();
        let mut res: hyper::client::Response = client.get(url).send()?;
        println!("Loaded: {:?}", res.url);
        let mut json_string = String::new();
        res.read_to_string(&mut json_string)?;
        Ok(json::parse(&json_string)?)
    }

    fn get_entrypoint(&self) -> Url {
        self.entrypoint.clone()
    }

    fn download_json(&self, url: Url) -> Result<JsonValue, Box<Error>> {
        let client = hyper::Client::new();
        let mut res: hyper::client::Response = client.get(url).send()?;
        println!("Loaded: {:?}", res.url);
        let mut json_string = String::new();
        res.read_to_string(&mut json_string)?;
        Ok(json::parse(&json_string)?)
    }
}