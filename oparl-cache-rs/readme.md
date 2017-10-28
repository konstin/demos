## Rust OParl Cache

[![Build Status](https://travis-ci.org/konstin/oparl-cache-rs.svg?branch=master)](https://travis-ci.org/konstin/oparl-cache-rs)

This is both a library and a CLI tool for caching the whole contents of an [OParl API](https://oparl.org)

## Usage

Install rust and its package manager, cargo.

Build and run with the default settings:

```bash
cargo run
```

List all available options:

```bash
cargo run -- --help
```

To use this as a library, include the `oparl_cache` crate, which offers implementations
of a file based storage and normal http based oparl servers.

## Notes

There is one big assumption made for the default `FileStorage`: There is not a folder ending with `.json` and a file
the same name with `.json`.

Also note that the acutual files are currently not downloaded.