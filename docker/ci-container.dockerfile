FROM rust:latest AS ci-container
RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk wasm-bindgen-cli
