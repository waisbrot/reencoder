# Re-encoder

Walks through video files and re-encodes them as HEVC.

Primarily a project about learning some new languages (Rust).

## Prereqs

* Postgres 11 or later running
* A database called `media` with user and password `media` (or whatever, it's passed as an argument)
* Schema and data as seen in `schema.sql`
* Entries in the `roots` table for directories that should be walked

## Running

Runs as a docker container:

```
docker run -d \
  --name reencoder \
  --privileged \
  -e RUST_LOG=reencoder=info \
  scan-to-postgres \
    --host tularemia.local
    --password media \
    --username media \
    --modules 'clean,scan,reencode'
```
