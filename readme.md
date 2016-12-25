## Rust OParl Cache

This repository contains both a library and CLI tool written in rust for loading the contents of an OParl API 
to an idiomatic file cache and to retrieve the cached files. Files provided by Paper are not downloaded. 

## Usage

You need rust and its package manager cargo first.

You can then use the CLI with the following command:

```bash
cargo run
```

To list all available options, run

```bash
cargo run -- --help
```

To use this as a library, use the provided crate, which is called `òparl_cache`.
