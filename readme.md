## Rust OParl Cache

[![Build Status](https://travis-ci.org/konstin/oparl-cache-rs.svg?branch=master)](https://travis-ci.org/konstin/oparl-cache-rs)

This is both a library and a CLI tool for caching the whole contents of an [OParl API](https://oparl.org)
This repository contains both a library and CLI written in rust for loading the contents of an OParl API 
to a simple file cache and to retrieve the cached files. Note that the actual documents provided by Paper are not
downloaded. 

## Usage

Install rust and its package manager, cargo.

Build and run with the default settingss:

```bash
cargo run
```

List all available options:

```bash
cargo run -- --help
```

To use this as a library, include the provided crate, which is called `òparl_cache`.
