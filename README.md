# `fast-dlt`

`fast-dlt` is a high-performance library designed for parsing Autosar DLT files.

**Please note that this crate is still a work in progress, so be prepared for potential bugs and incomplete features.**

## Features

The primary goal of `fast-dlt` is to offer lightning-fast DLT file reading. At this stage, it doesn't support creating or writing DLT messages, focusing solely on parsing and extracting data from existing DLT files.

## Examples

You can quickly print the contents of a DLT file with the following command:

```bash
cargo run --release --example print_file your/file.dlt
```

Counting the number of messages in a DLT file is straightforward with this command:

```bash
cargo run --release --example count your/file.dlt
```

## Getting Started

To get started with `fast-dlt`, you can include it as a dependency in your project's `Cargo.toml`:

```toml
[dependencies]
fast-dlt = "0.1.0"
```

Please make sure to check the latest version on [Crates.io](https://crates.io/crates/fast-dlt).

## License

`fast-dlt` may be used under your choice of the Apache 2 or MIT license.