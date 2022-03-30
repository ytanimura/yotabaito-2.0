FROM rust:latest AS ci-container
RUN rustup default stable && rustup update
RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk wasm-bindgen-cli
