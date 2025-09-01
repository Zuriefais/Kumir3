run:
    RUST_LOG=info cargo run --package kumir_gui

#!/bin/fish
run_web:
  cd crates/kumir_gui && \
  RUSTFLAGS='--cfg=web_sys_unstable_apis --cfg=getrandom_backend="wasm_js" -C target-feature=+atomics,+bulk-memory' \
  trunk serve

run_lang:
    RUST_LOG=info cargo run --package kumir_lang

run_runtime:
    RUST_LOG=info cargo run --package kumir_runtime

build:
    cargo build --release --package kumir_gui

build_web:
    cd crates/kumir_gui && \
    RUSTFLAGS="--cfg=web_sys_unstable_apis \
               --cfg=getrandom_backend=\"wasm_js\" \
               -C target-feature=+atomics,+bulk-memory \
               -C opt-level=z \
               -C codegen-units=1 \
               -C strip=debuginfo" \
    trunk build --release && \
    if command -v wasm-opt >/dev/null 2>&1; then \
        echo "✅ Running wasm-opt for size optimization..."; \
        for f in dist/*.wasm; do \
            if [ -f "$f" ]; then \
                echo "Before: $(stat -f%z "$f" 2>/dev/null || stat -c%s "$f") bytes → $f"; \
                wasm-opt -Oz -o "$f.opt" "$f" && mv "$f.opt" "$f"; \
                echo "After: $(stat -f%z "$f" 2>/dev/null || stat -c%s "$f") bytes"; \
            fi; \
        done; \
    else \
        echo "⚠️ wasm-opt not found, skipping extra optimization."; \
        echo "👉 Install with: brew install binaryen (macOS) or apt install binaryen (Linux)"; \
    fi
