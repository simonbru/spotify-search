# spotify-search
Search for tracks in JSON files produced by simonbru/spotify-backup.

## Run
Copy `src/config.example.rs` to `src/config.rs`.

Set adequate values in `src/config.rs`.

Run `cargo run --release YOUR_KEYWORD`

## Install
Copy `target/release/spotify-search` where you want or run `cargo install --path .` to install to Cargo's default location.
