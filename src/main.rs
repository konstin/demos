//! This file defines a CLI for the OParlCache crate
//! Use `cargo run -- --help` to get information on the available options

extern crate oparl_cache;
#[macro_use] extern crate clap;

use oparl_cache::{Storage, Cacher};

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
    let cache_status_file = matches.value_of("cache_status_file").unwrap_or(oparl_cache::DEFAULT_CACHE_STATUS_FILE);

    let storage = Storage::new(entrypoint, schemadir, cachedir, cache_status_file);
    let status = storage.load_to_cache();

    if let Err(err) = status {
        println!("✗ Loading failed: {}", err.description());
    } else {
        println!("✓ Succesfully loaded to cache");
    }
}