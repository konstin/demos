use std::collections::HashMap;
use std::error::Error;

use json::JsonValue;
use reqwest::Url;
use reqwest::IntoUrl;

use oparl_cache::Server;

pub struct MockingServer {
    entrypoint: Url,
    responses: HashMap<Url, JsonValue>,
}

impl MockingServer {
    pub fn new(entrypoint: Url) -> MockingServer {
        return MockingServer { entrypoint: entrypoint, responses: HashMap::new() }
    }

    pub fn add_response(&mut self, url: Url, response: JsonValue) {
        self.responses.insert(url, response);
    }

    pub fn with_response<T: IntoUrl>(mut self, url: T, response: JsonValue) -> MockingServer {
        self.responses.insert(url.into_url().unwrap(), response);
        self
    }
}

impl Server for MockingServer {
    fn get_json(&self, url: Url) -> Result<JsonValue, Box<Error>> {
        println!("{}", url.as_str());
        Ok(self.responses[&url].clone())
    }

    fn get_entrypoint(&self) -> Url {
        return self.entrypoint.clone();
    }
}
