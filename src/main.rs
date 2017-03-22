//! This file defines a CLI for the OParlCache crate
//! Use `cargo run -- --help` to get information on the available options

extern crate oparl_cache;
#[macro_use] extern crate clap;
extern crate hyper;

use hyper::client::IntoUrl;

use oparl_cache::{Cacher, FileStorage, CommonServer, DEFAULT_CACHE_STATUS_FILE};

fn main() {
    let matches = clap_app!(OParl_Cache_Rust =>
        (about: "Allows writing the data from an OParl API to a file cache.")
        (@arg entrypoint: "The url of the entrypoint")
        (@arg cachedir: -c --cache "The directory where the API responses will be saved")
        (@arg schemadir: -s --schema "The path of the folder with the OParl schema")
        (@arg cache_status_file: --cachestatus "The file where the information concerning the \
                                                 cache status gets stored")
    ).get_matches();

    let entrypoint = matches.value_of("entrypoint").unwrap_or("http://localhost:8080/oparl/v1.0/");
    let cachedir = matches.value_of("cachedir").unwrap_or("/home/konsti/cache-rust/");
    let schemadir = matches.value_of("schemadir").unwrap_or("/home/konsti/oparl/schema/");
    let cache_status_file = matches.value_of("cache_status_file").unwrap_or(DEFAULT_CACHE_STATUS_FILE);

    let entrypoint = match entrypoint.into_url() {
        Ok(ok) => ok,
        Err(err) => {
            println!("Invalid URL for the entrypoint: {}", entrypoint);
            println!("Error: {}", err);
            return
        }
    };

    let server = CommonServer::new(entrypoint);
    let storage = FileStorage::new(schemadir, cachedir, cache_status_file).unwrap();
    let status = storage.cache(server);

    if let Err(err) = status {
        println!("✗ Loading failed: {}", err.description());
    } else {
        println!("✓ Succesfully loaded to cache");
    }
}