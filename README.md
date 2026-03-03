# Instant Finder (ifind)

## Description

Rust-based Command Line Tool (CLI) that persists directory structure and files in CBOR format, for instantaneous querying of large-scale cloud-based and local files.  

## Implementation

Populates a vector (vec) of  structs containg file details, after recursively traversing a directory (using walkdir crate), and persisting it to CBOR format (using serde crate).

CLI parsing is implement using Clap crate, with the following options:

```
Usage: instant-find [OPTIONS] [QUERY] [COMMAND]

Commands:
  update
  search
  help    Print this message or the help of the given subcommand(s)

Arguments:
  [QUERY]

Options:
  -e, --extension <EXTENSION>
  -h, --help                   Print help
```

Use with *dbxfs* for indexing large DropBox accounts.
