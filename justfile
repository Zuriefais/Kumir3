run:
    RUST_LOG=info cargo run --package kumir_gui

#!/bin/fish
run_web:
    cd crates/kumir_gui && RUSTFLAGS=--cfg=web_sys_unstable_apis trunk serve

run_lang:
    RUST_LOG=info cargo run --package kumir_lang

build:
    cargo build --release --package kumir_gui
