# ebook-name-from-metadata

Small CLI utility to automatically rename ebook files according to their metadata.
Only EPUB and PDF are supported.

Many ebook distributors provide files with names that are _extremely_ safe - all lowercase, alphanumeric only, without spaces.
These are good to ensure wide compatibility, but look bad on local file system.
They also might be hard to find using common system utilities.

This tool reads author and title from file metadata and renames it according to these values.
`--slugify` option is provided for people who still want to keep files safe for shell expansions.

## Installation

[Install rust and cargo on your system](https://www.rust-lang.org/tools/install).

Clone this repository.

In repository directory, execute `cargo build --locked -r`.

A compiled program will be available in `target/release/ebook-name-from-metadata`.
Copy it or save a full path somewhere.
