//! This file defines a CLI for the OParlCache crate
//! Use `cargo run -- --help` to get information on the available options

extern crate oparl_cache;
#[macro_use] extern crate clap;

use oparl_cache::OParlCache;

fn main() {
    let matches = clap_app!(OParl_Cache_Rust =>
        (about: "Allows writing the data from an OParl API to a file cache.")
        (@arg entrypoint: -e --entrypoint "The url of the entrypoint")
        (@arg debug: -d ... "Sets the level of debugging information")
    ).get_matches();

    let entrypoint = matches.value_of("entrypoint").unwrap_or("http://localhost:8080/oparl/v1.0/");

    let cache = OParlCache::new();
    cache.load_to_cache(entrypoint);
}