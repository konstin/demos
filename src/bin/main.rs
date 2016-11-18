extern crate oparl_cache;

use oparl_cache::OParlCache;

fn main() {
    let mut cache = OParlCache::new();
    cache.load_to_cache("http://localhost:8080/oparl/v1.0/");
}