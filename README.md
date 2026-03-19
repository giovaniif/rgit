# rgit

A tiny Git implementation in Rust — built to understand how Git works under the hood.

## What it does

`rgit` reimplements Git's core object model: content-addressable storage, SHA-1 hashing, zlib compression, and the blob/tree/commit object types. It's compatible with real Git's hash algorithm, so `rgit hash-object` and `git hash-object` produce the same output.

## Commands

### `init`
Initialize a new repository.
```bash
rgit init
```
Creates a `.rgit/` directory with `objects/`, `refs/heads/`, and a `HEAD` pointing to `refs/heads/main`.

### `hash-object`
Compute (and optionally store) a blob object.
```bash
rgit hash-object <file>           # print SHA-1 hash
rgit hash-object <file> --write   # store the object and print hash
```

### `cat-file`
Read a stored object.
```bash
rgit cat-file <hash>           # print raw bytes
rgit cat-file <hash> --pretty  # print as text
```

## Build & run

```bash
cargo build --release
./target/release/rgit init
```

Or directly via Cargo:
```bash
cargo run -- init
cargo run -- hash-object hello.txt --write
cargo run -- cat-file <hash> --pretty
```

## Tests

```bash
cargo test
```

All 8 tests cover hashing, object storage, read/write round-trips, tree formatting, commit formatting, and a full end-to-end flow.

## How it works

Git stores everything as objects in `.git/objects/`. Each object is identified by the SHA-1 hash of its content (prefixed with a type header), compressed with zlib, and written to a path like `.git/objects/ab/cdef1234...`.

`rgit` follows the same layout under `.rgit/`:

- **Blob** — raw file contents
- **Tree** — a directory snapshot (list of mode/name/hash entries)
- **Commit** — a tree hash, optional parent, author info, and message

The hash computation matches Git exactly, so objects written by `rgit` can be read by `git` and vice versa.
