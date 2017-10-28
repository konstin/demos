use url_serde;
use url::Url;
use external_list::ExternalList;
use reqwest;
use std::error::Error;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;


pub type ExternalListUrl<T> = OParlUrl<ExternalList<T>>;

/// Some really ugly API design to work around https://github.com/serde-rs/serde/issues/1048
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum OParlUrl<T: DeserializeOwned> {
    #[serde(with = "url_serde")]
    UsedBySerde(Url),
    NotUsedBySerde(PhantomData<T>)
}

impl<T: DeserializeOwned> OParlUrl<T> {
    pub fn get_url(&self) -> Url {
        match self {
            &OParlUrl::UsedBySerde(ref url) => url.clone(),
            // Also a helper againt https://github.com/serde-rs/serde/issues/1048
            _ => panic!("This is a bug")
        }
    }

    pub fn try_into(&self) -> Result<T, Box<Error>> {
        Ok(reqwest::get(self.get_url())?.json()?)
    }
}

