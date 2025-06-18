run:
    RUST_LOG=info cargo run --package kumir_gui

run_lang:
    RUST_LOG=info cargo run --package kumir_lang

build:
    cargo build --release --package kumir_gui
