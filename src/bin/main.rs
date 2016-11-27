//! This file defines a CLI for the OParlCache crate
//! Use `cargo run -- --help` to get information on the available options

extern crate oparl_cache;
#[macro_use] extern crate clap;

use oparl_cache::OParlCache;

fn main() {
    let matches = clap_app!(OParl_Cache_Rust =>
        (about: "Allows writing the data from an OParl API to a file cache.")
        (@arg entrypoint: "The url of the entrypoint")
        (@arg cachedir: -c --cache "The directory where the API responses will be saved")
        (@arg schemadir: -s --schema "The path of the folder with the OParl schema")
    ).get_matches();

    let entrypoint = matches.value_of("entrypoint").unwrap_or("http://localhost:8080/oparl/v1.0/");
    let cachedir = matches.value_of("cachedir").unwrap_or("/home/konsti/cache-rust/");
    let schemadir = matches.value_of("schemadir").unwrap_or("/home/konsti/oparl/schema/");

    let cache = OParlCache::new(entrypoint, schemadir, cachedir);
    cache.load_to_cache();
}