cargo check --release --no-default-features --target=aarch64-unknown-linux-gnu
cargo check --release --no-default-features --target=aarch64-unknown-linux-musl
cargo check --release --no-default-features --target=x86_64-unknown-linux-gnu
cargo check --release --no-default-features --target=x86_64-unknown-linux-musl
cargo check --release --no-default-features --target=x86_64-pc-windows-gnu
# cargo check --release --no-default-features --target=x86_64-android-linux-gnu
# cargo check --release --no-default-features --target=wasm32-wasi-preview1-threads