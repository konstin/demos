//! This file defines a CLI for the OParlCache crate
//! Use `cargo run -- --help` to get information on the available options

extern crate oparl_cache;
#[macro_use]
extern crate clap;
extern crate reqwest;

use std::error::Error;
use std::path::Path;

use reqwest::IntoUrl;

use oparl_cache::{Cacher, FileStorage, CommonServer};
use oparl_cache::file_storage::CACHE_STATUS_FILE;

/// List the servers cached in a storage
fn list(storage: FileStorage) -> Result<(), Box<Error>> {
    let servers = storage.get_cached_servers()?;
    println!("The following servers have been cached:");
    for i in servers.members() {
        println!(" - {}", i.as_str().ok_or("The entry isn't a url")?);
    }
    return Ok(());
}

fn main() {
    let matches = clap_app!(OParl_Cache_Rust =>
        (about: "Allows writing the data from an OParl API to a file cache.")
        (@arg entrypoint: "The url of the entrypoint")
        (@arg cachedir: -c --cache "The directory where the API responses will be saved")
        (@arg schemadir: -s --schema "The path of the folder with the OParl schema")
        (@arg cache_status_file: --cachestatus "The file where the information concerning the \
                                                 cache status gets stored")
        (@subcommand list =>
            (about: "List the servers cached in this storage")
        )
    )
        .get_matches();

    let entrypoint = matches.value_of("entrypoint").unwrap_or("http://localhost:8080/oparl/v1.0/");
    let cachedir = matches.value_of("cachedir").unwrap_or("/home/konsti/cache-rust/");
    let schemadir = matches.value_of("schemadir").unwrap_or("/home/konsti/oparl/schema/");
    let cache_status_file = matches.value_of("cache_status_file")
        .unwrap_or(CACHE_STATUS_FILE);

    let entrypoint = match entrypoint.into_url() {
        Ok(ok) => ok,
        Err(err) => {
            println!("Invalid URL for the entrypoint: {}", entrypoint);
            println!("Error: {}", err);
            return;
        }
    };

    let server = CommonServer::new(entrypoint);
    let storage = FileStorage::new(Path::new(schemadir), Path::new(cachedir).to_owned(), cache_status_file).unwrap();

    if matches.is_present("list") {
        let result = list(storage);
        if let Err(err) = result {
            println!("The file listing the servers has been corrupted ({})", err.description());
        }
        return;
    }

    let status = storage.cache(server);

    if let Err(err) = status {
        println!("✗ Loading failed: {}", err.description());
    } else {
        println!("✓ Done");
    }
}
