default:
    @just --list

test:
    cargo test --workspace

build:
    cargo build --workspace -q

fmt:
    cargo fmt --all

clippy:
    cargo clippy --workspace --all-targets --all-features -D warnings

bench:
    @echo "bench stubs; add Criterion later"
